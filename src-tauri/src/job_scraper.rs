// Job URL scraping functionality
// Extracts job descriptions from various job board URLs

use crate::errors::CareerBenchError;
use scraper::{Html, Selector};

/// Result of scraping a job URL
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ScrapedJobData {
    pub title: Option<String>,
    pub company: Option<String>,
    pub location: Option<String>,
    pub description: String,
    pub source: String, // e.g., "LinkedIn", "Indeed", "Generic"
}

/// Detect the job board type from URL
pub fn detect_job_board(url: &str) -> &str {
    let url_lower = url.to_lowercase();
    if url_lower.contains("linkedin.com") {
        "LinkedIn"
    } else if url_lower.contains("indeed.com") {
        "Indeed"
    } else if url_lower.contains("glassdoor.com") {
        "Glassdoor"
    } else if url_lower.contains("monster.com") {
        "Monster"
    } else if url_lower.contains("ziprecruiter.com") {
        "ZipRecruiter"
    } else if url_lower.contains("dice.com") {
        "Dice"
    } else {
        "Generic"
    }
}

/// Scrape job data from a URL
pub async fn scrape_job_url(url: &str) -> Result<ScrapedJobData, CareerBenchError> {
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36")
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| CareerBenchError::FileSystem(crate::errors::FileSystemError::IoError(
            format!("Failed to create HTTP client: {}", e)
        )))?;

    // Fetch the page
    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| CareerBenchError::FileSystem(crate::errors::FileSystemError::IoError(
            format!("Failed to fetch URL: {}", e)
        )))?;

    if !response.status().is_success() {
        return Err(CareerBenchError::FileSystem(crate::errors::FileSystemError::IoError(
            format!("HTTP error: {}", response.status())
        )));
    }

    let html = response
        .text()
        .await
        .map_err(|e| CareerBenchError::FileSystem(crate::errors::FileSystemError::IoError(
            format!("Failed to read response: {}", e)
        )))?;

    let document = Html::parse_document(&html);
    let source = detect_job_board(url);

    // Try job board-specific scrapers first
    let result = match source {
        "LinkedIn" => scrape_linkedin(&document, url),
        "Indeed" => scrape_indeed(&document, url),
        "Glassdoor" => scrape_glassdoor(&document, url),
        _ => scrape_generic(&document, url),
    };

    result.map(|mut data| {
        data.source = source.to_string();
        data
    })
}

/// Scrape LinkedIn job posting
fn scrape_linkedin(document: &Html, _url: &str) -> Result<ScrapedJobData, CareerBenchError> {
    let mut data = ScrapedJobData {
        title: None,
        company: None,
        location: None,
        description: String::new(),
        source: "LinkedIn".to_string(),
    };

    // LinkedIn job title (multiple possible selectors)
    let title_selectors = [
        "h1.job-details-jobs-unified-top-card__job-title",
        "h1[data-test-id='job-title']",
        ".job-details-jobs-unified-top-card__job-title",
        "h1",
    ];

    for selector_str in &title_selectors {
        if let Ok(selector) = Selector::parse(selector_str) {
            if let Some(element) = document.select(&selector).next() {
                data.title = Some(element.text().collect::<String>().trim().to_string());
                break;
            }
        }
    }

    // LinkedIn company name
    let company_selectors = [
        "a.job-details-jobs-unified-top-card__company-name",
        "a[data-test-id='job-company-name']",
        ".job-details-jobs-unified-top-card__company-name",
    ];

    for selector_str in &company_selectors {
        if let Ok(selector) = Selector::parse(selector_str) {
            if let Some(element) = document.select(&selector).next() {
                data.company = Some(element.text().collect::<String>().trim().to_string());
                break;
            }
        }
    }

    // LinkedIn location
    let location_selectors = [
        "span.job-details-jobs-unified-top-card__bullet",
        ".job-details-jobs-unified-top-card__primary-description-without-tagline",
    ];

    for selector_str in &location_selectors {
        if let Ok(selector) = Selector::parse(selector_str) {
            if let Some(element) = document.select(&selector).next() {
                let text = element.text().collect::<String>();
                if text.contains("·") || text.contains(",") {
                    data.location = Some(text.trim().to_string());
                    break;
                }
            }
        }
    }

    // LinkedIn job description
    let desc_selectors = [
        "div.jobs-description__content",
        "div[data-test-id='job-description']",
        ".jobs-description-content__text",
        "div.description__text",
    ];

    for selector_str in &desc_selectors {
        if let Ok(selector) = Selector::parse(selector_str) {
            let elements: Vec<_> = document.select(&selector).collect();
            if !elements.is_empty() {
                data.description = elements
                    .iter()
                    .map(|e| e.text().collect::<String>())
                    .collect::<Vec<_>>()
                    .join("\n\n")
                    .trim()
                    .to_string();
                break;
            }
        }
    }

    // Fallback: try to extract from meta tags or structured data
    if data.description.is_empty() {
        data.description = extract_from_meta_tags(document);
    }

    Ok(data)
}

/// Scrape Indeed job posting
fn scrape_indeed(document: &Html, _url: &str) -> Result<ScrapedJobData, CareerBenchError> {
    let mut data = ScrapedJobData {
        title: None,
        company: None,
        location: None,
        description: String::new(),
        source: "Indeed".to_string(),
    };

    // Indeed job title
    let title_selector = Selector::parse("h2.jobTitle, h1.jobsearch-JobInfoHeader-title").ok();
    if let Some(selector) = title_selector {
        if let Some(element) = document.select(&selector).next() {
            data.title = Some(element.text().collect::<String>().trim().to_string());
        }
    }

    // Indeed company name
    let company_selector = Selector::parse("span[data-testid='inlineHeader-companyName'], .companyName").ok();
    if let Some(selector) = company_selector {
        if let Some(element) = document.select(&selector).next() {
            data.company = Some(element.text().collect::<String>().trim().to_string());
        }
    }

    // Indeed location
    let location_selector = Selector::parse("div[data-testid='job-location'], .jobLocation").ok();
    if let Some(selector) = location_selector {
        if let Some(element) = document.select(&selector).next() {
            data.location = Some(element.text().collect::<String>().trim().to_string());
        }
    }

    // Indeed job description
    let desc_selector = Selector::parse("div#jobDescriptionText, div.jobsearch-jobDescriptionText").ok();
    if let Some(selector) = desc_selector {
        let elements: Vec<_> = document.select(&selector).collect();
        if !elements.is_empty() {
            data.description = elements
                .iter()
                .map(|e| e.text().collect::<String>())
                .collect::<Vec<_>>()
                .join("\n\n")
                .trim()
                .to_string();
        }
    }

    // Fallback
    if data.description.is_empty() {
        data.description = extract_from_meta_tags(document);
    }

    Ok(data)
}

/// Scrape Glassdoor job posting
fn scrape_glassdoor(document: &Html, _url: &str) -> Result<ScrapedJobData, CareerBenchError> {
    let mut data = ScrapedJobData {
        title: None,
        company: None,
        location: None,
        description: String::new(),
        source: "Glassdoor".to_string(),
    };

    // Glassdoor selectors (these may need adjustment based on actual site structure)
    let title_selector = Selector::parse("h1[data-test='jobTitle'], .jobTitle").ok();
    if let Some(selector) = title_selector {
        if let Some(element) = document.select(&selector).next() {
            data.title = Some(element.text().collect::<String>().trim().to_string());
        }
    }

    let company_selector = Selector::parse("span[data-test='employerName'], .employerName").ok();
    if let Some(selector) = company_selector {
        if let Some(element) = document.select(&selector).next() {
            data.company = Some(element.text().collect::<String>().trim().to_string());
        }
    }

    let desc_selector = Selector::parse("div[data-test='jobDescription'], .jobDescription").ok();
    if let Some(selector) = desc_selector {
        let elements: Vec<_> = document.select(&selector).collect();
        if !elements.is_empty() {
            data.description = elements
                .iter()
                .map(|e| e.text().collect::<String>())
                .collect::<Vec<_>>()
                .join("\n\n")
                .trim()
                .to_string();
        }
    }

    if data.description.is_empty() {
        data.description = extract_from_meta_tags(document);
    }

    Ok(data)
}

/// Generic scraper for unknown job boards
fn scrape_generic(document: &Html, _url: &str) -> Result<ScrapedJobData, CareerBenchError> {
    let mut data = ScrapedJobData {
        title: None,
        company: None,
        location: None,
        description: String::new(),
        source: "Generic".to_string(),
    };

    // Try to extract from common HTML5 semantic elements
    let title_selector = Selector::parse("h1, h2.job-title, .job-title").ok();
    if let Some(selector) = title_selector {
        if let Some(element) = document.select(&selector).next() {
            let text = element.text().collect::<String>().trim().to_string();
            if !text.is_empty() && text.len() < 200 {
                data.title = Some(text);
            }
        }
    }

    // Try to extract description from common content areas
    let desc_selectors = [
        "div.job-description",
        "div.description",
        "article",
        "main",
        "div.content",
    ];

    for selector_str in &desc_selectors {
        if let Ok(selector) = Selector::parse(selector_str) {
            let elements: Vec<_> = document.select(&selector).collect();
            if !elements.is_empty() {
                let text = elements
                    .iter()
                    .map(|e| e.text().collect::<String>())
                    .collect::<Vec<_>>()
                    .join("\n\n")
                    .trim()
                    .to_string();
                
                // Only use if it's substantial content (more than 100 chars)
                if text.len() > 100 {
                    data.description = text;
                    break;
                }
            }
        }
    }

    // Fallback to meta tags
    if data.description.is_empty() {
        data.description = extract_from_meta_tags(document);
    }

    // Extract from JSON-LD structured data if available
    if data.description.is_empty() || data.title.is_none() {
        if let Ok((title, desc)) = extract_from_json_ld(document) {
            if data.title.is_none() {
                data.title = title;
            }
            if data.description.is_empty() {
                data.description = desc;
            }
        }
    }

    Ok(data)
}

/// Extract job data from meta tags
fn extract_from_meta_tags(document: &Html) -> String {
    let meta_selectors = [
        "meta[property='og:description']",
        "meta[name='description']",
        "meta[property='description']",
    ];

    for selector_str in &meta_selectors {
        if let Ok(selector) = Selector::parse(selector_str) {
            if let Some(element) = document.select(&selector).next() {
                if let Some(content) = element.value().attr("content") {
                    if !content.trim().is_empty() {
                        return content.trim().to_string();
                    }
                }
            }
        }
    }

    String::new()
}

/// Extract job data from JSON-LD structured data
fn extract_from_json_ld(document: &Html) -> Result<(Option<String>, String), CareerBenchError> {
    let selector = Selector::parse("script[type='application/ld+json']").unwrap();
    let mut title = None;
    let mut description = String::new();

    for element in document.select(&selector) {
        if let Some(text) = element.text().next() {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(text) {
                // Try to extract from JobPosting schema
                if let Some(job_posting) = json.get("@type")
                    .and_then(|t| if t == "JobPosting" { Some(&json) } else { None })
                    .or_else(|| json.get("@type").and_then(|_| json.get("jobPosting")))
                {
                    if let Some(t) = job_posting.get("title").and_then(|v| v.as_str()) {
                        title = Some(t.to_string());
                    }
                    if let Some(d) = job_posting.get("description").and_then(|v| v.as_str()) {
                        description = d.to_string();
                    }
                }
            }
        }
    }

    Ok((title, description))
}

