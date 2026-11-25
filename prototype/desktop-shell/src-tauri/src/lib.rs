mod conversation_store;
mod companion_settings;
mod engine_settings;
mod llama_client;
mod model_catalog;
mod llama_process;
mod llama_runtime;
mod runtime;
mod runtime_manager;

use tauri::{ipc::Channel, State};
use runtime::{ConversationMessage, GenerationRequest, RuntimeConfig};
use runtime_manager::{RuntimeManager, RuntimeStatusSnapshot};
use conversation_store::{AwarenessHistoryEntry, AwarenessItem, AwarenessSettings, AwarenessSuggestion, AwarenessSuppression, ConversationStore, ConversationSummary, ImportResult, SavedMemory, StoredMessage, UserProfile};
use engine_settings::{EngineSettings, EngineSettingsStore};
use companion_settings::{CompanionSettings, CompanionSettingsStore};
use model_catalog::{LocalModel, ModelCatalog, ModelVerification};
use tauri::Manager;


#[tauri::command]
fn list_conversations(store: State<'_, ConversationStore>) -> Result<Vec<ConversationSummary>, String> {
    store.list()
}

#[tauri::command]
fn create_conversation(store: State<'_, ConversationStore>, title: String) -> Result<ConversationSummary, String> {
    store.create(&title)
}

#[tauri::command]
fn get_conversation_messages(store: State<'_, ConversationStore>, conversation_id: i64) -> Result<Vec<StoredMessage>, String> {
    store.messages(conversation_id)
}

#[tauri::command]
fn append_conversation_message(
    store: State<'_, ConversationStore>, conversation_id: i64, role: String, content: String,
) -> Result<(), String> {
    store.append(conversation_id, &role, &content)
}

#[tauri::command]
fn rename_conversation(
    store: State<'_, ConversationStore>, conversation_id: i64, title: String,
) -> Result<(), String> {
    store.rename(conversation_id, &title)
}

#[tauri::command]
fn delete_conversation(store: State<'_, ConversationStore>, conversation_id: i64) -> Result<(), String> {
    store.delete(conversation_id)
}

#[tauri::command]
fn export_conversations(store: State<'_, ConversationStore>) -> Result<String, String> {
    store.export_json()
}

#[tauri::command]
fn import_conversations(store: State<'_, ConversationStore>, json: String) -> Result<ImportResult, String> {
    store.import_json(&json)
}

#[tauri::command]
fn clear_conversations(store: State<'_, ConversationStore>) -> Result<usize, String> {
    store.clear_all()
}

#[tauri::command]
fn list_saved_memories(store: State<'_, ConversationStore>) -> Result<Vec<SavedMemory>, String> {
    store.list_memories()
}

#[tauri::command]
fn create_saved_memory(store: State<'_, ConversationStore>, content: String) -> Result<SavedMemory, String> {
    store.create_memory(&content)
}

#[tauri::command]
fn update_saved_memory(store: State<'_, ConversationStore>, id: i64, content: String) -> Result<(), String> {
    store.update_memory(id, &content)
}

#[tauri::command]
fn delete_saved_memory(store: State<'_, ConversationStore>, id: i64) -> Result<(), String> {
    store.delete_memory(id)
}

#[tauri::command]
fn clear_saved_memories(store: State<'_, ConversationStore>) -> Result<usize, String> {
    store.clear_memories()
}

#[tauri::command]
fn get_memory_enabled(store: State<'_, ConversationStore>) -> Result<bool, String> {
    store.memory_enabled()
}

#[tauri::command]
fn set_memory_enabled(store: State<'_, ConversationStore>, enabled: bool) -> Result<bool, String> {
    store.set_memory_enabled(enabled)
}

#[tauri::command]
fn export_saved_memories(store: State<'_, ConversationStore>) -> Result<String, String> {
    store.export_memories()
}

#[tauri::command]
fn list_awareness_history(store: State<'_, ConversationStore>) -> Result<Vec<AwarenessHistoryEntry>, String> {
    store.list_awareness_history()
}

#[tauri::command]
fn clear_awareness_history(store: State<'_, ConversationStore>) -> Result<usize, String> {
    store.clear_awareness_history()
}

#[tauri::command]
fn list_awareness_suppressions(store: State<'_, ConversationStore>) -> Result<Vec<AwarenessSuppression>, String> {
    store.list_awareness_suppressions()
}

#[tauri::command]
fn clear_awareness_suppressions(store: State<'_, ConversationStore>) -> Result<usize, String> {
    store.clear_awareness_suppressions()
}

#[tauri::command]
fn list_awareness_suggestions(store: State<'_, ConversationStore>) -> Result<Vec<AwarenessSuggestion>, String> {
    store.list_awareness_suggestions()
}

#[tauri::command]
fn approve_awareness_suggestion(store: State<'_, ConversationStore>, id: i64) -> Result<AwarenessItem, String> {
    store.approve_awareness_suggestion(id)
}

#[tauri::command]
fn reject_awareness_suggestion(store: State<'_, ConversationStore>, id: i64) -> Result<(), String> {
    store.reject_awareness_suggestion(id)
}

#[tauri::command]
fn list_awareness_items(store: State<'_, ConversationStore>) -> Result<Vec<AwarenessItem>, String> {
    store.list_awareness_items()
}

#[tauri::command]
fn create_awareness_item(store: State<'_, ConversationStore>, content: String, category: String) -> Result<AwarenessItem, String> {
    store.create_awareness_item(&content, &category)
}

#[tauri::command]
fn update_awareness_item(store: State<'_, ConversationStore>, id: i64, content: String, category: String) -> Result<(), String> {
    store.update_awareness_item(id, &content, &category)
}

#[tauri::command]
fn delete_awareness_item(store: State<'_, ConversationStore>, id: i64) -> Result<(), String> {
    store.delete_awareness_item(id)
}

#[tauri::command]
fn clear_awareness_items(store: State<'_, ConversationStore>) -> Result<usize, String> {
    store.clear_awareness_items()
}

#[tauri::command]
fn export_awareness_items(store: State<'_, ConversationStore>) -> Result<String, String> {
    store.export_awareness_items()
}

#[tauri::command]
fn record_learned_awareness_item(
    store: State<'_, ConversationStore>,
    content: String,
    category: String,
    source_type: String,
    source_record_id: Option<i64>,
    knowledge_type: String,
    confidence: f64,
) -> Result<Option<AwarenessItem>, String> {
    store.record_learned_awareness_item(
        &content, &category, &source_type, source_record_id, &knowledge_type, confidence,
    )
}

#[tauri::command]
fn get_awareness_settings(store: State<'_, ConversationStore>) -> Result<AwarenessSettings, String> {
    store.get_awareness_settings()
}

#[tauri::command]
fn save_awareness_settings(
    store: State<'_, ConversationStore>,
    settings: AwarenessSettings,
) -> Result<AwarenessSettings, String> {
    store.save_awareness_settings(settings)
}

#[tauri::command]
fn get_user_profile(store: State<'_, ConversationStore>) -> Result<UserProfile, String> {
    store.get_user_profile()
}

#[tauri::command]
fn save_user_profile(store: State<'_, ConversationStore>, profile: UserProfile) -> Result<UserProfile, String> {
    store.save_user_profile(profile)
}

#[tauri::command]
fn list_local_models(catalog: State<'_, ModelCatalog>) -> Result<Vec<LocalModel>, String> { catalog.list() }

#[tauri::command]
fn register_local_model(catalog: State<'_, ModelCatalog>, file_path: String, display_name: String, source: String, license: String) -> Result<LocalModel, String> {
    catalog.register(&file_path, &display_name, &source, &license)
}

#[tauri::command]
fn remove_local_model(catalog: State<'_, ModelCatalog>, id: String) -> Result<bool, String> { catalog.remove(&id) }

#[tauri::command]
fn verify_local_model(catalog: State<'_, ModelCatalog>, id: String) -> Result<ModelVerification, String> {
    catalog.verify(&id)
}

#[tauri::command]
fn get_app_status() -> &'static str {
    "The midSunday Rust core is running. The desktop shell is connected."
}

#[tauri::command]
fn get_engine_settings(settings: State<'_, EngineSettingsStore>) -> Result<EngineSettings, String> {
    settings.get()
}

#[tauri::command]
fn save_engine_settings(settings: State<'_, EngineSettingsStore>, value: EngineSettings) -> Result<EngineSettings, String> {
    settings.save(value.clone())?;
    Ok(value)
}

#[tauri::command]
fn get_companion_settings(store: State<'_, CompanionSettingsStore>) -> Result<CompanionSettings, String> {
    store.get()
}

#[tauri::command]
fn save_companion_settings(
    store: State<'_, CompanionSettingsStore>,
    value: CompanionSettings,
) -> Result<CompanionSettings, String> {
    store.save(value)
}

#[tauri::command]
fn get_runtime_status(manager: State<'_, RuntimeManager>) -> Result<RuntimeStatusSnapshot, String> {
    manager.snapshot()
}

#[tauri::command]
async fn start_local_runtime(
    manager: State<'_, RuntimeManager>,
    settings: State<'_, EngineSettingsStore>,
    catalog: State<'_, ModelCatalog>,
    executable_path: String,
    model_path: String,
    host: String,
    port: u16,
    context_tokens: u32,
    cpu_threads: Option<u16>,
) -> Result<RuntimeStatusSnapshot, String> {
    let verification = catalog.verify_path(&model_path)?;
    if !verification.valid {
        return Err(format!("Model verification failed ({}): {}", verification.status, verification.detail));
    }
    settings.save(EngineSettings { executable_path: executable_path.clone(), model_path: model_path.clone(), context_tokens, cpu_threads })?;
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
    system_prompt: String,
    prompt: String,
    history: Vec<ConversationMessage>,
    max_tokens: u32,
    temperature: f32,
    top_p: f32,
    on_delta: Channel<String>,
) -> Result<(), String> {
    let manager = manager.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        manager.generate_with_callback(GenerationRequest { system_prompt, prompt, history, max_tokens, temperature, top_p }, |delta| {
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
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            let database_path = data_dir.join("midsunday.sqlite3");
            let store = ConversationStore::open(&database_path)
                .map_err(|error| std::io::Error::new(std::io::ErrorKind::Other, error))?;
            let settings = EngineSettingsStore::open(&data_dir.join("engine-settings.json"))
                .map_err(|error| std::io::Error::new(std::io::ErrorKind::Other, error))?;
            let models = ModelCatalog::open(&data_dir.join("models.json"))
                .map_err(|error| std::io::Error::new(std::io::ErrorKind::Other, error))?;
            let companions = CompanionSettingsStore::open(&data_dir.join("companion-settings.json"))
                .map_err(|error| std::io::Error::new(std::io::ErrorKind::Other, error))?;
            app.manage(store);
            app.manage(settings);
            app.manage(models);
            app.manage(companions);
            Ok(())
        })
        .plugin(tauri_plugin_dialog::init())
        .manage(RuntimeManager::default())
        .invoke_handler(tauri::generate_handler![
            get_app_status,
            list_local_models,
            register_local_model,
            remove_local_model,
            verify_local_model,
            list_conversations,
            create_conversation,
            get_conversation_messages,
            append_conversation_message,
            rename_conversation,
            delete_conversation,
            export_conversations,
            import_conversations,
            clear_conversations,
            list_saved_memories,
            create_saved_memory,
            update_saved_memory,
            delete_saved_memory,
            clear_saved_memories,
            get_memory_enabled,
            set_memory_enabled,
            export_saved_memories,
            get_awareness_settings,
            save_awareness_settings,
            record_learned_awareness_item,
            list_awareness_items,
            list_awareness_suggestions,
            approve_awareness_suggestion,
            reject_awareness_suggestion,
            list_awareness_history,
            clear_awareness_history,
            list_awareness_suppressions,
            clear_awareness_suppressions,
            create_awareness_item,
            update_awareness_item,
            delete_awareness_item,
            clear_awareness_items,
            export_awareness_items,
            get_user_profile,
            save_user_profile,
            get_engine_settings,
            save_engine_settings,
            get_companion_settings,
            save_companion_settings,
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
