use std::{path::Path, time::Duration};
use serde::Serialize;
use crate::llama_runtime::{LlamaRuntimeSession, RecoveryPolicy};
use crate::runtime::{GenerationRequest, RuntimeConfig, RuntimeStatus};

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeStatusSnapshot { pub status: String, pub detail: Option<String> }

pub struct RuntimeManager {
    session: Option<LlamaRuntimeSession>,
    status: RuntimeStatus,
    detail: Option<String>,
}
impl Default for RuntimeManager {
    fn default() -> Self {
        Self { session: None, status: RuntimeStatus::NotInstalled, detail: None }
    }
}
impl RuntimeManager {
    pub fn snapshot(&self) -> RuntimeStatusSnapshot {
        RuntimeStatusSnapshot { status: status_name(self.status).into(), detail: self.detail.clone() }
    }
    pub fn start(&mut self, config: RuntimeConfig, executable: &str, model: &str,
        startup: Duration, poll: Duration, request: Duration) -> Result<(), String> {
        if self.session.is_some() { return Err("A runtime session already exists; stop it or use recovery first.".into()); }
        self.status = RuntimeStatus::Starting;
        self.detail = None;
        match LlamaRuntimeSession::start(&config, Path::new(executable), Path::new(model), startup, poll, request) {
            Ok(session) => { self.session = Some(session); self.status = RuntimeStatus::Ready; Ok(()) }
            Err(error) => { self.status = RuntimeStatus::Failed; let d = format!("{error:?}"); self.detail = Some(d.clone()); Err(d) }
        }
    }
    pub fn generate(&mut self, request: GenerationRequest) -> Result<String, String> {
        let result = self.session.as_mut().ok_or_else(|| "No local runtime session is active.".to_owned())?.generate(&request);
        match result {
            Ok(text) => { self.status = self.session.as_ref().unwrap().status(); self.detail = None; Ok(text) }
            Err(error) => { self.status = self.session.as_ref().unwrap().status(); let d = format!("{error:?}"); self.detail = Some(d.clone()); Err(d) }
        }
    }
    pub fn stop(&mut self) -> Result<(), String> {
        if self.session.is_none() {
            if self.status != RuntimeStatus::NotInstalled { self.status = RuntimeStatus::Stopped; self.detail = None; }
            return Ok(());
        }
        let result = self.session.as_mut().unwrap().stop();
        match result {
            Ok(()) => { self.session = None; self.status = RuntimeStatus::Stopped; self.detail = None; Ok(()) }
            Err(error) => { self.status = self.session.as_ref().unwrap().status(); let d = format!("{error:?}"); self.detail = Some(d.clone()); Err(d) }
        }
    }
    pub fn recover(&mut self, executable: &str, model: &str, startup: Duration, poll: Duration) -> Result<(), String> {
        let result = self.session.as_mut().ok_or_else(|| "There is no existing session to recover.".to_owned())?
            .recover(Path::new(executable), Path::new(model), startup, poll, RecoveryPolicy::default());
        match result {
            Ok(()) => { self.status = self.session.as_ref().unwrap().status(); self.detail = None; Ok(()) }
            Err(error) => { self.status = self.session.as_ref().unwrap().status(); let d = format!("{error:?}"); self.detail = Some(d.clone()); Err(d) }
        }
    }
}
fn status_name(s: RuntimeStatus) -> &'static str {
    match s { RuntimeStatus::NotInstalled=>"notInstalled", RuntimeStatus::Stopped=>"stopped",
        RuntimeStatus::Starting=>"starting", RuntimeStatus::Ready=>"ready",
        RuntimeStatus::Generating=>"generating", RuntimeStatus::Failed=>"failed" }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn starts_not_installed() {
        let m=RuntimeManager::default(); assert_eq!(m.snapshot().status,"notInstalled");
    }
    #[test] fn rejects_generation_without_session() {
        let mut m=RuntimeManager::default();
        let r=GenerationRequest{prompt:"hi".into(),max_tokens:16,temperature:0.7,top_p:0.9};
        assert!(m.generate(r).unwrap_err().contains("No local runtime"));
    }
    #[test] fn invalid_configuration_fails_without_launching_a_process() {
        let mut m=RuntimeManager::default();
        let c=RuntimeConfig{host:"0.0.0.0".into(),..RuntimeConfig::default()};
        assert!(m.start(c,"missing","missing",Duration::from_secs(1),Duration::from_millis(1),Duration::from_secs(1)).is_err());
        assert_eq!(m.snapshot().status,"failed");
    }
}
