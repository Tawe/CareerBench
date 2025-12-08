use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

/// Rate limiter using token bucket algorithm
/// 
/// This rate limiter allows a certain number of requests per time window.
/// When the limit is reached, requests are delayed until tokens are available.
#[derive(Debug, Clone)]
pub struct RateLimiter {
    /// Maximum number of requests allowed in the time window
    max_requests: u32,
    /// Time window in seconds
    window_seconds: u64,
    /// Current number of available tokens
    tokens: Arc<Mutex<u32>>,
    /// Timestamp of the last token refill
    last_refill: Arc<Mutex<Instant>>,
}

impl RateLimiter {
    /// Create a new rate limiter
    /// 
    /// # Arguments
    /// * `max_requests` - Maximum number of requests allowed in the time window
    /// * `window_seconds` - Time window in seconds
    pub fn new(max_requests: u32, window_seconds: u64) -> Self {
        Self {
            max_requests,
            window_seconds,
            tokens: Arc::new(Mutex::new(max_requests)),
            last_refill: Arc::new(Mutex::new(Instant::now())),
        }
    }

    /// Create a rate limiter with default OpenAI limits
    /// OpenAI free tier: 3 requests per minute
    /// OpenAI paid tier: 60 requests per minute (we'll use 50 to be safe)
    pub fn openai_default() -> Self {
        // Conservative limit: 50 requests per minute
        Self::new(50, 60)
    }

    /// Create a rate limiter with default Anthropic limits
    /// Anthropic: 50 requests per minute
    pub fn anthropic_default() -> Self {
        Self::new(50, 60)
    }

    /// Wait for a token to become available
    /// 
    /// This will block until a token is available, refilling tokens
    /// based on the elapsed time since the last refill.
    pub async fn acquire(&self) {
        loop {
            let mut tokens = self.tokens.lock().await;
            let mut last_refill = self.last_refill.lock().await;
            
            // Refill tokens based on elapsed time
            let now = Instant::now();
            let elapsed = now.duration_since(*last_refill);
            let window_duration = Duration::from_secs(self.window_seconds);
            
            if elapsed >= window_duration {
                // Full window has passed, refill all tokens
                *tokens = self.max_requests;
                *last_refill = now;
            } else {
                // Calculate how many tokens to refill based on elapsed time
                let tokens_to_refill = (elapsed.as_secs_f64() / window_duration.as_secs_f64() * self.max_requests as f64) as u32;
                if tokens_to_refill > 0 {
                    *tokens = (*tokens + tokens_to_refill).min(self.max_requests);
                    *last_refill = now;
                }
            }
            
            // If we have tokens available, use one and return
            if *tokens > 0 {
                *tokens -= 1;
                return;
            }
            
            // No tokens available, calculate wait time
            drop(tokens);
            drop(last_refill);
            
            // Wait for the next token to become available
            let wait_time = window_duration - elapsed;
            if wait_time.as_secs() > 0 || wait_time.as_millis() > 0 {
                tokio::time::sleep(wait_time).await;
            } else {
                // Very small wait time, just yield to avoid tight loop
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        }
    }

    /// Try to acquire a token without blocking
    /// 
    /// Returns `true` if a token was acquired, `false` if no tokens are available
    #[allow(dead_code)]
    pub async fn try_acquire(&self) -> bool {
        let mut tokens = self.tokens.lock().await;
        let mut last_refill = self.last_refill.lock().await;
        
        // Refill tokens based on elapsed time
        let now = Instant::now();
        let elapsed = now.duration_since(*last_refill);
        let window_duration = Duration::from_secs(self.window_seconds);
        
        if elapsed >= window_duration {
            // Full window has passed, refill all tokens
            *tokens = self.max_requests;
            *last_refill = now;
        } else {
            // Calculate how many tokens to refill based on elapsed time
            let tokens_to_refill = (elapsed.as_secs_f64() / window_duration.as_secs_f64() * self.max_requests as f64) as u32;
            if tokens_to_refill > 0 {
                *tokens = (*tokens + tokens_to_refill).min(self.max_requests);
                *last_refill = now;
            }
        }
        
        // If we have tokens available, use one
        if *tokens > 0 {
            *tokens -= 1;
            true
        } else {
            false
        }
    }

    /// Get the current number of available tokens (for monitoring)
    #[allow(dead_code)]
    pub async fn available_tokens(&self) -> u32 {
        let mut tokens = self.tokens.lock().await;
        let mut last_refill = self.last_refill.lock().await;
        
        // Refill tokens based on elapsed time
        let now = Instant::now();
        let elapsed = now.duration_since(*last_refill);
        let window_duration = Duration::from_secs(self.window_seconds);
        
        if elapsed >= window_duration {
            *tokens = self.max_requests;
            *last_refill = now;
        } else {
            let tokens_to_refill = (elapsed.as_secs_f64() / window_duration.as_secs_f64() * self.max_requests as f64) as u32;
            if tokens_to_refill > 0 {
                *tokens = (*tokens + tokens_to_refill).min(self.max_requests);
                *last_refill = now;
            }
        }
        
        *tokens
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, Instant};

    #[tokio::test]
    async fn test_rate_limiter_acquires_tokens() {
        let limiter = RateLimiter::new(2, 1); // 2 requests per second
        
        // Should be able to acquire 2 tokens immediately
        let start = Instant::now();
        limiter.acquire().await;
        limiter.acquire().await;
        let elapsed = start.elapsed();
        
        // Should be fast (no waiting)
        assert!(elapsed.as_millis() < 100);
    }

    #[tokio::test]
    async fn test_rate_limiter_blocks_when_exhausted() {
        let limiter = RateLimiter::new(1, 1); // 1 request per second
        
        // Acquire the only token
        limiter.acquire().await;
        
        // Next acquire should wait
        let start = Instant::now();
        limiter.acquire().await;
        let elapsed = start.elapsed();
        
        // Should have waited approximately 1 second
        assert!(elapsed.as_secs() >= 1);
        assert!(elapsed.as_secs() < 2); // But not too long
    }

    #[tokio::test]
    async fn test_rate_limiter_refills_over_time() {
        let limiter = RateLimiter::new(2, 1); // 2 requests per second
        
        // Exhaust tokens
        limiter.acquire().await;
        limiter.acquire().await;
        
        // Wait for refill
        sleep(Duration::from_millis(1100)).await;
        
        // Should be able to acquire again
        let start = Instant::now();
        limiter.acquire().await;
        let elapsed = start.elapsed();
        
        // Should be fast (token was refilled)
        assert!(elapsed.as_millis() < 100);
    }

    #[tokio::test]
    async fn test_try_acquire() {
        let limiter = RateLimiter::new(1, 1);
        
        // First acquire should succeed
        assert!(limiter.try_acquire().await);
        
        // Second acquire should fail
        assert!(!limiter.try_acquire().await);
        
        // After waiting, should succeed again
        sleep(Duration::from_millis(1100)).await;
        assert!(limiter.try_acquire().await);
    }

    #[tokio::test]
    async fn test_available_tokens() {
        let limiter = RateLimiter::new(5, 1);
        
        // Should start with all tokens
        assert_eq!(limiter.available_tokens().await, 5);
        
        // After acquiring one, should have 4
        limiter.acquire().await;
        assert_eq!(limiter.available_tokens().await, 4);
        
        // After waiting, should refill
        sleep(Duration::from_millis(1100)).await;
        assert_eq!(limiter.available_tokens().await, 5);
    }
}

