use crate::ai::errors::AiProviderError;
use std::time::Duration;
use tokio::time::sleep;

/// Configuration for retry behavior
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts (not including the initial attempt)
    pub max_retries: u32,
    /// Initial delay before first retry (in milliseconds)
    pub initial_delay_ms: u64,
    /// Maximum delay between retries (in milliseconds)
    pub max_delay_ms: u64,
    /// Multiplier for exponential backoff
    pub backoff_multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay_ms: 500,
            max_delay_ms: 10000,
            backoff_multiplier: 2.0,
        }
    }
}

/// Determines if an error is transient and should be retried
pub fn is_retryable_error(error: &AiProviderError) -> bool {
    match error {
        // Network errors are usually transient
        AiProviderError::NetworkError(_) => true,
        // Rate limits are transient - wait and retry
        AiProviderError::RateLimitExceeded => true,
        // These are permanent errors - don't retry
        AiProviderError::InvalidApiKey => false,
        AiProviderError::InvalidResponse(_) => false,
        AiProviderError::ValidationError(_) => false,
        AiProviderError::ModelNotFound => false,
        AiProviderError::Unknown(_) => {
            // Unknown errors might be transient, but be conservative
            // Only retry if the error message suggests a network issue
            false
        }
    }
}

/// Retry an async operation with exponential backoff
/// 
/// # Arguments
/// * `operation` - The async operation to retry
/// * `config` - Retry configuration
/// 
/// # Returns
/// * `Ok(T)` - If the operation succeeds (on any attempt)
/// * `Err(AiProviderError)` - If all retries are exhausted or a non-retryable error occurs
pub async fn retry_with_backoff<F, Fut, T>(
    mut operation: F,
    config: RetryConfig,
) -> Result<T, AiProviderError>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, AiProviderError>>,
{
    let mut last_error: Option<AiProviderError> = None;
    let mut delay_ms = config.initial_delay_ms;

    // Initial attempt + retries
    for attempt in 0..=config.max_retries {
        match operation().await {
            Ok(result) => {
                // Success - return immediately
                if attempt > 0 {
                    log::info!("AI operation succeeded after {} retry attempts", attempt);
                }
                return Ok(result);
            }
            Err(error) => {
                // Check if error is retryable
                if !is_retryable_error(&error) {
                    // Non-retryable error - return immediately
                    log::warn!("Non-retryable error encountered: {}", error);
                    return Err(error);
                }

                last_error = Some(error.clone());

                // If this was the last attempt, don't wait
                if attempt >= config.max_retries {
                    log::warn!(
                        "AI operation failed after {} attempts. Last error: {}",
                        attempt + 1,
                        error
                    );
                    break;
                }

                // Log retry attempt
                log::info!(
                    "AI operation failed (attempt {}/{}): {}. Retrying in {}ms...",
                    attempt + 1,
                    config.max_retries + 1,
                    error,
                    delay_ms
                );

                // Wait before retrying
                sleep(Duration::from_millis(delay_ms)).await;

                // Calculate next delay with exponential backoff
                delay_ms = (delay_ms as f64 * config.backoff_multiplier) as u64;
                delay_ms = delay_ms.min(config.max_delay_ms);
            }
        }
    }

    // All retries exhausted
    Err(last_error.unwrap_or_else(|| {
        AiProviderError::Unknown("Operation failed after retries".to_string())
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    #[tokio::test]
    async fn test_retry_succeeds_on_first_attempt() {
        let config = RetryConfig {
            max_retries: 3,
            initial_delay_ms: 10,
            max_delay_ms: 100,
            backoff_multiplier: 2.0,
        };

        let result = retry_with_backoff(|| async { Ok("success") }, config).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "success");
    }

    #[tokio::test]
    async fn test_retry_succeeds_after_retries() {
        let config = RetryConfig {
            max_retries: 3,
            initial_delay_ms: 10,
            max_delay_ms: 100,
            backoff_multiplier: 2.0,
        };

        let attempts = AtomicU32::new(0);
        let result = retry_with_backoff(
            || {
                let attempts = &attempts;
                async move {
                    let count = attempts.fetch_add(1, Ordering::SeqCst);
                    if count < 2 {
                        Err(AiProviderError::NetworkError("Temporary failure".to_string()))
                    } else {
                        Ok("success")
                    }
                }
            },
            config,
        )
        .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "success");
        assert_eq!(attempts.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_retry_fails_on_non_retryable_error() {
        let config = RetryConfig {
            max_retries: 3,
            initial_delay_ms: 10,
            max_delay_ms: 100,
            backoff_multiplier: 2.0,
        };

        let attempts = AtomicU32::new(0);
        let result = retry_with_backoff(
            || {
                let attempts = &attempts;
                async move {
                    attempts.fetch_add(1, Ordering::SeqCst);
                    Err(AiProviderError::InvalidApiKey)
                }
            },
            config,
        )
        .await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AiProviderError::InvalidApiKey));
        // Should only attempt once (no retries for non-retryable errors)
        assert_eq!(attempts.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_retry_exhausts_all_attempts() {
        let config = RetryConfig {
            max_retries: 2,
            initial_delay_ms: 10,
            max_delay_ms: 100,
            backoff_multiplier: 2.0,
        };

        let attempts = AtomicU32::new(0);
        let result = retry_with_backoff(
            || {
                let attempts = &attempts;
                async move {
                    attempts.fetch_add(1, Ordering::SeqCst);
                    Err(AiProviderError::NetworkError("Persistent failure".to_string()))
                }
            },
            config,
        )
        .await;

        assert!(result.is_err());
        // Should attempt initial + max_retries = 3 times
        assert_eq!(attempts.load(Ordering::SeqCst), 3);
    }

    #[test]
    fn test_is_retryable_error() {
        assert!(is_retryable_error(&AiProviderError::NetworkError("test".to_string())));
        assert!(is_retryable_error(&AiProviderError::RateLimitExceeded));
        assert!(!is_retryable_error(&AiProviderError::InvalidApiKey));
        assert!(!is_retryable_error(&AiProviderError::InvalidResponse("test".to_string())));
        assert!(!is_retryable_error(&AiProviderError::ValidationError("test".to_string())));
    }
}

