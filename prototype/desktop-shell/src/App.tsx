import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

type BackendState = "checking" | "connected" | "error";

export default function App() {
  const [backendState, setBackendState] = useState<BackendState>("checking");
  const [backendMessage, setBackendMessage] = useState("Checking the local Rust core…");

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
        setBackendMessage("The local core could not be reached.");
        setBackendState("error");
      });
    return () => { active = false; };
  }, []);

  return (
    <main className="app-shell">
      <header className="topbar">
        <div className="brand">
          <div className="brand-mark" aria-hidden="true">m</div>
          <div>
            <div className="brand-name">midSunday</div>
            <div className="brand-subtitle">AI Personal Companion</div>
          </div>
        </div>
        <span className="prototype-label">EARLY PROTOTYPE</span>
      </header>

      <section className="welcome">
        <p className="eyebrow">A quieter kind of intelligence</p>
        <h1>Your space.<br /><span>Your companion.</span></h1>
        <p className="intro">
          A private, local-first companion designed to be useful, personal, and
          available even when you are offline.
        </p>
      </section>

      <section className="status-card" aria-live="polite">
        <div className="status-heading">
          <div>
            <p className="eyebrow">FOUNDATION CHECK</p>
            <h2>Desktop shell</h2>
          </div>
          <span className={`status-indicator status-${backendState}`}>
            <span className="status-dot" />
            {backendState === "checking" ? "Checking" : backendState === "connected" ? "Connected" : "Unavailable"}
          </span>
        </div>
        <p className="status-message">{backendMessage}</p>
        <div className="status-footnote">
          No AI model is loaded. No account or cloud connection is required for this check.
        </div>
      </section>

      <footer className="footer">
        <span>Built to keep your data yours.</span>
        <span>Prototype 0.1.0</span>
      </footer>
    </main>
  );
}
