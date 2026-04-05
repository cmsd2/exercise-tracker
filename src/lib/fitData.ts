export interface FitSession {
  total_elapsed_time?: number;
  total_timer_time?: number;
  total_distance?: number;
  total_calories?: number;
  avg_heart_rate?: number;
  max_heart_rate?: number;
  avg_cadence?: number;
  max_cadence?: number;
  avg_power?: number;
  max_power?: number;
  avg_speed?: number;
  max_speed?: number;
  total_ascent?: number;
  total_descent?: number;
  avg_altitude?: number;
  max_altitude?: number;
  min_altitude?: number;
  sport?: string;
  sub_sport?: string;
  [key: string]: unknown;
}

export interface FitRecord {
  timestamp?: string;
  heart_rate?: number;
  cadence?: number;
  power?: number;
  speed?: number;
  altitude?: number;
  position_lat?: number;
  position_long?: number;
  distance?: number;
  [key: string]: unknown;
}

export interface FitLap {
  timestamp?: string;
  start_time?: string;
  total_elapsed_time?: number;
  total_timer_time?: number;
  total_distance?: number;
  total_calories?: number;
  avg_heart_rate?: number;
  max_heart_rate?: number;
  avg_cadence?: number;
  avg_power?: number;
  avg_speed?: number;
  max_speed?: number;
  [key: string]: unknown;
}

export interface ParsedFitData {
  session: FitSession | null;
  records: FitRecord[];
  laps: FitLap[];
}

export interface TimeSeriesPoint {
  time: number; // elapsed seconds
  value: number;
}

export function parseFitData(fitDataJson: string | null): ParsedFitData {
  if (!fitDataJson) return { session: null, records: [], laps: [] };
  try {
    const parsed = JSON.parse(fitDataJson);
    const session: FitSession | null = parsed?.session ?? null;
    const records: FitRecord[] = parsed?.records ?? [];
    const laps: FitLap[] = parsed?.laps ?? [];
    return { session, records, laps };
  } catch {
    return { session: null, records: [], laps: [] };
  }
}

export function extractCoordinates(records: FitRecord[]): [number, number][] {
  return records
    .filter((r) => r.position_lat != null && r.position_long != null)
    .map((r) => [r.position_lat!, r.position_long!]);
}

export function extractTimeSeries(
  records: FitRecord[],
  field: keyof FitRecord,
): TimeSeriesPoint[] {
  if (records.length === 0) return [];

  const firstTimestamp = records[0]?.timestamp
    ? new Date(records[0].timestamp).getTime()
    : 0;

  return records
    .filter((r) => r[field] != null && typeof r[field] === "number")
    .map((r) => {
      const t = r.timestamp
        ? (new Date(r.timestamp).getTime() - firstTimestamp) / 1000
        : 0;
      return { time: t, value: r[field] as number };
    });
}

export function hasField(records: FitRecord[], field: keyof FitRecord): boolean {
  return records.some((r) => r[field] != null && typeof r[field] === "number");
}
