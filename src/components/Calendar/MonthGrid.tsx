import {
  startOfMonth,
  endOfMonth,
  startOfWeek,
  endOfWeek,
  eachDayOfInterval,
  isSameDay,
  isSameMonth,
  isToday,
} from "date-fns";
import { Activity } from "../../lib/tauri";
import { DayCell } from "./DayCell";

interface MonthGridProps {
  currentDate: Date;
  activities: Activity[];
  selectedDate: Date | null;
  onSelectDate: (date: Date) => void;
}

const DAY_HEADERS = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];

export function MonthGrid({
  currentDate,
  activities,
  selectedDate,
  onSelectDate,
}: MonthGridProps) {
  const monthStart = startOfMonth(currentDate);
  const monthEnd = endOfMonth(currentDate);
  const gridStart = startOfWeek(monthStart, { weekStartsOn: 1 });
  const gridEnd = endOfWeek(monthEnd, { weekStartsOn: 1 });
  const days = eachDayOfInterval({ start: gridStart, end: gridEnd });

  const activitiesByDay = groupByDay(activities);

  return (
    <div>
      <div
        style={{
          display: "grid",
          gridTemplateColumns: "repeat(7, 1fr)",
          gap: 1,
          marginBottom: 1,
        }}
      >
        {DAY_HEADERS.map((d) => (
          <div
            key={d}
            style={{
              textAlign: "center",
              fontSize: "var(--font-size-xs)",
              fontWeight: 600,
              color: "var(--color-text-secondary)",
              padding: "var(--spacing-xs) 0",
            }}
          >
            {d}
          </div>
        ))}
      </div>
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
            <DayCell
              key={key}
              date={day}
              activities={activitiesByDay[key] ?? []}
              isToday={isToday(day)}
              isCurrentMonth={isSameMonth(day, currentDate)}
              isSelected={selectedDate != null && isSameDay(day, selectedDate)}
              compact={true}
              onClick={() => onSelectDate(day)}
            />
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
