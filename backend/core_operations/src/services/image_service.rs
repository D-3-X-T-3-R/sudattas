// Image upload service for Cloudflare R2
// Generates presigned URLs for direct browser → R2 uploads

use aws_sdk_s3::{
    config::{Credentials, Region},
    presigning::PresigningConfig,
    Client as S3Client, Config as S3Config,
};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ImageError {
    #[error("S3/R2 error: {0}")]
    S3Error(String),
    
    #[error("Invalid file type: {0}. Allowed: {1}")]
    InvalidFileType(String, String),
    
    #[error("File too large: {0}MB. Max: {1}MB")]
    FileTooLarge(usize, usize),
    
    #[error("Invalid file name")]
    InvalidFileName,
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
}

/// Allowed image types
const ALLOWED_TYPES: &[&str] = &["image/jpeg", "image/png", "image/webp"];
const MAX_SIZE_MB: usize = 5;

/// Image upload configuration
#[derive(Clone)]
pub struct ImageConfig {
    pub r2_account_id: String,
    pub r2_access_key_id: String,
    pub r2_secret_access_key: String,
    pub r2_endpoint: String,
    pub r2_bucket_name: String,
    pub r2_public_url: String, // e.g., https://cdn.sudattas.com
    pub max_size_mb: usize,
}

impl ImageConfig {
    /// Load from environment variables
    pub fn from_env() -> Result<Self, ImageError> {
        Ok(Self {
            r2_account_id: std::env::var("R2_ACCOUNT_ID")
                .map_err(|_| ImageError::ConfigError("R2_ACCOUNT_ID not set".to_string()))?,
            r2_access_key_id: std::env::var("R2_ACCESS_KEY_ID")
                .map_err(|_| ImageError::ConfigError("R2_ACCESS_KEY_ID not set".to_string()))?,
            r2_secret_access_key: std::env::var("R2_SECRET_ACCESS_KEY")
                .map_err(|_| ImageError::ConfigError("R2_SECRET_ACCESS_KEY not set".to_string()))?,
            r2_endpoint: std::env::var("R2_ENDPOINT")
                .map_err(|_| ImageError::ConfigError("R2_ENDPOINT not set".to_string()))?,
            r2_bucket_name: std::env::var("R2_BUCKET_NAME")
                .map_err(|_| ImageError::ConfigError("R2_BUCKET_NAME not set".to_string()))?,
            r2_public_url: std::env::var("R2_PUBLIC_URL")
                .map_err(|_| ImageError::ConfigError("R2_PUBLIC_URL not set".to_string()))?,
            max_size_mb: std::env::var("MAX_IMAGE_SIZE_MB")
                .unwrap_or("5".to_string())
                .parse()
                .unwrap_or(MAX_SIZE_MB),
        })
    }
}

/// Image upload service
pub struct ImageService {
    client: S3Client,
    config: ImageConfig,
}

impl ImageService {
    /// Create a new image service
    pub fn new(config: ImageConfig) -> Result<Self, ImageError> {
        let credentials = Credentials::new(
            &config.r2_access_key_id,
            &config.r2_secret_access_key,
            None,
            None,
            "r2",
        );
        
        let s3_config = S3Config::builder()
            .endpoint_url(&config.r2_endpoint)
            .region(Region::new("auto"))
            .credentials_provider(credentials)
            .build();
        
        let client = S3Client::from_conf(s3_config);
        
        Ok(Self { client, config })
    }
    
    /// Validate file before upload
    pub fn validate_file(
        &self,
        file_name: &str,
        content_type: &str,
        file_size_mb: usize,
    ) -> Result<(), ImageError> {
        // Check file type
        if !ALLOWED_TYPES.contains(&content_type) {
            return Err(ImageError::InvalidFileType(
                content_type.to_string(),
                ALLOWED_TYPES.join(", "),
            ));
        }
        
        // Check file size
        if file_size_mb > self.config.max_size_mb {
            return Err(ImageError::FileTooLarge(file_size_mb, self.config.max_size_mb));
        }
        
        // Check file name
        if file_name.is_empty() || file_name.contains("..") {
            return Err(ImageError::InvalidFileName);
        }
        
        Ok(())
    }
    
    /// Generate a presigned URL for uploading
    /// 
    /// Example:
    /// ```
    /// let upload_url = image_service.generate_presigned_upload_url(
    ///     "products/saree-123/hero.webp",
    ///     "image/webp",
    ///     Duration::from_secs(300) // 5 minutes
    /// ).await?;
    /// 
    /// // Return this URL to frontend
    /// // Frontend does: PUT {upload_url} with image binary
    /// ```
    pub async fn generate_presigned_upload_url(
        &self,
        object_key: &str,
        content_type: &str,
        expiration: Duration,
    ) -> Result<String, ImageError> {
        let presigning_config = PresigningConfig::expires_in(expiration)
            .map_err(|e| ImageError::S3Error(e.to_string()))?;
        
        let presigned_request = self
            .client
            .put_object()
            .bucket(&self.config.r2_bucket_name)
            .key(object_key)
            .content_type(content_type)
            .presigned(presigning_config)
            .await
            .map_err(|e| ImageError::S3Error(e.to_string()))?;
        
        Ok(presigned_request.uri().to_string())
    }
    
    /// Generate public URL for an uploaded image
    pub fn get_public_url(&self, object_key: &str) -> String {
        format!("{}/{}", self.config.r2_public_url, object_key)
    }
    
    /// Delete an image from R2
    pub async fn delete_image(&self, object_key: &str) -> Result<(), ImageError> {
        self.client
            .delete_object()
            .bucket(&self.config.r2_bucket_name)
            .key(object_key)
            .send()
            .await
            .map_err(|e| ImageError::S3Error(e.to_string()))?;
        
        Ok(())
    }
    
    /// Generate multiple size variants for responsive images
    pub fn generate_variants(&self, base_path: &str, file_name: &str) -> ImageVariants {
        let name_without_ext = file_name.trim_end_matches(".webp")
            .trim_end_matches(".jpg")
            .trim_end_matches(".jpeg")
            .trim_end_matches(".png");
        
        ImageVariants {
            original: format!("{}/{}.webp", base_path, name_without_ext),
            large: format!("{}/{}-large.webp", base_path, name_without_ext),
            medium: format!("{}/{}-medium.webp", base_path, name_without_ext),
            thumbnail: format!("{}/{}-thumb.webp", base_path, name_without_ext),
        }
    }
}

/// Image size variants for responsive images
#[derive(Debug, Serialize, Deserialize)]
pub struct ImageVariants {
    pub original: String,     // Full size (1600x2400)
    pub large: String,        // Large/zoom (1600x2400)
    pub medium: String,       // Gallery (800x1200)
    pub thumbnail: String,    // Thumbnail (200x300)
}

/// Request to generate presigned upload URL
#[derive(Debug, Deserialize)]
pub struct PresignedUploadRequest {
    pub file_name: String,
    pub content_type: String,
    pub file_size_mb: usize,
    pub product_sku: Option<String>, // For organizing in folders
}

/// Response with presigned URL
#[derive(Debug, Serialize)]
pub struct PresignedUploadResponse {
    pub upload_url: String,
    pub object_key: String,
    pub public_url: String,
    pub expires_in_seconds: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_validate_file() {
        let config = ImageConfig {
            r2_account_id: "test".to_string(),
            r2_access_key_id: "test".to_string(),
            r2_secret_access_key: "test".to_string(),
            r2_endpoint: "https://test.r2.cloudflarestorage.com".to_string(),
            r2_bucket_name: "test".to_string(),
            r2_public_url: "https://cdn.test.com".to_string(),
            max_size_mb: 5,
        };
        
        let service = ImageService::new(config).unwrap();
        
        // Valid file
        assert!(service.validate_file("test.webp", "image/webp", 2).is_ok());
        
        // Invalid type
        assert!(service.validate_file("test.pdf", "application/pdf", 2).is_err());
        
        // Too large
        assert!(service.validate_file("test.webp", "image/webp", 10).is_err());
        
        // Invalid name
        assert!(service.validate_file("../etc/passwd", "image/webp", 2).is_err());
    }
    
    #[test]
    fn test_generate_variants() {
        let config = ImageConfig {
            r2_account_id: "test".to_string(),
            r2_access_key_id: "test".to_string(),
            r2_secret_access_key: "test".to_string(),
            r2_endpoint: "https://test.r2.cloudflarestorage.com".to_string(),
            r2_bucket_name: "test".to_string(),
            r2_public_url: "https://cdn.test.com".to_string(),
            max_size_mb: 5,
        };
        
        let service = ImageService::new(config).unwrap();
        let variants = service.generate_variants("products/saree-123", "hero.webp");
        
        assert_eq!(variants.original, "products/saree-123/hero.webp");
        assert_eq!(variants.large, "products/saree-123/hero-large.webp");
        assert_eq!(variants.medium, "products/saree-123/hero-medium.webp");
        assert_eq!(variants.thumbnail, "products/saree-123/hero-thumb.webp");
    }
    
    #[test]
    fn test_get_public_url() {
        let config = ImageConfig {
            r2_account_id: "test".to_string(),
            r2_access_key_id: "test".to_string(),
            r2_secret_access_key: "test".to_string(),
            r2_endpoint: "https://test.r2.cloudflarestorage.com".to_string(),
            r2_bucket_name: "test".to_string(),
            r2_public_url: "https://cdn.sudattas.com".to_string(),
            max_size_mb: 5,
        };
        
        let service = ImageService::new(config).unwrap();
        let url = service.get_public_url("products/saree-123/hero.webp");
        
        assert_eq!(url, "https://cdn.sudattas.com/products/saree-123/hero.webp");
    }
}
