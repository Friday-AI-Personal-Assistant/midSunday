mod llama_client;
mod llama_process;
mod runtime;

#[tauri::command]
fn get_app_status() -> &'static str {
    "The midSunday Rust core is running. The desktop shell is connected."
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![get_app_status])
        .run(tauri::generate_context!())
        .expect("error while running midSunday");
}
