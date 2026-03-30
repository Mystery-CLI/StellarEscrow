import AWS from 'aws-sdk';
import { Router, Request, Response, NextFunction } from 'express';
import multer from 'multer';
import sharp from 'sharp';
import crypto from 'crypto';
import path from 'path';
import fs from 'fs';
import { z } from 'zod';

// ============================================================================
// CONFIGURATION & TYPES
// ============================================================================

interface UploadedFile {
  fieldname: string;
  originalname: string;
  encoding: string;
  mimetype: string;
  size: number;
  buffer: Buffer;
}

interface FileMetadata {
  fileId: string;
  userId: string;
  fileName: string;
  originalName: string;
  mimeType: string;
  size: number;
  compressed: boolean;
  originalSize?: number;
  uploadedAt: Date;
  expiresAt?: Date;
  permissions: string[];
}

interface SignedUrlResponse {
  fileId: string;
  url: string;
  expiresIn: number;
  expiresAt: Date;
}

// File type whitelist
const ALLOWED_FILE_TYPES = {
  images: ['image/png', 'image/jpeg', 'image/webp'],
  documents: ['application/pdf'],
} as const;

const FILE_SIZE_LIMITS = {
  image: 5 * 1024 * 1024, // 5MB
  document: 10 * 1024 * 1024, // 10MB
} as const;

const SIGNED_URL_EXPIRATION = 24 * 60 * 60; // 24 hours

// ============================================================================
// S3 CLIENT & STORAGE SERVICE
// ============================================================================

class S3StorageService {
  private s3: AWS.S3;
  private bucketName: string;
  private region: string;
  private fileMetadataMap: Map<string, FileMetadata> = new Map();

  constructor() {
    this.s3 = new AWS.S3({
      region: process.env.AWS_REGION || 'us-east-1',
      accessKeyId: process.env.AWS_ACCESS_KEY_ID,
      secretAccessKey: process.env.AWS_SECRET_ACCESS_KEY,
    });

    this.bucketName = process.env.S3_BUCKET_NAME || 'stellar-escrow-storage';
    this.region = process.env.AWS_REGION || 'us-east-1';

    this.initializeBucket();
  }

  /**
   * Initialize S3 bucket with proper configuration
   */
  private async initializeBucket(): Promise<void> {
    try {
      // Check if bucket exists
      await this.s3.headBucket({ Bucket: this.bucketName }).promise();
      console.log(`✓ S3 bucket '${this.bucketName}' exists`);

      // Enable versioning for data protection
      await this.s3
        .putBucketVersioning({
          Bucket: this.bucketName,
          VersioningConfiguration: { Status: 'Enabled' },
        })
        .promise();

      // Enable encryption
      await this.s3
        .putBucketEncryption({
          Bucket: this.bucketName,
          ServerSideEncryptionConfiguration: {
            Rules: [
              {
                ApplyServerSideEncryptionByDefault: {
                  SSEAlgorithm: 'AES256',
                },
              },
            ],
          },
        })
        .promise();

      // Block public access
      await this.s3
        .putPublicAccessBlockConfiguration({
          Bucket: this.bucketName,
          PublicAccessBlockConfiguration: {
            BlockPublicAcls: true,
            BlockPublicPolicy: true,
            IgnorePublicAcls: true,
            RestrictPublicBuckets: true,
          },
        })
        .promise();

      console.log('✓ S3 bucket security configuration applied');
    } catch (error) {
      console.error('Failed to initialize S3 bucket:', error);
    }
  }

  /**
   * Upload file to S3 with metadata
   */
  async uploadFile(
    userId: string,
    file: UploadedFile,
    fileType: 'avatar' | 'document'
  ): Promise<FileMetadata> {
    const fileId = this.generateFileId();
    const key = this.generateS3Key(userId, fileId, fileType);
    let uploadBuffer = file.buffer;
    let compressed = false;
    let originalSize = file.size;

    // Compress images
    if (fileType === 'avatar' && file.mimetype.startsWith('image/')) {
      uploadBuffer = await sharp(file.buffer)
        .resize(400, 400, {
          fit: 'cover',
          position: 'center',
        })
        .webp({ quality: 80 })
        .toBuffer();

      compressed = true;
      console.log(`✓ Image compressed: ${originalSize} → ${uploadBuffer.length} bytes`);
    }

    // Upload to S3 with server-side encryption
    const uploadParams: AWS.S3.PutObjectRequest = {
      Bucket: this.bucketName,
      Key: key,
      Body: uploadBuffer,
      ContentType: fileType === 'avatar' && compressed ? 'image/webp' : file.mimetype,
      ServerSideEncryption: 'AES256',
      Metadata: {
        'user-id': userId,
        'file-type': fileType,
        'original-name': file.originalname,
        'upload-timestamp': new Date().toISOString(),
      },
      // Enable multipart upload for large files
      StorageClass: 'STANDARD_IA', // Infrequent access for cost optimization
    };

    try {
      await this.s3.putObject(uploadParams).promise();
      console.log(`✓ File uploaded to S3: ${key}`);
    } catch (error) {
      console.error('S3 upload failed:', error);
      throw new Error('Failed to upload file to storage');
    }

    // Store metadata
    const metadata: FileMetadata = {
      fileId,
      userId,
      fileName: path.basename(key),
      originalName: file.originalname,
      mimeType: compressed ? 'image/webp' : file.mimetype,
      size: uploadBuffer.length,
      compressed,
      originalSize,
      uploadedAt: new Date(),
      expiresAt: this.calculateExpiration(fileType),
      permissions: ['owner', 'read'],
    };

    this.fileMetadataMap.set(fileId, metadata);

    return metadata;
  }

  /**
   * Generate signed URL for file download (time-limited, secure)
   */
  generateSignedUrl(
    fileId: string,
    userId: string,
    expiresIn: number = SIGNED_URL_EXPIRATION
  ): SignedUrlResponse {
    const metadata = this.fileMetadataMap.get(fileId);

    if (!metadata) {
      throw new Error('File not found');
    }

    if (metadata.userId !== userId) {
      throw new Error('Unauthorized: User does not own this file');
    }

    const key = `${userId}/${fileId}/${metadata.fileName}`;
    const expiresAt = new Date(Date.now() + expiresIn * 1000);

    const url = this.s3.getSignedUrl('getObject', {
      Bucket: this.bucketName,
      Key: key,
      Expires: expiresIn,
      ResponseContentDisposition: `attachment; filename="${metadata.originalName}"`,
      ServerSideEncryption: 'AES256',
    });

    return {
      fileId,
      url,
      expiresIn,
      expiresAt,
    };
  }

  /**
   * Download file from S3 (verify permissions first)
   */
  async downloadFile(fileId: string, userId: string): Promise<Buffer> {
    const metadata = this.fileMetadataMap.get(fileId);

    if (!metadata) {
      throw new Error('File not found');
    }

    // Verify ownership
    if (metadata.userId !== userId) {
      throw new Error('Unauthorized: User does not own this file');
    }

    const key = `${userId}/${fileId}/${metadata.fileName}`;

    try {
      const params: AWS.S3.GetObjectRequest = {
        Bucket: this.bucketName,
        Key: key,
      };

      const data = await this.s3.getObject(params).promise();
      console.log(`✓ File downloaded: ${key}`);

      return data.Body as Buffer;
    } catch (error) {
      console.error('S3 download failed:', error);
      throw new Error('Failed to download file');
    }
  }

  /**
   * Delete file from S3 (permanent, owner-only)
   */
  async deleteFile(fileId: string, userId: string): Promise<void> {
    const metadata = this.fileMetadataMap.get(fileId);

    if (!metadata) {
      throw new Error('File not found');
    }

    if (metadata.userId !== userId) {
      throw new Error('Unauthorized: Cannot delete other users\' files');
    }

    const key = `${userId}/${fileId}/${metadata.fileName}`;

    try {
      // Delete current version
      await this.s3.deleteObject({ Bucket: this.bucketName, Key: key }).promise();

      // Optionally delete all versions (if versioning is enabled)
      const versions = await this.s3
        .listObjectVersions({ Bucket: this.bucketName, Prefix: key })
        .promise();

      if (versions.Versions) {
        for (const version of versions.Versions) {
          await this.s3
            .deleteObject({
              Bucket: this.bucketName,
              Key: version.Key!,
              VersionId: version.VersionId,
            })
            .promise();
        }
      }

      this.fileMetadataMap.delete(fileId);
      console.log(`✓ File deleted: ${key}`);
    } catch (error) {
      console.error('S3 deletion failed:', error);
      throw new Error('Failed to delete file');
    }
  }

  /**
   * Get file metadata without download
   */
  getFileMetadata(fileId: string, userId: string): FileMetadata {
    const metadata = this.fileMetadataMap.get(fileId);

    if (!metadata) {
      throw new Error('File not found');
    }

    if (metadata.userId !== userId) {
      throw new Error('Unauthorized: Cannot access other users\' files');
    }

    return metadata;
  }

  /**
   * List all files for user
   */
  listUserFiles(userId: string): FileMetadata[] {
    return Array.from(this.fileMetadataMap.values()).filter(
      (meta) => meta.userId === userId
    );
  }

  // =========================================================================
  // PRIVATE HELPERS
  // =========================================================================

  private generateFileId(): string {
    return `file_${crypto.randomBytes(16).toString('hex')}`;
  }

  private generateS3Key(userId: string, fileId: string, fileType: string): string {
    return `${fileType}/${userId}/${fileId}/data`;
  }

  private calculateExpiration(fileType: 'avatar' | 'document'): Date | undefined {
    if (fileType === 'document') {
      // Documents expire after 7 days
      return new Date(Date.now() + 7 * 24 * 60 * 60 * 1000);
    }
    // Avatars don't expire
    return undefined;
  }
}

// ============================================================================
// VALIDATION MIDDLEWARE
// ============================================================================

const uploadSchema = z.object({
  fileType: z.enum(['avatar', 'document']),
  userId: z.string().min(1),
});

function validateFileType(fileType: string, mimetype: string): boolean {
  if (fileType === 'avatar') {
    return ALLOWED_FILE_TYPES.images.includes(mimetype as any);
  } else if (fileType === 'document') {
    return ALLOWED_FILE_TYPES.documents.includes(mimetype as any);
  }
  return false;
}

function validateFileSize(fileType: string, size: number): boolean {
  const limit = fileType === 'avatar' ? FILE_SIZE_LIMITS.image : FILE_SIZE_LIMITS.document;
  return size <= limit;
}

// ============================================================================
// EXPRESS MIDDLEWARE & ROUTES
// ============================================================================

const storage = new S3StorageService();
const upload = multer({ storage: multer.memoryStorage(), limits: { fileSize: 50 * 1024 * 1024 } });
const router = Router();

/**
 * POST /upload
 * Upload a file (avatar or legal document)
 */
router.post(
  '/upload',
  upload.single('file'),
  async (req: Request, res: Response, next: NextFunction) => {
    try {
      // Validate request body
      const { fileType, userId } = uploadSchema.parse(req.body);

      // Validate file exists
      if (!req.file) {
        return res.status(400).json({ error: 'No file provided' });
      }

      // Validate file type
      if (!validateFileType(fileType, req.file.mimetype)) {
        return res.status(400).json({
          error: `Invalid file type for ${fileType}. Allowed: ${
            fileType === 'avatar' ? 'PNG, JPEG, WebP' : 'PDF'
          }`,
        });
      }

      // Validate file size
      if (!validateFileSize(fileType, req.file.size)) {
        const limit = fileType === 'avatar' ? '5MB' : '10MB';
        return res.status(400).json({
          error: `File exceeds size limit of ${limit}`,
        });
      }

      // Upload to S3
      const metadata = await storage.uploadFile(userId, req.file, fileType);

      res.status(201).json({
        success: true,
        fileId: metadata.fileId,
        fileName: metadata.fileName,
        size: metadata.size,
        compressed: metadata.compressed,
        uploadedAt: metadata.uploadedAt,
      });
    } catch (error) {
      console.error('Upload error:', error);
      res.status(500).json({ error: 'Upload failed' });
    }
  }
);

/**
 * GET /download/:fileId
 * Download file with signed URL (temporary access)
 */
router.get('/download/:fileId', (req: Request, res: Response, next: NextFunction) => {
  try {
    const { fileId } = req.params;
    const userId = req.headers['x-user-id'] as string;

    if (!userId) {
      return res.status(401).json({ error: 'User ID required' });
    }

    const signedUrl = storage.generateSignedUrl(fileId, userId);

    res.json({
      fileId: signedUrl.fileId,
      url: signedUrl.url,
      expiresIn: signedUrl.expiresIn,
      expiresAt: signedUrl.expiresAt,
    });
  } catch (error) {
    console.error('Download error:', error);
    res.status(500).json({ error: 'Download failed' });
  }
});

/**
 * GET /files
 * List all files for authenticated user
 */
router.get('/files', (req: Request, res: Response, next: NextFunction) => {
  try {
    const userId = req.headers['x-user-id'] as string;

    if (!userId) {
      return res.status(401).json({ error: 'User ID required' });
    }

    const files = storage.listUserFiles(userId);

    res.json({
      success: true,
      count: files.length,
      files: files.map((f) => ({
        fileId: f.fileId,
        originalName: f.originalName,
        size: f.size,
        mimeType: f.mimeType,
        uploadedAt: f.uploadedAt,
        expiresAt: f.expiresAt,
      })),
    });
  } catch (error) {
    console.error('List files error:', error);
    res.status(500).json({ error: 'Failed to list files' });
  }
});

/**
 * GET /files/:fileId/metadata
 * Get file metadata (for preview info)
 */
router.get('/files/:fileId/metadata', (req: Request, res: Response, next: NextFunction) => {
  try {
    const { fileId } = req.params;
    const userId = req.headers['x-user-id'] as string;

    if (!userId) {
      return res.status(401).json({ error: 'User ID required' });
    }

    const metadata = storage.getFileMetadata(fileId, userId);

    res.json({
      success: true,
      fileId: metadata.fileId,
      fileName: metadata.fileName,
      originalName: metadata.originalName,
      size: metadata.size,
      originalSize: metadata.originalSize,
      mimeType: metadata.mimeType,
      compressed: metadata.compressed,
      uploadedAt: metadata.uploadedAt,
      expiresAt: metadata.expiresAt,
    });
  } catch (error) {
    console.error('Metadata error:', error);
    res.status(500).json({ error: 'Failed to get metadata' });
  }
});

/**
 * DELETE /files/:fileId
 * Permanently delete a file (owner only)
 */
router.delete('/files/:fileId', async (req: Request, res: Response, next: NextFunction) => {
  try {
    const { fileId } = req.params;
    const userId = req.headers['x-user-id'] as string;

    if (!userId) {
      return res.status(401).json({ error: 'User ID required' });
    }

    await storage.deleteFile(fileId, userId);

    res.json({
      success: true,
      message: 'File deleted successfully',
    });
  } catch (error) {
    console.error('Delete error:', error);
    res.status(500).json({ error: 'Failed to delete file' });
  }
});

export { router as fileStorageRouter, S3StorageService };
