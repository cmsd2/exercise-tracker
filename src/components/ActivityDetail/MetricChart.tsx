import {
  LineChart,
  Line,
  XAxis,
  YAxis,
  Tooltip,
  ResponsiveContainer,
  CartesianGrid,
} from "recharts";
import type { TimeSeriesPoint } from "../../lib/fitData";
import { formatDuration } from "../../lib/units";

interface MetricChartProps {
  data: TimeSeriesPoint[];
  label: string;
  color: string;
  unit: string;
}

export function MetricChart({ data, label, color, unit }: MetricChartProps) {
  if (data.length === 0) return null;

  return (
    <div style={{ width: "100%", height: 280 }}>
      <h4
        style={{
          fontSize: "var(--font-size-sm)",
          color: "var(--color-text-secondary)",
          marginBottom: "var(--spacing-sm)",
        }}
      >
        {label}
      </h4>
      <ResponsiveContainer width="100%" height={250}>
        <LineChart data={data}>
          <CartesianGrid
            strokeDasharray="3 3"
            stroke="var(--color-border)"
          />
          <XAxis
            dataKey="time"
            tickFormatter={(t: number) => formatDuration(t)}
            stroke="var(--color-text-secondary)"
            fontSize={11}
            interval="preserveStartEnd"
          />
          <YAxis
            stroke="var(--color-text-secondary)"
            fontSize={11}
            width={50}
            tickFormatter={(v: number) => Math.round(v).toString()}
          />
          <Tooltip
            labelFormatter={(t) => formatDuration(Number(t))}
            formatter={(v) => [`${Math.round(Number(v))} ${unit}`, label]}
            contentStyle={{
              background: "var(--color-bg-secondary)",
              border: "1px solid var(--color-border)",
              borderRadius: "var(--radius-sm)",
              fontSize: "var(--font-size-sm)",
            }}
          />
          <Line
            type="monotone"
            dataKey="value"
            stroke={color}
            strokeWidth={1.5}
            dot={false}
            isAnimationActive={false}
          />
        </LineChart>
      </ResponsiveContainer>
    </div>
  );
}
