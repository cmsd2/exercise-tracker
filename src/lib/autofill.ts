export interface TriadFields {
  duration_secs: number | null;
  distance_m: number | null;
  pace_s_per_m: number | null;
}

export type TriadField = "duration_secs" | "distance_m" | "pace_s_per_m";

/**
 * Auto-fill the third field given any two of (duration, distance, pace).
 *
 * The constraint is: duration_secs = distance_m * pace_s_per_m
 *
 * `lastChanged` indicates which field the user just edited.
 * If all three are present, recompute the one the user did NOT just change,
 * preferring to recompute pace (the derived value).
 */
export function computeAutofill(
  fields: TriadFields,
  lastChanged: TriadField
): TriadFields {
  const { duration_secs, distance_m, pace_s_per_m } = fields;
  const result = { ...fields };

  const filled = [duration_secs, distance_m, pace_s_per_m].filter(
    (v) => v != null && v > 0
  ).length;

  if (filled < 2) {
    return result;
  }

  if (lastChanged === "duration_secs") {
    if (duration_secs != null && distance_m != null && distance_m > 0) {
      result.pace_s_per_m = duration_secs / distance_m;
    } else if (
      duration_secs != null &&
      pace_s_per_m != null &&
      pace_s_per_m > 0
    ) {
      result.distance_m = duration_secs / pace_s_per_m;
    }
  } else if (lastChanged === "distance_m") {
    if (distance_m != null && duration_secs != null && duration_secs > 0) {
      result.pace_s_per_m = duration_secs / distance_m;
    } else if (distance_m != null && pace_s_per_m != null && pace_s_per_m > 0) {
      result.duration_secs = distance_m * pace_s_per_m;
    }
  } else if (lastChanged === "pace_s_per_m") {
    if (pace_s_per_m != null && distance_m != null && distance_m > 0) {
      result.duration_secs = distance_m * pace_s_per_m;
    } else if (
      pace_s_per_m != null &&
      duration_secs != null &&
      duration_secs > 0
    ) {
      result.distance_m = duration_secs / pace_s_per_m;
    }
  }

  return result;
}
