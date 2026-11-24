//! Local inference runtime contracts and safe defaults.
//!
//! This module defines the boundary for a future local inference adapter.
//! It does not launch a process, open a socket, download a model, or perform
//! inference. Those behaviours will be implemented only after the lifecycle
//! and security contracts are tested.

use std::net::IpAddr;
use std::path::Path;
use serde::Deserialize;

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

/// A prior user or assistant turn supplied as context for the next response.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ConversationRole { User, Assistant }

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct ConversationMessage { pub role: ConversationRole, pub content: String }

/// A bounded request passed from the application coordinator to a runtime.
#[derive(Clone, Debug, PartialEq)]
pub struct GenerationRequest {
    pub prompt: String,
    pub history: Vec<ConversationMessage>,
    pub max_tokens: u32,
    pub temperature: f32,
    pub top_p: f32,
}

impl GenerationRequest {
    pub fn validate(&self) -> Result<(), GenerationRequestError> {
        if self.prompt.trim().is_empty() {
            return Err(GenerationRequestError::EmptyPrompt);
        }
        if self.prompt.len() > 32_768 {
            return Err(GenerationRequestError::PromptTooLarge);
        }
        if self.history.len() > 64 { return Err(GenerationRequestError::TooManyHistoryMessages); }
        let mut history_bytes = 0usize;
        for message in &self.history {
            if message.content.trim().is_empty() || message.content.len() > 32_768 { return Err(GenerationRequestError::InvalidHistoryMessage); }
            history_bytes = history_bytes.saturating_add(message.content.len());
        }
        if history_bytes > 131_072 { return Err(GenerationRequestError::HistoryTooLarge); }
        if self.max_tokens == 0 || self.max_tokens > 8_192 {
            return Err(GenerationRequestError::InvalidTokenLimit);
        }
        if !self.temperature.is_finite() || !(0.0..=2.0).contains(&self.temperature) {
            return Err(GenerationRequestError::InvalidTemperature);
        }
        if !self.top_p.is_finite() || !(0.0..=1.0).contains(&self.top_p) {
            return Err(GenerationRequestError::InvalidTopP);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GenerationRequestError {
    EmptyPrompt,
    PromptTooLarge,
    InvalidTokenLimit,
    InvalidTemperature,
    InvalidTopP,
    TooManyHistoryMessages,
    InvalidHistoryMessage,
    HistoryTooLarge,
}

/// Errors exposed by a concrete runtime adapter.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeError {
    InvalidConfiguration,
    InvalidRequest,
    InvalidState,
    ProcessUnavailable,
    CommunicationFailed,
}

/// The lifecycle state machine prevents impossible runtime transitions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RuntimeLifecycle {
    status: RuntimeStatus,
}

impl Default for RuntimeLifecycle {
    fn default() -> Self {
        Self {
            status: RuntimeStatus::NotInstalled,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeTransitionError {
    InvalidTransition {
        from: RuntimeStatus,
        to: RuntimeStatus,
    },
}

impl RuntimeLifecycle {
    pub fn status(&self) -> RuntimeStatus {
        self.status
    }

    pub fn transition(
        &mut self,
        next: RuntimeStatus,
    ) -> Result<(), RuntimeTransitionError> {
        use RuntimeStatus::*;

        let allowed = matches!(
            (self.status, next),
            (NotInstalled, Stopped)
                | (Stopped, Starting)
                | (Starting, Ready)
                | (Starting, Failed)
                | (Starting, Stopped)
                | (Ready, Generating)
                | (Ready, Stopped)
                | (Ready, Failed)
                | (Generating, Ready)
                | (Generating, Stopped)
                | (Generating, Failed)
                | (Failed, Stopped)
                | (Failed, Starting)
        );

        if !allowed {
            return Err(RuntimeTransitionError::InvalidTransition {
                from: self.status,
                to: next,
            });
        }

        self.status = next;
        Ok(())
    }
}

/// Replaceable boundary for local model engines.
///
/// Implementations must use validated paths and argument arrays, bind only to
/// loopback, never invoke a shell, and never forward prompts outside the device.
pub trait LocalModelRuntime {
    fn status(&self) -> RuntimeStatus;
    fn start(&mut self, config: &RuntimeConfig, model_path: &Path)
        -> Result<(), RuntimeError>;
    fn generate(&mut self, request: &GenerationRequest) -> Result<String, RuntimeError>;
    fn stop(&mut self) -> Result<(), RuntimeError>;
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

    fn valid_request() -> GenerationRequest {
        GenerationRequest {
            history: Vec::new(),
            prompt: "Hello, midSunday.".to_owned(),
            max_tokens: 256,
            temperature: 0.7,
            top_p: 0.9,
        }
    }

    #[test]
    fn valid_generation_request_is_accepted() {
        assert_eq!(valid_request().validate(), Ok(()));
    }

    #[test]
    fn empty_generation_prompt_is_rejected() {
        let mut request = valid_request();
        request.prompt = "   ".to_owned();
        assert_eq!(
            request.validate(),
            Err(GenerationRequestError::EmptyPrompt)
        );
    }

    #[test]
    fn oversized_generation_prompt_is_rejected() {
        let mut request = valid_request();
        request.prompt = "x".repeat(32_769);
        assert_eq!(request.validate(), Err(GenerationRequestError::PromptTooLarge));
    }

    #[test]
    fn generation_token_limit_is_bounded() {
        let mut request = valid_request();
        request.max_tokens = 8_193;
        assert_eq!(
            request.validate(),
            Err(GenerationRequestError::InvalidTokenLimit)
        );
    }

    #[test]
    fn non_finite_temperature_is_rejected() {
        let mut request = valid_request();
        request.temperature = f32::NAN;
        assert_eq!(
            request.validate(),
            Err(GenerationRequestError::InvalidTemperature)
        );
    }

    #[test]
    fn top_p_outside_supported_range_is_rejected() {
        let mut request = valid_request();
        request.top_p = 1.1;
        assert_eq!(request.validate(), Err(GenerationRequestError::InvalidTopP));
    }


    #[test]
    fn generation_history_is_bounded() {
        let mut request = valid_request();
        request.history = (0..65).map(|_| ConversationMessage { role: ConversationRole::User, content: "hi".to_owned() }).collect();
        assert_eq!(request.validate(), Err(GenerationRequestError::TooManyHistoryMessages));
    }

    #[test]
    fn empty_history_message_is_rejected() {
        let mut request = valid_request();
        request.history.push(ConversationMessage { role: ConversationRole::Assistant, content: "  ".to_owned() });
        assert_eq!(request.validate(), Err(GenerationRequestError::InvalidHistoryMessage));
    }

    #[test]
    fn lifecycle_accepts_a_normal_generation_cycle() {
        let mut lifecycle = RuntimeLifecycle::default();
        assert_eq!(lifecycle.transition(RuntimeStatus::Stopped), Ok(()));
        assert_eq!(lifecycle.transition(RuntimeStatus::Starting), Ok(()));
        assert_eq!(lifecycle.transition(RuntimeStatus::Ready), Ok(()));
        assert_eq!(lifecycle.transition(RuntimeStatus::Generating), Ok(()));
        assert_eq!(lifecycle.transition(RuntimeStatus::Ready), Ok(()));
        assert_eq!(lifecycle.status(), RuntimeStatus::Ready);
    }

    #[test]
    fn lifecycle_rejects_generation_before_runtime_is_ready() {
        let mut lifecycle = RuntimeLifecycle::default();
        assert_eq!(
            lifecycle.transition(RuntimeStatus::Generating),
            Err(RuntimeTransitionError::InvalidTransition {
                from: RuntimeStatus::NotInstalled,
                to: RuntimeStatus::Generating,
            })
        );
    }

    #[test]
    fn lifecycle_allows_recovery_after_failure() {
        let mut lifecycle = RuntimeLifecycle::default();
        lifecycle.transition(RuntimeStatus::Stopped).unwrap();
        lifecycle.transition(RuntimeStatus::Starting).unwrap();
        lifecycle.transition(RuntimeStatus::Failed).unwrap();
        lifecycle.transition(RuntimeStatus::Stopped).unwrap();
        lifecycle.transition(RuntimeStatus::Starting).unwrap();
        assert_eq!(lifecycle.status(), RuntimeStatus::Starting);
    }
}
