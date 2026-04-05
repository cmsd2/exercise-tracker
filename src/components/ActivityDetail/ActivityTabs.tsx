interface ActivityTabsProps {
  tabs: string[];
  activeTab: string;
  onTabChange: (tab: string) => void;
}

export function ActivityTabs({ tabs, activeTab, onTabChange }: ActivityTabsProps) {
  return (
    <div className="segmented-control" style={{ marginBottom: "var(--spacing-md)" }}>
      {tabs.map((tab) => (
        <button
          key={tab}
          className={activeTab === tab ? "active" : ""}
          onClick={() => onTabChange(tab)}
        >
          {tab}
        </button>
      ))}
    </div>
  );
}
