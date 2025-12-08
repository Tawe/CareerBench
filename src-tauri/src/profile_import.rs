// Profile import functionality for parsing resumes/CVs and extracting profile data

use crate::commands::{UserProfile, Experience, Skill, Education, Certification, PortfolioItem};
use crate::errors::CareerBenchError;
use std::fs;
use std::path::Path;

/// Extract text from a PDF file
pub fn extract_text_from_pdf(file_path: &Path) -> Result<String, CareerBenchError> {
    let bytes = fs::read(file_path)
        .map_err(|e| CareerBenchError::FileSystem(crate::errors::FileSystemError::IoError(
            format!("Failed to read PDF file {}: {}", file_path.to_string_lossy(), e)
        )))?;
    
    let text = pdf_extract::extract_text_from_mem(&bytes)
        .map_err(|e| CareerBenchError::FileSystem(crate::errors::FileSystemError::IoError(
            format!("Failed to parse PDF file {}: {}", file_path.to_string_lossy(), e)
        )))?;
    
    Ok(text)
}

/// Extract text from a DOCX file
pub fn extract_text_from_docx(file_path: &Path) -> Result<String, CareerBenchError> {
    let bytes = fs::read(file_path)
        .map_err(|e| CareerBenchError::FileSystem(crate::errors::FileSystemError::IoError(
            format!("Failed to read DOCX file {}: {}", file_path.to_string_lossy(), e)
        )))?;
    
    // Parse DOCX file
    let docx = docx_rs::read_docx(&bytes)
        .map_err(|e| CareerBenchError::FileSystem(crate::errors::FileSystemError::IoError(
            format!("Failed to parse DOCX file {}: {}", file_path.to_string_lossy(), e)
        )))?;
    
    // Extract text from all paragraphs
    let mut text = String::new();
    
    // Access the document and extract text from paragraphs
    // docx-rs 0.4 structure: docx.document has a children field which is a Vec<DocumentChild>
    for child in &docx.document.children {
        match child {
            docx_rs::DocumentChild::Paragraph(paragraph) => {
                // Extract text from paragraph runs
                for child in &paragraph.children {
                    match child {
                        docx_rs::ParagraphChild::Run(run) => {
                            for child in &run.children {
                                match child {
                                    docx_rs::RunChild::Text(text_content) => {
                                        text.push_str(&text_content.text);
                                    }
                                    docx_rs::RunChild::Tab(_) => {
                                        text.push('\t');
                                    }
                                    docx_rs::RunChild::Break(_) => {
                                        text.push('\n');
                                    }
                                    _ => {}
                                }
                            }
                        }
                        _ => {}
                    }
                }
                text.push('\n');
            }
            docx_rs::DocumentChild::Table(_) => {
                // Tables are complex - for now, just add a separator
                text.push_str("\n[Table]\n");
            }
            _ => {}
        }
    }
    
    Ok(text.trim().to_string())
}

/// Extract text from a plain text file
pub fn extract_text_from_txt(file_path: &Path) -> Result<String, CareerBenchError> {
    fs::read_to_string(file_path)
        .map_err(|e| CareerBenchError::FileSystem(crate::errors::FileSystemError::IoError(
            format!("Failed to read text file {}: {}", file_path.to_string_lossy(), e)
        )))
}

/// Extract text from a resume file (PDF, DOCX, or TXT)
pub fn extract_text_from_resume(file_path: &Path) -> Result<String, CareerBenchError> {
    let extension = file_path.extension()
        .and_then(|ext| ext.to_str())
        .ok_or_else(|| CareerBenchError::FileSystem(crate::errors::FileSystemError::IoError(
            format!("File {} has no extension", file_path.to_string_lossy())
        )))?;
    
    let extension_lower = extension.to_lowercase();
    match extension_lower.as_str() {
        "pdf" => extract_text_from_pdf(file_path),
        "docx" => extract_text_from_docx(file_path),
        "doc" => extract_text_from_docx(file_path), // Try DOCX parser for .doc files
        "txt" => extract_text_from_txt(file_path),
        _ => Err(CareerBenchError::FileSystem(crate::errors::FileSystemError::IoError(
            format!("Unsupported file type: {} (supported: pdf, docx, txt)", extension)
        ))),
    }
}

/// Structure for parsed resume data (before AI extraction)
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ParsedResumeText {
    pub text: String,
    pub file_path: String,
}

/// Structure for AI-extracted profile data from resume
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ExtractedProfileData {
    pub profile: Option<UserProfile>,
    pub experience: Vec<Experience>,
    pub skills: Vec<Skill>,
    pub education: Vec<Education>,
    pub certifications: Vec<Certification>,
    pub portfolio: Vec<PortfolioItem>,
}

