import { useState, useEffect } from "react";
import { format } from "date-fns";
import {
  Activity,
  ACTIVITY_TYPE_LABELS,
  ACTIVITY_SUB_TYPE_LABELS,
  HR_ZONE_LABELS,
  ActivitySubType,
  getActivity,
} from "../../lib/tauri";
import {
  formatDuration,
  formatDistance,
  paceToDisplayString,
  DistanceUnit,
  PaceUnit,
} from "../../lib/units";
import { ACTIVITY_TYPE_COLORS } from "./DayCell";
import { RouteMap } from "./RouteMap";

interface DayDetailProps {
  date: Date;
  activities: Activity[];
  onViewActivity?: (id: string) => void;
}

export function DayDetail({ date, activities, onViewActivity }: DayDetailProps) {
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
            <ActivityCard
              key={a.id}
              activity={a}
              distanceUnit={distanceUnit}
              paceUnit={paceUnit}
              onViewActivity={onViewActivity}
            />
          ))}
        </div>
      )}
    </div>
  );
}

function ActivityCard({
  activity: a,
  distanceUnit,
  paceUnit,
  onViewActivity,
}: {
  activity: Activity;
  distanceUnit: DistanceUnit;
  paceUnit: PaceUnit;
  onViewActivity?: (id: string) => void;
}) {
  const [expanded, setExpanded] = useState(false);
  const [routeCoords, setRouteCoords] = useState<[number, number][]>([]);

  useEffect(() => {
    if (!expanded || a.source !== "garmin") return;
    let cancelled = false;
    getActivity(a.id).then((full) => {
      if (cancelled || !full.fit_data) return;
      try {
        const parsed = JSON.parse(full.fit_data);
        const records: unknown[] = parsed?.records ?? [];
        const coords: [number, number][] = records
          .filter(
            (r: any) => r.position_lat != null && r.position_long != null,
          )
          .map((r: any) => [r.position_lat, r.position_long]);
        setRouteCoords(coords);
      } catch {
        // ignore parse errors
      }
    });
    return () => { cancelled = true; };
  }, [expanded, a.id, a.source]);

  return (
    <div
      onClick={() => setExpanded((v) => !v)}
      style={{
        display: "flex",
        gap: "var(--spacing-sm)",
        alignItems: "start",
        padding: "var(--spacing-sm)",
        background: "var(--color-bg)",
        borderRadius: "var(--radius-sm)",
        border: "1px solid var(--color-border)",
        cursor: "pointer",
        transition: "background 0.15s",
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
        {/* Always-visible header */}
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
          {/* Summary metrics inline when collapsed */}
          {!expanded && a.duration_secs != null && (
            <span
              style={{
                fontSize: "var(--font-size-xs)",
                color: "var(--color-text-secondary)",
              }}
            >
              {formatDuration(a.duration_secs)}
            </span>
          )}
          {!expanded && a.distance_m != null && (
            <span
              style={{
                fontSize: "var(--font-size-xs)",
                color: "var(--color-text-secondary)",
              }}
            >
              {formatDistance(a.distance_m, distanceUnit)}
            </span>
          )}
        </div>

        {/* Expanded details */}
        {expanded && (
          <div style={{ marginTop: "var(--spacing-sm)" }}>
            <div
              style={{
                display: "flex",
                gap: "var(--spacing-md)",
                fontSize: "var(--font-size-sm)",
                color: "var(--color-text-secondary)",
              }}
            >
              {a.duration_secs != null && (
                <div>
                  <span style={{ color: "var(--color-text-secondary)" }}>
                    Duration:{" "}
                  </span>
                  <span style={{ color: "var(--color-text)" }}>
                    {formatDuration(a.duration_secs)}
                  </span>
                </div>
              )}
              {a.distance_m != null && (
                <div>
                  <span style={{ color: "var(--color-text-secondary)" }}>
                    Distance:{" "}
                  </span>
                  <span style={{ color: "var(--color-text)" }}>
                    {formatDistance(a.distance_m, distanceUnit)}
                  </span>
                </div>
              )}
              {a.pace_s_per_m != null && (
                <div>
                  <span style={{ color: "var(--color-text-secondary)" }}>
                    Pace:{" "}
                  </span>
                  <span style={{ color: "var(--color-text)" }}>
                    {paceToDisplayString(a.pace_s_per_m, paceUnit)} {paceUnit}
                  </span>
                </div>
              )}
            </div>

            {/* Badges */}
            <div
              style={{
                display: "flex",
                gap: "var(--spacing-sm)",
                marginTop: "var(--spacing-xs)",
                flexWrap: "wrap",
              }}
            >
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

            {a.notes && (
              <div
                style={{
                  marginTop: "var(--spacing-xs)",
                  fontSize: "var(--font-size-sm)",
                  color: "var(--color-text-secondary)",
                  fontStyle: "italic",
                }}
              >
                {a.notes}
              </div>
            )}

            {routeCoords.length > 0 && (
              <div onClick={(e) => e.stopPropagation()}>
                <RouteMap coordinates={routeCoords} />
              </div>
            )}

            {onViewActivity && (
              <button
                className="btn btn-secondary"
                style={{
                  marginTop: "var(--spacing-sm)",
                  fontSize: "var(--font-size-xs)",
                  padding: "var(--spacing-xs) var(--spacing-md)",
                }}
                onClick={(e) => {
                  e.stopPropagation();
                  onViewActivity(a.id);
                }}
              >
                View Details &rarr;
              </button>
            )}
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
  );
}
