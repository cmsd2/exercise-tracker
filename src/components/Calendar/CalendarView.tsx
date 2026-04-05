import { useState, useEffect, useCallback } from "react";
import {
  startOfMonth,
  endOfMonth,
  startOfWeek,
  endOfWeek,
  addMonths,
  addWeeks,
  format,
  isSameDay,
} from "date-fns";
import { Activity, listActivities } from "../../lib/tauri";
import { MonthGrid } from "./MonthGrid";
import { WeekGrid } from "./WeekGrid";
import { DayDetail } from "./DayDetail";

type ViewMode = "week" | "month";

export function CalendarView() {
  const [viewMode, setViewMode] = useState<ViewMode>("month");
  const [currentDate, setCurrentDate] = useState(new Date());
  const [selectedDate, setSelectedDate] = useState<Date>(new Date());
  const [activities, setActivities] = useState<Activity[]>([]);
  const [loading, setLoading] = useState(false);

  const getDateRange = useCallback(() => {
    if (viewMode === "month") {
      const monthStart = startOfMonth(currentDate);
      const monthEnd = endOfMonth(currentDate);
      return {
        start: startOfWeek(monthStart, { weekStartsOn: 1 }),
        end: endOfWeek(monthEnd, { weekStartsOn: 1 }),
      };
    } else {
      return {
        start: startOfWeek(currentDate, { weekStartsOn: 1 }),
        end: endOfWeek(currentDate, { weekStartsOn: 1 }),
      };
    }
  }, [viewMode, currentDate]);

  useEffect(() => {
    const { start, end } = getDateRange();
    setLoading(true);
    listActivities({
      date_from: format(start, "yyyy-MM-dd'T'HH:mm:ss"),
      date_to: format(end, "yyyy-MM-dd'T'HH:mm:ss"),
    })
      .then(setActivities)
      .catch(console.error)
      .finally(() => setLoading(false));
  }, [getDateRange]);

  function navigate(direction: -1 | 1) {
    setCurrentDate((d) =>
      viewMode === "month" ? addMonths(d, direction) : addWeeks(d, direction)
    );
  }

  function goToday() {
    const today = new Date();
    setCurrentDate(today);
    setSelectedDate(today);
  }

  const periodLabel =
    viewMode === "month"
      ? format(currentDate, "MMMM yyyy")
      : (() => {
          const ws = startOfWeek(currentDate, { weekStartsOn: 1 });
          const we = endOfWeek(currentDate, { weekStartsOn: 1 });
          return `${format(ws, "MMM d")} – ${format(we, "MMM d, yyyy")}`;
        })();

  const selectedActivities = activities.filter((a) =>
    isSameDay(new Date(a.date), selectedDate)
  );

  return (
    <div>
      <h2 style={{ marginBottom: "var(--spacing-lg)" }}>Calendar</h2>

      {/* Controls */}
      <div
        style={{
          display: "flex",
          gap: "var(--spacing-md)",
          alignItems: "center",
          marginBottom: "var(--spacing-md)",
          flexWrap: "wrap",
        }}
      >
        {/* View toggle */}
        <div className="segmented-control">
          {(["week", "month"] as const).map((mode) => (
            <button
              key={mode}
              className={viewMode === mode ? "active" : ""}
              onClick={() => {
                setViewMode(mode);
                }}
            >
              {mode === "week" ? "Week" : "Month"}
            </button>
          ))}
        </div>

        {/* Navigation */}
        <div
          style={{
            display: "flex",
            alignItems: "center",
            gap: "var(--spacing-sm)",
          }}
        >
          <button className="btn btn-secondary" onClick={() => navigate(-1)}>
            &larr;
          </button>
          <span
            style={{
              minWidth: 180,
              textAlign: "center",
              fontWeight: 600,
              fontSize: "var(--font-size-base)",
            }}
          >
            {periodLabel}
          </span>
          <button className="btn btn-secondary" onClick={() => navigate(1)}>
            &rarr;
          </button>
        </div>

        <button className="btn btn-secondary" onClick={goToday}>
          Today
        </button>

        {loading && (
          <span
            style={{
              fontSize: "var(--font-size-sm)",
              color: "var(--color-text-secondary)",
            }}
          >
            Loading...
          </span>
        )}
      </div>

      {/* Grid */}
      {viewMode === "month" ? (
        <MonthGrid
          currentDate={currentDate}
          activities={activities}
          selectedDate={selectedDate}
          onSelectDate={setSelectedDate}
        />
      ) : (
        <WeekGrid
          currentDate={currentDate}
          activities={activities}
          selectedDate={selectedDate}
          onSelectDate={setSelectedDate}
        />
      )}

      {/* Day detail */}
      <DayDetail date={selectedDate} activities={selectedActivities} />
    </div>
  );
}
