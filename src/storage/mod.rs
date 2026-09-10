//! Object storage for course attachments. See [`file_storage`].

pub mod file_storage;

pub use file_storage::{mime_for, FileStorage, S3FileStorage, StorageError};
