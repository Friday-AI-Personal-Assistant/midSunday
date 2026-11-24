mod llama_client;
mod llama_process;
mod llama_runtime;
mod runtime;
mod runtime_manager;

use std::sync::Mutex;
use tauri::State;
use runtime::{GenerationRequest, RuntimeConfig};
use runtime_manager::{RuntimeManager, RuntimeStatusSnapshot};

#[tauri::command]
fn get_app_status() -> &'static str {
    "The midSunday Rust core is running. The desktop shell is connected."
}
#[tauri::command]
fn get_runtime_status(manager: State<'_, Mutex<RuntimeManager>>) -> Result<RuntimeStatusSnapshot, String> {
    Ok(manager.lock().map_err(|_| "Runtime manager unavailable".to_owned())?.snapshot())
}
#[tauri::command]
fn start_local_runtime(
    manager: State<'_, Mutex<RuntimeManager>>, executable_path: String, model_path: String,
    host: String, port: u16, context_tokens: u32, cpu_threads: Option<u16>,
) -> Result<RuntimeStatusSnapshot, String> {
    let mut m=manager.lock().map_err(|_| "Runtime manager unavailable".to_owned())?;
    m.start(RuntimeConfig{host,port,context_tokens,cpu_threads},&executable_path,&model_path,
        std::time::Duration::from_secs(60),std::time::Duration::from_millis(250),std::time::Duration::from_secs(15))?;
    Ok(m.snapshot())
}
#[tauri::command]
fn generate_local_response(
    manager: State<'_, Mutex<RuntimeManager>>, prompt: String, max_tokens: u32, temperature: f32, top_p: f32,
) -> Result<String,String> {
    manager.lock().map_err(|_| "Runtime manager unavailable".to_owned())?
        .generate(GenerationRequest{prompt,max_tokens,temperature,top_p})
}
#[tauri::command]
fn stop_local_runtime(manager: State<'_, Mutex<RuntimeManager>>) -> Result<RuntimeStatusSnapshot,String> {
    let mut m=manager.lock().map_err(|_| "Runtime manager unavailable".to_owned())?;
    m.stop()?; Ok(m.snapshot())
}
#[tauri::command]
fn recover_local_runtime(
    manager: State<'_, Mutex<RuntimeManager>>, executable_path: String, model_path: String,
) -> Result<RuntimeStatusSnapshot,String> {
    let mut m=manager.lock().map_err(|_| "Runtime manager unavailable".to_owned())?;
    m.recover(&executable_path,&model_path,std::time::Duration::from_secs(60),std::time::Duration::from_millis(250))?;
    Ok(m.snapshot())
}
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(Mutex::new(RuntimeManager::default()))
        .invoke_handler(tauri::generate_handler![
            get_app_status,get_runtime_status,start_local_runtime,generate_local_response,
            stop_local_runtime,recover_local_runtime
        ])
        .run(tauri::generate_context!())
        .expect("error while running midSunday");
}
