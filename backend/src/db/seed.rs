use serde_json::json;
use sqlx::{Sqlite, Transaction};

pub async fn seed_site_content_tx(tx: &mut Transaction<'_, Sqlite>) -> Result<(), sqlx::Error> {
    for (section, content) in default_site_content() {
        // Step 1: Check if this content section already exists (Idempotency)
        let exists: Option<(String,)> =
            sqlx::query_as("SELECT section FROM site_content WHERE section = ?")
                .bind(section)
                .fetch_optional(&mut **tx)
                .await?;

        if exists.is_some() {
            continue;
        }

        // Step 2: Persist the default JSON content
        sqlx::query("INSERT INTO site_content (section, content_json) VALUES (?, ?)")
            .bind(section)
            .bind(content.to_string())
            .execute(&mut **tx)
            .await?;
    }

    Ok(())
}

fn default_site_content() -> Vec<(&'static str, serde_json::Value)> {
    vec![
        (
            "hero",
            json!({
                "badgeText": "Persönlicher Blog",
                "title": {
                    "line1": "Gedanken, Projekte",
                    "line2": "& Dinge dazwischen"
                },
                "subtitle": "Persönliche Notizen über Technik, Ideen und alles, was mich beschäftigt.",
                "subline": "Ausprobiert, durchdacht und ehrlich aufgeschrieben.",
                "primaryCta": {
                    "label": "Beiträge lesen",
                    "target": { "type": "section", "value": "stories" }
                },
                "secondaryCta": {
                    "label": "Über diesen Blog",
                    "target": { "type": "section", "value": "about" }
                }
            }),
        ),
        (
            "site_meta",
            json!({
                "title": "minos – Persönlicher Blog",
                "description": "Persönliche Notizen über Technik, Projekte, Ideen und alles dazwischen."
            }),
        ),
        (
            "about",
            json!({
                "eyebrow": "Warum ich schreibe",
                "lead": "Ich schreibe, um Dinge wirklich zu verstehen – und um meine Gedanken nicht zu verlieren.",
                "paragraphs": [
                    concat!(
                        "Dieser Blog ist mein digitales Notizbuch. Ich teile, was ich lerne, ",
                        "woran ich arbeite und welche Fragen mich gerade begleiten."
                    ),
                    concat!(
                        "Die Themen dürfen wechseln. Was bleibt, ist eine persönliche Perspektive, ",
                        "ehrliche Neugier und der Wunsch, Gedanken sauber zu Ende zu denken."
                    )
                ]
            }),
        ),
        (
            "cta_section",
            json!({
                "title": "Neue Notizen per Mail",
                "description": "Ich melde mich, wenn es einen neuen Gedanken oder Beitrag zu teilen gibt."
            }),
        ),
        (
            "header",
            json!({
                "brand": {
                    "name": "minos",
                    "tagline": "Persönlicher Blog",
                    "icon": "Terminal"
                },
                "navItems": [
                    { "id": "stories", "label": "Beiträge", "type": "section", "value": "stories" },
                    { "id": "about", "label": "Über diesen Blog", "type": "section", "value": "about" }
                ],
                "cta": {
                    "guestLabel": "Login",
                    "authLabel": "Admin",
                    "icon": "Lock"
                }
            }),
        ),
        (
            "footer",
            json!({
                "brand": {
                    "title": "minos",
                    "description": "Persönliche Notizen über Technik, Projekte, Ideen und alles dazwischen.",
                    "icon": "Terminal"
                },
                "quickLinks": [
                    { "label": "Beiträge", "target": { "type": "section", "value": "stories" } },
                    { "label": "Über diesen Blog", "target": { "type": "section", "value": "about" } }
                ],
                "contactLinks": [],
                "bottom": {
                    "copyright": "© {year} minos.",
                    "signature": "Persönlich notiert"
                }
            }),
        ),
    ]
}

pub async fn insert_default_tutorials_tx(
    tx: &mut Transaction<'_, Sqlite>,
) -> Result<(), sqlx::Error> {
    let tutorials = vec![
        (
            "1",
            "Grundlegende Befehle",
            "Lerne die wichtigsten Linux-Befehle für die tägliche Arbeit im Terminal.",
            "Terminal",
            "from-blue-500 to-cyan-500",
            vec![
                "ls", "cd", "pwd", "mkdir", "rm", "cp", "mv", "cat", "grep", "find", "chmod",
                "chown",
            ],
        ),
        (
            "2",
            "Dateisystem & Navigation",
            "Verstehe die Linux-Dateistruktur und navigiere effizient durch Verzeichnisse.",
            "FolderTree",
            "from-green-500 to-emerald-500",
            vec![
                "Verzeichnisstruktur",
                "Absolute vs. Relative Pfade",
                "Symlinks",
                "Mount Points",
            ],
        ),
        (
            "3",
            "Text-Editoren",
            "Beherrsche vim, nano und andere Editoren für die Arbeit in der Kommandozeile.",
            "FileText",
            "from-purple-500 to-pink-500",
            vec!["vim Basics", "nano Befehle", "sed & awk", "Regex Patterns"],
        ),
        (
            "4",
            "Prozessverwaltung",
            "Verwalte und überwache Prozesse effektiv in deinem Linux-System.",
            "Settings",
            "from-orange-500 to-red-500",
            vec![
                "ps",
                "top",
                "htop",
                "kill",
                "pkill",
                "Background Jobs",
                "systemctl",
            ],
        ),
        (
            "5",
            "Berechtigungen & Sicherheit",
            "Verstehe Benutzerrechte, Gruppen und Sicherheitskonzepte.",
            "Shield",
            "from-indigo-500 to-blue-500",
            vec!["User & Groups", "chmod & chown", "sudo & su", "SSH & Keys"],
        ),
        (
            "6",
            "Netzwerk-Grundlagen",
            "Konfiguriere Netzwerke und nutze wichtige Netzwerk-Tools.",
            "Network",
            "from-teal-500 to-green-500",
            vec![
                "ip & ifconfig",
                "ping",
                "traceroute",
                "netstat",
                "ss",
                "curl & wget",
            ],
        ),
        (
            "7",
            "Bash Scripting",
            "Automatisiere Aufgaben mit Shell-Scripts und Bash-Programmierung.",
            "Database",
            "from-yellow-500 to-orange-500",
            vec![
                "Variables & Loops",
                "If-Statements",
                "Functions",
                "Cron Jobs",
            ],
        ),
        (
            "8",
            "System Administration",
            "Erweiterte Admin-Aufgaben und Systemwartung.",
            "Server",
            "from-red-500 to-pink-500",
            vec![
                "Package Manager",
                "Logs & Monitoring",
                "Backup & Recovery",
                "Performance Tuning",
            ],
        ),
    ];

    for (id, title, description, icon, color, topics) in tutorials {
        let topics_vec: Vec<String> = topics.into_iter().map(|topic| topic.to_string()).collect();

        if sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM tutorials WHERE id = ?")
            .bind(id)
            .fetch_one(&mut **tx)
            .await?
            > 0
        {
            continue;
        }

        if let Err(err) = crate::handlers::tutorials::validate_icon(icon) {
            tracing::warn!(
                "Skipping default tutorial '{}' due to invalid icon: {}",
                id,
                err
            );
            continue;
        }

        if let Err(err) = crate::handlers::tutorials::validate_color(color) {
            tracing::warn!(
                "Skipping default tutorial '{}' due to invalid color: {}",
                id,
                err
            );
            continue;
        }

        let topics_json = serde_json::to_string(&topics_vec).map_err(|e| {
            sqlx::Error::Protocol(format!(
                "Failed to serialize topics for default tutorial '{}': {}",
                id, e
            ))
        })?;

        sqlx::query(concat!(
            "INSERT INTO tutorials ",
            "(id, title, description, icon, color, topics, content, version) ",
            "VALUES (?, ?, ?, ?, ?, ?, ?, 1)"
        ))
        .bind(id)
        .bind(title)
        .bind(description)
        .bind(icon)
        .bind(color)
        .bind(topics_json)
        .bind("")
        .execute(&mut **tx)
        .await?;

        crate::repositories::tutorials::replace_tutorial_topics_tx(tx, id, &topics_vec).await?;
    }

    Ok(())
}
