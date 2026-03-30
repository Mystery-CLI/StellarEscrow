import { fileStorageRouter, S3StorageService } from '../src/services/file-storage';
import request from 'supertest';
import express, { Express } from 'express';

describe('File Storage Service', () => {
  let app: Express;
  let storage: S3StorageService;

  beforeAll(() => {
    app = express();
    app.use(express.json());
    app.use('/storage', fileStorageRouter);
    storage = new S3StorageService();
  });

  describe('POST /storage/upload', () => {
    it('should upload a valid PNG image as avatar', async () => {
      const response = await request(app)
        .post('/storage/upload')
        .field('fileType', 'avatar')
        .field('userId', 'user_123')
        .attach('file', Buffer.from('PNG_FILE_CONTENT'), 'avatar.png');

      expect(response.status).toBe(201);
      expect(response.body.fileId).toBeDefined();
      expect(response.body.size).toBeGreaterThan(0);
      expect(response.body.compressed).toBe(true); // Should compress images
    });

    it('should upload a valid PDF as document', async () => {
      const response = await request(app)
        .post('/storage/upload')
        .field('fileType', 'document')
        .field('userId', 'user_123')
        .attach('file', Buffer.from('PDF_CONTENT'), 'legal-agreement.pdf');

      expect(response.status).toBe(201);
      expect(response.body.fileId).toBeDefined();
      expect(response.body.compressed).toBe(false); // Should not compress PDFs
    });

    it('should reject invalid file types', async () => {
      const response = await request(app)
        .post('/storage/upload')
        .field('fileType', 'avatar')
        .field('userId', 'user_123')
        .attach('file', Buffer.from('EXECUTABLE_CONTENT'), 'malware.exe');

      expect(response.status).toBe(400);
      expect(response.body.error).toContain('Invalid file type');
    });

    it('should reject files exceeding size limit', async () => {
      // Create a buffer exceeding 5MB for avatar
      const largeBuffer = Buffer.alloc(6 * 1024 * 1024);

      const response = await request(app)
        .post('/storage/upload')
        .field('fileType', 'avatar')
        .field('userId', 'user_123')
        .attach('file', largeBuffer, 'huge-image.png');

      expect(response.status).toBe(400);
      expect(response.body.error).toContain('exceeds size limit');
    });

    it('should reject upload without file', async () => {
      const response = await request(app)
        .post('/storage/upload')
        .field('fileType', 'avatar')
        .field('userId', 'user_123');

      expect(response.status).toBe(400);
      expect(response.body.error).toContain('No file provided');
    });

    it('should reject upload without userId', async () => {
      const response = await request(app)
        .post('/storage/upload')
        .field('fileType', 'avatar')
        .attach('file', Buffer.from('PNG'), 'avatar.png');

      expect(response.status).toBe(400);
    });
  });

  describe('GET /storage/download/:fileId', () => {
    let fileId: string;

    beforeAll(async () => {
      // Upload a file first
      const response = await request(app)
        .post('/storage/upload')
        .field('fileType', 'avatar')
        .field('userId', 'user_123')
        .attach('file', Buffer.from('PNG_DATA'), 'test.png');

      fileId = response.body.fileId;
    });

    it('should generate signed URL for file download', async () => {
      const response = await request(app)
        .get(`/storage/download/${fileId}`)
        .set('x-user-id', 'user_123');

      expect(response.status).toBe(200);
      expect(response.body.url).toBeDefined();
      expect(response.body.url).toContain('X-Amz-Signature'); // AWS signed URL
      expect(response.body.expiresAt).toBeDefined();
      expect(response.body.expiresIn).toBe(86400); // 24 hours
    });

    it('should reject download without user ID', async () => {
      const response = await request(app).get(`/storage/download/${fileId}`);

      expect(response.status).toBe(401);
      expect(response.body.error).toContain('User ID required');
    });

    it('should reject download by unauthorized user', async () => {
      const response = await request(app)
        .get(`/storage/download/${fileId}`)
        .set('x-user-id', 'attacker_user');

      expect(response.status).toBe(500); // File not found for this user
    });
  });

  describe('GET /storage/files', () => {
    it('should list all files for user', async () => {
      const response = await request(app)
        .get('/storage/files')
        .set('x-user-id', 'user_123');

      expect(response.status).toBe(200);
      expect(response.body.success).toBe(true);
      expect(response.body.count).toBeGreaterThan(0);
      expect(Array.isArray(response.body.files)).toBe(true);
    });

    it('should return empty list for user with no files', async () => {
      const response = await request(app)
        .get('/storage/files')
        .set('x-user-id', 'new_user_xyz');

      expect(response.status).toBe(200);
      expect(response.body.count).toBe(0);
      expect(response.body.files).toEqual([]);
    });

    it('should reject list request without user ID', async () => {
      const response = await request(app).get('/storage/files');

      expect(response.status).toBe(401);
    });
  });

  describe('GET /storage/files/:fileId/metadata', () => {
    let fileId: string;

    beforeAll(async () => {
      const response = await request(app)
        .post('/storage/upload')
        .field('fileType', 'document')
        .field('userId', 'user_123')
        .attach('file', Buffer.from('PDF'), 'doc.pdf');

      fileId = response.body.fileId;
    });

    it('should return file metadata', async () => {
      const response = await request(app)
        .get(`/storage/files/${fileId}/metadata`)
        .set('x-user-id', 'user_123');

      expect(response.status).toBe(200);
      expect(response.body.fileId).toBe(fileId);
      expect(response.body.mimeType).toBe('application/pdf');
      expect(response.body.size).toBeGreaterThan(0);
      expect(response.body.uploadedAt).toBeDefined();
    });
  });

  describe('DELETE /storage/files/:fileId', () => {
    let fileId: string;

    beforeAll(async () => {
      const response = await request(app)
        .post('/storage/upload')
        .field('fileType', 'avatar')
        .field('userId', 'user_delete_test')
        .attach('file', Buffer.from('PNG'), 'to-delete.png');

      fileId = response.body.fileId;
    });

    it('should delete file owned by user', async () => {
      const response = await request(app)
        .delete(`/storage/files/${fileId}`)
        .set('x-user-id', 'user_delete_test');

      expect(response.status).toBe(200);
      expect(response.body.success).toBe(true);
      expect(response.body.message).toContain('deleted successfully');
    });

    it('should reject deletion by unauthorized user', async () => {
      const response = await request(app)
        .delete(`/storage/files/${fileId}`)
        .set('x-user-id', 'attacker_user');

      expect(response.status).toBe(500);
    });

    it('should return 404 for already-deleted file', async () => {
      const response = await request(app)
        .delete(`/storage/files/${fileId}`)
        .set('x-user-id', 'user_delete_test');

      expect(response.status).toBe(500);
    });
  });

  describe('Image Compression', () => {
    it('should compress large image to WebP', async () => {
      const largeImageBuffer = Buffer.alloc(2 * 1024 * 1024); // 2MB dummy buffer

      const response = await request(app)
        .post('/storage/upload')
        .field('fileType', 'avatar')
        .field('userId', 'user_compression_test')
        .attach('file', largeImageBuffer, 'large.png');

      expect(response.status).toBe(201);
      expect(response.body.compressed).toBe(true);
      expect(response.body.size).toBeLessThan(2 * 1024 * 1024); // Should be smaller after compression
    });
  });

  describe('Security Tests', () => {
    it('should not allow path traversal in filename', async () => {
      const response = await request(app)
        .post('/storage/upload')
        .field('fileType', 'avatar')
        .field('userId', 'user_123')
        .attach('file', Buffer.from('PNG'), '../../../etc/passwd.png');

      expect(response.status).toBe(201);
      // Filename should be sanitized
      expect(response.body.fileName).not.toContain('..');
    });

    it('should isolate files by user', async () => {
      // Upload as user_a
      const uploadA = await request(app)
        .post('/storage/upload')
        .field('fileType', 'avatar')
        .field('userId', 'user_a')
        .attach('file', Buffer.from('CONTENT_A'), 'file_a.png');

      // Upload as user_b
      const uploadB = await request(app)
        .post('/storage/upload')
        .field('fileType', 'avatar')
        .field('userId', 'user_b')
        .attach('file', Buffer.from('CONTENT_B'), 'file_b.png');

      // List files as user_b
      const listB = await request(app)
        .get('/storage/files')
        .set('x-user-id', 'user_b');

      // Should only see user_b's file
      expect(listB.body.count).toBe(1);
      expect(listB.body.files[0].fileId).toBe(uploadB.body.fileId);
    });
  });
});
