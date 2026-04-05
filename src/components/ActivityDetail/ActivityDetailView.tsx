import { useState, useEffect, useMemo } from "react";
import { format } from "date-fns";
import {
  Activity,
  ACTIVITY_TYPE_LABELS,
  ACTIVITY_SUB_TYPE_LABELS,
  HR_ZONE_LABELS,
  ActivitySubType,
  getActivity,
  deleteActivity,
} from "../../lib/tauri";
import {
  parseFitData,
  extractCoordinates,
  extractTimeSeries,
} from "../../lib/fitData";
import { ActivityTabs } from "./ActivityTabs";
import { SummaryCards } from "./SummaryCards";
import { LapsTable } from "./LapsTable";
import { MetricChart } from "./MetricChart";
import { RouteMap } from "../Calendar/RouteMap";
import { DeleteButton } from "../DeleteButton";

interface ActivityDetailViewProps {
  id: string;
  onBack: () => void;
}

export function ActivityDetailView({ id, onBack }: ActivityDetailViewProps) {
  const [activity, setActivity] = useState<Activity | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  async function handleDelete() {
    await deleteActivity(id);
    onBack();
  }

  useEffect(() => {
    let cancelled = false;
    setLoading(true);
    setError(null);
    getActivity(id)
      .then((a) => {
        if (!cancelled) setActivity(a);
      })
      .catch((e) => {
        if (!cancelled) setError(String(e));
      })
      .finally(() => {
        if (!cancelled) setLoading(false);
      });
    return () => { cancelled = true; };
  }, [id]);

  const fitData = useMemo(
    () => parseFitData(activity?.fit_data ?? null),
    [activity?.fit_data],
  );

  const coordinates = useMemo(
    () => extractCoordinates(fitData.records),
    [fitData.records],
  );

  const hrData = useMemo(
    () => extractTimeSeries(fitData.records, "heart_rate"),
    [fitData.records],
  );
  const cadenceData = useMemo(
    () => extractTimeSeries(fitData.records, "cadence"),
    [fitData.records],
  );
  const powerData = useMemo(
    () => extractTimeSeries(fitData.records, "power"),
    [fitData.records],
  );
  const speedData = useMemo(
    () => extractTimeSeries(fitData.records, "speed"),
    [fitData.records],
  );
  const altitudeData = useMemo(
    () => extractTimeSeries(fitData.records, "altitude"),
    [fitData.records],
  );

  // Build available tabs
  const availableTabs = useMemo(() => {
    const tabs = ["Overview"];
    if (coordinates.length > 0) tabs.push("Map");
    if (hrData.length > 0) tabs.push("Heart Rate");
    if (
      cadenceData.length > 0 ||
      powerData.length > 0 ||
      speedData.length > 0
    ) {
      tabs.push("Performance");
    }
    if (altitudeData.length > 0) tabs.push("Elevation");
    return tabs;
  }, [coordinates, hrData, cadenceData, powerData, speedData, altitudeData]);

  const [activeTab, setActiveTab] = useState("Overview");

  // Reset tab if it becomes unavailable
  useEffect(() => {
    if (!availableTabs.includes(activeTab)) {
      setActiveTab("Overview");
    }
  }, [availableTabs, activeTab]);

  if (loading) {
    return (
      <div>
        <button className="btn btn-secondary" onClick={onBack}>
          &larr; Back
        </button>
        <p style={{ marginTop: "var(--spacing-md)", color: "var(--color-text-secondary)" }}>
          Loading...
        </p>
      </div>
    );
  }

  if (error || !activity) {
    return (
      <div>
        <button className="btn btn-secondary" onClick={onBack}>
          &larr; Back
        </button>
        <p className="error-message" style={{ marginTop: "var(--spacing-md)" }}>
          {error ?? "Activity not found"}
        </p>
      </div>
    );
  }

  return (
    <div>
      {/* Header */}
      <div
        style={{
          display: "flex",
          alignItems: "center",
          gap: "var(--spacing-md)",
          marginBottom: "var(--spacing-md)",
        }}
      >
        <button className="btn btn-secondary" onClick={onBack}>
          &larr; Back
        </button>
        <div style={{ flex: 1 }}>
          <h2 style={{ margin: 0 }}>
            {ACTIVITY_TYPE_LABELS[activity.activity_type]}
          </h2>
          <div
            style={{
              fontSize: "var(--font-size-sm)",
              color: "var(--color-text-secondary)",
            }}
          >
            {format(new Date(activity.date), "EEEE, MMMM d, yyyy 'at' HH:mm")}
          </div>
        </div>
        <DeleteButton onDelete={handleDelete} />
      </div>

      {/* Badges */}
      <div
        style={{
          display: "flex",
          gap: "var(--spacing-sm)",
          flexWrap: "wrap",
          marginBottom: "var(--spacing-md)",
        }}
      >
        {activity.sub_type && (
          <span
            style={{
              fontSize: "var(--font-size-xs)",
              padding: "1px 6px",
              borderRadius: "var(--radius-sm)",
              background: "var(--color-bg-tertiary)",
            }}
          >
            {ACTIVITY_SUB_TYPE_LABELS[activity.sub_type as ActivitySubType]}
          </span>
        )}
        {activity.source === "garmin" && (
          <span className="badge-garmin">Garmin</span>
        )}
        {activity.is_commute && (
          <span
            style={{
              fontSize: "var(--font-size-xs)",
              padding: "1px 6px",
              borderRadius: "var(--radius-sm)",
              background: "rgba(52, 199, 89, 0.15)",
              color: "var(--color-success)",
            }}
          >
            Commute
          </span>
        )}
        {activity.is_race && (
          <span
            style={{
              fontSize: "var(--font-size-xs)",
              padding: "1px 6px",
              borderRadius: "var(--radius-sm)",
              background: "rgba(255, 149, 0, 0.15)",
              color: "#ff9500",
            }}
          >
            Race
          </span>
        )}
        {activity.hr_zone && (
          <span
            style={{
              fontSize: "var(--font-size-xs)",
              padding: "1px 6px",
              borderRadius: "var(--radius-sm)",
              background: "var(--color-bg-tertiary)",
            }}
          >
            {HR_ZONE_LABELS[activity.hr_zone]}
          </span>
        )}
      </div>

      {/* Summary cards */}
      <SummaryCards activity={activity} session={fitData.session} />

      {/* Tabs */}
      {availableTabs.length > 1 && (
        <ActivityTabs
          tabs={availableTabs}
          activeTab={activeTab}
          onTabChange={setActiveTab}
        />
      )}

      {/* Tab content */}
      {activeTab === "Overview" && (
        <div>
          {activity.notes && (
            <div
              className="card"
              style={{
                marginBottom: "var(--spacing-md)",
                fontSize: "var(--font-size-sm)",
                color: "var(--color-text-secondary)",
                fontStyle: "italic",
              }}
            >
              {activity.notes}
            </div>
          )}
          <LapsTable laps={fitData.laps} />
        </div>
      )}

      {activeTab === "Map" && coordinates.length > 0 && (
        <RouteMap coordinates={coordinates} />
      )}

      {activeTab === "Heart Rate" && (
        <MetricChart
          data={hrData}
          label="Heart Rate"
          color="#ff3b30"
          unit="bpm"
        />
      )}

      {activeTab === "Performance" && (
        <div
          style={{
            display: "flex",
            flexDirection: "column",
            gap: "var(--spacing-lg)",
          }}
        >
          {speedData.length > 0 && (
            <MetricChart
              data={speedData}
              label="Speed"
              color="#0a84ff"
              unit="m/s"
            />
          )}
          {cadenceData.length > 0 && (
            <MetricChart
              data={cadenceData}
              label="Cadence"
              color="#34c759"
              unit="rpm"
            />
          )}
          {powerData.length > 0 && (
            <MetricChart
              data={powerData}
              label="Power"
              color="#ff9500"
              unit="W"
            />
          )}
        </div>
      )}

      {activeTab === "Elevation" && (
        <MetricChart
          data={altitudeData}
          label="Elevation"
          color="#a2845e"
          unit="m"
        />
      )}
    </div>
  );
}
