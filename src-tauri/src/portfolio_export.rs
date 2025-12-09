//! Portfolio export and generation functionality

use crate::db::get_connection;
use crate::errors::CareerBenchError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct PortfolioItem {
    pub id: Option<i64>,
    pub title: String,
    pub url: Option<String>,
    pub description: Option<String>,
    pub role: Option<String>,
    pub tech_stack: Option<String>,
    pub highlighted: bool,
}

/// Export portfolio as HTML
pub fn export_portfolio_html(
    portfolio_items: &[PortfolioItem],
    include_highlighted_only: bool,
) -> String {
    let items = if include_highlighted_only {
        portfolio_items.iter().filter(|item| item.highlighted).collect::<Vec<_>>()
    } else {
        portfolio_items.iter().collect::<Vec<_>>()
    };

    let mut html = String::from(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Portfolio</title>
    <style>
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, sans-serif;
            line-height: 1.6;
            color: #333;
            max-width: 900px;
            margin: 0 auto;
            padding: 2rem;
            background-color: #f9fafb;
        }
        h1 {
            color: #1f2937;
            border-bottom: 3px solid #6366f1;
            padding-bottom: 0.5rem;
            margin-bottom: 2rem;
        }
        .portfolio-item {
            background: white;
            border-radius: 8px;
            padding: 1.5rem;
            margin-bottom: 1.5rem;
            box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
        }
        .portfolio-item.highlighted {
            border-left: 4px solid #6366f1;
        }
        .portfolio-item h2 {
            margin-top: 0;
            color: #1f2937;
        }
        .portfolio-item h2 .highlight-badge {
            color: #f59e0b;
            margin-left: 0.5rem;
        }
        .portfolio-meta {
            color: #6b7280;
            font-size: 0.9rem;
            margin: 0.5rem 0;
        }
        .portfolio-meta a {
            color: #6366f1;
            text-decoration: none;
        }
        .portfolio-meta a:hover {
            text-decoration: underline;
        }
        .tech-stack {
            display: flex;
            flex-wrap: wrap;
            gap: 0.5rem;
            margin-top: 1rem;
        }
        .tech-tag {
            background-color: #ede9fe;
            color: #6366f1;
            padding: 0.25rem 0.75rem;
            border-radius: 9999px;
            font-size: 0.875rem;
        }
        .portfolio-description {
            margin-top: 1rem;
            white-space: pre-wrap;
        }
        @media print {
            body {
                background-color: white;
            }
            .portfolio-item {
                page-break-inside: avoid;
            }
        }
    </style>
</head>
<body>
    <h1>Portfolio</h1>
"#,
    );

    for item in items {
        html.push_str("    <div class=\"portfolio-item");
        if item.highlighted {
            html.push_str(" highlighted");
        }
        html.push_str("\">\n");
        html.push_str(&format!("        <h2>{}", item.title));
        if item.highlighted {
            html.push_str(" <span class=\"highlight-badge\">★</span>");
        }
        html.push_str("</h2>\n");

        if let Some(url) = &item.url {
            html.push_str(&format!(
                "        <p class=\"portfolio-meta\"><a href=\"{}\" target=\"_blank\">{}</a></p>\n",
                url, url
            ));
        }

        if let Some(role) = &item.role {
            html.push_str(&format!("        <p class=\"portfolio-meta\">{}</p>\n", role));
        }

        if let Some(description) = &item.description {
            html.push_str("        <div class=\"portfolio-description\">\n");
            html.push_str(&html_escape(description));
            html.push_str("        </div>\n");
        }

        if let Some(tech_stack) = &item.tech_stack {
            html.push_str("        <div class=\"tech-stack\">\n");
            for tech in tech_stack.split(',') {
                let tech = tech.trim();
                if !tech.is_empty() {
                    html.push_str(&format!(
                        "            <span class=\"tech-tag\">{}</span>\n",
                        html_escape(tech)
                    ));
                }
            }
            html.push_str("        </div>\n");
        }

        html.push_str("    </div>\n");
    }

    html.push_str(
        r#"</body>
</html>"#,
    );

    html
}

/// Export portfolio as Markdown
pub fn export_portfolio_markdown(
    portfolio_items: &[PortfolioItem],
    include_highlighted_only: bool,
) -> String {
    let items = if include_highlighted_only {
        portfolio_items.iter().filter(|item| item.highlighted).collect::<Vec<_>>()
    } else {
        portfolio_items.iter().collect::<Vec<_>>()
    };

    let mut markdown = String::from("# Portfolio\n\n");

    for item in items {
        markdown.push_str("## ");
        markdown.push_str(&item.title);
        if item.highlighted {
            markdown.push_str(" ⭐");
        }
        markdown.push_str("\n\n");

        if let Some(url) = &item.url {
            markdown.push_str(&format!("**URL:** [{}]({})\n\n", url, url));
        }

        if let Some(role) = &item.role {
            markdown.push_str(&format!("**Role:** {}\n\n", role));
        }

        if let Some(description) = &item.description {
            markdown.push_str(&format!("{}\n\n", description));
        }

        if let Some(tech_stack) = &item.tech_stack {
            markdown.push_str("**Tech Stack:** ");
            let tech_list: Vec<&str> = tech_stack
                .split(',')
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .collect();
            markdown.push_str(&tech_list.join(", "));
            markdown.push_str("\n\n");
        }

        markdown.push_str("---\n\n");
    }

    markdown
}

/// Export portfolio as plain text
pub fn export_portfolio_text(
    portfolio_items: &[PortfolioItem],
    include_highlighted_only: bool,
) -> String {
    let items = if include_highlighted_only {
        portfolio_items.iter().filter(|item| item.highlighted).collect::<Vec<_>>()
    } else {
        portfolio_items.iter().collect::<Vec<_>>()
    };

    let mut text = String::from("PORTFOLIO\n");
    text.push_str(&"=".repeat(50));
    text.push_str("\n\n");

    for (idx, item) in items.iter().enumerate() {
        text.push_str(&format!("{}. {}\n", idx + 1, item.title));
        if item.highlighted {
            text.push_str("   [HIGHLIGHTED]\n");
        }
        text.push_str("\n");

        if let Some(url) = &item.url {
            text.push_str(&format!("   URL: {}\n", url));
        }

        if let Some(role) = &item.role {
            text.push_str(&format!("   Role: {}\n", role));
        }

        if let Some(description) = &item.description {
            text.push_str(&format!("   Description:\n   {}\n", description));
        }

        if let Some(tech_stack) = &item.tech_stack {
            text.push_str("   Tech Stack: ");
            let tech_list: Vec<&str> = tech_stack
                .split(',')
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .collect();
            text.push_str(&tech_list.join(", "));
            text.push_str("\n");
        }

        text.push_str("\n");
    }

    text
}

/// Helper function to escape HTML special characters
fn html_escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

/// Get portfolio items linked to an application
pub fn get_portfolio_for_application(
    application_id: i64,
) -> Result<Vec<PortfolioItem>, CareerBenchError> {
    let conn = get_connection()?;

    let mut stmt = conn.prepare(
        "SELECT p.id, p.title, p.url, p.description, p.role, p.tech_stack, p.highlighted
         FROM portfolio_items p
         INNER JOIN application_portfolio_links l ON p.id = l.portfolio_item_id
         WHERE l.application_id = ?
         ORDER BY p.highlighted DESC, p.id DESC"
    )?;

    let rows = stmt.query_map([application_id], |row| {
        Ok(PortfolioItem {
            id: row.get(0)?,
            title: row.get(1)?,
            url: row.get(2)?,
            description: row.get(3)?,
            role: row.get(4)?,
            tech_stack: row.get(5)?,
            highlighted: row.get::<_, i64>(6)? != 0,
        })
    })?;

    let mut items = Vec::new();
    for row_result in rows {
        items.push(row_result?);
    }

    Ok(items)
}

/// Link portfolio items to an application
pub fn link_portfolio_to_application(
    application_id: i64,
    portfolio_item_ids: &[i64],
) -> Result<(), CareerBenchError> {
    let conn = get_connection()?;

    // Remove existing links
    conn.execute(
        "DELETE FROM application_portfolio_links WHERE application_id = ?",
        [application_id],
    )?;

    // Add new links
    for portfolio_id in portfolio_item_ids {
        conn.execute(
            "INSERT OR IGNORE INTO application_portfolio_links (application_id, portfolio_item_id, created_at)
             VALUES (?, ?, datetime('now'))",
            [application_id, *portfolio_id],
        )?;
    }

    Ok(())
}

/// Get applications linked to a portfolio item
pub fn get_applications_for_portfolio(
    portfolio_item_id: i64,
) -> Result<Vec<i64>, CareerBenchError> {
    let conn = get_connection()?;

    let mut stmt = conn.prepare(
        "SELECT application_id FROM application_portfolio_links WHERE portfolio_item_id = ?"
    )?;

    let rows = stmt.query_map([portfolio_item_id], |row| {
        row.get(0)
    })?;

    let mut application_ids = Vec::new();
    for row_result in rows {
        application_ids.push(row_result?);
    }

    Ok(application_ids)
}
