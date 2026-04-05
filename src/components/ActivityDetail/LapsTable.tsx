import type { FitLap } from "../../lib/fitData";
import { formatDuration } from "../../lib/units";

interface LapsTableProps {
  laps: FitLap[];
}

export function LapsTable({ laps }: LapsTableProps) {
  if (laps.length === 0) return null;

  return (
    <div style={{ overflowX: "auto" }}>
      <h4
        style={{
          fontSize: "var(--font-size-sm)",
          color: "var(--color-text-secondary)",
          marginBottom: "var(--spacing-sm)",
        }}
      >
        Laps
      </h4>
      <table
        style={{
          width: "100%",
          borderCollapse: "collapse",
          fontSize: "var(--font-size-sm)",
        }}
      >
        <thead>
          <tr
            style={{
              borderBottom: "1px solid var(--color-border)",
              textAlign: "left",
            }}
          >
            <th style={{ padding: "var(--spacing-xs) var(--spacing-sm)", color: "var(--color-text-secondary)", fontWeight: 500 }}>
              Lap
            </th>
            <th style={{ padding: "var(--spacing-xs) var(--spacing-sm)", color: "var(--color-text-secondary)", fontWeight: 500 }}>
              Time
            </th>
            <th style={{ padding: "var(--spacing-xs) var(--spacing-sm)", color: "var(--color-text-secondary)", fontWeight: 500 }}>
              Distance
            </th>
            <th style={{ padding: "var(--spacing-xs) var(--spacing-sm)", color: "var(--color-text-secondary)", fontWeight: 500 }}>
              Avg HR
            </th>
            <th style={{ padding: "var(--spacing-xs) var(--spacing-sm)", color: "var(--color-text-secondary)", fontWeight: 500 }}>
              Avg Cadence
            </th>
          </tr>
        </thead>
        <tbody>
          {laps.map((lap, i) => (
            <tr
              key={i}
              style={{ borderBottom: "1px solid var(--color-border)" }}
            >
              <td style={{ padding: "var(--spacing-xs) var(--spacing-sm)" }}>
                {i + 1}
              </td>
              <td style={{ padding: "var(--spacing-xs) var(--spacing-sm)" }}>
                {lap.total_elapsed_time != null
                  ? formatDuration(lap.total_elapsed_time)
                  : "—"}
              </td>
              <td style={{ padding: "var(--spacing-xs) var(--spacing-sm)" }}>
                {lap.total_distance != null
                  ? `${(lap.total_distance / 1000).toFixed(2)} km`
                  : "—"}
              </td>
              <td style={{ padding: "var(--spacing-xs) var(--spacing-sm)" }}>
                {lap.avg_heart_rate != null
                  ? `${Math.round(lap.avg_heart_rate)} bpm`
                  : "—"}
              </td>
              <td style={{ padding: "var(--spacing-xs) var(--spacing-sm)" }}>
                {lap.avg_cadence != null
                  ? `${Math.round(lap.avg_cadence)}`
                  : "—"}
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
