use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::FromRow;

/// Represents dynamic content for a site section.
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct SiteContent {
    /// The section identifier (e.g., "features", "cta").
    pub section: String,
    /// JSON string containing the section content.
    pub content_json: String,
    /// ISO 8601 timestamp of last update.
    pub updated_at: String,
}

/// Response payload for site content.
#[derive(Debug, Serialize)]
pub struct SiteContentResponse {
    /// The section identifier.
    pub section: String,
    /// Parsed JSON content.
    pub content: Value,
    /// Last updated timestamp.
    pub updated_at: String,
}

/// List response for multiple content sections.
#[derive(Debug, Serialize)]
pub struct SiteContentListResponse {
    /// List of content items.
    pub items: Vec<SiteContentResponse>,
}

/// Payload to update a site section's content.
#[derive(Debug, Deserialize)]
pub struct UpdateSiteContentRequest {
    /// The new content as a JSON value.
    pub content: Value,
}

/// Represents a standalone page in the site structure.
#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct SitePage {
    /// Unique UUID for the page.
    pub id: String,
    /// URL slug for the page.
    pub slug: String,
    /// Page title.
    pub title: String,
    /// SEO description.
    pub description: String,
    /// Text label for the navigation link (if enabled).
    pub nav_label: Option<String>,
    /// Whether this page appears in the main navigation.
    pub show_in_nav: bool,
    /// Sorting order in navigation.
    pub order_index: i64,
    /// Whether the page is publicly visible.
    pub is_published: bool,
    /// JSON string representing the hero section configuration.
    pub hero_json: String,
    /// JSON string representing the page layout configuration.
    pub layout_json: String,
    /// Creation timestamp.
    pub created_at: String,
    /// Last update timestamp.
    pub updated_at: String,
}

/// Public response for a site page.
#[derive(Debug, Serialize)]
pub struct SitePageResponse {
    /// The page ID.
    pub id: String,
    /// The URL slug.
    pub slug: String,
    /// The page title.
    pub title: String,
    /// The page description.
    pub description: String,
    /// The navigation label.
    pub nav_label: Option<String>,
    /// Navigation visibility status.
    pub show_in_nav: bool,
    /// Navigation sort order.
    pub order_index: i64,
    /// Publication status.
    pub is_published: bool,
    /// Parsed hero object.
    pub hero: Value,
    /// Parsed layout object.
    pub layout: Value,
    /// Creation timestamp.
    pub created_at: String,
    /// Update timestamp.
    pub updated_at: String,
}

/// List response for site pages.
#[derive(Debug, Serialize)]
pub struct SitePageListResponse {
    /// Collection of pages.
    pub items: Vec<SitePageResponse>,
}

/// Response combining a page with its associated posts.
#[derive(Debug, Serialize)]
pub struct SitePageWithPostsResponse {
    /// The full page details.
    pub page: SitePageResponse,
    /// List of posts belonging to this page.
    pub posts: Vec<SitePostResponse>,
}

/// Response containing detailed view of a single post and its parent page.
#[derive(Debug, Serialize)]
pub struct SitePostDetailResponse {
    /// The parent page details.
    pub page: SitePageResponse,
    /// The post details.
    pub post: SitePostResponse,
}

/// Payload to create a new site page.
#[derive(Debug, Deserialize)]
pub struct CreateSitePageRequest {
    /// The URL slug.
    pub slug: String,
    /// The page title.
    pub title: String,
    /// Optional description.
    pub description: Option<String>,
    /// Navigation label (optional).
    pub nav_label: Option<String>,
    /// Whether to show in navigation (default: false).
    #[serde(default)]
    pub show_in_nav: bool,
    /// Sort order (optional).
    pub order_index: Option<i64>,
    /// Whether published immediately (default: false).
    #[serde(default)]
    pub is_published: bool,
    /// Hero section config (default: null/empty).
    #[serde(default)]
    pub hero: Value,
    /// Layout config (default: null/empty).
    #[serde(default)]
    pub layout: Value,
}

/// Payload to update an existing page.
#[derive(Debug, Deserialize)]
pub struct UpdateSitePageRequest {
    /// Update slug unique.
    pub slug: Option<String>,
    /// Update title.
    pub title: Option<String>,
    /// Update description.
    pub description: Option<String>,
    /// Update nav label. Double Option allows clearing the label.
    pub nav_label: Option<Option<String>>,
    /// Update visibility in nav.
    pub show_in_nav: Option<bool>,
    /// Update sort order.
    pub order_index: Option<i64>,
    /// Update publication status.
    pub is_published: Option<bool>,
    /// Update hero config.
    pub hero: Option<Value>,
    /// Update layout config.
    pub layout: Option<Value>,
}

/// Represents a blog post or page content item.
#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct SitePost {
    /// Unique UUID.
    pub id: String,
    /// ID of the parent page.
    pub page_id: String,
    /// Post title.
    pub title: String,
    /// URL slug.
    pub slug: String,
    /// Short summary.
    pub excerpt: String,
    /// Main content (Markdown).
    pub content_markdown: String,
    /// Public visibility status.
    pub is_published: bool,
    /// Whether comments are enabled.
    pub allow_comments: bool,
    /// Timestamp when published (optional).
    pub published_at: Option<String>,
    /// Sort order.
    pub order_index: i64,
    /// Creation timestamp.
    pub created_at: String,
    /// Last update timestamp.
    pub updated_at: String,
}

/// Public response for a site post.
#[derive(Debug, Serialize)]
pub struct SitePostResponse {
    /// Post ID.
    pub id: String,
    /// Parent Page ID.
    pub page_id: String,
    /// Title.
    pub title: String,
    /// Slug.
    pub slug: String,
    /// Excerpt.
    pub excerpt: String,
    /// Content (Markdown).
    pub content_markdown: String,
    /// Publication status.
    pub is_published: bool,
    /// Comment status.
    pub allow_comments: bool,
    /// Publishing timestamp.
    pub published_at: Option<String>,
    /// Sort order.
    pub order_index: i64,
    /// Creation time.
    pub created_at: String,
    /// Update time.
    pub updated_at: String,
}

/// List response for posts.
#[derive(Debug, Serialize)]
pub struct SitePostListResponse {
    /// List of post items.
    pub items: Vec<SitePostResponse>,
}

/// Payload to create a new blog post.
#[derive(Debug, Deserialize)]
pub struct CreateSitePostRequest {
    /// Post title.
    pub title: String,
    /// URL slug.
    pub slug: String,
    /// Short summary.
    pub excerpt: Option<String>,
    /// Markdown body.
    pub content_markdown: String,
    /// Whether public (default: false).
    #[serde(default)]
    pub is_published: bool,
    /// Enable comments (defaults to true).
    #[serde(default = "default_allow_comments")]
    pub allow_comments: bool,
    /// Optional publish date.
    pub published_at: Option<String>,
    /// Sort order.
    pub order_index: Option<i64>,
}

/// Helper to default `allow_comments` to true.
fn default_allow_comments() -> bool {
    true
}

/// Payload to update an existing post.
#[derive(Debug, Deserialize)]
pub struct UpdateSitePostRequest {
    /// Update title.
    pub title: Option<String>,
    /// Update slug.
    pub slug: Option<String>,
    /// Update excerpt.
    pub excerpt: Option<String>,
    /// Update markdown content.
    pub content_markdown: Option<String>,
    /// Update publication status.
    pub is_published: Option<bool>,
    /// Update comment status.
    pub allow_comments: Option<bool>,
    /// Update publish date (Double Option to clear).
    pub published_at: Option<Option<String>>,
    /// Update sort order.
    pub order_index: Option<i64>,
}

/// Item in the navigation menu.
#[derive(Debug, Serialize, Deserialize)]
pub struct NavigationItemResponse {
    /// Page ID.
    pub id: String,
    /// Page slug.
    pub slug: String,
    /// Display label.
    pub label: String,
    /// Sort order.
    pub order_index: i64,
}

/// Full navigation structure.
#[derive(Debug, Serialize, Deserialize)]
pub struct NavigationResponse {
    /// List of navigation link items.
    pub items: Vec<NavigationItemResponse>,
}
