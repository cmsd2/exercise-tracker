import { useEffect } from "react";
import { useActivityStore } from "../../store/activityStore";
import {
  ACTIVITY_TYPES,
  ACTIVITY_TYPE_LABELS,
  ACTIVITY_SUB_TYPE_LABELS,
  ActivityType,
  Activity,
  ActivitySubType,
} from "../../lib/tauri";
import {
  formatDuration,
  formatDistance,
  paceToDisplayString,
  DistanceUnit,
  PaceUnit,
} from "../../lib/units";
import { HR_ZONE_LABELS } from "../../lib/tauri";

export function HistoryView() {
  const { activities, filter, fetchActivities, setFilter, deleteActivity } =
    useActivityStore();
  const distanceUnit: DistanceUnit = "km";
  const paceUnit: PaceUnit = "min/km";

  useEffect(() => {
    fetchActivities();
  }, []);

  async function handleDelete(id: string) {
    if (confirm("Delete this activity?")) {
      await deleteActivity(id);
    }
  }

  return (
    <div>
      <h2 style={{ marginBottom: "var(--spacing-lg)" }}>History</h2>

      <div
        style={{
          display: "flex",
          gap: "var(--spacing-md)",
          marginBottom: "var(--spacing-lg)",
          flexWrap: "wrap",
          alignItems: "end",
        }}
      >
        <div className="form-group" style={{ marginBottom: 0 }}>
          <label>Activity Type</label>
          <select
            className="form-input"
            value={filter.activity_type ?? ""}
            onChange={(e) =>
              setFilter({
                activity_type: (e.target.value as ActivityType) || undefined,
              })
            }
          >
            <option value="">All</option>
            {ACTIVITY_TYPES.map((t) => (
              <option key={t} value={t}>
                {ACTIVITY_TYPE_LABELS[t]}
              </option>
            ))}
          </select>
        </div>

        <div className="form-group" style={{ marginBottom: 0 }}>
          <label>From</label>
          <input
            type="date"
            className="form-input"
            value={filter.date_from?.slice(0, 10) ?? ""}
            onChange={(e) =>
              setFilter({
                date_from: e.target.value
                  ? e.target.value + "T00:00:00"
                  : undefined,
              })
            }
          />
        </div>

        <div className="form-group" style={{ marginBottom: 0 }}>
          <label>To</label>
          <input
            type="date"
            className="form-input"
            value={filter.date_to?.slice(0, 10) ?? ""}
            onChange={(e) =>
              setFilter({
                date_to: e.target.value
                  ? e.target.value + "T23:59:59"
                  : undefined,
              })
            }
          />
        </div>
      </div>

      {activities.length === 0 ? (
        <p style={{ color: "var(--color-text-secondary)" }}>
          No activities found. Log your first activity!
        </p>
      ) : (
        <div style={{ display: "flex", flexDirection: "column", gap: "var(--spacing-sm)" }}>
          {activities.map((activity) => (
            <ActivityCard
              key={activity.id}
              activity={activity}
              distanceUnit={distanceUnit}
              paceUnit={paceUnit}
              onDelete={() => handleDelete(activity.id)}
            />
          ))}
        </div>
      )}
    </div>
  );
}

interface ActivityCardProps {
  activity: Activity;
  distanceUnit: DistanceUnit;
  paceUnit: PaceUnit;
  onDelete: () => void;
}

function ActivityCard({
  activity,
  distanceUnit,
  paceUnit,
  onDelete,
}: ActivityCardProps) {
  const dateStr = new Date(activity.date).toLocaleDateString(undefined, {
    weekday: "short",
    year: "numeric",
    month: "short",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  });

  return (
    <div className="card">
      <div
        style={{
          display: "flex",
          justifyContent: "space-between",
          alignItems: "start",
        }}
      >
        <div>
          <div
            style={{
              display: "flex",
              gap: "var(--spacing-sm)",
              alignItems: "center",
              marginBottom: "var(--spacing-xs)",
            }}
          >
            <strong>
              {ACTIVITY_TYPE_LABELS[activity.activity_type]}
            </strong>
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
          <div
            style={{
              fontSize: "var(--font-size-sm)",
              color: "var(--color-text-secondary)",
            }}
          >
            {dateStr}
          </div>
        </div>
        <button
          className="btn btn-danger"
          style={{ fontSize: "var(--font-size-xs)", padding: "2px 8px" }}
          onClick={onDelete}
        >
          Delete
        </button>
      </div>

      <div
        style={{
          display: "flex",
          gap: "var(--spacing-lg)",
          marginTop: "var(--spacing-sm)",
          fontSize: "var(--font-size-sm)",
        }}
      >
        {activity.duration_secs != null && (
          <div>
            <span style={{ color: "var(--color-text-secondary)" }}>
              Duration:{" "}
            </span>
            {formatDuration(activity.duration_secs)}
          </div>
        )}
        {activity.distance_m != null && (
          <div>
            <span style={{ color: "var(--color-text-secondary)" }}>
              Distance:{" "}
            </span>
            {formatDistance(activity.distance_m, distanceUnit)}
          </div>
        )}
        {activity.pace_s_per_m != null && (
          <div>
            <span style={{ color: "var(--color-text-secondary)" }}>
              Pace:{" "}
            </span>
            {paceToDisplayString(activity.pace_s_per_m, paceUnit)} {paceUnit}
          </div>
        )}
      </div>

      {activity.notes && (
        <div
          style={{
            marginTop: "var(--spacing-sm)",
            fontSize: "var(--font-size-sm)",
            color: "var(--color-text-secondary)",
            fontStyle: "italic",
          }}
        >
          {activity.notes}
        </div>
      )}
    </div>
  );
}
