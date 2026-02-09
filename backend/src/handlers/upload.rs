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
    models::{ErrorResponse, UploadResponse},
    security::auth,
};
use axum::{extract::Multipart, http::StatusCode, Json};
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
) -> Result<Json<UploadResponse>, (StatusCode, Json<ErrorResponse>)> {
    // SECURITY: Ensure only authorized administrators can upload assets
    if claims.role != "admin" {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ErrorResponse {
                error: "Insufficient permissions".to_string(),
            }),
        ));
    }

    // Iterate through multipart fields
    while let Some(mut field) = multipart.next_field().await.map_err(|err| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: format!("Failed to process multipart field: {}", err),
            }),
        )
    })? {
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
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(ErrorResponse {
                        error: format!("Invalid file extension. Allowed: {:?}", ALLOWED_EXTENSIONS),
                    }),
                ));
            }

            // Peek at the first chunk of data to perform MIME type detection (magic bytes)
            let first_chunk = match field.chunk().await.map_err(|err| {
                tracing::error!("Failed to read first chunk: {}", err);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse {
                        error: "Failed to read file".to_string(),
                    }),
                )
            })? {
                Some(chunk) => chunk,
                None => {
                    return Err((
                        StatusCode::BAD_REQUEST,
                        Json(ErrorResponse {
                            error: "File is empty".to_string(),
                        }),
                    ))
                }
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

                // SECURITY: Reject if the content type (magic bytes) represents an extension we don't allow,
                // or if it obviously contradicts the provided file extension.
                if ALLOWED_EXTENSIONS.contains(&normalized_detected)
                    && normalized_detected != normalized_ext
                {
                    return Err((
                        StatusCode::BAD_REQUEST,
                        Json(ErrorResponse {
                            error: format!(
                                "File extension mismatch. Expected '{}', but detected '{}'",
                                ext, detected_ext
                            ),
                        }),
                    ));
                }
            } else {
                // REJECT if we can't determine what it is; this is safer than allowing mystery blobs.
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(ErrorResponse {
                        error: "Could not determine file type from magic bytes".to_string(),
                    }),
                ));
            }

            // Generate a random ID for the filename to prevent path injection and name collisions
            let id = Uuid::new_v4();
            let new_filename = format!("{}.{}", id, ext);

            // Resolve the upload directory from environment or default to local "uploads"
            let upload_dir = std::env::var("UPLOAD_DIR").unwrap_or_else(|_| "uploads".to_string());
            let upload_path_base = PathBuf::from(upload_dir);

            // BOOTSTRAP: Ensure the physical directory exists
            if !upload_path_base.exists() {
                fs::create_dir_all(&upload_path_base).await.map_err(|err| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ErrorResponse {
                            error: format!("Failed to create uploads directory: {}", err),
                        }),
                    )
                })?;
            }

            let filepath = upload_path_base.join(&new_filename);
            // Create a temporary file path
            let temp_filename = format!("{}.tmp", id);
            let temp_filepath = upload_path_base.join(&temp_filename);

            // Open the temp file for writing
            let mut file = match tokio::fs::File::create(&temp_filepath).await {
                Ok(file) => file,
                Err(e) => {
                    tracing::error!(
                        "Failed to create temp file {}: {}",
                        temp_filepath.display(),
                        e
                    );
                    return Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ErrorResponse {
                            error: "Failed to create file".to_string(),
                        }),
                    ));
                }
            };

            use tokio::io::AsyncWriteExt; // Required for write_all and flush

            // Write the first chunk
            if let Err(e) = file.write_all(&first_chunk).await {
                tracing::error!(
                    "Failed to write first chunk to {}: {}",
                    temp_filepath.display(),
                    e
                );
                let _ = tokio::fs::remove_file(&temp_filepath).await;
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse {
                        error: "Failed to write file".to_string(),
                    }),
                ));
            }

            let mut total_size = first_chunk.len();

            // Stream the remaining chunks of the multipart field
            loop {
                let chunk_option = match field.chunk().await {
                    Ok(opt) => opt,
                    Err(err) => {
                        tracing::error!("Failed to read chunk: {}", err);
                        let _ = tokio::fs::remove_file(&temp_filepath).await;
                        return Err((
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(ErrorResponse {
                                error: format!("Failed to read file: {}", err),
                            }),
                        ));
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
                    return Err((
                        StatusCode::BAD_REQUEST,
                        Json(ErrorResponse {
                            error: format!("File too large. Max size: {} bytes", MAX_FILE_SIZE),
                        }),
                    ));
                }

                // Write chunk to disk
                if let Err(e) = file.write_all(&chunk).await {
                    tracing::error!(
                        "Failed to write chunk to {}: {}",
                        temp_filepath.display(),
                        e
                    );
                    let _ = tokio::fs::remove_file(&temp_filepath).await;
                    return Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ErrorResponse {
                            error: "Failed to write file".to_string(),
                        }),
                    ));
                }
            }

            // Sync buffers to disk
            if let Err(e) = file.flush().await {
                tracing::error!("Failed to flush file {}: {}", temp_filepath.display(), e);
                let _ = tokio::fs::remove_file(&temp_filepath).await;
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse {
                        error: "Failed to save file".to_string(),
                    }),
                ));
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
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse {
                        error: "Failed to save file".to_string(),
                    }),
                ));
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
    Err((
        StatusCode::BAD_REQUEST,
        Json(ErrorResponse {
            error: "No file found in request".to_string(),
        }),
    ))
}
