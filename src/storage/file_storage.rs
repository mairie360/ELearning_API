//! File storage abstraction over the Scaleway S3 bucket that holds the course
//! attachments.
//!
//! `course_attachments.file_url` stores the **S3 object key**, never a public
//! URL — the bucket is private. To let a client view a file, the API mints a
//! short-lived presigned `GET` URL with `response-content-disposition = inline`
//! so the browser renders the PDF/video in place instead of downloading it.
//!
//! Only the read path is implemented today. Upload/delete (`put_object` /
//! `delete_object`) would be added to [`FileStorage`] here.

use std::collections::HashMap;

use s3::creds::Credentials;
use s3::{Bucket, Region};

use mairie360_api_lib::env_manager::{get_critical_env_var, get_env_var};

/// Default lifetime of a presigned URL when `S3_PRESIGN_TTL_SECS` is unset.
const DEFAULT_PRESIGN_TTL_SECS: u32 = 900;

/// Anything that can hand out a viewable URL for a stored object.
#[async_trait::async_trait]
pub trait FileStorage: Send + Sync {
    /// Returns a URL that renders the object at `key` inline in a browser.
    ///
    /// `content_type` is forced onto the response (`response-content-type`) so
    /// the object is served with the right MIME regardless of what was set at
    /// upload time. See [`mime_for`].
    async fn presigned_view_url(
        &self,
        key: &str,
        content_type: &str,
    ) -> Result<String, StorageError>;
}

/// Maps the `attachment_type` enum text stored in Postgres to a MIME type.
pub fn mime_for(file_type: &str) -> &'static str {
    match file_type {
        "pdf" => "application/pdf",
        "video" => "video/mp4",
        _ => "application/octet-stream",
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StorageError {
    /// The S3 client could not be built (bad credentials, bad region, …).
    Config(String),
    /// Signing the URL failed.
    Presign(String),
}

impl std::fmt::Display for StorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StorageError::Config(msg) => write!(f, "file storage misconfigured: {msg}"),
            StorageError::Presign(msg) => write!(f, "could not presign object URL: {msg}"),
        }
    }
}

impl std::error::Error for StorageError {}

/// [`FileStorage`] backed by an S3-compatible bucket (Scaleway Object Storage).
pub struct S3FileStorage {
    bucket: Box<Bucket>,
    ttl_secs: u32,
}

impl S3FileStorage {
    /// Builds the client from explicit settings. No network I/O.
    pub fn new(
        bucket: &str,
        region: &str,
        endpoint: &str,
        access_key: &str,
        secret_key: &str,
        ttl_secs: u32,
    ) -> Result<Self, StorageError> {
        let credentials = Credentials::new(Some(access_key), Some(secret_key), None, None, None)
            .map_err(|e| StorageError::Config(e.to_string()))?;
        let region = Region::Custom {
            region: region.to_string(),
            endpoint: endpoint.to_string(),
        };
        // `with_path_style` keeps the bucket in the path (`<endpoint>/<bucket>/<key>`);
        // Scaleway supports both, path-style avoids DNS/vhost surprises.
        let bucket = Bucket::new(bucket, region, credentials)
            .map_err(|e| StorageError::Config(e.to_string()))?
            .with_path_style();
        Ok(Self { bucket, ttl_secs })
    }

    /// Builds the client from the `S3_*` environment variables.
    ///
    /// Panics (like the rest of the config, via `get_critical_env_var`) if a
    /// required variable is missing. `S3_PRESIGN_TTL_SECS` is optional.
    pub fn from_env() -> Result<Self, StorageError> {
        let ttl_secs = get_env_var("S3_PRESIGN_TTL_SECS")
            .and_then(|v| v.parse().ok())
            .unwrap_or(DEFAULT_PRESIGN_TTL_SECS);
        Self::new(
            &get_critical_env_var("S3_BUCKET"),
            &get_critical_env_var("S3_REGION"),
            &get_critical_env_var("S3_ENDPOINT"),
            &get_critical_env_var("S3_ACCESS_KEY"),
            &get_critical_env_var("S3_SECRET_KEY"),
            ttl_secs,
        )
    }
}

#[async_trait::async_trait]
impl FileStorage for S3FileStorage {
    async fn presigned_view_url(
        &self,
        key: &str,
        content_type: &str,
    ) -> Result<String, StorageError> {
        let mut queries = HashMap::new();
        queries.insert(
            "response-content-disposition".to_string(),
            "inline".to_string(),
        );
        queries.insert(
            "response-content-type".to_string(),
            content_type.to_string(),
        );
        self.bucket
            .presign_get(key, self.ttl_secs, Some(queries))
            .await
            .map_err(|e| StorageError::Presign(e.to_string()))
    }
}
