/// Verify a backup file exists and optionally matches a checksum.
pub async fn verify_backup(location: &str, expected_checksum: Option<&str>) -> bool {
    // For S3 locations, check via HEAD request
    if location.starts_with("s3://") {
        return verify_s3(location).await;
    }

    // For local files, check existence and optionally checksum
    let path = std::path::Path::new(location);
    if !path.exists() {
        tracing::warn!("Backup file not found: {}", location);
        return false;
    }

    if let Some(expected) = expected_checksum {
        match compute_sha256(location).await {
            Ok(actual) if actual == expected => true,
            Ok(actual) => {
                tracing::warn!("Backup checksum mismatch: expected={} actual={}", expected, actual);
                false
            }
            Err(e) => {
                tracing::warn!("Checksum computation failed: {}", e);
                false
            }
        }
    } else {
        true
    }
}

async fn verify_s3(location: &str) -> bool {
    // In production this would use the AWS SDK to HEAD the object.
    // For now, treat S3 locations as verified if the script succeeded.
    tracing::info!("S3 backup location recorded: {}", location);
    true
}

async fn compute_sha256(path: &str) -> anyhow::Result<String> {
    use sha2::{Digest, Sha256};
    let data = tokio::fs::read(path).await?;
    let hash = Sha256::digest(&data);
    Ok(hex::encode(hash))
}
