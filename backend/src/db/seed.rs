use serde_json::json;
use sqlx::{Sqlite, Transaction};

pub async fn seed_site_content_tx(
    tx: &mut Transaction<'_, Sqlite>,
) -> Result<(), sqlx::Error> {
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
                "badgeText": "Professionelles Linux Training",
                "title": {
                    "line1": "Lerne Linux",
                    "line2": "von Grund auf"
                },
                "subtitle": "Dein umfassendes Tutorial für Linux – von den Basics bis zu Advanced-Techniken.",
                "subline": "Interaktiv, modern und praxisnah.",
                "primaryCta": {
                    "label": "Los geht's",
                    "target": { "type": "section", "value": "tutorials" }
                },
                "secondaryCta": {
                    "label": "Mehr erfahren",
                    "target": { "type": "section", "value": "tutorials" }
                },
                "features": [
                    {
                        "icon": "Book",
                        "title": "Schritt für Schritt",
                        "description": "Strukturiert lernen mit klaren Beispielen",
                        "color": "from-blue-500 to-cyan-500"
                    },
                    {
                        "icon": "Code",
                        "title": "Praktische Befehle",
                        "description": "Direkt anwendbare Kommandos",
                        "color": "from-purple-500 to-pink-500"
                    },
                    {
                        "icon": "Zap",
                        "title": "Modern & Aktuell",
                        "description": "Neueste Best Practices",
                        "color": "from-orange-500 to-red-500"
                    }
                ]
            }),
        ),
        (
            "tutorial_section",
            json!({
                "title": "Tutorial Inhalte",
                "description": "Umfassende Lernmodule für alle Erfahrungsstufen – vom Anfänger bis zum Profi",
                "heading": "Bereit anzufangen?",
                "ctaDescription": "Wähle ein Thema aus und starte deine Linux-Lernreise noch heute!",
                "ctaPrimary": {
                    "label": "Tutorial starten",
                    "target": { "type": "section", "value": "home" }
                },
                "ctaSecondary": {
                    "label": "Mehr erfahren",
                    "target": { "type": "section", "value": "home" }
                },
                "tutorialCardButton": "Zum Tutorial"
            }),
        ),
        (
            "site_meta",
            json!({
                "title": "Linux Tutorial - Lerne Linux Schritt für Schritt",
                "description": "Lerne Linux von Grund auf - Interaktiv, modern und praxisnah."
            }),
        ),
        (
            "header",
            json!({
                "brand": {
                    "name": "Linux Tutorial",
                    "tagline": "",
                    "icon": "Terminal"
                },
                "navItems": [
                    { "id": "home", "label": "Home", "type": "section" },
                    { "id": "grundlagen", "label": "Grundlagen", "type": "route", "path": "/grundlagen" },
                    { "id": "befehle", "label": "Befehle", "type": "section" },
                    { "id": "praxis", "label": "Praxis", "type": "section" },
                    { "id": "advanced", "label": "Advanced", "type": "section" }
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
                    "title": "Linux Tutorial",
                    "description": "Dein umfassendes Tutorial für Linux - von den Basics bis zu Advanced Techniken.",
                    "icon": "Terminal"
                },
                "quickLinks": [
                    { "label": "Grundlagen", "target": { "type": "section", "value": "grundlagen" } },
                    { "label": "Befehle", "target": { "type": "section", "value": "befehle" } },
                    { "label": "Praxis", "target": { "type": "section", "value": "praxis" } },
                    { "label": "Advanced", "target": { "type": "section", "value": "advanced" } }
                ],
                "contactLinks": [
                    { "label": "GitHub", "href": "https://github.com", "icon": "Github" },
                    { "label": "E-Mail", "href": "mailto:info@example.com", "icon": "Mail" }
                ],
                "bottom": {
                    "copyright": "© {year} Linux Tutorial. Alle Rechte vorbehalten.",
                    "signature": "Gemacht mit Herz für die Linux Community"
                }
            }),
        ),
        (
            "grundlagen_page",
            json!({
                "hero": {
                    "badge": "Grundlagenkurs",
                    "title": "Starte deine Linux-Reise mit einem starken Fundament",
                    "description": "In diesem Grundlagenbereich begleiten wir dich von den allerersten Schritten im Terminal bis hin zu sicheren Arbeitsabläufen. Nach diesem Kurs bewegst du dich selbstbewusst in der Linux-Welt.",
                    "icon": "BookOpen"
                },
                "highlights": [
                    {
                        "icon": "BookOpen",
                        "title": "Terminal Basics verstehen",
                        "description": "Lerne die wichtigsten Shell-Befehle, arbeite sicher mit Dateien und nutze Pipes, um Aufgaben zu automatisieren."
                    },
                    {
                        "icon": "Compass",
                        "title": "Linux-Philosophie kennenlernen",
                        "description": "Verstehe das Zusammenspiel von Kernel, Distribution, Paketverwaltung und warum Linux so flexibel einsetzbar ist."
                    },
                    {
                        "icon": "Layers",
                        "title": "Praxisnahe Übungen",
                        "description": "Setze das Erlernte direkt in kleinen Projekten um – von der Benutzerverwaltung bis zum Einrichten eines Webservers."
                    },
                    {
                        "icon": "ShieldCheck",
                        "title": "Sicher arbeiten",
                        "description": "Erhalte Best Practices für Benutzerrechte, sudo, SSH und weitere Sicherheitsmechanismen."
                    }
                ],
                "modules": {
                    "title": "Module im Grundlagenkurs",
                    "description": "Unsere Tutorials bauen logisch aufeinander auf. Jedes Modul enthält praxisnahe Beispiele, Schritt-für-Schritt Anleitungen und kleine Wissenschecks, damit du deinen Fortschritt direkt sehen kannst.",
                    "items": [
                        "Einstieg in die Shell: Navigation, grundlegende Befehle, Dateiverwaltung",
                        "Linux-Systemaufbau: Kernel, Distributionen, Paketmanager verstehen und nutzen",
                        "Benutzer & Rechte: Arbeiten mit sudo, Gruppen und Dateiberechtigungen",
                        "Wichtige Tools: SSH, einfache Netzwerkanalyse und nützliche Utilities für den Alltag"
                    ],
                    "summary": [
                        "Über 40 praxisnahe Lessons",
                        "Schritt-für-Schritt Guides mit Screenshots & Code-Beispielen",
                        "Übungen und Checklisten zum Selbstüberprüfen"
                    ]
                },
                "cta": {
                    "title": "Bereit für den nächsten Schritt?",
                    "description": "Wechsel zur Startseite und wähle das Modul, das am besten zu dir passt, oder tauche direkt in die Praxis- und Advanced-Themen ein, sobald du die Grundlagen sicher beherrschst.",
                    "primary": { "label": "Zur Startseite", "href": "/" },
                    "secondary": { "label": "Tutorials verwalten", "href": "/admin" }
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
            sqlx::Error::Protocol(
                format!(
                    "Failed to serialize topics for default tutorial '{}': {}",
                    id, e
                )
                .into(),
            )
        })?;

        sqlx::query(
            "INSERT INTO tutorials (id, title, description, icon, color, topics, content, version) VALUES (?, ?, ?, ?, ?, ?, ?, 1)"
        )
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
