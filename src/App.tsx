import { useState } from "react";
import { TabBar, Tab } from "./components/TabBar";
import { LogActivityView } from "./components/LogActivity/LogActivityView";
import { HistoryView } from "./components/History/HistoryView";
import { ProgressView } from "./components/Progress/ProgressView";
import { CalendarView } from "./components/Calendar/CalendarView";
import { GarminSyncView } from "./components/Garmin/GarminSyncView";
import { ActivityDetailView } from "./components/ActivityDetail/ActivityDetailView";
import "./styles/global.css";

function App() {
  const [activeTab, setActiveTab] = useState<Tab>("log");
  const [detailActivityId, setDetailActivityId] = useState<string | null>(null);

  const handleViewActivity = (id: string) => setDetailActivityId(id);

  return (
    <div className="app">
      <nav className="app-nav">
        <TabBar activeTab={activeTab} onTabChange={(tab) => { setActiveTab(tab); setDetailActivityId(null); }} />
      </nav>
      <main className="app-main">
        {detailActivityId && (
          <ActivityDetailView
            id={detailActivityId}
            onBack={() => setDetailActivityId(null)}
          />
        )}
        <div style={{ display: detailActivityId ? "none" : undefined }}>
          {activeTab === "log" && <LogActivityView />}
          {activeTab === "history" && (
            <HistoryView onViewActivity={handleViewActivity} />
          )}
          {activeTab === "calendar" && (
            <CalendarView onViewActivity={handleViewActivity} />
          )}
          {activeTab === "progress" && <ProgressView />}
          {activeTab === "garmin" && <GarminSyncView />}
        </div>
      </main>
    </div>
  );
}

export default App;
