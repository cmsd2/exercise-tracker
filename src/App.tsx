import { useState } from "react";
import { TabBar, Tab } from "./components/TabBar";
import { LogActivityView } from "./components/LogActivity/LogActivityView";
import { HistoryView } from "./components/History/HistoryView";
import { ProgressView } from "./components/Progress/ProgressView";
import { GarminSyncView } from "./components/Garmin/GarminSyncView";
import "./styles/global.css";

function App() {
  const [activeTab, setActiveTab] = useState<Tab>("log");

  return (
    <div className="app">
      <nav className="app-nav">
        <TabBar activeTab={activeTab} onTabChange={setActiveTab} />
      </nav>
      <main className="app-main">
        {activeTab === "log" && <LogActivityView />}
        {activeTab === "history" && <HistoryView />}
        {activeTab === "progress" && <ProgressView />}
        {activeTab === "garmin" && <GarminSyncView />}
      </main>
    </div>
  );
}

export default App;
