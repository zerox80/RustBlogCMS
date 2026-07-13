use super::*;

/// Validates a tutorial ID for length and character safety.
/// Used to prevent path injection and ensure URL compatibility.
pub(crate) fn validate_tutorial_id(id: &str) -> Result<(), String> {
    // Check length bounds to prevent buffer overflow or DoS attacks
    if id.is_empty() || id.len() > 100 {
        return Err("Invalid tutorial ID (must be 1-100 characters)".to_string());
    }

    // Ensure only safe characters for database and URL usage
    if !id
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.')
    {
        return Err(
            "Tutorial ID contains invalid characters (allowed: alphanumeric, -, _, .)".to_string(),
        );
    }
    Ok(())
}

/// Validates the core text content of a tutorial.
pub(super) fn validate_tutorial_data(
    title: &str,
    description: &str,
    content: &str,
) -> Result<(), String> {
    // Title validation
    let title_trimmed = title.trim();
    if title_trimmed.is_empty() {
        return Err("Title cannot be empty".to_string());
    }
    if title_trimmed.len() > 200 {
        return Err("Title too long (max 200 characters)".to_string());
    }

    // Description validation
    let description_trimmed = description.trim();
    if description_trimmed.is_empty() {
        return Err("Description cannot be empty".to_string());
    }
    if description_trimmed.len() > 1000 {
        return Err("Description too long (max 1000 characters)".to_string());
    }

    // Markdown content validation
    let content_trimmed = content.trim();
    if content_trimmed.is_empty() {
        return Err("Content cannot be empty".to_string());
    }
    if content_trimmed.len() > 100_000 {
        return Err("Content too long (max 100,000 characters)".to_string());
    }
    Ok(())
}

/// Validates that the provided icon name is within the allowed Lucide whitelist.
pub(crate) fn validate_icon(icon: &str) -> Result<(), String> {
    /// Whitelist of Lucide icon identifiers used in the frontend
    const ALLOWED_ICONS: &[&str] = &[
        "Terminal",   // Command line and shell tutorials
        "FolderTree", // File system and directory tutorials
        "FileText",   // Text editing and file manipulation
        "Settings",   // System configuration and settings
        "Shield",     // Security and permissions
        "Network",    // Networking and connectivity
        "Database",   // Database and data management
        "Server",     // Server administration and services
    ];

    if ALLOWED_ICONS.contains(&icon) {
        Ok(())
    } else {
        Err(format!(
            "Invalid icon '{}'. Must be one of: {:?}",
            icon, ALLOWED_ICONS
        ))
    }
}

/// Validates a Tailwind CSS gradient string.
/// Ensures the format 'from-COLOR [via-COLOR] to-COLOR' is followed.
pub(crate) fn validate_color(color: &str) -> Result<(), String> {
    const MAX_SEGMENT_LEN: usize = 32;

    /// Checks if a single tailwind class segment is valid (e.g. 'from-blue-500')
    fn validate_segment(segment: &str, prefix: &str) -> bool {
        // Handle responsive modifiers (e.g., dark:from-..., md:hover:to-...)
        // We look at the last part after ':' or the whole string if no ':'
        let base_class = segment.split(':').next_back().unwrap_or(segment);

        if !base_class.starts_with(prefix) {
            return false;
        }
        let suffix = &base_class[prefix.len()..];
        !suffix.is_empty()
            && suffix.len() <= MAX_SEGMENT_LEN
            && suffix
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-')
    }

    let segments: Vec<&str> = color.split_whitespace().collect();
    // Allow more complex gradients but ensure we have at least from and to
    // Typically 2 or 3 parts: from-... [via-...] to-...
    // But could be more with responsive? No, typically "from-X to-Y" is the base structure.
    // We stick to 2 or 3 segments for simplicity of storage/validation as per original design.

    // Gradients must have 2 (from/to) or 3 (from/via/to) segments
    if !(segments.len() == 2 || segments.len() == 3) {
        return Err(
            "Invalid color gradient. Expected Tailwind style 'from-… [via-…] to-…' format."
                .to_string(),
        );
    }

    // Note: The logic below assumes the order is always (modifiers:)?from -> (modifiers:)?via -> (modifiers:)?to
    // This might be too strict if user writes "to-red-500 from-blue-500", but Tailwind usually encourages ordered.
    // The original code enforced order segments[0]=from, segments[1]=via/to. We keep this but allow modifiers.

    // Validate 'from-' segment
    if !validate_segment(segments[0], "from-") {
        return Err(
            "Invalid color gradient: 'from-*' segment malformed, too long, or missing.".to_string(),
        );
    }

    if segments.len() == 3 {
        // Validation for middle segment - check if it is 'via-' or 'to-'?
        // Original code expected: 0=from, 1=via, 2=to.
        // Validate internal 'via-' segment
        if !validate_segment(segments[1], "via-") {
            return Err(
                "Invalid color gradient: Middle segment must be 'via-*' in a 3-part gradient."
                    .to_string(),
            );
        }
        // Validate 'to-' segment
        if !validate_segment(segments[2], "to-") {
            return Err("Invalid color gradient: Last segment must be 'to-*'.".to_string());
        }
    } else if !validate_segment(segments[1], "to-") {
        // Validate 'to-' segment for 2-part gradient
        return Err("Invalid color gradient: Last segment must be 'to-*'.".to_string());
    }

    Ok(())
}

/// Sanitizes a list of topics.
/// Normalizes to lowercase, removes duplicates, and trims long strings.
pub(super) fn sanitize_topics(topics: &[String]) -> Result<Vec<String>, String> {
    // SECURITY: Limit number of topics to prevent indexing DoS
    if topics.len() > 20 {
        return Err("Too many topics (max 20)".to_string());
    }

    let mut sanitized = Vec::with_capacity(topics.len());
    let mut seen = HashSet::new();

    for topic in topics {
        let trimmed = topic.trim();
        if trimmed.is_empty() {
            continue;
        }

        // ENFORCEMENT: Truncate excessively long topic names
        let limited: String = if trimmed.len() > 100 {
            trimmed.chars().take(100).collect()
        } else {
            trimmed.to_string()
        };

        // Normalize to lowercase for duplicate detection
        let canonical = limited
            .chars()
            .map(|c| c.to_ascii_lowercase())
            .collect::<String>();

        if !seen.insert(canonical) {
            return Err("Duplicate topics are not allowed".to_string());
        }

        sanitized.push(limited);
    }

    // Requirements
    if sanitized.is_empty() {
        return Err("At least one topic is required".to_string());
    }

    Ok(sanitized)
}
