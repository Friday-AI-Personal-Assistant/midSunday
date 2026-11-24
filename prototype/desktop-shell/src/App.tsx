import { useEffect, useRef, useState, type FormEvent, type KeyboardEvent } from "react";
import { Channel, invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";

type BackendState = "checking" | "connected" | "error";
type Section = "chat" | "characters" | "settings";
type Message = { id: number; role: "user" | "assistant" | "system"; text: string };
type RuntimeStatusSnapshot = { status: string; detail: string | null };
type EngineSettings = { executablePath: string; modelPath: string; contextTokens: number; cpuThreads: number | null };
type LocalModel = { id: string; displayName: string; filePath: string; sizeBytes: number; sha256: string; source: string; license: string; addedAt: number };
type ConversationSummary = { id: number; title: string; createdAt: number; updatedAt: number };
type StoredMessage = { id: number; conversationId: number; role: "user" | "assistant"; content: string; createdAt: number };

const MAX_HISTORY_MESSAGES = 64;
const MAX_HISTORY_MESSAGE_BYTES = 32_768;
const MAX_HISTORY_TOTAL_BYTES = 131_072;
const MAX_PROMPT_BYTES = 32_768;

function buildBoundedHistory(messages: Message[]) {
  const encoder = new TextEncoder();
  const selected: Array<{ role: "user" | "assistant"; content: string }> = [];
  let totalBytes = 0;

  for (const message of [...messages].reverse()) {
    if (message.role !== "user" && message.role !== "assistant") continue;
    if (!message.text.trim()) continue;

    const byteLength = encoder.encode(message.text).byteLength;
    if (byteLength > MAX_HISTORY_MESSAGE_BYTES) continue;
    if (totalBytes + byteLength > MAX_HISTORY_TOTAL_BYTES) continue;

    selected.push({ role: message.role, content: message.text });
    totalBytes += byteLength;
    if (selected.length >= MAX_HISTORY_MESSAGES) break;
  }

  return selected.reverse();
}

const personalities = [
  { name: "Best friend", detail: "Easygoing, honest and supportive", mark: "B" },
  { name: "Mentor", detail: "Thoughtful guidance and perspective", mark: "M" },
  { name: "Professional", detail: "Focused, clear and organised", mark: "P" },
  { name: "Custom character", detail: "Shape a companion of your own", mark: "+" },
];

export default function App() {
  const [backendState, setBackendState] = useState<BackendState>("checking");
  const [backendMessage, setBackendMessage] = useState("Checking the local Rust core…");
  const [section, setSection] = useState<Section>("chat");
  const [personality, setPersonality] = useState("Best friend");
  const [draft, setDraft] = useState("");
  const [messages, setMessages] = useState<Message[]>([]);
  const [conversations, setConversations] = useState<ConversationSummary[]>([]);
  const [activeConversationId, setActiveConversationId] = useState<number | null>(null);
  const [conversationName, setConversationName] = useState("A fresh start");
  const [memoryEnabled, setMemoryEnabled] = useState(false);
  const [startWithWindows, setStartWithWindows] = useState(false);
  const [runtimeStatus, setRuntimeStatus] = useState<RuntimeStatusSnapshot>({ status: "notInstalled", detail: null });
  const [runtimeBusy, setRuntimeBusy] = useState(false);
  const [runtimeError, setRuntimeError] = useState("");
  const [executablePath, setExecutablePath] = useState("");
  const [modelPath, setModelPath] = useState("");
  const [contextTokens, setContextTokens] = useState("2048");
  const [cpuThreads, setCpuThreads] = useState("");
  const [localModels, setLocalModels] = useState<LocalModel[]>([]);
  const [modelDisplayName, setModelDisplayName] = useState("");
  const [modelSource, setModelSource] = useState("");
  const [modelLicense, setModelLicense] = useState("");
  const [modelCatalogBusy, setModelCatalogBusy] = useState(false);
  const [isGenerating, setIsGenerating] = useState(false);
  const [importBusy, setImportBusy] = useState(false);
  const importFileRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    let active = true;
    invoke<string>("get_app_status")
      .then((message) => {
        if (!active) return;
        setBackendMessage(message);
        setBackendState("connected");
      })
      .catch(() => {
        if (!active) return;
        setBackendMessage("The local Rust core could not be reached.");
        setBackendState("error");
      });
    const refreshRuntimeStatus = () => {
      invoke<RuntimeStatusSnapshot>("get_runtime_status")
        .then((snapshot) => { if (active) setRuntimeStatus(snapshot); })
        .catch(() => { if (active) setRuntimeError("Could not read local engine status."); });
    };
    invoke<ConversationSummary[]>("list_conversations")
      .then(async (items) => {
        if (!active || items.length === 0) return;
        setConversations(items);
        const latest = items[0];
        const saved = await invoke<StoredMessage[]>("get_conversation_messages", { conversationId: latest.id });
        if (!active) return;
        setActiveConversationId(latest.id);
        setConversationName(latest.title);
        setMessages(saved.map((message) => ({ id: message.id, role: message.role, text: message.content })));
      })
      .catch(() => { if (active) setRuntimeError("Could not load local conversations."); });
    invoke<LocalModel[]>("list_local_models").then((items) => { if (active) setLocalModels(items); }).catch(() => { if (active) setRuntimeError("Could not load the local model library."); });
    invoke<EngineSettings>("get_engine_settings")
      .then((settings) => {
        if (!active) return;
        setExecutablePath(settings.executablePath);
        setModelPath(settings.modelPath);
        setContextTokens(String(settings.contextTokens));
        setCpuThreads(settings.cpuThreads === null ? "" : String(settings.cpuThreads));
      })
      .catch(() => { if (active) setRuntimeError("Could not load saved local engine settings."); });
    refreshRuntimeStatus();
    const statusTimer = window.setInterval(refreshRuntimeStatus, 1500);
    return () => { active = false; window.clearInterval(statusTimer); };
  }, []);

  function startNewChat() {
    if (isGenerating) void cancelGeneration();
    setMessages([]);
    setActiveConversationId(null);
    setDraft("");
    setConversationName("A fresh start");
    setSection("chat");
  }

  async function openConversation(conversation: ConversationSummary) {
    if (isGenerating) return;
    try {
      const saved = await invoke<StoredMessage[]>("get_conversation_messages", { conversationId: conversation.id });
      setActiveConversationId(conversation.id);
      setConversationName(conversation.title);
      setMessages(saved.map((message) => ({ id: message.id, role: message.role, text: message.content })));
      setDraft("");
      setRuntimeError("");
      setSection("chat");
    } catch (error) { setRuntimeError(`Could not open conversation: ${String(error)}`); }
  }

  async function renameConversation(conversation: ConversationSummary) {
    const title = window.prompt("Rename conversation", conversation.title);
    if (title === null) return;
    try {
      await invoke<void>("rename_conversation", { conversationId: conversation.id, title });
      const items = await invoke<ConversationSummary[]>("list_conversations");
      setConversations(items);
      if (activeConversationId === conversation.id) setConversationName(title.trim());
    } catch (error) { setRuntimeError(`Could not rename conversation: ${String(error)}`); }
  }

  async function deleteConversation(conversation: ConversationSummary) {
    if (!window.confirm(`Delete “${conversation.title}” and its saved messages from this device?`)) return;
    try {
      await invoke<void>("delete_conversation", { conversationId: conversation.id });
      const items = await invoke<ConversationSummary[]>("list_conversations");
      setConversations(items);
      if (activeConversationId === conversation.id) {
        setActiveConversationId(null);
        setMessages([]);
        setConversationName("A fresh start");
      }
    } catch (error) { setRuntimeError(`Could not delete conversation: ${String(error)}`); }
  }

  async function exportConversations() {
    try {
      const json = await invoke<string>("export_conversations");
      const blob = new Blob([json], { type: "application/json;charset=utf-8" });
      const url = URL.createObjectURL(blob);
      const anchor = document.createElement("a");
      anchor.href = url;
      anchor.download = `midsunday-conversations-${new Date().toISOString().slice(0, 10)}.json`;
      document.body.appendChild(anchor);
      anchor.click();
      anchor.remove();
      URL.revokeObjectURL(url);
      setRuntimeError("");
    } catch (error) {
      setRuntimeError(`Could not export conversations: ${String(error)}`);
    }
  }


  async function importConversations(event: React.ChangeEvent<HTMLInputElement>) {
    const file = event.target.files?.[0];
    event.target.value = "";
    if (!file) return;
    if (isGenerating || importBusy) return;
    if (file.size > 20 * 1024 * 1024) {
      setRuntimeError("That backup is larger than the 20 MB import limit.");
      return;
    }
    setImportBusy(true);
    setRuntimeError("");
    try {
      const json = await file.text();
      const result = await invoke<{ imported: number; skippedDuplicates: number }>("import_conversations", { json });
      const items = await invoke<ConversationSummary[]>("list_conversations");
      setConversations(items);
      if (activeConversationId === null && items.length > 0) {
        const latest = items[0];
        const saved = await invoke<StoredMessage[]>("get_conversation_messages", { conversationId: latest.id });
        setActiveConversationId(latest.id);
        setConversationName(latest.title);
        setMessages(saved.map((message) => ({ id: message.id, role: message.role, text: message.content })));
      }
      setRuntimeError(`Import complete: ${result.imported} imported, ${result.skippedDuplicates} exact duplicates skipped.`);
    } catch (error) {
      setRuntimeError(`Could not import conversation backup: ${String(error)}`);
    } finally {
      setImportBusy(false);
    }
  }

  async function clearAllConversations() {
    if (isGenerating) return;
    if (!window.confirm("Permanently delete every saved conversation and message from this device? Export your history first if you want to keep a copy. This does not delete future personal memories.")) return;
    try {
      await invoke<number>("clear_conversations");
      setConversations([]);
      setActiveConversationId(null);
      setMessages([]);
      setConversationName("A fresh start");
      setDraft("");
      setRuntimeError("");
    } catch (error) {
      setRuntimeError(`Could not clear conversation history: ${String(error)}`);
    }
  }

  async function sendMessage(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const text = draft.trim();
    if (!text || isGenerating) return;
    if (runtimeStatus.status !== "ready") {
      setRuntimeError("Start a local model from Settings before sending a message.");
      return;
    }

    if (new TextEncoder().encode(text).byteLength > MAX_PROMPT_BYTES) {
      setRuntimeError("That message is too large. Please keep each message under 32 KB.");
      return;
    }
    const history = buildBoundedHistory(messages);
    let conversationId = activeConversationId;
    try {
      if (conversationId === null) {
        const created = await invoke<ConversationSummary>("create_conversation", { title: text.slice(0, 80) || "New conversation" });
        conversationId = created.id;
        setActiveConversationId(created.id);
      }
      await invoke<void>("append_conversation_message", { conversationId, role: "user", content: text });
      setConversations(await invoke<ConversationSummary[]>("list_conversations"));
    } catch (error) {
      setRuntimeError(`Could not save this conversation locally: ${String(error)}`);
      return;
    }
    const userId = Date.now();
    const assistantId = userId + 1;
    setMessages((current) => [
      ...current,
      { id: userId, role: "user", text },
      { id: assistantId, role: "assistant", text: "" },
    ]);
    setConversationName(text.length > 28 ? text.slice(0, 28) + "…" : text);
    setDraft("");
    setRuntimeError("");
    setIsGenerating(true);

    let responseText = "";
    const channel = new Channel<string>();
    channel.onmessage = (delta) => {
      responseText += delta;
      setMessages((current) => current.map((message) =>
        message.id === assistantId ? { ...message, text: message.text + delta } : message
      ));
    };

    try {
      await invoke<void>("generate_local_response", {
        prompt: text,
        history,
        maxTokens: 512,
        temperature: 0.7,
        topP: 0.9,
        onDelta: channel,
      });
    } catch (error) {
      const detail = String(error);
      setMessages((current) => current.map((message) =>
        message.id === assistantId
          ? { ...message, text: message.text || (detail.includes("cancel") ? "Generation stopped." : "I couldn't generate a response. Check the local engine status in Settings.") }
          : message
      ));
      responseText = responseText || (detail.includes("cancel") ? "Generation stopped." : "I couldn't generate a response. Check the local engine status in Settings.");
      setRuntimeError(detail);
    } finally {
      try {
        if (conversationId !== null && responseText.trim()) {
          await invoke<void>("append_conversation_message", { conversationId, role: "assistant", content: responseText });
          setConversations(await invoke<ConversationSummary[]>("list_conversations"));
        }
      } catch (error) {
        setRuntimeError(`The reply appeared, but could not be saved: ${String(error)}`);
      }
      setIsGenerating(false);
    }
  }

  async function browseForExecutable() {
    try {
      const selected = await open({ multiple: false, directory: false, title: "Select llama-server executable" });
      if (typeof selected === "string") setExecutablePath(selected);
    } catch (error) { setRuntimeError(`Could not open the file picker: ${String(error)}`); }
  }

  async function browseForModel() {
    try {
      const selected = await open({ multiple: false, directory: false, title: "Select GGUF model file", filters: [{ name: "GGUF model", extensions: ["gguf"] }] });
      if (typeof selected === "string") { setModelPath(selected); const fileName = selected.split(/[\\/]/).pop() ?? "Local model"; setModelDisplayName(fileName.replace(/\.gguf$/i, "")); }
    } catch (error) { setRuntimeError(`Could not open the file picker: ${String(error)}`); }
  }


  async function registerLocalModel() {
    if (!modelPath.trim()) { setRuntimeError("Choose an existing GGUF file first."); return; }
    if (!modelDisplayName.trim()) { setRuntimeError("Give this model a name before adding it to the library."); return; }
    setModelCatalogBusy(true); setRuntimeError("");
    try {
      const model = await invoke<LocalModel>("register_local_model", { filePath: modelPath.trim(), displayName: modelDisplayName.trim(), source: modelSource.trim(), license: modelLicense.trim() });
      setLocalModels(await invoke<LocalModel[]>("list_local_models")); setModelPath(model.filePath);
    } catch (error) { setRuntimeError(String(error)); }
    finally { setModelCatalogBusy(false); }
  }
  async function removeLocalModel(model: LocalModel) {
    if (!window.confirm(`Remove "${model.displayName}" from the midSunday library? The GGUF file itself will not be deleted.`)) return;
    try { await invoke<boolean>("remove_local_model", { id: model.id }); setLocalModels(await invoke<LocalModel[]>("list_local_models")); }
    catch (error) { setRuntimeError(`Could not remove model from the library: ${String(error)}`); }
  }
  function formatModelSize(bytes: number) {
    if (bytes >= 1024 ** 3) return `${(bytes / 1024 ** 3).toFixed(2)} GB`;
    if (bytes >= 1024 ** 2) return `${(bytes / 1024 ** 2).toFixed(1)} MB`;
    return `${bytes.toLocaleString()} bytes`;
  }

  async function startLocalRuntime() {
    if (!executablePath.trim() || !modelPath.trim()) {
      setRuntimeError("Enter both the llama-server executable path and the GGUF model path.");
      return;
    }
    const parsedContext = Number(contextTokens);
    const parsedThreads = cpuThreads.trim() === "" ? null : Number(cpuThreads);
    if (!Number.isInteger(parsedContext) || parsedContext < 512 || parsedContext > 32768) {
      setRuntimeError("Context length must be a whole number from 512 to 32,768.");
      return;
    }
    if (parsedThreads !== null && (!Number.isInteger(parsedThreads) || parsedThreads < 1 || parsedThreads > 256)) {
      setRuntimeError("CPU threads must be a whole number from 1 to 256, or left blank for automatic.");
      return;
    }
    setRuntimeBusy(true);
    setRuntimeError("");
    try {
      const snapshot = await invoke<RuntimeStatusSnapshot>("start_local_runtime", {
        executablePath: executablePath.trim(),
        modelPath: modelPath.trim(),
        host: "127.0.0.1",
        port: 9931,
        contextTokens: parsedContext,
        cpuThreads: parsedThreads,
      });
      setRuntimeStatus(snapshot);
    } catch (error) {
      setRuntimeError(String(error));
      const snapshot = await invoke<RuntimeStatusSnapshot>("get_runtime_status").catch(() => null);
      if (snapshot) setRuntimeStatus(snapshot);
    } finally {
      setRuntimeBusy(false);
    }
  }

  async function stopLocalRuntime() {
    setRuntimeBusy(true);
    setRuntimeError("");
    try {
      const snapshot = await invoke<RuntimeStatusSnapshot>("stop_local_runtime");
      setRuntimeStatus(snapshot);
    } catch (error) {
      setRuntimeError(String(error));
    } finally {
      setRuntimeBusy(false);
    }
  }

  async function cancelGeneration() {
    try {
      await invoke<boolean>("cancel_local_generation");
    } catch (error) {
      setRuntimeError(String(error));
    }
  }

  function handleComposerKey(event: KeyboardEvent<HTMLTextAreaElement>) {
    if (event.key === "Enter" && !event.shiftKey) {
      event.preventDefault();
      event.currentTarget.form?.requestSubmit();
    }
  }

  const statusLabel = backendState === "connected" ? "Core connected" : backendState === "checking" ? "Checking core" : "Core unavailable";

  return (
    <main className="workspace">
      <aside className="sidebar">
        <div className="brand-row">
          <div className="brand-mark">m</div>
          <div className="brand-copy"><strong>midSunday</strong><span>AI PERSONAL COMPANION</span></div>
        </div>

        <button className="new-chat-button" onClick={startNewChat}><span>＋</span> New conversation</button>

        <div className="side-label">WORKSPACE</div>
        <nav className="main-nav" aria-label="Main navigation">
          <button className={section === "chat" ? "nav-item active" : "nav-item"} onClick={() => setSection("chat")}><span className="nav-icon">◷</span> Conversations</button>
          <button className={section === "characters" ? "nav-item active" : "nav-item"} onClick={() => setSection("characters")}><span className="nav-icon">✧</span> Companions</button>
          <button className={section === "settings" ? "nav-item active" : "nav-item"} onClick={() => setSection("settings")}><span className="nav-icon">⚙</span> Settings</button>
        </nav>

        <div className="side-label recent-label">RECENT</div>
        <div className="conversation-list">{conversations.slice(0, 8).map((conversation) => <div className="conversation-list-row" key={conversation.id}><button className={activeConversationId === conversation.id ? "recent-chat selected" : "recent-chat"} onClick={() => void openConversation(conversation)}><span className="recent-dot" />{conversation.title}</button><button className="conversation-action" title="Rename conversation" aria-label={`Rename ${conversation.title}`} onClick={() => void renameConversation(conversation)}>✎</button><button className="conversation-action delete" title="Delete conversation" aria-label={`Delete ${conversation.title}`} onClick={() => void deleteConversation(conversation)}>×</button></div>)}</div>

        <div className="sidebar-bottom">
          <div className="privacy-mini"><span className="privacy-icon">⌑</span><div><strong>Private by design</strong><span>Your space stays yours.</span></div></div>
          <div className="profile-row"><div className="profile-avatar">m</div><div className="profile-copy"><strong>Local workspace</strong><span>No account connected</span></div><span className="more-mark">···</span></div>
        </div>
      </aside>

      <section className="main-panel">
        <header className="topbar">
          <div className="breadcrumb"><span>Workspace</span><b>/</b><strong>{section === "chat" ? "Conversations" : section === "characters" ? "Companions" : "Settings"}</strong></div>
          <div className="topbar-right"><span className="offline-pill"><i /> Local-first</span><span className="version-label">0.1.0 · Prototype</span></div>
        </header>

        {section === "chat" && (
          <div className="chat-layout">
            <div className="chat-heading">
              <div><p className="eyebrow">YOUR PERSONAL SPACE</p><h1>{messages.length ? conversationName : "Good to have you here."}</h1><p className="heading-subtitle">A little space to think, plan, and talk things through.</p></div>
              <button className="icon-button" title="Start a new conversation" onClick={startNewChat}>＋</button>
            </div>

            <div className="conversation-area">
              {messages.length === 0 ? (
                <div className="empty-state">
                  <div className="orb-wrap"><div className="orb"><span>m</span></div><i className="orb-spark spark-one">✦</i><i className="orb-spark spark-two">✧</i></div>
                  <h2>What’s on your mind?</h2>
                  <p>Start with a thought, a question, or something you want to get done.</p>
                  <div className="suggestion-grid">
                    <button onClick={() => setDraft("Help me organise my day")}><span>↗</span><strong>Plan my day</strong><small>Turn a busy day into a clear plan</small></button>
                    <button onClick={() => setDraft("I want to think through an idea")}><span>✧</span><strong>Explore an idea</strong><small>Work through a thought together</small></button>
                    <button onClick={() => setDraft("Help me learn something new")}><span>⌘</span><strong>Learn something</strong><small>Break a topic into simple steps</small></button>
                  </div>
                </div>
              ) : (
                <div className="message-list" aria-live="polite">
                  <div className="date-divider"><span>SAVED ON THIS DEVICE</span></div>
                  {messages.map((message) => (
                    <div className={`message-row ${message.role === "user" ? "user-message" : "assistant-message"}`} key={message.id}>
                      <div className="message-avatar">{message.role === "user" ? "Y" : "m"}</div>
                      <div className="message-bubble">{message.text || (isGenerating && message.role === "assistant" ? "Thinking…" : "")}</div>
                    </div>
                  ))}
                </div>
              )}
            </div>

            <div className="composer-wrap">
              <form className="composer" onSubmit={sendMessage}>
                <textarea value={draft} onChange={(event) => setDraft(event.target.value)} onKeyDown={handleComposerKey} placeholder="Message your companion…" rows={2} aria-label="Message your companion" />
                <div className="composer-tools"><div className="composer-hint"><span>↵</span> to send <span className="hint-divider">·</span> Shift + ↵ for a new line</div><div className="composer-actions">{isGenerating ? <button className="stop-button" type="button" onClick={cancelGeneration}>Stop generating</button> : <><span className="voice-disabled" title="Voice will be added in a later phase">◖ Voice</span><button className="send-button" type="submit" disabled={!draft.trim() || runtimeStatus.status !== "ready"} aria-label="Send message">↑</button></>}</div></div>
              </form>
              <p className="composer-disclaimer">{runtimeStatus.status === "ready" ? "Local model connected · Conversations are stored on this device" : "Connect a local model in Settings to start chatting · Local conversation storage"}</p>
            </div>
          </div>
        )}

        {section === "characters" && (
          <div className="page-content">
            <p className="eyebrow">MAKE IT YOURS</p><h1>Your companions.</h1><p className="page-lead">Different ways of talking, shaped around what you need.</p>
            <div className="personality-grid">
              {personalities.map((item) => <button className={personality === item.name ? "personality-card selected" : "personality-card"} key={item.name} onClick={() => setPersonality(item.name)}><span className="personality-mark">{item.mark}</span><strong>{item.name}</strong><small>{item.detail}</small><span className="select-state">{personality === item.name ? "Selected" : "Choose companion"}</span></button>)}
            </div>
            <div className="info-strip"><span>✧</span><p>Personality selection is a visual prototype setting. A real character engine, custom profiles, and voice styles will come later.</p></div>
          </div>
        )}

        {section === "settings" && (
          <div className="page-content settings-page">
            <p className="eyebrow">YOUR CONTROL CENTRE</p><h1>Settings.</h1><p className="page-lead">Clear controls. No surprises.</p>
            <section className="settings-section runtime-settings"><div className="settings-section-title"><div><h2>Local AI engine</h2><p>Connect an existing llama.cpp server executable and a GGUF model stored on this device.</p></div><span className="settings-symbol">◈</span></div>
              <div className="runtime-status-line"><span className={runtimeStatus.status === "ready" ? "core-dot good" : runtimeStatus.status === "failed" ? "core-dot bad" : "core-dot"} />Engine status: <strong>{runtimeStatus.status}</strong></div>
              <div className="path-field"><label htmlFor="server-path">llama-server executable</label><div className="path-input-row"><input id="server-path" value={executablePath} onChange={(event) => setExecutablePath(event.target.value)} placeholder="Choose the llama-server file…" spellCheck={false} /><button type="button" className="browse-button" onClick={browseForExecutable}>Browse</button></div></div>
              <div className="path-field"><label htmlFor="model-path">GGUF model file</label><div className="path-input-row"><input id="model-path" value={modelPath} onChange={(event) => setModelPath(event.target.value)} placeholder="Choose a .gguf model…" spellCheck={false} /><button type="button" className="browse-button" onClick={browseForModel}>Browse</button></div></div>
              <div className="path-field"><label htmlFor="context-tokens">Context length (tokens)</label><input id="context-tokens" type="number" min="512" max="32768" step="256" value={contextTokens} onChange={(event) => setContextTokens(event.target.value)} disabled={runtimeBusy || runtimeStatus.status === "ready"} /></div>
              <div className="path-field"><label htmlFor="cpu-threads">CPU threads (blank = automatic)</label><input id="cpu-threads" type="number" min="1" max="256" step="1" value={cpuThreads} onChange={(event) => setCpuThreads(event.target.value)} placeholder="Automatic" disabled={runtimeBusy || runtimeStatus.status === "ready"} /></div>
              <div className="runtime-actions"><button className="primary-action" onClick={startLocalRuntime} disabled={runtimeBusy || runtimeStatus.status === "ready"}>{runtimeBusy ? "Please wait…" : "Start local engine"}</button><button className="secondary-action" onClick={stopLocalRuntime} disabled={runtimeBusy || runtimeStatus.status === "notInstalled" || runtimeStatus.status === "stopped"}>Stop engine</button></div>
              {runtimeError && <p className="runtime-error" role="alert">{runtimeError}</p>}
              <p className="settings-footnote">Nothing is downloaded automatically. You must already have a compatible llama.cpp server executable and GGUF model. The engine listens only on this device (127.0.0.1).</p>
            </section>
            <section className="settings-section">
              <div className="settings-section-title"><div><h2>Local model library</h2><p>Register GGUF files already on this device. midSunday records their details and SHA-256 fingerprint without copying or downloading them.</p></div><span className="settings-symbol">▧</span></div>
              <div className="path-field"><label htmlFor="model-display-name">Model name</label><input id="model-display-name" value={modelDisplayName} onChange={(event) => setModelDisplayName(event.target.value)} placeholder="e.g. My local assistant" maxLength={120} /></div>
              <div className="path-field"><label htmlFor="model-source">Source (optional)</label><input id="model-source" value={modelSource} onChange={(event) => setModelSource(event.target.value)} placeholder="Where did you get this model?" maxLength={512} /></div>
              <div className="path-field"><label htmlFor="model-license">Licence (optional)</label><input id="model-license" value={modelLicense} onChange={(event) => setModelLicense(event.target.value)} placeholder="Record the model's licence" maxLength={512} /></div>
              <div className="runtime-actions"><button className="secondary-action" type="button" onClick={registerLocalModel} disabled={modelCatalogBusy || !modelPath.trim()}>{modelCatalogBusy ? "Fingerprinting model…" : "Add selected GGUF to library"}</button></div>
              {localModels.length === 0 ? <p className="settings-footnote">No models registered yet. Browse for an existing GGUF file above, then add it here.</p> : <div className="model-library-list">{localModels.map((model) => <div className="setting-row" key={model.id}><div><strong>{model.displayName}</strong><p>{formatModelSize(model.sizeBytes)} · SHA-256: {model.sha256.slice(0, 16)}…</p><p>{model.source || "Source not recorded"} · {model.license || "Licence not recorded"}</p><p className="model-path-text">{model.filePath}</p></div><div className="model-library-actions"><button className="secondary-action" type="button" onClick={() => { setModelPath(model.filePath); setModelDisplayName(model.displayName); setModelSource(model.source); setModelLicense(model.license); }}>Use model</button><button className="danger-action" type="button" onClick={() => void removeLocalModel(model)}>Remove</button></div></div>)}</div>}
              <p className="settings-footnote">Removing an entry only removes its catalogue record. It never deletes the original model file. Always check the model's licence before redistribution or commercial use.</p>
            </section>
            <section className="settings-section"><div className="settings-section-title"><div><h2>Conversation data</h2><p>Your chat history stays in the local app database. Export a portable copy or clear it from this device.</p></div><span className="settings-symbol">▤</span></div>
              <div className="setting-row"><div><strong>Export conversation history</strong><p>Save all conversations and messages as a versioned JSON file. This export does not include future personal memories or model files.</p></div><button className="secondary-action" type="button" onClick={() => void exportConversations()}>Export JSON</button></div>
              <div className="setting-row"><div><strong>Import conversation backup</strong><p>Merge a versioned JSON backup into this device. Existing chats are never overwritten; exact duplicates are skipped.</p></div><><input ref={importFileRef} className="visually-hidden-file" type="file" accept=".json,application/json" onChange={(event) => void importConversations(event)} /><button className="secondary-action" type="button" onClick={() => importFileRef.current?.click()} disabled={importBusy || isGenerating}>{importBusy ? "Importing…" : "Import JSON"}</button></></div>
              <div className="setting-row"><div><strong>Clear all conversation history</strong><p>Delete every saved conversation and its messages. This does not affect separate memory data.</p></div><button className="danger-action" type="button" onClick={() => void clearAllConversations()} disabled={isGenerating}>Clear history</button></div>
            </section>
            <section className="settings-section"><div className="settings-section-title"><div><h2>Privacy & memory</h2><p>Choose what midSunday may remember on this device.</p></div><span className="settings-symbol">⌑</span></div>
              <div className="setting-row"><div><strong>Local memory</strong><p>Allow the future memory system to keep useful details locally.</p></div><button className={memoryEnabled ? "toggle on" : "toggle"} role="switch" aria-checked={memoryEnabled} onClick={() => setMemoryEnabled(!memoryEnabled)}><i /></button></div>
              <div className="setting-row"><div><strong>Cloud connection</strong><p>Cloud features are not connected or enabled in this prototype.</p></div><span className="locked-label">OFF</span></div>
            </section>
            <section className="settings-section"><div className="settings-section-title"><div><h2>System</h2><p>Hardware detection and resource controls are planned.</p></div><span className="settings-symbol">⌘</span></div>
              <div className="setting-row"><div><strong>Hardware profile</strong><p>CPU, RAM, GPU and storage information will appear here.</p></div><span className="pending-label">Not detected</span></div>
              <div className="setting-row"><div><strong>Start with Windows</strong><p>Launch midSunday when you sign in. Not implemented yet.</p></div><button className={startWithWindows ? "toggle on" : "toggle"} role="switch" aria-checked={startWithWindows} onClick={() => setStartWithWindows(!startWithWindows)}><i /></button></div>
            </section>
            <div className="settings-footnote">These switches are temporary UI demonstrations. They do not save settings or activate background features yet.</div>
          </div>
        )}

        <footer className="statusbar">
          <div className="core-status"><span className={backendState === "connected" ? "core-dot good" : backendState === "error" ? "core-dot bad" : "core-dot"} />{statusLabel}</div>
          <div className="statusbar-right"><span>AI engine <b>{runtimeStatus.status === "ready" ? "Ready" : runtimeStatus.status}</b></span><span>Hardware <b>Not detected</b></span><span className="status-detail" title={backendMessage}>Rust foundation</span></div>
        </footer>
      </section>
    </main>
  );
}
