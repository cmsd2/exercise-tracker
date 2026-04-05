import { format } from "date-fns";
import {
  Activity,
  ACTIVITY_TYPE_LABELS,
  ACTIVITY_SUB_TYPE_LABELS,
  HR_ZONE_LABELS,
  ActivitySubType,
} from "../../lib/tauri";
import {
  formatDuration,
  formatDistance,
  paceToDisplayString,
  DistanceUnit,
  PaceUnit,
} from "../../lib/units";
import { ACTIVITY_TYPE_COLORS } from "./DayCell";

interface DayDetailProps {
  date: Date;
  activities: Activity[];
}

export function DayDetail({ date, activities }: DayDetailProps) {
  const distanceUnit: DistanceUnit = "km";
  const paceUnit: PaceUnit = "min/km";

  return (
    <div
      style={{
        marginTop: "var(--spacing-md)",
        padding: "var(--spacing-md)",
        background: "var(--color-bg-secondary)",
        borderRadius: "var(--radius-md)",
        border: "1px solid var(--color-border)",
      }}
    >
      <h3
        style={{
          fontSize: "var(--font-size-base)",
          margin: 0,
          marginBottom: "var(--spacing-sm)",
        }}
      >
        {format(date, "EEEE, MMMM d, yyyy")}
      </h3>

      {activities.length === 0 ? (
        <p
          style={{
            color: "var(--color-text-secondary)",
            fontSize: "var(--font-size-sm)",
            margin: 0,
          }}
        >
          No activities
        </p>
      ) : (
        <div
          style={{
            display: "flex",
            flexDirection: "column",
            gap: "var(--spacing-sm)",
          }}
        >
          {activities.map((a) => (
            <div
              key={a.id}
              style={{
                display: "flex",
                gap: "var(--spacing-sm)",
                alignItems: "start",
                padding: "var(--spacing-sm)",
                background: "var(--color-bg)",
                borderRadius: "var(--radius-sm)",
                border: "1px solid var(--color-border)",
              }}
            >
              <span
                style={{
                  width: 10,
                  height: 10,
                  borderRadius: "50%",
                  flexShrink: 0,
                  marginTop: 4,
                  background: ACTIVITY_TYPE_COLORS[a.activity_type],
                }}
              />
              <div style={{ flex: 1, minWidth: 0 }}>
                <div
                  style={{
                    display: "flex",
                    gap: "var(--spacing-sm)",
                    alignItems: "center",
                    flexWrap: "wrap",
                  }}
                >
                  <strong style={{ fontSize: "var(--font-size-sm)" }}>
                    {ACTIVITY_TYPE_LABELS[a.activity_type]}
                  </strong>
                  {a.sub_type && (
                    <span
                      style={{
                        fontSize: "var(--font-size-xs)",
                        padding: "1px 6px",
                        borderRadius: "var(--radius-sm)",
                        background: "var(--color-bg-tertiary)",
                      }}
                    >
                      {ACTIVITY_SUB_TYPE_LABELS[a.sub_type as ActivitySubType]}
                    </span>
                  )}
                  {a.source === "garmin" && (
                    <span className="badge-garmin">Garmin</span>
                  )}
                  {a.is_commute && (
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
                  {a.is_race && (
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
                  {a.hr_zone && (
                    <span
                      style={{
                        fontSize: "var(--font-size-xs)",
                        padding: "1px 6px",
                        borderRadius: "var(--radius-sm)",
                        background: "var(--color-bg-tertiary)",
                      }}
                    >
                      {HR_ZONE_LABELS[a.hr_zone]}
                    </span>
                  )}
                </div>

                <div
                  style={{
                    display: "flex",
                    gap: "var(--spacing-md)",
                    marginTop: 4,
                    fontSize: "var(--font-size-sm)",
                    color: "var(--color-text-secondary)",
                  }}
                >
                  {a.duration_secs != null && (
                    <span>{formatDuration(a.duration_secs)}</span>
                  )}
                  {a.distance_m != null && (
                    <span>{formatDistance(a.distance_m, distanceUnit)}</span>
                  )}
                  {a.pace_s_per_m != null && (
                    <span>
                      {paceToDisplayString(a.pace_s_per_m, paceUnit)} {paceUnit}
                    </span>
                  )}
                </div>

                {a.notes && (
                  <div
                    style={{
                      marginTop: 4,
                      fontSize: "var(--font-size-sm)",
                      color: "var(--color-text-secondary)",
                      fontStyle: "italic",
                    }}
                  >
                    {a.notes}
                  </div>
                )}
              </div>

              <span
                style={{
                  fontSize: "var(--font-size-xs)",
                  color: "var(--color-text-secondary)",
                  whiteSpace: "nowrap",
                }}
              >
                {format(new Date(a.date), "HH:mm")}
              </span>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
