//! Per-client IPC rate limit: a token bucket per connection. Over-limit lines are dropped.

use std::time::Instant;

/// Sustained IPC lines per second, per client.
pub const IPC_LINES_PER_SEC: f64 = 100.0;
/// Lines a client may send at once before the sustained rate applies.
pub const IPC_BURST: f64 = 200.0;

/// Per-client limits, from `[daemon]` config. `Default` is the built-in values.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct IpcLimits {
    pub lines_per_sec: f64,
    pub burst: f64,
}

impl Default for IpcLimits {
    fn default() -> Self {
        Self { lines_per_sec: IPC_LINES_PER_SEC, burst: IPC_BURST }
    }
}

pub struct RateLimiter {
    capacity: f64,
    refill_per_sec: f64,
    tokens: f64,
    last: Instant,
}

impl RateLimiter {
    /// Starts with a full bucket.
    pub fn new(refill_per_sec: f64, capacity: f64, now: Instant) -> Self {
        Self { capacity, refill_per_sec, tokens: capacity, last: now }
    }

    /// Takes one token if one is available. Returns `false` when the bucket is empty.
    pub fn allow(&mut self, now: Instant) -> bool {
        let elapsed = now.saturating_duration_since(self.last).as_secs_f64();
        self.tokens = (self.tokens + elapsed * self.refill_per_sec).min(self.capacity);
        self.last = now;
        if self.tokens >= 1.0 {
            self.tokens -= 1.0;
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_rate_limiter_allows_burst_then_denies() {
        let t0 = Instant::now();
        let mut rl = RateLimiter::new(1.0, 3.0, t0);
        assert!(rl.allow(t0));
        assert!(rl.allow(t0));
        assert!(rl.allow(t0));
        assert!(!rl.allow(t0));
    }

    #[test]
    fn test_rate_limiter_refills_over_time() {
        let t0 = Instant::now();
        let mut rl = RateLimiter::new(1.0, 3.0, t0);
        for _ in 0..3 {
            assert!(rl.allow(t0));
        }
        assert!(!rl.allow(t0));
        assert!(rl.allow(t0 + Duration::from_secs(1)));
        assert!(!rl.allow(t0 + Duration::from_secs(1)));
    }

    #[test]
    fn test_rate_limiter_caps_at_capacity() {
        let t0 = Instant::now();
        let mut rl = RateLimiter::new(1.0, 3.0, t0);
        for _ in 0..3 {
            assert!(rl.allow(t0));
        }
        let later = t0 + Duration::from_secs(10);
        for _ in 0..3 {
            assert!(rl.allow(later));
        }
        assert!(!rl.allow(later));
    }
}
