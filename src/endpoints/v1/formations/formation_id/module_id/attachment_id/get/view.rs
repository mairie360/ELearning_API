use utoipa::ToSchema;

/// A ready-to-use URL that renders the attachment inline in a browser. The
/// client does not need to know it points at an S3 bucket.
#[derive(Debug, serde::Serialize, ToSchema)]
pub struct GetAttachmentUrlView {
    /// URL signée à durée de vie limitée. À redemander à chaque ouverture plutôt qu'à stocker.
    #[schema(
        example = "https://storage.mairie360.fr/elearning/rgpd-principes.pdf?X-Amz-Expires=900&X-Amz-Signature=..."
    )]
    pub url: String,
}

impl GetAttachmentUrlView {
    pub fn new(url: String) -> Self {
        Self { url }
    }
}
