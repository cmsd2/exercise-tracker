import { Activity } from "../../lib/tauri";
import { FitSession } from "../../lib/fitData";
import { formatDuration, formatDistance, paceToDisplayString } from "../../lib/units";

interface SummaryCardsProps {
  activity: Activity;
  session: FitSession | null;
}

interface MetricCardProps {
  label: string;
  value: string;
}

function MetricCard({ label, value }: MetricCardProps) {
  return (
    <div
      style={{
        flex: "1 1 120px",
        padding: "var(--spacing-sm) var(--spacing-md)",
        background: "var(--color-bg-secondary)",
        borderRadius: "var(--radius-sm)",
        border: "1px solid var(--color-border)",
        textAlign: "center",
      }}
    >
      <div
        style={{
          fontSize: "var(--font-size-xs)",
          color: "var(--color-text-secondary)",
          marginBottom: 2,
        }}
      >
        {label}
      </div>
      <div style={{ fontSize: "var(--font-size-base)", fontWeight: 600 }}>
        {value}
      </div>
    </div>
  );
}

export function SummaryCards({ activity, session }: SummaryCardsProps) {
  const cards: { label: string; value: string }[] = [];

  if (activity.duration_secs != null) {
    cards.push({ label: "Duration", value: formatDuration(activity.duration_secs) });
  }
  if (activity.distance_m != null) {
    cards.push({ label: "Distance", value: formatDistance(activity.distance_m, "km") });
  }
  if (activity.pace_s_per_m != null) {
    cards.push({
      label: "Pace",
      value: `${paceToDisplayString(activity.pace_s_per_m, "min/km")} /km`,
    });
  }
  if (session?.total_calories != null) {
    cards.push({ label: "Calories", value: `${Math.round(session.total_calories)}` });
  }
  if (session?.avg_heart_rate != null) {
    cards.push({ label: "Avg HR", value: `${Math.round(session.avg_heart_rate)} bpm` });
  }
  if (session?.max_heart_rate != null) {
    cards.push({ label: "Max HR", value: `${Math.round(session.max_heart_rate)} bpm` });
  }

  if (cards.length === 0) return null;

  return (
    <div
      style={{
        display: "flex",
        gap: "var(--spacing-sm)",
        flexWrap: "wrap",
        marginBottom: "var(--spacing-md)",
      }}
    >
      {cards.map((c) => (
        <MetricCard key={c.label} label={c.label} value={c.value} />
      ))}
    </div>
  );
}
