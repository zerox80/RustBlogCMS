use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::convert::TryFrom;

/// Represents a coding tutorial.
#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct Tutorial {
    /// Unique UUID.
    pub id: String,
    /// Tutorial title.
    pub title: String,
    /// Description/Summary.
    pub description: String,
    /// Icon identifier (e.g., "rust", "react").
    pub icon: String,
    /// Hex color code for branding.
    pub color: String,
    /// JSON string containing the list of topics/sections.
    pub topics: String,
    /// Main content (Markdown/HTML mixed).
    pub content: String,
    /// Version number for optimistic concurrency.
    pub version: i64,
    /// Creation timestamp.
    pub created_at: String,
    /// Update timestamp.
    pub updated_at: String,
}

/// Payload to create a new tutorial.
#[derive(Debug, Deserialize)]
pub struct CreateTutorialRequest {
    /// Title.
    pub title: String,
    /// Description.
    pub description: String,
    /// Icon ID.
    pub icon: String,
    /// Hex color.
    pub color: String,
    /// List of topics (will be serialized to JSON).
    pub topics: Vec<String>,
    /// Content body.
    pub content: String,
    /// Optional ID (for pre-determined UUIDs).
    pub id: Option<String>,
}

/// Payload to update an existing tutorial.
#[derive(Debug, Deserialize)]
pub struct UpdateTutorialRequest {
    /// Update title.
    pub title: Option<String>,
    /// Update description.
    pub description: Option<String>,
    /// Update icon.
    pub icon: Option<String>,
    /// Update color.
    pub color: Option<String>,
    /// Update topics list.
    pub topics: Option<Vec<String>>,
    /// Update content.
    pub content: Option<String>,
}

/// Public response for a tutorial.
#[derive(Debug, Serialize)]
pub struct TutorialResponse {
    /// ID.
    pub id: String,
    /// Title.
    pub title: String,
    /// Description.
    pub description: String,
    /// Icon.
    pub icon: String,
    /// Color.
    pub color: String,
    /// Parsed topics list.
    pub topics: Vec<String>,
    /// Content.
    pub content: String,
    /// Version.
    pub version: i64,
    /// Created at.
    pub created_at: String,
    /// Updated at.
    pub updated_at: String,
}

/// Summary response (excludes heavy content).
#[derive(Debug, Serialize)]
pub struct TutorialSummaryResponse {
    /// ID.
    pub id: String,
    /// Title.
    pub title: String,
    /// Description.
    pub description: String,
    /// Icon.
    pub icon: String,
    /// Color.
    pub color: String,
    /// Parsed topics.
    pub topics: Vec<String>,
    /// Version.
    pub version: i64,
    /// Created at.
    pub created_at: String,
    /// Updated at.
    pub updated_at: String,
}

impl TryFrom<Tutorial> for TutorialResponse {
    type Error = String;

    /// Converts database model to response model, parsing JSON topics.
    fn try_from(tutorial: Tutorial) -> Result<Self, Self::Error> {
        // Parse the JSON topics string into a Vec<String>
        // Gracefully handle parsing errors by logging and returning empty list
        let topics: Vec<String> = serde_json::from_str(&tutorial.topics).unwrap_or_else(|e| {
            tracing::error!(
                "Failed to parse topics JSON for tutorial {}: {}. Topics JSON: '{}'",
                tutorial.id,
                e,
                tutorial.topics
            );
            Vec::new()
        });

        Ok(TutorialResponse {
            id: tutorial.id,
            title: tutorial.title,
            description: tutorial.description,
            icon: tutorial.icon,
            color: tutorial.color,
            topics,
            content: tutorial.content,
            version: tutorial.version,
            created_at: tutorial.created_at,
            updated_at: tutorial.updated_at,
        })
    }
}

impl TryFrom<Tutorial> for TutorialSummaryResponse {
    type Error = String;

    /// Converts database model to summary response, parsing JSON topics.
    fn try_from(tutorial: Tutorial) -> Result<Self, Self::Error> {
        let topics: Vec<String> = serde_json::from_str(&tutorial.topics).unwrap_or_else(|e| {
            tracing::error!(
                "Failed to parse topics JSON for tutorial {}: {}. Topics JSON: '{}'",
                tutorial.id,
                e,
                tutorial.topics
            );
            Vec::new()
        });

        Ok(TutorialSummaryResponse {
            id: tutorial.id,
            title: tutorial.title,
            description: tutorial.description,
            icon: tutorial.icon,
            color: tutorial.color,
            topics,
            version: tutorial.version,
            created_at: tutorial.created_at,
            updated_at: tutorial.updated_at,
        })
    }
}

/// Standard error response.
#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    /// The error message.
    pub error: String,
}

/// Response for file uploads.
#[derive(Debug, Serialize, Deserialize)]
pub struct UploadResponse {
    /// The URL of the uploaded file.
    pub url: String,
}
