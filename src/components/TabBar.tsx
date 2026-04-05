export type Tab = "log" | "history" | "calendar" | "progress" | "garmin";

interface TabBarProps {
  activeTab: Tab;
  onTabChange: (tab: Tab) => void;
}

const TABS: { id: Tab; label: string }[] = [
  { id: "log", label: "Log" },
  { id: "history", label: "History" },
  { id: "calendar", label: "Calendar" },
  { id: "progress", label: "Progress" },
  { id: "garmin", label: "Garmin" },
];

export function TabBar({ activeTab, onTabChange }: TabBarProps) {
  return (
    <div className="tab-bar">
      {TABS.map((tab) => (
        <button
          key={tab.id}
          className={activeTab === tab.id ? "active" : ""}
          onClick={() => onTabChange(tab.id)}
        >
          {tab.label}
        </button>
      ))}
    </div>
  );
}
