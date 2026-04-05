export type DistanceUnit = "km" | "mi";
export type PaceUnit = "min/km" | "min/mi";
export type SpeedUnit = "km/h" | "mph";

const KM_PER_MILE = 1.60934;

// Distance conversions

export function metresToKm(metres: number): number {
  return metres / 1000;
}

export function metresToMiles(metres: number): number {
  return metres / 1000 / KM_PER_MILE;
}

export function metresToDisplay(metres: number, unit: DistanceUnit): number {
  return unit === "km" ? metresToKm(metres) : metresToMiles(metres);
}

export function displayToMetres(value: number, unit: DistanceUnit): number {
  return unit === "km" ? value * 1000 : value * 1000 * KM_PER_MILE;
}

// Pace conversions
// Internal: seconds per metre
// Display: "MM:SS" per km or per mile

export function paceToMinPerUnit(
  sPerM: number,
  unit: PaceUnit
): { minutes: number; seconds: number } {
  const metresPerUnit = unit === "min/km" ? 1000 : 1000 * KM_PER_MILE;
  const totalSeconds = sPerM * metresPerUnit;
  const minutes = Math.floor(totalSeconds / 60);
  const seconds = Math.round(totalSeconds % 60);
  return { minutes, seconds };
}

export function paceToDisplayString(sPerM: number, unit: PaceUnit): string {
  const { minutes, seconds } = paceToMinPerUnit(sPerM, unit);
  return `${minutes}:${seconds.toString().padStart(2, "0")}`;
}

export function parsePaceDisplay(
  display: string,
  unit: PaceUnit
): number | null {
  const match = display.match(/^(\d+):(\d{1,2})$/);
  if (!match) return null;
  const minutes = parseInt(match[1], 10);
  const seconds = parseInt(match[2], 10);
  if (seconds >= 60) return null;
  const totalSeconds = minutes * 60 + seconds;
  const metresPerUnit = unit === "min/km" ? 1000 : 1000 * KM_PER_MILE;
  return totalSeconds / metresPerUnit;
}

// Duration formatting

export function formatDuration(secs: number): string {
  const hours = Math.floor(secs / 3600);
  const minutes = Math.floor((secs % 3600) / 60);
  const seconds = Math.round(secs % 60);

  if (hours > 0) {
    return `${hours}:${minutes.toString().padStart(2, "0")}:${seconds.toString().padStart(2, "0")}`;
  }
  return `${minutes}:${seconds.toString().padStart(2, "0")}`;
}

export function parseDuration(display: string): number | null {
  const parts = display.split(":").map((p) => parseInt(p, 10));
  if (parts.some(isNaN)) return null;

  if (parts.length === 3) {
    const [h, m, s] = parts;
    if (m >= 60 || s >= 60) return null;
    return h * 3600 + m * 60 + s;
  }
  if (parts.length === 2) {
    const [m, s] = parts;
    if (s >= 60) return null;
    return m * 60 + s;
  }
  return null;
}

export function formatDistance(metres: number, unit: DistanceUnit): string {
  const value = metresToDisplay(metres, unit);
  return `${value.toFixed(2)} ${unit}`;
}
