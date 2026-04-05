import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { useGarminStore } from "../../store/garminStore";
import { useActivityStore } from "../../store/activityStore";
import { SyncProgress } from "../../lib/tauri";

export function GarminSyncView() {
  const {
    connected,
    syncing,
    syncProgress,
    lastSyncResult,
    error,
    checkAuth,
    startLogin,
    disconnect,
    syncActivities,
    clearError,
  } = useGarminStore();

  const { fetchActivities } = useActivityStore();

  const today = new Date().toISOString().slice(0, 10);
  const thirtyDaysAgo = new Date(Date.now() - 30 * 86400000)
    .toISOString()
    .slice(0, 10);

  const [startDate, setStartDate] = useState(thirtyDaysAgo);
  const [endDate, setEndDate] = useState(today);
  const [showDisconnectConfirm, setShowDisconnectConfirm] = useState(false);

  useEffect(() => {
    checkAuth();
  }, []);

  // Listen for sync progress events
  useEffect(() => {
    const unlisten = listen<SyncProgress>("garmin-sync-progress", (event) => {
      useGarminStore.setState({ syncProgress: event.payload });
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  // Listen for auth completion from the login window
  useEffect(() => {
    const unlisten = listen("garmin-auth-complete", () => {
      checkAuth();
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  async function handleConnect() {
    await startLogin();
  }

  async function handleSync() {
    await syncActivities(startDate, endDate);
    // Refresh activity list after sync
    await fetchActivities();
  }

  async function handleDisconnect() {
    setShowDisconnectConfirm(true);
  }

  async function confirmDisconnect() {
    await disconnect();
    setShowDisconnectConfirm(false);
  }

  return (
    <div>
      <h2 style={{ marginBottom: "var(--spacing-lg)" }}>Garmin Sync</h2>

      {error && (
        <div className="error-message" style={{ marginBottom: "var(--spacing-md)" }}>
          {error}
          <button
            onClick={clearError}
            style={{
              marginLeft: "var(--spacing-sm)",
              background: "none",
              border: "none",
              color: "var(--color-danger)",
              cursor: "pointer",
              textDecoration: "underline",
            }}
          >
            Dismiss
          </button>
        </div>
      )}

      {/* Connection status */}
      <div className="card" style={{ marginBottom: "var(--spacing-lg)" }}>
        <div
          style={{
            display: "flex",
            justifyContent: "space-between",
            alignItems: "center",
          }}
        >
          <div>
            <div style={{ fontWeight: 600, marginBottom: "var(--spacing-xs)" }}>
              Garmin Connect
            </div>
            <div
              style={{
                fontSize: "var(--font-size-sm)",
                color: connected
                  ? "var(--color-success, #34c759)"
                  : "var(--color-text-secondary)",
              }}
            >
              {connected ? "Connected" : "Not connected"}
            </div>
          </div>
          <div style={{ display: "flex", gap: "var(--spacing-sm)", alignItems: "center" }}>
            {connected ? (
              showDisconnectConfirm ? (
                <>
                  <span style={{ fontSize: "var(--font-size-sm)", color: "var(--color-text-secondary)" }}>
                    Remove credentials?
                  </span>
                  <button className="btn btn-secondary" style={{ color: "var(--color-danger)" }} onClick={confirmDisconnect}>
                    Yes
                  </button>
                  <button className="btn btn-secondary" onClick={() => setShowDisconnectConfirm(false)}>
                    No
                  </button>
                </>
              ) : (
                <button className="btn btn-secondary" onClick={handleDisconnect}>
                  Disconnect
                </button>
              )
            ) : (
              <button className="btn btn-primary" onClick={handleConnect}>
                Connect to Garmin
              </button>
            )}
          </div>
        </div>
      </div>

      {/* Import section - only show when connected */}
      {connected && (
        <>
          <div className="card" style={{ marginBottom: "var(--spacing-lg)" }}>
            <div style={{ fontWeight: 600, marginBottom: "var(--spacing-md)" }}>
              Import Activities
            </div>
            <div
              style={{
                display: "flex",
                gap: "var(--spacing-md)",
                alignItems: "end",
                flexWrap: "wrap",
              }}
            >
              <div className="form-group" style={{ marginBottom: 0 }}>
                <label>Start Date</label>
                <input
                  type="date"
                  className="form-input"
                  value={startDate}
                  onChange={(e) => setStartDate(e.target.value)}
                />
              </div>
              <div className="form-group" style={{ marginBottom: 0 }}>
                <label>End Date</label>
                <input
                  type="date"
                  className="form-input"
                  value={endDate}
                  onChange={(e) => setEndDate(e.target.value)}
                />
              </div>
              <button
                className="btn btn-primary"
                onClick={handleSync}
                disabled={syncing}
              >
                {syncing ? "Importing..." : "Import Activities"}
              </button>
            </div>

            {/* Progress */}
            {syncing && syncProgress && (
              <div
                style={{
                  marginTop: "var(--spacing-md)",
                  fontSize: "var(--font-size-sm)",
                  color: "var(--color-text-secondary)",
                }}
              >
                {syncProgress.kind === "Started" &&
                  `Found ${syncProgress.total} activities...`}
                {syncProgress.kind === "Activity" &&
                  `Importing ${syncProgress.current}/${syncProgress.total}...`}
                {syncProgress.kind === "Skipped" &&
                  `Skipped ${syncProgress.current}/${syncProgress.total}: ${syncProgress.reason}`}
              </div>
            )}
          </div>

          {/* Results */}
          {lastSyncResult && lastSyncResult.kind === "Finished" && (
            <div className="card">
              <div
                style={{ fontWeight: 600, marginBottom: "var(--spacing-sm)" }}
              >
                Import Complete
              </div>
              <div
                style={{
                  display: "grid",
                  gridTemplateColumns: "repeat(3, 1fr)",
                  gap: "var(--spacing-md)",
                  textAlign: "center",
                }}
              >
                <div>
                  <div
                    style={{
                      fontSize: "var(--font-size-xl)",
                      fontWeight: 600,
                      color: "var(--color-primary)",
                    }}
                  >
                    {lastSyncResult.imported ?? 0}
                  </div>
                  <div
                    style={{
                      fontSize: "var(--font-size-xs)",
                      color: "var(--color-text-secondary)",
                    }}
                  >
                    imported
                  </div>
                </div>
                <div>
                  <div
                    style={{
                      fontSize: "var(--font-size-xl)",
                      fontWeight: 600,
                    }}
                  >
                    {lastSyncResult.skipped ?? 0}
                  </div>
                  <div
                    style={{
                      fontSize: "var(--font-size-xs)",
                      color: "var(--color-text-secondary)",
                    }}
                  >
                    skipped
                  </div>
                </div>
                <div>
                  <div
                    style={{
                      fontSize: "var(--font-size-xl)",
                      fontWeight: 600,
                      color:
                        (lastSyncResult.errors ?? 0) > 0
                          ? "var(--color-danger)"
                          : undefined,
                    }}
                  >
                    {lastSyncResult.errors ?? 0}
                  </div>
                  <div
                    style={{
                      fontSize: "var(--font-size-xs)",
                      color: "var(--color-text-secondary)",
                    }}
                  >
                    errors
                  </div>
                </div>
              </div>
            </div>
          )}
        </>
      )}

      {/* Info when not connected */}
      {!connected && (
        <div
          style={{
            color: "var(--color-text-secondary)",
            fontSize: "var(--font-size-sm)",
            lineHeight: 1.6,
          }}
        >
          <p>
            Connect your Garmin account to automatically import your cardio
            activities. Supported activity types: Run, Cycle, Swim, Row, Walk,
            and Hike.
          </p>
          <p style={{ marginTop: "var(--spacing-sm)" }}>
            Duplicate activities are automatically detected and skipped during
            import.
          </p>
        </div>
      )}
    </div>
  );
}
