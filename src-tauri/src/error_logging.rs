//! Error logging and monitoring utilities
//! 
//! This module provides utilities for logging errors with context and
//! tracking error metrics for monitoring purposes.

use crate::errors::CareerBenchError;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

/// Error metrics for monitoring
#[derive(Debug, Clone)]
struct ErrorMetrics {
    /// Total error count
    total_errors: u64,
    /// Errors by type
    errors_by_type: HashMap<String, u64>,
    /// Errors by context/operation
    errors_by_context: HashMap<String, u64>,
    /// Recent errors (last 100)
    recent_errors: Vec<ErrorRecord>,
}

/// Record of a single error occurrence
#[derive(Debug, Clone)]
pub struct ErrorRecord {
    /// Timestamp (Unix epoch seconds)
    pub timestamp: u64,
    /// Error type
    pub error_type: String,
    /// Context/operation where error occurred
    pub context: String,
    /// Error message
    pub message: String,
    /// Whether error was recoverable
    pub recoverable: bool,
}

impl ErrorMetrics {
    fn new() -> Self {
        Self {
            total_errors: 0,
            errors_by_type: HashMap::new(),
            errors_by_context: HashMap::new(),
            recent_errors: Vec::new(),
        }
    }
    
    fn record_error(&mut self, error: &CareerBenchError, context: &str) {
        self.total_errors += 1;
        
        // Record by error type
        let error_type = match error {
            CareerBenchError::Database(_) => "Database",
            CareerBenchError::AiProvider(_) => "AiProvider",
            CareerBenchError::Validation(_) => "Validation",
            CareerBenchError::Configuration(_) => "Configuration",
            CareerBenchError::FileSystem(_) => "FileSystem",
            CareerBenchError::Application(_) => "Application",
        };
        
        *self.errors_by_type.entry(error_type.to_string()).or_insert(0) += 1;
        *self.errors_by_context.entry(context.to_string()).or_insert(0) += 1;
        
        // Record recent error
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let recoverable = match error {
            CareerBenchError::Database(_) => false,
            CareerBenchError::AiProvider(e) => matches!(
                e,
                crate::errors::AiProviderError::NetworkError(_) 
                | crate::errors::AiProviderError::RateLimitExceeded
                | crate::errors::AiProviderError::InvalidResponse(_)
            ),
            CareerBenchError::Validation(_) => true,
            CareerBenchError::Configuration(_) => false,
            CareerBenchError::FileSystem(_) => false,
            CareerBenchError::Application(_) => true,
        };
        
        let record = ErrorRecord {
            timestamp,
            error_type: error_type.to_string(),
            context: context.to_string(),
            message: get_short_error_message(error),
            recoverable,
        };
        
        self.recent_errors.push(record);
        
        // Keep only last 100 errors
        if self.recent_errors.len() > 100 {
            self.recent_errors.remove(0);
        }
    }
}

// Global error metrics (thread-safe)
static ERROR_METRICS: Mutex<Option<Arc<Mutex<ErrorMetrics>>>> = Mutex::new(None);

/// Initialize error metrics tracking
pub fn init_error_metrics() {
    let mut metrics_guard = ERROR_METRICS.lock().unwrap();
    if metrics_guard.is_none() {
        *metrics_guard = Some(Arc::new(Mutex::new(ErrorMetrics::new())));
    }
}

/// Record an error in metrics
pub fn record_error_metric(error: &CareerBenchError, context: &str) {
    if let Ok(metrics_guard) = ERROR_METRICS.lock() {
        if let Some(metrics) = metrics_guard.as_ref() {
            if let Ok(mut m) = metrics.lock() {
                m.record_error(error, context);
            }
        }
    }
}

/// Get error statistics
pub fn get_error_stats() -> Option<(u64, HashMap<String, u64>, HashMap<String, u64>)> {
    if let Ok(metrics_guard) = ERROR_METRICS.lock() {
        if let Some(metrics) = metrics_guard.as_ref() {
            if let Ok(m) = metrics.lock() {
                return Some((
                    m.total_errors,
                    m.errors_by_type.clone(),
                    m.errors_by_context.clone(),
                ));
            }
        }
    }
    None
}

/// Get recent error records
pub fn get_recent_errors(limit: usize) -> Vec<ErrorRecord> {
    if let Ok(metrics_guard) = ERROR_METRICS.lock() {
        if let Some(metrics) = metrics_guard.as_ref() {
            if let Ok(m) = metrics.lock() {
                let len = m.recent_errors.len();
                let start = if len > limit { len - limit } else { 0 };
                return m.recent_errors[start..].to_vec();
            }
        }
    }
    Vec::new()
}

use crate::errors::get_short_error_message;

