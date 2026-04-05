import { Activity, ActivityType, ACTIVITY_TYPE_LABELS } from "../../lib/tauri";
import { formatDuration } from "../../lib/units";

export const ACTIVITY_TYPE_COLORS: Record<ActivityType, string> = {
  run: "#34c759",
  cycle: "#0071e3",
  swim: "#5ac8fa",
  row: "#ff9500",
  walk: "#af52de",
  hike: "#a2845e",
};

interface DayCellProps {
  date: Date;
  activities: Activity[];
  isToday: boolean;
  isCurrentMonth: boolean;
  isSelected: boolean;
  compact: boolean;
  onClick: () => void;
}

export function DayCell({
  date,
  activities,
  isToday,
  isCurrentMonth,
  isSelected,
  compact,
  onClick,
}: DayCellProps) {
  const dayNumber = date.getDate();

  return (
    <button
      onClick={onClick}
      style={{
        display: "flex",
        flexDirection: "column",
        alignItems: compact ? "center" : "flex-start",
        padding: "var(--spacing-xs)",
        minHeight: compact ? 72 : 120,
        background: isSelected
          ? "var(--color-primary)"
          : "var(--color-bg)",
        border: "1px solid var(--color-border)",
        borderRadius: "var(--radius-sm)",
        cursor: "pointer",
        opacity: isCurrentMonth ? 1 : 0.35,
        position: "relative",
        overflow: "hidden",
        textAlign: "left",
        width: "100%",
        color: isSelected ? "#fff" : "var(--color-text)",
        transition: "background 0.15s",
      }}
    >
      <span
        style={{
          fontSize: "var(--font-size-sm)",
          fontWeight: isToday ? 700 : 400,
          width: 24,
          height: 24,
          lineHeight: "24px",
          textAlign: "center",
          borderRadius: "50%",
          background: isToday && !isSelected ? "var(--color-primary)" : "transparent",
          color: isToday && !isSelected ? "#fff" : isSelected ? "#fff" : "inherit",
        }}
      >
        {dayNumber}
      </span>

      {compact ? (
        <CompactDots activities={activities} isSelected={isSelected} />
      ) : (
        <DetailedEntries activities={activities} isSelected={isSelected} />
      )}
    </button>
  );
}

function CompactDots({
  activities,
  isSelected,
}: {
  activities: Activity[];
  isSelected: boolean;
}) {
  if (activities.length === 0) return null;

  const shown = activities.slice(0, 3);
  const extra = activities.length - shown.length;

  return (
    <div
      style={{
        display: "flex",
        gap: 3,
        marginTop: 2,
        flexWrap: "wrap",
        justifyContent: "center",
        alignItems: "center",
      }}
    >
      {shown.map((a, i) => (
        <span
          key={i}
          style={{
            width: 7,
            height: 7,
            borderRadius: "50%",
            background: isSelected
              ? "rgba(255,255,255,0.8)"
              : ACTIVITY_TYPE_COLORS[a.activity_type],
          }}
        />
      ))}
      {extra > 0 && (
        <span
          style={{
            fontSize: 9,
            lineHeight: 1,
            color: isSelected ? "rgba(255,255,255,0.8)" : "var(--color-text-secondary)",
          }}
        >
          +{extra}
        </span>
      )}
    </div>
  );
}

function DetailedEntries({
  activities,
  isSelected,
}: {
  activities: Activity[];
  isSelected: boolean;
}) {
  if (activities.length === 0) return null;

  const shown = activities.slice(0, 3);
  const extra = activities.length - shown.length;

  return (
    <div
      style={{
        display: "flex",
        flexDirection: "column",
        gap: 2,
        marginTop: 2,
        width: "100%",
        overflow: "hidden",
      }}
    >
      {shown.map((a, i) => (
        <div
          key={i}
          style={{
            display: "flex",
            alignItems: "center",
            gap: 4,
            fontSize: 10,
            lineHeight: 1.2,
            whiteSpace: "nowrap",
            overflow: "hidden",
            textOverflow: "ellipsis",
          }}
        >
          <span
            style={{
              width: 6,
              height: 6,
              borderRadius: "50%",
              flexShrink: 0,
              background: isSelected
                ? "rgba(255,255,255,0.8)"
                : ACTIVITY_TYPE_COLORS[a.activity_type],
            }}
          />
          <span
            style={{
              color: isSelected ? "rgba(255,255,255,0.9)" : "var(--color-text-secondary)",
            }}
          >
            {ACTIVITY_TYPE_LABELS[a.activity_type]}
            {a.duration_secs != null && ` ${formatDuration(a.duration_secs)}`}
          </span>
        </div>
      ))}
      {extra > 0 && (
        <span
          style={{
            fontSize: 9,
            color: isSelected ? "rgba(255,255,255,0.8)" : "var(--color-text-secondary)",
          }}
        >
          +{extra} more
        </span>
      )}
    </div>
  );
}
