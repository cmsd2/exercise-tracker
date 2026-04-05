import {
  startOfWeek,
  endOfWeek,
  eachDayOfInterval,
  isSameDay,
  isToday,
  format,
} from "date-fns";
import { Activity } from "../../lib/tauri";
import { DayCell } from "./DayCell";

interface WeekGridProps {
  currentDate: Date;
  activities: Activity[];
  selectedDate: Date | null;
  onSelectDate: (date: Date) => void;
}

export function WeekGrid({
  currentDate,
  activities,
  selectedDate,
  onSelectDate,
}: WeekGridProps) {
  const weekStart = startOfWeek(currentDate, { weekStartsOn: 1 });
  const weekEnd = endOfWeek(currentDate, { weekStartsOn: 1 });
  const days = eachDayOfInterval({ start: weekStart, end: weekEnd });

  const activitiesByDay = groupByDay(activities);

  return (
    <div>
      <div
        style={{
          display: "grid",
          gridTemplateColumns: "repeat(7, 1fr)",
          gap: 2,
        }}
      >
        {days.map((day) => {
          const key = dayKey(day);
          return (
            <div key={key} style={{ display: "flex", flexDirection: "column" }}>
              <div
                style={{
                  textAlign: "center",
                  fontSize: "var(--font-size-xs)",
                  fontWeight: 600,
                  color: "var(--color-text-secondary)",
                  padding: "var(--spacing-xs) 0",
                }}
              >
                {format(day, "EEE")}
              </div>
              <DayCell
                date={day}
                activities={activitiesByDay[key] ?? []}
                isToday={isToday(day)}
                isCurrentMonth={true}
                isSelected={
                  selectedDate != null && isSameDay(day, selectedDate)
                }
                compact={false}
                onClick={() => onSelectDate(day)}
              />
            </div>
          );
        })}
      </div>
    </div>
  );
}

function dayKey(date: Date): string {
  return `${date.getFullYear()}-${String(date.getMonth() + 1).padStart(2, "0")}-${String(date.getDate()).padStart(2, "0")}`;
}

function groupByDay(activities: Activity[]): Record<string, Activity[]> {
  const map: Record<string, Activity[]> = {};
  for (const a of activities) {
    const d = new Date(a.date);
    const key = dayKey(d);
    (map[key] ??= []).push(a);
  }
  return map;
}
