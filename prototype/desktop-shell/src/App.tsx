import { useEffect, useState, type FormEvent, type KeyboardEvent } from "react";
import { invoke } from "@tauri-apps/api/core";

type BackendState = "checking" | "connected" | "error";
type Section = "chat" | "characters" | "settings";
type Message = { id: number; role: "user" | "system"; text: string };

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
  const [conversationName, setConversationName] = useState("A fresh start");
  const [memoryEnabled, setMemoryEnabled] = useState(false);
  const [startWithWindows, setStartWithWindows] = useState(false);

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
    return () => { active = false; };
  }, []);

  function startNewChat() {
    setMessages([]);
    setDraft("");
    setConversationName("A fresh start");
    setSection("chat");
  }

  function sendMessage(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const text = draft.trim();
    if (!text) return;
    setMessages((current) => [...current, { id: Date.now(), role: "user", text }]);
    setConversationName(text.length > 28 ? text.slice(0, 28) + "…" : text);
    setDraft("");
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
        <button className="recent-chat" onClick={() => setSection("chat")}><span className="recent-dot" />{conversationName}</button>

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
                  <div className="date-divider"><span>THIS SESSION · NOT SAVED</span></div>
                  {messages.map((message) => (
                    <div className="message-row user-message" key={message.id}><div className="message-avatar">Y</div><div className="message-bubble">{message.text}</div></div>
                  ))}
                  <div className="prototype-note"><span>✧</span><div><strong>Your message is shown in this prototype.</strong><p>The local AI engine is not connected yet, so midSunday has not generated a reply. Messages currently remain only in this session and are cleared when the app closes.</p></div></div>
                </div>
              )}
            </div>

            <div className="composer-wrap">
              <form className="composer" onSubmit={sendMessage}>
                <textarea value={draft} onChange={(event) => setDraft(event.target.value)} onKeyDown={handleComposerKey} placeholder="Message your companion…" rows={2} aria-label="Message your companion" />
                <div className="composer-tools"><div className="composer-hint"><span>↵</span> to send <span className="hint-divider">·</span> Shift + ↵ for a new line</div><div className="composer-actions"><span className="voice-disabled" title="Voice will be added in a later phase">◖ Voice</span><button className="send-button" type="submit" disabled={!draft.trim()} aria-label="Send message">↑</button></div></div>
              </form>
              <p className="composer-disclaimer">Prototype workspace · No AI model or microphone is active</p>
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
          <div className="statusbar-right"><span>AI engine <b>Not installed</b></span><span>Hardware <b>Not detected</b></span><span className="status-detail" title={backendMessage}>Rust foundation</span></div>
        </footer>
      </section>
    </main>
  );
}
