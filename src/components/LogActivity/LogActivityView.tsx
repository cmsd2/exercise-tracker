import { useState } from "react";
import {
  ACTIVITY_TYPES,
  ACTIVITY_TYPE_LABELS,
  ACTIVITY_SUB_TYPES,
  ACTIVITY_SUB_TYPE_LABELS,
  HR_ZONES,
  HR_ZONE_LABELS,
  ActivityType,
  ActivitySubType,
  HrZone,
  CreateActivityParams,
} from "../../lib/tauri";
import { useActivityStore } from "../../store/activityStore";
import { computeAutofill, TriadField } from "../../lib/autofill";
import {
  formatDuration,
  parseDuration,
  metresToDisplay,
  displayToMetres,
  paceToDisplayString,
  parsePaceDisplay,
  DistanceUnit,
  PaceUnit,
} from "../../lib/units";

export function LogActivityView() {
  const { createActivity } = useActivityStore();

  const [activityType, setActivityType] = useState<ActivityType>("run");
  const [date, setDate] = useState(() => {
    const now = new Date();
    return now.toISOString().slice(0, 16);
  });
  const [durationDisplay, setDurationDisplay] = useState("");
  const [distanceDisplay, setDistanceDisplay] = useState("");
  const [paceDisplay, setPaceDisplay] = useState("");
  const [subType, setSubType] = useState<ActivitySubType | "">("");
  const [hrZone, setHrZone] = useState<HrZone | "">("");
  const [notes, setNotes] = useState("");
  const [isCommute, setIsCommute] = useState(false);
  const [isRace, setIsRace] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState(false);

  const distanceUnit: DistanceUnit = "km";
  const paceUnit: PaceUnit = "min/km";

  function handleTriadChange(field: TriadField, value: string) {
    let durationSecs: number | null = null;
    let distanceM: number | null = null;
    let paceSPerM: number | null = null;

    if (field === "duration_secs") {
      setDurationDisplay(value);
      durationSecs = parseDuration(value);
      distanceM = parseFloat(distanceDisplay) > 0
        ? displayToMetres(parseFloat(distanceDisplay), distanceUnit)
        : null;
      paceSPerM = parsePaceDisplay(paceDisplay, paceUnit);
    } else if (field === "distance_m") {
      setDistanceDisplay(value);
      distanceM = parseFloat(value) > 0
        ? displayToMetres(parseFloat(value), distanceUnit)
        : null;
      durationSecs = parseDuration(durationDisplay);
      paceSPerM = parsePaceDisplay(paceDisplay, paceUnit);
    } else if (field === "pace_s_per_m") {
      setPaceDisplay(value);
      paceSPerM = parsePaceDisplay(value, paceUnit);
      durationSecs = parseDuration(durationDisplay);
      distanceM = parseFloat(distanceDisplay) > 0
        ? displayToMetres(parseFloat(distanceDisplay), distanceUnit)
        : null;
    }

    const result = computeAutofill(
      {
        duration_secs: durationSecs,
        distance_m: distanceM,
        pace_s_per_m: paceSPerM,
      },
      field
    );

    if (field !== "duration_secs" && result.duration_secs != null) {
      setDurationDisplay(formatDuration(result.duration_secs));
    }
    if (field !== "distance_m" && result.distance_m != null) {
      setDistanceDisplay(
        metresToDisplay(result.distance_m, distanceUnit).toFixed(2)
      );
    }
    if (field !== "pace_s_per_m" && result.pace_s_per_m != null) {
      setPaceDisplay(paceToDisplayString(result.pace_s_per_m, paceUnit));
    }
  }

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    setError(null);
    setSuccess(false);

    const durationSecs = parseDuration(durationDisplay);
    const distanceM =
      parseFloat(distanceDisplay) > 0
        ? displayToMetres(parseFloat(distanceDisplay), distanceUnit)
        : undefined;
    const paceSPerM = parsePaceDisplay(paceDisplay, paceUnit) ?? undefined;

    if (durationSecs == null && distanceM == null) {
      setError("Please enter at least a duration or distance.");
      return;
    }

    const params: CreateActivityParams = {
      activity_type: activityType,
      date: date.length === 16 ? date + ":00" : date,
      duration_secs: durationSecs ?? undefined,
      distance_m: distanceM,
      pace_s_per_m: paceSPerM,
      hr_zone: hrZone || undefined,
      notes: notes || undefined,
      sub_type: subType || undefined,
      is_commute: isCommute || undefined,
      is_race: isRace || undefined,
    };

    try {
      const effect = await createActivity(params);
      if (effect.type === "ValidationError") {
        setError(effect.reason);
      } else {
        setSuccess(true);
        setDurationDisplay("");
        setDistanceDisplay("");
        setPaceDisplay("");
        setSubType("");
        setHrZone("");
        setNotes("");
        setIsCommute(false);
        setIsRace(false);
        setTimeout(() => setSuccess(false), 2000);
      }
    } catch (e) {
      setError(String(e));
    }
  }

  return (
    <div style={{ maxWidth: 500 }}>
      <h2 style={{ marginBottom: "var(--spacing-lg)" }}>Log Activity</h2>

      {error && <div className="error-message">{error}</div>}
      {success && (
        <div
          style={{
            padding: "var(--spacing-sm) var(--spacing-md)",
            background: "rgba(52, 199, 89, 0.1)",
            border: "1px solid var(--color-success)",
            borderRadius: "var(--radius-sm)",
            color: "var(--color-success)",
            fontSize: "var(--font-size-sm)",
            marginBottom: "var(--spacing-md)",
          }}
        >
          Activity logged!
        </div>
      )}

      <form onSubmit={handleSubmit}>
        <div className="form-group">
          <label>Activity Type</label>
          <div className="segmented-control">
            {ACTIVITY_TYPES.map((t) => (
              <button
                key={t}
                type="button"
                className={activityType === t ? "active" : ""}
                onClick={() => {
                  setActivityType(t);
                  setSubType("");
                }}
              >
                {ACTIVITY_TYPE_LABELS[t]}
              </button>
            ))}
          </div>
        </div>

        {ACTIVITY_SUB_TYPES[activityType].length > 0 && (
          <div className="form-group">
            <label>Sub-type</label>
            <div className="segmented-control">
              <button
                type="button"
                className={subType === "" ? "active" : ""}
                onClick={() => setSubType("")}
              >
                Default
              </button>
              {ACTIVITY_SUB_TYPES[activityType].map((st) => (
                <button
                  key={st}
                  type="button"
                  className={subType === st ? "active" : ""}
                  onClick={() => setSubType(st)}
                >
                  {ACTIVITY_SUB_TYPE_LABELS[st]}
                </button>
              ))}
            </div>
          </div>
        )}

        <div className="form-group">
          <label htmlFor="date">Date &amp; Time</label>
          <input
            id="date"
            type="datetime-local"
            className="form-input"
            value={date}
            onChange={(e) => setDate(e.target.value)}
          />
        </div>

        <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr 1fr", gap: "var(--spacing-md)" }}>
          <div className="form-group">
            <label htmlFor="duration">Duration</label>
            <input
              id="duration"
              type="text"
              className="form-input"
              placeholder="MM:SS"
              value={durationDisplay}
              onChange={(e) =>
                handleTriadChange("duration_secs", e.target.value)
              }
            />
          </div>

          <div className="form-group">
            <label htmlFor="distance">Distance ({distanceUnit})</label>
            <input
              id="distance"
              type="text"
              className="form-input"
              placeholder="0.00"
              value={distanceDisplay}
              onChange={(e) =>
                handleTriadChange("distance_m", e.target.value)
              }
            />
          </div>

          <div className="form-group">
            <label htmlFor="pace">Pace ({paceUnit})</label>
            <input
              id="pace"
              type="text"
              className="form-input"
              placeholder="MM:SS"
              value={paceDisplay}
              onChange={(e) =>
                handleTriadChange("pace_s_per_m", e.target.value)
              }
            />
          </div>
        </div>

        <div className="form-group">
          <label>Heart Rate Zone</label>
          <div className="segmented-control">
            <button
              type="button"
              className={hrZone === "" ? "active" : ""}
              onClick={() => setHrZone("")}
            >
              None
            </button>
            {HR_ZONES.map((z) => (
              <button
                key={z}
                type="button"
                className={hrZone === z ? "active" : ""}
                onClick={() => setHrZone(z)}
              >
                {HR_ZONE_LABELS[z]}
              </button>
            ))}
          </div>
        </div>

        <div className="form-group">
          <label htmlFor="notes">Notes</label>
          <textarea
            id="notes"
            className="form-input"
            rows={3}
            value={notes}
            onChange={(e) => setNotes(e.target.value)}
          />
        </div>

        <div
          style={{
            display: "flex",
            gap: "var(--spacing-lg)",
            marginBottom: "var(--spacing-md)",
          }}
        >
          <label style={{ display: "flex", alignItems: "center", gap: "var(--spacing-xs)", cursor: "pointer" }}>
            <input
              type="checkbox"
              checked={isCommute}
              onChange={(e) => setIsCommute(e.target.checked)}
            />
            Commute
          </label>
          <label style={{ display: "flex", alignItems: "center", gap: "var(--spacing-xs)", cursor: "pointer" }}>
            <input
              type="checkbox"
              checked={isRace}
              onChange={(e) => setIsRace(e.target.checked)}
            />
            Race
          </label>
        </div>

        <button type="submit" className="btn btn-primary">
          Save Activity
        </button>
      </form>
    </div>
  );
}
