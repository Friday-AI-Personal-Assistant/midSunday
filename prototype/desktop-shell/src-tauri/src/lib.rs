mod llama_client;
mod llama_process;
mod llama_runtime;
mod runtime;
mod runtime_manager;

use tauri::{ipc::Channel, State};
use runtime::{ConversationMessage, GenerationRequest, RuntimeConfig};
use runtime_manager::{RuntimeManager, RuntimeStatusSnapshot};

#[tauri::command]
fn get_app_status() -> &'static str {
    "The midSunday Rust core is running. The desktop shell is connected."
}

#[tauri::command]
fn get_runtime_status(manager: State<'_, RuntimeManager>) -> Result<RuntimeStatusSnapshot, String> {
    manager.snapshot()
}

#[tauri::command]
async fn start_local_runtime(
    manager: State<'_, RuntimeManager>,
    executable_path: String,
    model_path: String,
    host: String,
    port: u16,
    context_tokens: u32,
    cpu_threads: Option<u16>,
) -> Result<RuntimeStatusSnapshot, String> {
    let manager = manager.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        manager.start(
            RuntimeConfig { host, port, context_tokens, cpu_threads },
            &executable_path,
            &model_path,
            std::time::Duration::from_secs(60),
            std::time::Duration::from_millis(250),
            std::time::Duration::from_secs(15),
        )?;
        manager.snapshot()
    })
    .await
    .map_err(|error| format!("Runtime worker failed: {error}"))?
}

#[tauri::command]
async fn generate_local_response(
    manager: State<'_, RuntimeManager>,
    prompt: String,
    history: Vec<ConversationMessage>,
    max_tokens: u32,
    temperature: f32,
    top_p: f32,
    on_delta: Channel<String>,
) -> Result<(), String> {
    let manager = manager.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        manager.generate_with_callback(GenerationRequest { prompt, history, max_tokens, temperature, top_p }, |delta| {
            let _ = on_delta.send(delta.to_owned());
        })
    })
    .await
    .map_err(|error| format!("Generation worker failed: {error}"))?
}

#[tauri::command]
fn cancel_local_generation(manager: State<'_, RuntimeManager>) -> Result<bool, String> {
    manager.cancel_generation()
}

#[tauri::command]
async fn stop_local_runtime(
    manager: State<'_, RuntimeManager>,
) -> Result<RuntimeStatusSnapshot, String> {
    let manager = manager.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        manager.stop()?;
        manager.snapshot()
    })
    .await
    .map_err(|error| format!("Stop worker failed: {error}"))?
}

#[tauri::command]
async fn recover_local_runtime(
    manager: State<'_, RuntimeManager>,
    executable_path: String,
    model_path: String,
) -> Result<RuntimeStatusSnapshot, String> {
    let manager = manager.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        manager.recover(
            &executable_path,
            &model_path,
            std::time::Duration::from_secs(60),
            std::time::Duration::from_millis(250),
        )?;
        manager.snapshot()
    })
    .await
    .map_err(|error| format!("Recovery worker failed: {error}"))?
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(RuntimeManager::default())
        .invoke_handler(tauri::generate_handler![
            get_app_status,
            get_runtime_status,
            start_local_runtime,
            generate_local_response,
            cancel_local_generation,
            stop_local_runtime,
            recover_local_runtime
        ])
        .run(tauri::generate_context!())
        .expect("error while running midSunday");
}
