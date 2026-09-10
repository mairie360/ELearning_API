use utoipa::ToSchema;

/// A ready-to-use URL that renders the attachment inline in a browser. The
/// client does not need to know it points at an S3 bucket.
#[derive(Debug, serde::Serialize, ToSchema)]
pub struct GetAttachmentUrlView {
    pub url: String,
}

impl GetAttachmentUrlView {
    pub fn new(url: String) -> Self {
        Self { url }
    }
}
