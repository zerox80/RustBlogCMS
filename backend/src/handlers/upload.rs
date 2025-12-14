use crate::{
    security::auth,
    models::{ErrorResponse, UploadResponse},
};
use axum::{
    extract::{Multipart, State},
    http::StatusCode,
    Json,
};
use std::path::PathBuf;
use tokio::fs;
use uuid::Uuid;

const MAX_FILE_SIZE: usize = 10 * 1024 * 1024; // 10MB
const ALLOWED_EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "gif", "webp"];

pub async fn upload_image(
    claims: auth::Claims,
    mut multipart: Multipart,
) -> Result<Json<UploadResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Ensure user is admin
    if claims.role != "admin" {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ErrorResponse {
                error: "Insufficient permissions".to_string(),
            }),
        ));
    }

    while let Some(mut field) = multipart.next_field().await.map_err(|err| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: format!("Failed to process multipart field: {}", err),
            }),
        )
    })? {
        let name = field.name().unwrap_or("").to_string();

        if name == "file" {
            let file_name = field.file_name().unwrap_or("unknown").to_string();

            // Simple extension validation
            let ext = std::path::Path::new(&file_name)
                .extension()
                .and_then(|os_str| os_str.to_str())
                .unwrap_or("")
                .to_lowercase();

            if !ALLOWED_EXTENSIONS.contains(&ext.as_str()) {
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(ErrorResponse {
                        error: format!("Invalid file extension. Allowed: {:?}", ALLOWED_EXTENSIONS),
                    }),
                ));
            }

            // Get first chunk to validate magic bytes
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
                None => return Err((
                    StatusCode::BAD_REQUEST,
                    Json(ErrorResponse {
                        error: "File is empty".to_string(),
                    }),
                )),
            };

            // Validate file content using magic bytes
            if let Some(kind) = infer::get(&first_chunk) {
                let detected_ext = kind.extension();
                 // Verify the detected extension matches the file extension (prevent spoofing)
                let normalized_detected = if detected_ext == "jpeg" { "jpg" } else { detected_ext };
                let normalized_ext = if ext == "jpeg" { "jpg" } else { ext.as_str() };

                // Allow matches where magic bytes might be generic but extension is specific and allowed, 
                // but primarily check for obvious mismatches if detected extension is in our allowed list.
                // If infer detects something NOT in allowed list, reject.
                // If infer detects something in allowed list but different from extension, reject.
                
                if ALLOWED_EXTENSIONS.contains(&normalized_detected) && normalized_detected != normalized_ext {
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
                 // Could not infer type, but extension is allowed. 
                 // We might strictly require inference, but for now let's issue a warning or allow if it's a known text issue?
                 // For images, infer should usually work.
                 return Err((
                    StatusCode::BAD_REQUEST,
                    Json(ErrorResponse {
                        error: "Could not determine file type from magic bytes".to_string(),
                    }),
                ));
            }

            let id = Uuid::new_v4();
            let new_filename = format!("{}.{}", id, ext);
            let upload_dir = std::env::var("UPLOAD_DIR").unwrap_or_else(|_| "uploads".to_string());
            let upload_path_base = PathBuf::from(upload_dir);
             
            // Ensure uploads directory exists
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

            // Create file and write first chunk
            let mut file = match tokio::fs::File::create(&filepath).await {
                Ok(file) => file,
                Err(e) => {
                    tracing::error!("Failed to create file {}: {}", filepath.display(), e);
                    return Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ErrorResponse {
                            error: "Failed to create file".to_string(),
                        }),
                    ));
                }
            };

            use tokio::io::AsyncWriteExt; // Import trait for write_all
            
            if let Err(e) = file.write_all(&first_chunk).await {
                 tracing::error!("Failed to write first chunk to {}: {}", filepath.display(), e);
                 let _ = tokio::fs::remove_file(&filepath).await;
                 return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse {
                        error: "Failed to write file".to_string(),
                    }),
                ));
            }

            let mut total_size = first_chunk.len();

            while let Some(chunk) = field.chunk().await.map_err(|err| {
                tracing::error!("Failed to read chunk: {}", err);
                let _ = tokio::fs::remove_file(&filepath).await;
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse {
                        error: format!("Failed to read file: {}", err),
                    }),
                )
            })? {
                total_size += chunk.len();
                if total_size > MAX_FILE_SIZE {
                    let _ = tokio::fs::remove_file(&filepath).await;
                    return Err((
                        StatusCode::BAD_REQUEST,
                        Json(ErrorResponse {
                            error: format!("File too large. Max size: {} bytes", MAX_FILE_SIZE),
                        }),
                    ));
                }
                
                if let Err(e) = file.write_all(&chunk).await {
                     tracing::error!("Failed to write chunk to {}: {}", filepath.display(), e);
                     let _ = tokio::fs::remove_file(&filepath).await;
                     return Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ErrorResponse {
                            error: "Failed to write file".to_string(),
                        }),
                    ));
                }
            }

            if let Err(e) = file.flush().await {
                 tracing::error!("Failed to flush file {}: {}", filepath.display(), e);
                 let _ = tokio::fs::remove_file(&filepath).await;
                 return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse {
                        error: "Failed to save file".to_string(),
                    }),
                ));
            }

            tracing::info!("Successfully uploaded image: {}", filepath.display());

            return Ok(Json(UploadResponse {
                url: format!("/uploads/{}", new_filename),
            }));
        }
    }

    Err((
        StatusCode::BAD_REQUEST,
        Json(ErrorResponse {
            error: "No file found in request".to_string(),
        }),
    ))
}
