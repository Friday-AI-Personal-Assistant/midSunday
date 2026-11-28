import React from "react";

type AppErrorBoundaryState = { hasError: boolean };

/** Keeps a render failure inside the React tree from blanking the entire desktop window. */
export default class AppErrorBoundary extends React.Component<
  { children: React.ReactNode },
  AppErrorBoundaryState
> {
  state: AppErrorBoundaryState = { hasError: false };

  static getDerivedStateFromError(): AppErrorBoundaryState {
    return { hasError: true };
  }

  render() {
    if (this.state.hasError) {
      return (
        <main className="app-crash-screen" role="alert" aria-labelledby="app-crash-title">
          <section className="app-crash-card">
            <div className="app-crash-mark" aria-hidden="true">m</div>
            <p className="app-crash-eyebrow">midSunday · Recovery</p>
            <h1 id="app-crash-title">This screen ran into a problem</h1>
            <p>The interface could not finish rendering. Reloading midSunday may restore the screen. If the problem keeps happening, open Settings → Diagnostics after the app returns.</p>
            <button type="button" onClick={() => window.location.reload()}>Reload midSunday</button>
            <small>This recovery screen does not delete or reset your local data.</small>
          </section>
        </main>
      );
    }

    return this.props.children;
  }
}
