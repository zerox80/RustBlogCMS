//! Image Upload HTTP Handlers
//!
//! This module provides secure image upload capabilities with several safeguards:
//! - RBAC: Admin role required
//! - File size limits (10MB)
//! - Filename extension whitelisting
//! - Magic byte (MIME) inference to prevent extension spoofing
//! - Atomic-like file writing with cleanup on failure
//! - UUID-based filename generation to prevent collisions and path injection

use crate::{
    models::{
        bad_request, forbidden, internal_error, internal_error_plain, ApiError, UploadResponse,
    },
    security::auth,
};
use axum::{extract::Multipart, Json};
use std::path::PathBuf;
use tokio::fs;
use uuid::Uuid;

/// Maximum allowed file size for uploads (10 megabytes)
const MAX_FILE_SIZE: usize = 10 * 1024 * 1024;
/// List of allowed file extensions for image uploads
const ALLOWED_EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "gif", "webp"];

/// Processes a multipart form-data request to upload an image.
/// Implements strict security validations before saving to disk.
pub async fn upload_image(
    claims: auth::Claims,
    mut multipart: Multipart,
) -> Result<Json<UploadResponse>, ApiError> {
    // SECURITY: Ensure only authorized administrators can upload assets
    if claims.role != "admin" {
        return Err(forbidden("Insufficient permissions"));
    }

    // Iterate through multipart fields
    while let Some(mut field) = multipart
        .next_field()
        .await
        .map_err(|err| bad_request(format!("Failed to process multipart field: {}", err)))?
    {
        // We only care about fields named "file"
        let name = field.name().unwrap_or("").to_string();

        if name == "file" {
            let file_name = field.file_name().unwrap_or("unknown").to_string();

            // Extract and normalize the file extension
            let ext = std::path::Path::new(&file_name)
                .extension()
                .and_then(|os_str| os_str.to_str())
                .unwrap_or("")
                .to_lowercase();

            // VALIDATION: Extension must be in our whitelist
            if !ALLOWED_EXTENSIONS.contains(&ext.as_str()) {
                return Err(bad_request(format!(
                    "Invalid file extension. Allowed: {:?}",
                    ALLOWED_EXTENSIONS
                )));
            }

            // Peek at the first chunk of data to perform MIME type detection (magic bytes)
            let first_chunk = match field
                .chunk()
                .await
                .map_err(internal_error("Failed to read file"))?
            {
                Some(chunk) => chunk,
                None => return Err(bad_request("File is empty")),
            };

            // VALIDATION: Verify the file content matches an allowed image type
            if let Some(kind) = infer::get(&first_chunk) {
                let detected_ext = kind.extension();
                // Normalize "jpeg" vs "jpg" for comparison
                let normalized_detected = if detected_ext == "jpeg" {
                    "jpg"
                } else {
                    detected_ext
                };
                let normalized_ext = if ext == "jpeg" { "jpg" } else { ext.as_str() };

                // SECURITY: Reject outright if the detected content type is not one of our
                // allowed image formats. This must be checked independently of the mismatch
                // check below: a detected type outside the allowlist (e.g. exe, zip, pdf)
                // would otherwise pass through unrejected and be saved under the client's
                // claimed extension.
                if !ALLOWED_EXTENSIONS.contains(&normalized_detected) {
                    return Err(bad_request(format!(
                        "Invalid file content. Detected type '{}' is not an allowed image format",
                        detected_ext
                    )));
                }

                // SECURITY: Reject if the detected type contradicts the provided file extension.
                if normalized_detected != normalized_ext {
                    return Err(bad_request(format!(
                        "File extension mismatch. Expected '{}', but detected '{}'",
                        ext, detected_ext
                    )));
                }
            } else {
                // REJECT if we can't determine what it is; this is safer than allowing mystery blobs.
                return Err(bad_request("Could not determine file type from magic bytes"));
            }

            // Generate a random ID for the filename to prevent path injection and name collisions
            let id = Uuid::new_v4();
            let new_filename = format!("{}.{}", id, ext);

            // Resolve the upload directory from environment or default to local "uploads"
            let upload_dir = std::env::var("UPLOAD_DIR").unwrap_or_else(|_| "uploads".to_string());
            let upload_path_base = PathBuf::from(upload_dir);

            // BOOTSTRAP: Ensure the physical directory exists
            if !upload_path_base.exists() {
                fs::create_dir_all(&upload_path_base)
                    .await
                    .map_err(internal_error("Failed to create uploads directory"))?;
            }

            let filepath = upload_path_base.join(&new_filename);
            // Create a temporary file path
            let temp_filename = format!("{}.tmp", id);
            let temp_filepath = upload_path_base.join(&temp_filename);

            // Open the temp file for writing
            let mut file = tokio::fs::File::create(&temp_filepath).await.map_err(|e| {
                tracing::error!(
                    "Failed to create temp file {}: {}",
                    temp_filepath.display(),
                    e
                );
                internal_error_plain("Failed to create file")
            })?;

            use tokio::io::AsyncWriteExt; // Required for write_all and flush

            // Write the first chunk
            if let Err(e) = file.write_all(&first_chunk).await {
                tracing::error!(
                    "Failed to write first chunk to {}: {}",
                    temp_filepath.display(),
                    e
                );
                let _ = tokio::fs::remove_file(&temp_filepath).await;
                return Err(internal_error_plain("Failed to write file"));
            }

            let mut total_size = first_chunk.len();

            // Stream the remaining chunks of the multipart field
            loop {
                let chunk_option = match field.chunk().await {
                    Ok(opt) => opt,
                    Err(err) => {
                        tracing::error!("Failed to read chunk: {}", err);
                        let _ = tokio::fs::remove_file(&temp_filepath).await;
                        return Err(internal_error_plain("Failed to read file"));
                    }
                };

                let chunk = match chunk_option {
                    Some(c) => c,
                    None => break,
                };

                // ENFORCEMENT: Track total size to prevent Disk Space exhaustion (DoS)
                total_size += chunk.len();
                if total_size > MAX_FILE_SIZE {
                    let _ = tokio::fs::remove_file(&temp_filepath).await;
                    return Err(bad_request(format!(
                        "File too large. Max size: {} bytes",
                        MAX_FILE_SIZE
                    )));
                }

                // Write chunk to disk
                if let Err(e) = file.write_all(&chunk).await {
                    tracing::error!(
                        "Failed to write chunk to {}: {}",
                        temp_filepath.display(),
                        e
                    );
                    let _ = tokio::fs::remove_file(&temp_filepath).await;
                    return Err(internal_error_plain("Failed to write file"));
                }
            }

            // Sync buffers to disk
            if let Err(e) = file.flush().await {
                tracing::error!("Failed to flush file {}: {}", temp_filepath.display(), e);
                let _ = tokio::fs::remove_file(&temp_filepath).await;
                return Err(internal_error_plain("Failed to save file"));
            }

            // Atomic rename from temp to final
            if let Err(e) = tokio::fs::rename(&temp_filepath, &filepath).await {
                tracing::error!(
                    "Failed to rename temp file {} to {}: {}",
                    temp_filepath.display(),
                    filepath.display(),
                    e
                );
                let _ = tokio::fs::remove_file(&temp_filepath).await;
                return Err(internal_error_plain("Failed to save file"));
            }

            // SUCCESS path
            tracing::info!("Successfully uploaded image: {}", filepath.display());

            return Ok(Json(UploadResponse {
                // Return the public-facing URL
                url: format!("/uploads/{}", new_filename),
            }));
        }
    }

    // Default error if for some reason the "file" field was missing
    Err(bad_request("No file found in request"))
}
