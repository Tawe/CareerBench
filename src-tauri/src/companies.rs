//! Companies module for managing company information and linking jobs/applications

use crate::db::get_connection;
use crate::errors::CareerBenchError;
use serde::{Deserialize, Serialize};
use scraper::{Html, Selector};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Company {
    pub id: Option<i64>,
    pub name: String,
    pub website: Option<String>,
    pub industry: Option<String>,
    pub company_size: Option<String>,
    pub location: Option<String>,
    pub description: Option<String>,
    pub mission: Option<String>,
    pub vision: Option<String>,
    pub values: Option<String>,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompanyWithStats {
    pub id: Option<i64>,
    pub name: String,
    pub website: Option<String>,
    pub industry: Option<String>,
    pub company_size: Option<String>,
    pub location: Option<String>,
    pub description: Option<String>,
    pub mission: Option<String>,
    pub vision: Option<String>,
    pub values: Option<String>,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub job_count: i64,
    pub application_count: i64,
}

/// Create a new company
pub fn create_company(
    name: String,
    website: Option<String>,
    industry: Option<String>,
    company_size: Option<String>,
    location: Option<String>,
    description: Option<String>,
    mission: Option<String>,
    vision: Option<String>,
    values: Option<String>,
    notes: Option<String>,
) -> Result<i64, CareerBenchError> {
    let conn = get_connection()?;

    conn.execute(
        "INSERT INTO companies 
         (name, website, industry, company_size, location, description, mission, vision, \"values\", notes, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, datetime('now'), datetime('now'))",
        rusqlite::params![
            name,
            website,
            industry,
            company_size,
            location,
            description,
            mission,
            vision,
            values,
            notes
        ],
    )?;

    Ok(conn.last_insert_rowid())
}

/// Get all companies
pub fn get_companies(
    search_query: Option<&str>,
) -> Result<Vec<Company>, CareerBenchError> {
    let conn = get_connection()?;

    let mut query = "SELECT id, name, website, industry, company_size, location, description, mission, vision, \"values\", notes, created_at, updated_at
                     FROM companies".to_string();
    let mut params = Vec::new();

    if let Some(search) = search_query {
        query.push_str(" WHERE (name LIKE ? OR industry LIKE ? OR location LIKE ?)");
        let search_pattern = format!("%{}%", search);
        params.push(search_pattern.clone());
        params.push(search_pattern.clone());
        params.push(search_pattern);
    }

    query.push_str(" ORDER BY name ASC");

    let mut stmt = conn.prepare(&query)?;
    let rows = stmt.query_map(rusqlite::params_from_iter(params.iter()), |row| {
        Ok(Company {
            id: row.get(0)?,
            name: row.get(1)?,
            website: row.get(2)?,
            industry: row.get(3)?,
            company_size: row.get(4)?,
            location: row.get(5)?,
            description: row.get(6)?,
            mission: row.get(7)?,
            vision: row.get(8)?,
            values: row.get(9)?,
            notes: row.get(10)?,
            created_at: row.get(11)?,
            updated_at: row.get(12)?,
        })
    })?;

    let mut companies = Vec::new();
    for row_result in rows {
        companies.push(row_result?);
    }

    Ok(companies)
}

/// Get companies with statistics (job count, application count)
pub fn get_companies_with_stats(
    search_query: Option<&str>,
) -> Result<Vec<CompanyWithStats>, CareerBenchError> {
    let conn = get_connection()?;

    let mut query = "SELECT 
        c.id, c.name, c.website, c.industry, c.company_size, c.location, c.description, c.mission, c.vision, c.\"values\", c.notes, 
        c.created_at, c.updated_at,
        COALESCE((SELECT COUNT(*) FROM jobs WHERE company_id = c.id), 0) as job_count,
        COALESCE((SELECT COUNT(*) FROM applications WHERE company_id = c.id), 0) as application_count
        FROM companies c
    ".to_string();
    let mut params = Vec::new();

    if let Some(search) = search_query {
        query.push_str(" WHERE (c.name LIKE ? OR c.industry LIKE ? OR c.location LIKE ?)");
        let search_pattern = format!("%{}%", search);
        params.push(search_pattern.clone());
        params.push(search_pattern.clone());
        params.push(search_pattern);
    }

    query.push_str(" ORDER BY c.name ASC");

    let mut stmt = conn.prepare(&query)?;
    let rows = stmt.query_map(rusqlite::params_from_iter(params.iter()), |row| {
        Ok(CompanyWithStats {
            id: row.get(0)?,
            name: row.get(1)?,
            website: row.get(2)?,
            industry: row.get(3)?,
            company_size: row.get(4)?,
            location: row.get(5)?,
            description: row.get(6)?,
            mission: row.get(7)?,
            vision: row.get(8)?,
            values: row.get(9)?,
            notes: row.get(10)?,
            created_at: row.get(11)?,
            updated_at: row.get(12)?,
            job_count: row.get(13)?,
            application_count: row.get(14)?,
        })
    })?;

    let mut companies = Vec::new();
    for row_result in rows {
        companies.push(row_result?);
    }

    Ok(companies)
}

/// Get a single company by ID
pub fn get_company(company_id: i64) -> Result<Company, CareerBenchError> {
    let conn = get_connection()?;

    let company = conn.query_row(
        "SELECT id, name, website, industry, company_size, location, description, mission, vision, \"values\", notes, created_at, updated_at
         FROM companies WHERE id = ?",
        [company_id],
        |row| {
            Ok(Company {
                id: row.get(0)?,
                name: row.get(1)?,
                website: row.get(2)?,
                industry: row.get(3)?,
                company_size: row.get(4)?,
                location: row.get(5)?,
                description: row.get(6)?,
                mission: row.get(7)?,
                vision: row.get(8)?,
                values: row.get(9)?,
                notes: row.get(10)?,
                created_at: row.get(11)?,
                updated_at: row.get(12)?,
            })
        },
    )?;

    Ok(company)
}

/// Update a company
pub fn update_company(
    company_id: i64,
    name: Option<String>,
    website: Option<String>,
    industry: Option<String>,
    company_size: Option<String>,
    location: Option<String>,
    description: Option<String>,
    mission: Option<String>,
    vision: Option<String>,
    values: Option<String>,
    notes: Option<String>,
) -> Result<(), CareerBenchError> {
    let conn = get_connection()?;

    let mut updates = Vec::new();
    let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

    if let Some(n) = name {
        updates.push("name = ?");
        params.push(Box::new(n));
    }
    if let Some(w) = website {
        updates.push("website = ?");
        params.push(Box::new(w));
    }
    if let Some(i) = industry {
        updates.push("industry = ?");
        params.push(Box::new(i));
    }
    if let Some(cs) = company_size {
        updates.push("company_size = ?");
        params.push(Box::new(cs));
    }
    if let Some(l) = location {
        updates.push("location = ?");
        params.push(Box::new(l));
    }
    if let Some(d) = description {
        updates.push("description = ?");
        params.push(Box::new(d));
    }
    if let Some(m) = mission {
        updates.push("mission = ?");
        params.push(Box::new(m));
    }
    if let Some(v) = vision {
        updates.push("vision = ?");
        params.push(Box::new(v));
    }
    if let Some(v) = values {
        updates.push("\"values\" = ?");
        params.push(Box::new(v));
    }
    if let Some(n) = notes {
        updates.push("notes = ?");
        params.push(Box::new(n));
    }

    if updates.is_empty() {
        return Ok(()); // Nothing to update
    }

    updates.push("updated_at = datetime('now')");
    params.push(Box::new(company_id));

    let query = format!(
        "UPDATE companies SET {} WHERE id = ?",
        updates.join(", ")
    );

    let mut stmt = conn.prepare(&query)?;
    stmt.execute(rusqlite::params_from_iter(params.iter()))?;

    Ok(())
}

/// Delete a company
/// If force is true, automatically unlinks all jobs and applications before deleting
pub fn delete_company(company_id: i64, force: bool) -> Result<(), CareerBenchError> {
    let conn = get_connection()?;

    // Check if company is referenced by jobs or applications
    let job_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM jobs WHERE company_id = ?",
        [company_id],
        |row| row.get(0),
    )?;

    let app_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM applications WHERE company_id = ?",
        [company_id],
        |row| row.get(0),
    )?;

    if job_count > 0 || app_count > 0 {
        if force {
            // Force delete: unlink all references first
            log::info!("[companies] Force deleting company {}: unlinking {} job(s) and {} application(s)", 
                company_id, job_count, app_count);
            conn.execute("UPDATE jobs SET company_id = NULL WHERE company_id = ?", [company_id])?;
            conn.execute("UPDATE applications SET company_id = NULL WHERE company_id = ?", [company_id])?;
        } else {
            return Err(CareerBenchError::Validation(
                crate::errors::ValidationError::BusinessRule(format!(
                    "Cannot delete company: it is referenced by {} job(s) and {} application(s). Please unlink them first or use force delete.",
                    job_count, app_count
                ))
            ));
        }
    }

    conn.execute("DELETE FROM companies WHERE id = ?", [company_id])?;
    log::info!("[companies] Successfully deleted company {}", company_id);

    Ok(())
}

/// Link a job to a company
pub fn link_job_to_company(job_id: i64, company_id: i64) -> Result<(), CareerBenchError> {
    let conn = get_connection()?;

    // Verify company exists
    let _: i64 = conn.query_row(
        "SELECT id FROM companies WHERE id = ?",
        [company_id],
        |row| row.get(0),
    )?;

    conn.execute(
        "UPDATE jobs SET company_id = ? WHERE id = ?",
        [company_id, job_id],
    )?;

    Ok(())
}

/// Link an application to a company
pub fn link_application_to_company(application_id: i64, company_id: i64) -> Result<(), CareerBenchError> {
    let conn = get_connection()?;

    // Verify company exists
    let _: i64 = conn.query_row(
        "SELECT id FROM companies WHERE id = ?",
        [company_id],
        |row| row.get(0),
    )?;

    conn.execute(
        "UPDATE applications SET company_id = ? WHERE id = ?",
        [company_id, application_id],
    )?;

    Ok(())
}

/// Unlink a job from its company
pub fn unlink_job_from_company(job_id: i64) -> Result<(), CareerBenchError> {
    let conn = get_connection()?;

    conn.execute(
        "UPDATE jobs SET company_id = NULL WHERE id = ?",
        [job_id],
    )?;

    Ok(())
}

/// Unlink an application from its company
pub fn unlink_application_from_company(application_id: i64) -> Result<(), CareerBenchError> {
    let conn = get_connection()?;

    conn.execute(
        "UPDATE applications SET company_id = NULL WHERE id = ?",
        [application_id],
    )?;

    Ok(())
}

/// Scraped company information from website
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScrapedCompanyInfo {
    pub name: Option<String>,
    pub website: Option<String>,
    pub industry: Option<String>,
    pub company_size: Option<String>,
    pub location: Option<String>,
    pub description: Option<String>,
    pub raw_content: String,
}

/// Fetch a single page and return its HTML content
async fn fetch_page_content(
    client: &reqwest::Client,
    url: &str,
) -> Result<(String, Option<String>, Option<String>), CareerBenchError> {
    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| CareerBenchError::FileSystem(crate::errors::FileSystemError::IoError(
            format!("Failed to fetch {}: {}", url, e)
        )))?;

    if !response.status().is_success() {
        return Err(CareerBenchError::FileSystem(crate::errors::FileSystemError::IoError(
            format!("HTTP error {} for {}", response.status(), url)
        )));
    }

    let html = response
        .text()
        .await
        .map_err(|e| CareerBenchError::FileSystem(crate::errors::FileSystemError::IoError(
            format!("Failed to read response from {}: {}", url, e)
        )))?;

    let document = Html::parse_document(&html);
    let name = extract_company_name(&document, url);
    let description = extract_description(&document);

    Ok((html, name, description))
}

/// Extract content from a parsed HTML document
fn extract_page_content(document: &Html) -> String {
    let mut raw_content = String::new();
    
    // First, try to find main content areas (main, article, or specific sections)
    let main_selectors = [
        "main",
        "article",
        "[role='main']",
        ".main-content",
        "#main-content",
        ".content",
        "#content",
        "section[class*='about' i]",
        "div[class*='about' i]",
        "section[id*='about' i]",
        "div[id*='about' i]",
        "[data-section='about']",
        ".company-info",
        "#company-info",
        ".careers-content",
        "#careers-content",
    ];
    
    let mut found_content = false;
    for selector_str in &main_selectors {
        if let Ok(selector) = Selector::parse(selector_str) {
            for element in document.select(&selector) {
                let text = element.text().collect::<String>();
                // Filter out very short text (likely navigation or UI elements)
                let cleaned_text = text.trim();
                if cleaned_text.len() > 200 { // Only use if substantial content
                    raw_content.push_str(cleaned_text);
                    raw_content.push_str("\n\n");
                    found_content = true;
                }
            }
        }
    }
    
    // If no specific sections found, extract from body but filter navigation and UI elements
    if !found_content {
        let body_selector = Selector::parse("body").ok();
        if let Some(selector) = body_selector {
            if let Some(body) = document.select(&selector).next() {
                // Try to find h1, h2, and paragraphs first (main content)
                let content_selectors = ["h1", "h2", "h3", "p", "li"];
                for tag in &content_selectors {
                    if let Ok(sel) = Selector::parse(tag) {
                        for element in body.select(&sel) {
                            let text = element.text().collect::<String>().trim().to_string();
                            // Lower threshold to 10 chars to catch more content
                            if text.len() > 10 && text.len() < 2000 { // Reasonable length
                                raw_content.push_str(&text);
                                raw_content.push_str("\n");
                            }
                        }
                    }
                }
                
                // If still not enough content, extract from all body elements (more lenient)
                if raw_content.len() < 200 {
                    for element in body.select(&Selector::parse("*").unwrap()) {
                        let tag_name = element.value().name();
                        // Skip navigation, scripts, styles, and common UI elements
                        if tag_name != "script" && tag_name != "style" && tag_name != "noscript" 
                            && tag_name != "nav" && tag_name != "header" && tag_name != "footer"
                            && tag_name != "button" && tag_name != "a" && tag_name != "svg" {
                            let text = element.text().collect::<String>();
                            let cleaned = text.trim();
                            // Lower threshold to 5 chars to catch more content
                            if !cleaned.is_empty() && cleaned.len() > 5 && cleaned.len() < 1000 {
                                raw_content.push_str(cleaned);
                                raw_content.push_str(" ");
                            }
                        }
                    }
                }
                
                // Last resort: get all text from body, filtering out very short fragments
                if raw_content.len() < 100 {
                    let all_text = body.text().collect::<Vec<_>>();
                    for text in all_text {
                        let cleaned = text.trim();
                        if cleaned.len() > 10 {
                            raw_content.push_str(cleaned);
                            raw_content.push_str(" ");
                        }
                    }
                }
            }
        }
    }
    
    raw_content
}

/// Scrape company information from a website URL
/// Tries multiple pages (main, /about, /careers) to get comprehensive information
pub async fn scrape_company_website(url: &str) -> Result<ScrapedCompanyInfo, CareerBenchError> {
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        reqwest::header::USER_AGENT,
        reqwest::header::HeaderValue::from_static(
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"
        ),
    );
    headers.insert(
        reqwest::header::ACCEPT,
        reqwest::header::HeaderValue::from_static(
            "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7"
        ),
    );
    headers.insert(
        reqwest::header::ACCEPT_LANGUAGE,
        reqwest::header::HeaderValue::from_static("en-US,en;q=0.9"),
    );
    headers.insert(
        reqwest::header::ACCEPT_ENCODING,
        reqwest::header::HeaderValue::from_static("gzip, deflate, br"),
    );
    headers.insert(
        reqwest::header::HeaderName::from_static("sec-fetch-site"),
        reqwest::header::HeaderValue::from_static("none"),
    );
    headers.insert(
        reqwest::header::HeaderName::from_static("sec-fetch-mode"),
        reqwest::header::HeaderValue::from_static("navigate"),
    );
    headers.insert(
        reqwest::header::HeaderName::from_static("sec-fetch-user"),
        reqwest::header::HeaderValue::from_static("?1"),
    );
    headers.insert(
        reqwest::header::HeaderName::from_static("sec-fetch-dest"),
        reqwest::header::HeaderValue::from_static("document"),
    );
    headers.insert(
        reqwest::header::HeaderName::from_static("upgrade-insecure-requests"),
        reqwest::header::HeaderValue::from_static("1"),
    );

    let client = reqwest::Client::builder()
        .default_headers(headers)
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| CareerBenchError::FileSystem(crate::errors::FileSystemError::IoError(
            format!("Failed to create HTTP client: {}", e)
        )))?;

    // Parse base URL to construct paths
    // Extract the domain and protocol (e.g., https://www.affirm.com from https://www.affirm.com/some/path)
    let base_url = if let Some(protocol_end) = url.find("://") {
        let after_protocol = &url[protocol_end + 3..];
        if let Some(first_slash) = after_protocol.find('/') {
            // Has a path, extract just the domain (protocol + domain, without the path)
            url[..protocol_end + 3 + first_slash].to_string()
        } else {
            // No path, URL is already the base (e.g., https://www.affirm.com)
            url.to_string()
        }
    } else {
        // No protocol found, use URL as-is (shouldn't happen with valid URLs)
        url.to_string()
    };
    
    // Remove trailing slash from base_url if present
    let base_url = base_url.trim_end_matches('/').to_string();
    
    log::info!("[companies] Base URL extracted: '{}' from '{}'", base_url, url);

    // Try multiple pages in order of priority
    // Prioritize /about and /careers pages as they have the best company information
    let pages_to_try = vec![
        format!("{}/about", base_url),
        format!("{}/about-us", base_url),
        format!("{}/about/", base_url),
        format!("{}/careers", base_url),
        format!("{}/careers/", base_url),
        format!("{}/company", base_url),
        url.to_string(), // Original URL as fallback
    ];
    
    log::info!("[companies] Will try {} pages, starting with: {}", pages_to_try.len(), pages_to_try[0]);

    let mut all_content = String::new();
    let mut found_name = None;
    let mut found_description = None;
    let mut successful_pages = 0;

    // Try each page and accumulate content
    for (idx, page_url) in pages_to_try.iter().enumerate() {
        log::info!("[companies] Attempting to fetch page {}/{}: {}", idx + 1, pages_to_try.len(), page_url);
        match fetch_page_content(&client, page_url).await {
            Ok((html, name, description)) => {
                log::info!("[companies] Successfully fetched {} ({} bytes)", page_url, html.len());
                let document = Html::parse_document(&html);
                let page_content = extract_page_content(&document);
                
                log::info!("[companies] Extracted {} chars of content from {}", page_content.len(), page_url);
                
                if !page_content.is_empty() {
                    all_content.push_str(&page_content);
                    all_content.push_str("\n\n---\n\n"); // Separator between pages
                    successful_pages += 1;
                    
                    // Keep first found name and description
                    if found_name.is_none() && name.is_some() {
                        found_name = name.clone();
                        log::info!("[companies] Found company name: {:?}", found_name);
                    }
                    if found_description.is_none() && description.is_some() {
                        found_description = description.clone();
                        log::info!("[companies] Found description: {:?}", found_description.as_ref().map(|d| &d[..d.len().min(100)]));
                    }
                    
                    // If we got good content from about/careers pages, log it
                    if page_url.contains("/about") || page_url.contains("/careers") || page_url.contains("/company") {
                        log::info!("[companies] ✓ Found good content from: {} ({} chars)", page_url, page_content.len());
                    }
                } else {
                    log::warn!("[companies] Page {} returned empty content", page_url);
                }
            }
            Err(e) => {
                log::warn!("[companies] ✗ Failed to fetch {}: {}", page_url, e);
                continue; // Try next page
            }
        }
    }

    if all_content.is_empty() {
        return Err(CareerBenchError::FileSystem(crate::errors::FileSystemError::IoError(
            "Failed to fetch any content from company website".to_string()
        )));
    }

    log::info!("[companies] Successfully scraped {} page(s), total content: {} chars", successful_pages, all_content.len());

    // Clean up the content (remove excessive whitespace and short lines)
    let raw_content = all_content
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty() && l.len() > 5) // Filter out very short lines
        .collect::<Vec<_>>()
        .join("\n");
    
    // Try to extract basic info from meta tags (use first page's document)
    let first_doc = if let Ok((html, _, _)) = fetch_page_content(&client, url).await {
        Html::parse_document(&html)
    } else {
        // Fallback: parse the combined content
        Html::parse_document(&all_content)
    };
    
    let info = ScrapedCompanyInfo {
        name: found_name.or_else(|| extract_company_name(&first_doc, url)),
        website: Some(url.to_string()),
        industry: None,
        company_size: None,
        location: None,
        description: found_description.or_else(|| extract_description(&first_doc)),
        raw_content: raw_content.chars().take(15000).collect(), // Increased limit since we have more content
    };

    Ok(info)
}

/// Extract company name from page
fn extract_company_name(document: &Html, url: &str) -> Option<String> {
    // Try meta tags first (most reliable for company name)
    let meta_selector = Selector::parse("meta[property='og:site_name'], meta[name='application-name']").ok();
    if let Some(selector) = meta_selector {
        for element in document.select(&selector) {
            if let Some(content) = element.value().attr("content") {
                let cleaned = content.trim();
                // Remove common taglines/suffixes
                let cleaned = cleaned
                    .split(" | ")
                    .next()
                    .or_else(|| cleaned.split(" - ").next())
                    .unwrap_or(cleaned)
                    .trim();
                if !cleaned.is_empty() && cleaned.len() < 100 {
                    return Some(cleaned.to_string());
                }
            }
        }
    }
    
    // Try og:title but clean it more aggressively
    let og_title_selector = Selector::parse("meta[property='og:title']").ok();
    if let Some(selector) = og_title_selector {
        for element in document.select(&selector) {
            if let Some(content) = element.value().attr("content") {
                // Remove taglines, descriptions, etc. - take first part before | or -
                let cleaned = content
                    .split(" | ")
                    .next()
                    .or_else(|| content.split(" - ").next())
                    .unwrap_or(content)
                    .trim();
                // If it's too long, it's probably a tagline, skip it
                if !cleaned.is_empty() && cleaned.len() < 50 {
                    return Some(cleaned.to_string());
                }
            }
        }
    }
    
    // Try title tag (but be more aggressive about cleaning)
    let title_selector = Selector::parse("title").ok();
    if let Some(selector) = title_selector {
        if let Some(title) = document.select(&selector).next() {
            let text = title.text().collect::<String>();
            // Remove common suffixes and take first part
            let cleaned = text
                .split(" | ")
                .next()
                .or_else(|| text.split(" - ").next())
                .unwrap_or(&text)
                .trim();
            // Only use if it's short (likely just the company name)
            if !cleaned.is_empty() && cleaned.len() < 50 {
                return Some(cleaned.to_string());
            }
        }
    }
    
    // Try h1 (but only if it's short)
    let h1_selector = Selector::parse("h1").ok();
    if let Some(selector) = h1_selector {
        if let Some(h1) = document.select(&selector).next() {
            let text = h1.text().collect::<String>().trim().to_string();
            // Only use if it's short and looks like a company name
            if !text.is_empty() && text.len() < 50 && !text.contains("|") && !text.contains(" - ") {
                return Some(text);
            }
        }
    }
    
    // Fallback: extract from domain name
    if let Some(domain) = url.split("://").nth(1).and_then(|s| s.split('/').next()) {
        let domain = domain.replace("www.", "");
        if let Some(name) = domain.split('.').next() {
            if name.len() > 2 {
                // Capitalize first letter
                let mut chars = name.chars();
                if let Some(first) = chars.next() {
                    return Some(first.to_uppercase().collect::<String>() + chars.as_str());
                }
            }
        }
    }
    
    None
}

/// Extract description from page
fn extract_description(document: &Html) -> Option<String> {
    // Try meta description
    let meta_selector = Selector::parse("meta[name='description'], meta[property='og:description']").ok();
    if let Some(selector) = meta_selector {
        for element in document.select(&selector) {
            if let Some(content) = element.value().attr("content") {
                let desc = content.trim();
                if !desc.is_empty() && desc.len() > 20 {
                    return Some(desc.to_string());
                }
            }
        }
    }
    
    // Try first paragraph
    let p_selector = Selector::parse("p").ok();
    if let Some(selector) = p_selector {
        for element in document.select(&selector) {
            let text = element.text().collect::<String>().trim().to_string();
            if text.len() > 50 && text.len() < 500 {
                return Some(text);
            }
        }
    }
    
    None
}

/// Extract structured company information using AI
pub async fn extract_company_info_with_ai(scraped: &ScrapedCompanyInfo) -> Result<Company, CareerBenchError> {
    use crate::ai::resolver::ResolvedProvider;
    
    let provider = ResolvedProvider::resolve()
        .map_err(|e| CareerBenchError::AiProvider(crate::ai::errors::AiProviderError::Unknown(
            format!("Failed to resolve provider: {}", e)
        )))?;
    
    // Limit content to avoid token limits, but prioritize the beginning (usually most important info)
    let content_for_ai = if scraped.raw_content.len() > 12000 {
        // Take first 10000 chars (usually About/Careers pages) + last 2000 chars (footer/contact info)
        let first_part = scraped.raw_content.chars().take(10000).collect::<String>();
        let last_part = scraped.raw_content.chars().rev().take(2000).collect::<String>().chars().rev().collect::<String>();
        format!("{}\n\n[... content truncated ...]\n\n{}", first_part, last_part)
    } else {
        scraped.raw_content.clone()
    };
    
    log::info!("[companies] Sending {} chars of content to AI (from {} total chars)", 
        content_for_ai.len(), scraped.raw_content.len());
    
    // Use more content - increase to 6000 chars to get better context
    // Local models need more context to extract information properly
    let content_for_prompt = content_for_ai.chars().take(6000).collect::<String>();
    
    // Improved prompt with explicit instructions and example
    let prompt = format!(
        r#"Extract company information from this text and return ONLY valid JSON:

Text:
{}

Return JSON in this exact format (fill in the values from the text above):
{{
  "name": "Company Name",
  "website": "https://example.com",
  "industry": "Technology",
  "companySize": "100-500 employees",
  "location": "San Francisco, CA",
  "description": "Brief company description",
  "mission": "Company mission statement",
  "vision": "Company vision statement",
  "values": "Company values or culture"
}}

If information is not found in the text, use empty string "". Return ONLY the JSON object, no other text."#,
        content_for_prompt
    );
    
    let system_prompt = Some("You are a data extraction assistant. Extract company information from text and return ONLY valid JSON. Do not include any explanatory text, only the JSON object.");
    
    let response = provider.as_provider().call_llm(system_prompt, &prompt).await
        .map_err(|e| CareerBenchError::AiProvider(crate::ai::errors::AiProviderError::Unknown(
            format!("AI extraction failed: {}", e)
        )))?;
    
    log::info!("[companies] AI response length: {} chars", response.len());
    log::debug!("[companies] AI response preview (first 500 chars): {}", &response[..response.len().min(500)]);
    
    // Check if response looks like valid JSON (starts with {)
    let trimmed_response = response.trim();
    if !trimmed_response.starts_with('{') {
        log::warn!("[companies] AI response does not start with {{, attempting extraction...");
    }
    
    // Extract JSON from response (handles markdown code blocks)
    let json_str = extract_json_from_response(&response);
    log::info!("[companies] Extracted JSON length: {} chars", json_str.len());
    
    // Validate that extracted JSON looks reasonable
    if json_str.len() < 50 || !json_str.trim().starts_with('{') {
        log::error!("[companies] Extracted JSON appears invalid (too short or doesn't start with {{)");
        log::error!("[companies] Extracted content: {}", &json_str[..json_str.len().min(500)]);
    } else {
        log::debug!("[companies] Extracted JSON preview (first 500 chars): {}", &json_str[..json_str.len().min(500)]);
    }
    
    // Parse JSON response
    let extracted: serde_json::Value = match serde_json::from_str(&json_str) {
        Ok(val) => {
            log::info!("[companies] ✓ Successfully parsed JSON");
            val
        },
        Err(e) => {
            log::error!("[companies] ✗ Failed to parse AI response as JSON: {}", e);
            log::error!("[companies] Full extracted JSON (first 2000 chars): {}", &json_str[..json_str.len().min(2000)]);
            log::warn!("[companies] Falling back to scraped data only");
            // Fallback: return company with just scraped data
            let now = chrono::Utc::now().to_rfc3339();
            log::info!("[companies] Returning fallback company data: name={:?}", scraped.name);
            return Ok(Company {
                id: None,
                name: scraped.name.clone().unwrap_or_else(|| "Unknown Company".to_string()),
                website: scraped.website.clone(),
                industry: scraped.industry.clone(),
                company_size: scraped.company_size.clone(),
                location: scraped.location.clone(),
                description: scraped.description.clone(),
                mission: None,
                vision: None,
                values: None,
                notes: None,
                created_at: now.clone(),
                updated_at: now,
            });
        }
    };
    
    log::info!("[companies] Successfully parsed JSON, extracted fields:");
    log::info!("[companies]   - name: {:?}", extracted.get("name"));
    log::info!("[companies]   - industry: {:?}", extracted.get("industry"));
    log::info!("[companies]   - companySize: {:?}", extracted.get("companySize"));
    log::info!("[companies]   - location: {:?}", extracted.get("location"));
    log::info!("[companies]   - description: {:?}", extracted.get("description").as_ref().map(|d| {
        let desc_str = d.as_str().unwrap_or("");
        &desc_str[..desc_str.len().min(100)]
    }));
    
    // Check if AI returned all empty fields - if so, fall back to scraped data
    let ai_name = extracted["name"].as_str().map(|s| s.trim()).unwrap_or("");
    let ai_industry = extracted["industry"].as_str().map(|s| s.trim()).unwrap_or("");
    let ai_description = extracted["description"].as_str().map(|s| s.trim()).unwrap_or("");
    
    let ai_extracted_anything = !ai_name.is_empty() || !ai_industry.is_empty() || !ai_description.is_empty();
    
    if !ai_extracted_anything {
        log::warn!("[companies] AI returned all empty fields, falling back to scraped data");
        let now = chrono::Utc::now().to_rfc3339();
        return Ok(Company {
            id: None,
            name: scraped.name.clone().unwrap_or_else(|| "Unknown Company".to_string()),
            website: scraped.website.clone(),
            industry: scraped.industry.clone(),
            company_size: scraped.company_size.clone(),
            location: scraped.location.clone(),
            description: scraped.description.clone(),
            mission: None,
            vision: None,
            values: None,
            notes: None,
            created_at: now.clone(),
            updated_at: now,
        });
    }
    
    // Build Company struct from extracted data
    let now = chrono::Utc::now().to_rfc3339();
    
    let final_name = extracted["name"]
        .as_str()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .or_else(|| scraped.name.clone())
        .unwrap_or_else(|| "Unknown Company".to_string());
    
    let final_industry = extracted["industry"]
        .as_str()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .or_else(|| scraped.industry.clone());
    
    let final_location = extracted["location"]
        .as_str()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .or_else(|| scraped.location.clone());
    
    let final_company_size = extracted["companySize"]
        .as_str()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .or_else(|| scraped.company_size.clone());
    
    let final_description = extracted["description"]
        .as_str()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .or_else(|| scraped.description.clone());
    
    let final_mission = extracted["mission"]
        .as_str()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string());
    
    let final_vision = extracted["vision"]
        .as_str()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string());
    
    let final_values = extracted["values"]
        .as_str()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string());
    
    log::info!("[companies] Final extracted company data:");
    log::info!("[companies]   - name: {}", final_name);
    log::info!("[companies]   - industry: {:?}", final_industry);
    log::info!("[companies]   - location: {:?}", final_location);
    log::info!("[companies]   - company_size: {:?}", final_company_size);
    log::info!("[companies]   - description: {:?}", final_description.as_ref().map(|d| &d[..d.len().min(150)]));
    log::info!("[companies]   - mission: {:?}", final_mission.as_ref().map(|d| &d[..d.len().min(100)]));
    log::info!("[companies]   - vision: {:?}", final_vision.as_ref().map(|d| &d[..d.len().min(100)]));
    log::info!("[companies]   - values: {:?}", final_values.as_ref().map(|d| &d[..d.len().min(100)]));
    
    Ok(Company {
        id: None,
        name: final_name,
        website: extracted["website"]
            .as_str()
            .map(|s| s.to_string())
            .or_else(|| scraped.website.clone()),
        industry: final_industry,
        company_size: final_company_size,
        location: final_location,
        description: final_description,
        mission: final_mission,
        vision: final_vision,
        values: final_values,
        notes: None,
        created_at: now.clone(),
        updated_at: now,
    })
}

/// Extract JSON from AI response (handles markdown code blocks)
fn extract_json_from_response(text: &str) -> String {
    // First, try extracting from markdown code blocks
    if let Some(start) = text.find("```json") {
        let after_start = &text[start + 7..];
        if let Some(end) = after_start.find("```") {
            let candidate = after_start[..end].trim();
            if serde_json::from_str::<serde_json::Value>(candidate).is_ok() {
                return candidate.to_string();
            }
        }
        // If no closing ```, find the first '{' after ```json
        let trimmed = after_start.trim_start();
        if let Some(root_start) = trimmed.find('{') {
            let mut brace_count = 0;
            let mut root_end = None;
            for (i, ch) in trimmed[root_start..].char_indices() {
                match ch {
                    '{' => brace_count += 1,
                    '}' => {
                        brace_count -= 1;
                        if brace_count == 0 {
                            root_end = Some(root_start + i);
                            break;
                        }
                    }
                    _ => {}
                }
            }
            if let Some(end) = root_end {
                let json_candidate = &trimmed[root_start..=end];
                if serde_json::from_str::<serde_json::Value>(json_candidate).is_ok() {
                    return json_candidate.to_string();
                }
            }
        }
    }
    
    // Try to find JSON object by matching braces
    if let Some(end_pos) = text.rfind('}') {
        let mut brace_count = 0;
        let mut start_pos = None;
        for (i, ch) in text[..=end_pos].char_indices().rev() {
            match ch {
                '}' => brace_count += 1,
                '{' => {
                    brace_count -= 1;
                    if brace_count == 0 {
                        start_pos = Some(i);
                        break;
                    }
                }
                _ => {}
            }
        }
        if let Some(start) = start_pos {
            let json_candidate = &text[start..=end_pos];
            if serde_json::from_str::<serde_json::Value>(json_candidate).is_ok() {
                return json_candidate.to_string();
            }
        }
    }
    
    // Fallback: if no valid JSON found, return empty object
    // This allows the code to continue and use scraped data as fallback
    log::warn!("[companies] Could not extract valid JSON from response, returning empty object");
    "{}".to_string()
}

