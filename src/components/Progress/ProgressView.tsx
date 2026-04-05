import { useEffect, useState } from "react";
import { useActivityStore } from "../../store/activityStore";
import {
  ACTIVITY_TYPES,
  ACTIVITY_TYPE_LABELS,
  ActivityType,
} from "../../lib/tauri";
import {
  metresToDisplay,
  formatDuration,
  DistanceUnit,
} from "../../lib/units";
import {
  LineChart,
  Line,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
} from "recharts";

type Metric = "pace" | "distance" | "duration";
type TimeRange = "4w" | "3m" | "6m" | "1y";

const TIME_RANGE_LABELS: Record<TimeRange, string> = {
  "4w": "4 Weeks",
  "3m": "3 Months",
  "6m": "6 Months",
  "1y": "1 Year",
};

function getDateCutoff(range: TimeRange): Date {
  const now = new Date();
  switch (range) {
    case "4w":
      return new Date(now.getTime() - 28 * 24 * 60 * 60 * 1000);
    case "3m":
      return new Date(now.getFullYear(), now.getMonth() - 3, now.getDate());
    case "6m":
      return new Date(now.getFullYear(), now.getMonth() - 6, now.getDate());
    case "1y":
      return new Date(now.getFullYear() - 1, now.getMonth(), now.getDate());
  }
}

export function ProgressView() {
  const { activities, weeklySummaries, fetchActivities, fetchWeeklySummaries } =
    useActivityStore();

  const [activityType, setActivityType] = useState<ActivityType>("run");
  const [metric, setMetric] = useState<Metric>("pace");
  const [timeRange, setTimeRange] = useState<TimeRange>("3m");

  const distanceUnit: DistanceUnit = "km";

  useEffect(() => {
    fetchActivities();
    fetchWeeklySummaries(activityType);
  }, [activityType]);

  const cutoff = getDateCutoff(timeRange);
  const filtered = activities
    .filter((a) => a.activity_type === activityType)
    .filter((a) => new Date(a.date) >= cutoff)
    .sort((a, b) => new Date(a.date).getTime() - new Date(b.date).getTime());

  const chartData = filtered.map((a) => {
    const date = new Date(a.date).toLocaleDateString(undefined, {
      month: "short",
      day: "numeric",
    });
    let value: number | null = null;

    if (metric === "pace" && a.pace_s_per_m != null) {
      value = a.pace_s_per_m * 1000;
    } else if (metric === "distance" && a.distance_m != null) {
      value = metresToDisplay(a.distance_m, distanceUnit);
    } else if (metric === "duration" && a.duration_secs != null) {
      value = a.duration_secs / 60;
    }

    return { date, value };
  }).filter((d) => d.value != null);

  const metricLabel =
    metric === "pace"
      ? `Pace (s/km)`
      : metric === "distance"
        ? `Distance (${distanceUnit})`
        : "Duration (min)";

  // Weekly summary for current type
  const currentWeek = weeklySummaries.length > 0 ? weeklySummaries[0] : null;

  return (
    <div>
      <h2 style={{ marginBottom: "var(--spacing-lg)" }}>Progress</h2>

      {/* Weekly Summary */}
      <div
        style={{
          display: "grid",
          gridTemplateColumns: "repeat(3, 1fr)",
          gap: "var(--spacing-md)",
          marginBottom: "var(--spacing-xl)",
        }}
      >
        <div className="card" style={{ textAlign: "center" }}>
          <div style={{ fontSize: "var(--font-size-sm)", color: "var(--color-text-secondary)" }}>
            This Week
          </div>
          <div style={{ fontSize: "var(--font-size-xl)", fontWeight: 600 }}>
            {currentWeek?.activity_count ?? 0}
          </div>
          <div style={{ fontSize: "var(--font-size-xs)", color: "var(--color-text-secondary)" }}>
            activities
          </div>
        </div>
        <div className="card" style={{ textAlign: "center" }}>
          <div style={{ fontSize: "var(--font-size-sm)", color: "var(--color-text-secondary)" }}>
            Distance
          </div>
          <div style={{ fontSize: "var(--font-size-xl)", fontWeight: 600 }}>
            {currentWeek
              ? metresToDisplay(currentWeek.total_distance_m, distanceUnit).toFixed(1)
              : "0.0"}
          </div>
          <div style={{ fontSize: "var(--font-size-xs)", color: "var(--color-text-secondary)" }}>
            {distanceUnit}
          </div>
        </div>
        <div className="card" style={{ textAlign: "center" }}>
          <div style={{ fontSize: "var(--font-size-sm)", color: "var(--color-text-secondary)" }}>
            Time
          </div>
          <div style={{ fontSize: "var(--font-size-xl)", fontWeight: 600 }}>
            {currentWeek
              ? formatDuration(currentWeek.total_duration_secs)
              : "0:00"}
          </div>
          <div style={{ fontSize: "var(--font-size-xs)", color: "var(--color-text-secondary)" }}>
            total
          </div>
        </div>
      </div>

      {/* Controls */}
      <div
        style={{
          display: "flex",
          gap: "var(--spacing-md)",
          marginBottom: "var(--spacing-lg)",
          flexWrap: "wrap",
        }}
      >
        <div className="segmented-control">
          {ACTIVITY_TYPES.map((t) => (
            <button
              key={t}
              className={activityType === t ? "active" : ""}
              onClick={() => setActivityType(t)}
            >
              {ACTIVITY_TYPE_LABELS[t]}
            </button>
          ))}
        </div>

        <div className="segmented-control">
          {(["pace", "distance", "duration"] as Metric[]).map((m) => (
            <button
              key={m}
              className={metric === m ? "active" : ""}
              onClick={() => setMetric(m)}
            >
              {m.charAt(0).toUpperCase() + m.slice(1)}
            </button>
          ))}
        </div>

        <div className="segmented-control">
          {(Object.keys(TIME_RANGE_LABELS) as TimeRange[]).map((r) => (
            <button
              key={r}
              className={timeRange === r ? "active" : ""}
              onClick={() => setTimeRange(r)}
            >
              {TIME_RANGE_LABELS[r]}
            </button>
          ))}
        </div>
      </div>

      {/* Chart */}
      {chartData.length === 0 ? (
        <p style={{ color: "var(--color-text-secondary)" }}>
          No data for the selected filters.
        </p>
      ) : (
        <div style={{ width: "100%", height: 400 }}>
          <ResponsiveContainer>
            <LineChart data={chartData}>
              <CartesianGrid strokeDasharray="3 3" stroke="var(--color-border)" />
              <XAxis
                dataKey="date"
                tick={{ fontSize: 12, fill: "var(--color-text-secondary)" }}
              />
              <YAxis
                tick={{ fontSize: 12, fill: "var(--color-text-secondary)" }}
                label={{
                  value: metricLabel,
                  angle: -90,
                  position: "insideLeft",
                  style: {
                    fontSize: 12,
                    fill: "var(--color-text-secondary)",
                  },
                }}
                reversed={metric === "pace"}
              />
              <Tooltip />
              <Line
                type="monotone"
                dataKey="value"
                stroke="var(--color-primary)"
                strokeWidth={2}
                dot={{ r: 4, fill: "var(--color-primary)" }}
              />
            </LineChart>
          </ResponsiveContainer>
        </div>
      )}
    </div>
  );
}
