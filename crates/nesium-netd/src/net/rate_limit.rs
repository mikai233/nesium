//! Rate limiting utilities for the netplay server.
//!
//! Provides IP-based connection rate limiting and per-connection message
//! rate limiting to protect against DDoS attacks and misbehaving clients.

use std::net::IpAddr;
use std::num::NonZeroU32;
use std::sync::Arc;

use dashmap::DashMap;
use governor::clock::DefaultClock;
use governor::middleware::NoOpMiddleware;
use governor::state::{InMemoryState, NotKeyed};
use governor::{Quota, RateLimiter};

/// Type alias for a simple rate limiter (not keyed).
pub type SimpleRateLimiter = RateLimiter<NotKeyed, InMemoryState, DefaultClock, NoOpMiddleware>;

/// Rate limiting configuration.
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    /// Max new connections per IP per second (0 = disabled).
    pub conn_per_ip_per_sec: u32,
    /// Max messages per connection per second (0 = disabled).
    pub msg_per_conn_per_sec: u32,
    /// Burst multiplier (how many times the per-second rate is allowed in a burst).
    pub burst_multiplier: u32,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            conn_per_ip_per_sec: 10,
            msg_per_conn_per_sec: 100,
            burst_multiplier: 3,
        }
    }
}

impl RateLimitConfig {
    /// Returns true if IP-based connection rate limiting is enabled.
    pub fn conn_limit_enabled(&self) -> bool {
        self.conn_per_ip_per_sec > 0
    }

    /// Returns true if per-connection message rate limiting is enabled.
    pub fn msg_limit_enabled(&self) -> bool {
        self.msg_per_conn_per_sec > 0
    }
}

/// IP-based connection rate limiter.
///
/// Tracks connection attempts per IP address and rejects connections
/// that exceed the configured rate.
pub struct IpRateLimiter {
    /// Map of IP -> rate limiter.
    limiters: DashMap<IpAddr, SimpleRateLimiter>,
    /// Configuration.
    config: RateLimitConfig,
}

impl IpRateLimiter {
    /// Create a new IP rate limiter with the given configuration.
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            limiters: DashMap::new(),
            config,
        }
    }

    /// Check if a connection from the given IP should be allowed.
    ///
    /// Returns `true` if the connection is allowed, `false` if rate limited.
    pub fn check(&self, ip: IpAddr) -> bool {
        if !self.config.conn_limit_enabled() {
            return true;
        }

        let limiter = self.limiters.entry(ip).or_insert_with(|| {
            let burst = self.config.conn_per_ip_per_sec * self.config.burst_multiplier;
            let quota =
                Quota::per_second(NonZeroU32::new(self.config.conn_per_ip_per_sec).unwrap())
                    .allow_burst(NonZeroU32::new(burst).unwrap());
            RateLimiter::direct(quota)
        });

        limiter.check().is_ok()
    }

    /// Periodically clean up stale entries (IPs that haven't connected recently).
    ///
    /// Call this periodically to prevent memory growth from ephemeral IPs.
    pub fn cleanup_stale(&self, max_entries: usize) {
        if self.limiters.len() > max_entries {
            // Simple strategy: remove oldest entries until we're under the limit.
            // Since DashMap doesn't track insertion order, we just remove random entries.
            let to_remove = self.limiters.len() - max_entries;
            let keys: Vec<_> = self
                .limiters
                .iter()
                .take(to_remove)
                .map(|e| *e.key())
                .collect();
            for key in keys {
                self.limiters.remove(&key);
            }
        }
    }
}

/// Per-connection message rate limiter.
///
/// Wraps a simple rate limiter to track message rates for a single connection.
#[derive(Clone)]
pub struct ConnRateLimiter {
    limiter: Arc<SimpleRateLimiter>,
}

impl ConnRateLimiter {
    /// Create a new per-connection rate limiter with the given configuration.
    pub fn new(config: &RateLimitConfig) -> Option<Self> {
        if !config.msg_limit_enabled() {
            return None;
        }

        let burst = config.msg_per_conn_per_sec * config.burst_multiplier;
        let quota = Quota::per_second(NonZeroU32::new(config.msg_per_conn_per_sec)?)
            .allow_burst(NonZeroU32::new(burst)?);

        Some(Self {
            limiter: Arc::new(RateLimiter::direct(quota)),
        })
    }

    /// Check if a message should be allowed.
    ///
    /// Returns `true` if allowed, `false` if rate limited.
    pub fn check(&self) -> bool {
        self.limiter.check().is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    #[test]
    fn test_ip_rate_limiter_allows_within_limit() {
        let config = RateLimitConfig {
            conn_per_ip_per_sec: 5,
            msg_per_conn_per_sec: 100,
            burst_multiplier: 2,
        };
        let limiter = IpRateLimiter::new(config);
        let ip = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));

        // Should allow burst of 10 (5 * 2)
        for _ in 0..10 {
            assert!(limiter.check(ip));
        }
    }

    #[test]
    fn test_ip_rate_limiter_rejects_over_limit() {
        let config = RateLimitConfig {
            conn_per_ip_per_sec: 2,
            msg_per_conn_per_sec: 100,
            burst_multiplier: 1,
        };
        let limiter = IpRateLimiter::new(config);
        let ip = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));

        // Should allow burst of 2
        assert!(limiter.check(ip));
        assert!(limiter.check(ip));
        // Third should be rejected
        assert!(!limiter.check(ip));
    }

    #[test]
    fn test_conn_rate_limiter_allows_within_limit() {
        let config = RateLimitConfig {
            conn_per_ip_per_sec: 10,
            msg_per_conn_per_sec: 5,
            burst_multiplier: 2,
        };
        let limiter = ConnRateLimiter::new(&config).unwrap();

        // Should allow burst of 10 (5 * 2)
        for _ in 0..10 {
            assert!(limiter.check());
        }
    }

    #[test]
    fn test_disabled_rate_limiting() {
        let config = RateLimitConfig {
            conn_per_ip_per_sec: 0,
            msg_per_conn_per_sec: 0,
            burst_multiplier: 3,
        };

        // IP limiter should always allow when disabled
        let ip_limiter = IpRateLimiter::new(config.clone());
        let ip = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
        for _ in 0..1000 {
            assert!(ip_limiter.check(ip));
        }

        // Conn limiter should return None when disabled
        assert!(ConnRateLimiter::new(&config).is_none());
    }
}
