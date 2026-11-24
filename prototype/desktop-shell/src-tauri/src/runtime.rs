//! Local inference runtime contracts and safe defaults.
//!
//! This module intentionally does not launch a model process yet. It defines
//! the validated configuration boundary for the upcoming llama.cpp adapter.

use std::net::IpAddr;

/// Runtime implementation selected by the application.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeKind {
    LlamaCppServer,
}

/// High-level runtime lifecycle state exposed to the application.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeStatus {
    NotInstalled,
    Stopped,
    Starting,
    Ready,
    Generating,
    Failed,
}

/// Loopback-only server configuration for a local inference process.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeConfig {
    pub host: String,
    pub port: u16,
    pub context_tokens: u32,
    pub cpu_threads: Option<u16>,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_owned(),
            port: 9931,
            context_tokens: 2_048,
            cpu_threads: None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeConfigError {
    InvalidAddress,
    NonLoopbackAddress,
    ZeroContextLength,
    ZeroCpuThreads,
}

impl RuntimeConfig {
    /// Reject invalid or non-loopback bind addresses before a server is started.
    pub fn validate(&self) -> Result<(), RuntimeConfigError> {
        let address: IpAddr = self
            .host
            .parse()
            .map_err(|_| RuntimeConfigError::InvalidAddress)?;

        if !address.is_loopback() {
            return Err(RuntimeConfigError::NonLoopbackAddress);
        }

        if self.context_tokens == 0 {
            return Err(RuntimeConfigError::ZeroContextLength);
        }

        if self.cpu_threads == Some(0) {
            return Err(RuntimeConfigError::ZeroCpuThreads);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_configuration_is_valid_and_loopback_only() {
        assert_eq!(RuntimeConfig::default().validate(), Ok(()));
    }

    #[test]
    fn ipv6_loopback_is_valid() {
        let config = RuntimeConfig {
            host: "::1".to_owned(),
            ..RuntimeConfig::default()
        };
        assert_eq!(config.validate(), Ok(()));
    }

    #[test]
    fn public_bind_addresses_are_rejected() {
        let config = RuntimeConfig {
            host: "0.0.0.0".to_owned(),
            ..RuntimeConfig::default()
        };
        assert_eq!(
            config.validate(),
            Err(RuntimeConfigError::NonLoopbackAddress)
        );
    }

    #[test]
    fn invalid_addresses_are_rejected() {
        let config = RuntimeConfig {
            host: "localhost; rm -rf /".to_owned(),
            ..RuntimeConfig::default()
        };
        assert_eq!(config.validate(), Err(RuntimeConfigError::InvalidAddress));
    }

    #[test]
    fn zero_context_is_rejected() {
        let config = RuntimeConfig {
            context_tokens: 0,
            ..RuntimeConfig::default()
        };
        assert_eq!(
            config.validate(),
            Err(RuntimeConfigError::ZeroContextLength)
        );
    }

    #[test]
    fn zero_cpu_threads_is_rejected_when_explicit() {
        let config = RuntimeConfig {
            cpu_threads: Some(0),
            ..RuntimeConfig::default()
        };
        assert_eq!(
            config.validate(),
            Err(RuntimeConfigError::ZeroCpuThreads)
        );
    }
}
