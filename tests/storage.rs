//! Tests unitaires du module `src/storage/` — aucun accès réseau : la signature
//! d'une URL presignée S3 est purement locale.

use std::sync::Mutex;

use elearning_api::storage::file_storage::{mime_for, FileStorage, S3FileStorage, StorageError};

// ---------------------------------------------------------------------------
// mime_for
// ---------------------------------------------------------------------------

#[test]
fn mime_for_maps_known_types() {
    assert_eq!(mime_for("pdf"), "application/pdf");
    assert_eq!(mime_for("video"), "video/mp4");
    assert_eq!(mime_for("something-else"), "application/octet-stream");
}

#[test]
fn storage_error_displays_context() {
    assert_eq!(
        StorageError::Config("boom".into()).to_string(),
        "file storage misconfigured: boom"
    );
    assert_eq!(
        StorageError::Presign("boom".into()).to_string(),
        "could not presign object URL: boom"
    );
}

#[tokio::test]
#[serial_test::serial]
async fn from_env_reads_s3_variables() {
    // SAFETY: `#[serial]` keeps env-mutating tests from overlapping.
    for (k, v) in [
        ("S3_BUCKET", "mairie360-elearning"),
        ("S3_REGION", "fr-par"),
        ("S3_ENDPOINT", "https://s3.fr-par.scw.cloud"),
        ("S3_ACCESS_KEY", "SCWXXXXXXXXXXXXXXXXX"),
        ("S3_SECRET_KEY", "00000000-0000-0000-0000-000000000000"),
        ("S3_PRESIGN_TTL_SECS", "120"),
    ] {
        std::env::set_var(k, v);
    }

    let storage = S3FileStorage::from_env().expect("builds from env");
    let url = storage
        .presigned_view_url("courses/1/guide.pdf", "application/pdf")
        .await
        .expect("presign");
    assert!(url.contains("X-Amz-Expires=120"));

    for k in [
        "S3_BUCKET",
        "S3_REGION",
        "S3_ENDPOINT",
        "S3_ACCESS_KEY",
        "S3_SECRET_KEY",
        "S3_PRESIGN_TTL_SECS",
    ] {
        std::env::remove_var(k);
    }
}

// ---------------------------------------------------------------------------
// MockFileStorage — double de test réutilisable
// ---------------------------------------------------------------------------

/// Records every `(key, content_type)` it is asked to sign and hands back a
/// canned URL, so callers can be tested without an S3 bucket.
struct MockFileStorage {
    calls: Mutex<Vec<(String, String)>>,
    url: String,
}

impl MockFileStorage {
    fn new(url: &str) -> Self {
        Self {
            calls: Mutex::new(Vec::new()),
            url: url.to_string(),
        }
    }
}

#[async_trait::async_trait]
impl FileStorage for MockFileStorage {
    async fn presigned_view_url(
        &self,
        key: &str,
        content_type: &str,
    ) -> Result<String, StorageError> {
        self.calls
            .lock()
            .unwrap()
            .push((key.to_string(), content_type.to_string()));
        Ok(self.url.clone())
    }
}

#[tokio::test]
async fn mock_records_call_and_returns_url() {
    let storage = MockFileStorage::new("https://example.test/canned");

    let url = storage
        .presigned_view_url("courses/1/guide.pdf", "application/pdf")
        .await
        .expect("mock never fails");

    assert_eq!(url, "https://example.test/canned");
    assert_eq!(
        *storage.calls.lock().unwrap(),
        vec![(
            "courses/1/guide.pdf".to_string(),
            "application/pdf".to_string()
        )]
    );
}

// ---------------------------------------------------------------------------
// S3FileStorage — signature réelle, hors-ligne
// ---------------------------------------------------------------------------

#[tokio::test]
async fn s3_presign_is_offline_and_inline() {
    let storage = S3FileStorage::new(
        "mairie360-elearning",
        "fr-par",
        "https://s3.fr-par.scw.cloud",
        "SCWXXXXXXXXXXXXXXXXX",
        "00000000-0000-0000-0000-000000000000",
        900,
    )
    .expect("client builds from static credentials");

    let url = storage
        .presigned_view_url("courses/1/modules/2/guide.pdf", "application/pdf")
        .await
        .expect("presigning is a local computation");

    assert!(url.starts_with("https://s3.fr-par.scw.cloud/mairie360-elearning/"));
    assert!(url.contains("courses/1/modules/2/guide.pdf"));
    assert!(url.contains("X-Amz-Signature="));
    assert!(url.contains("X-Amz-Expires=900"));
    assert!(url.contains("response-content-disposition=inline"));
    assert!(!url.contains("attachment"));
    assert!(url.contains("response-content-type=application%2Fpdf"));
}
