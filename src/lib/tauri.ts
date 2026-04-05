import { invoke } from "@tauri-apps/api/core";

export type ActivityType = "run" | "cycle" | "swim" | "row" | "walk" | "hike";
export type HrZone = "Zone1" | "Zone2" | "Zone3" | "Zone4" | "Zone5";
export type ActivitySubType =
  | "treadmill"
  | "trail"
  | "track"
  | "indoor"
  | "road"
  | "mountain"
  | "pool"
  | "open-water"
  | "indoor-row"
  | "casual";

export const ACTIVITY_TYPES: ActivityType[] = [
  "run",
  "cycle",
  "swim",
  "row",
  "walk",
  "hike",
];

export const ACTIVITY_TYPE_LABELS: Record<ActivityType, string> = {
  run: "Run",
  cycle: "Cycle",
  swim: "Swim",
  row: "Row",
  walk: "Walk",
  hike: "Hike",
};

export const ACTIVITY_SUB_TYPES: Record<ActivityType, ActivitySubType[]> = {
  run: ["treadmill", "trail", "track"],
  cycle: ["indoor", "road", "mountain"],
  swim: ["pool", "open-water"],
  row: ["indoor-row"],
  walk: ["casual"],
  hike: [],
};

export const ACTIVITY_SUB_TYPE_LABELS: Record<ActivitySubType, string> = {
  treadmill: "Treadmill",
  trail: "Trail",
  track: "Track",
  indoor: "Indoor",
  road: "Road",
  mountain: "Mountain",
  pool: "Pool",
  "open-water": "Open Water",
  "indoor-row": "Indoor",
  casual: "Casual",
};

export const HR_ZONES: HrZone[] = [
  "Zone1",
  "Zone2",
  "Zone3",
  "Zone4",
  "Zone5",
];

export const HR_ZONE_LABELS: Record<HrZone, string> = {
  Zone1: "Zone 1",
  Zone2: "Zone 2",
  Zone3: "Zone 3",
  Zone4: "Zone 4",
  Zone5: "Zone 5",
};

export interface Activity {
  id: string;
  activity_type: ActivityType;
  date: string;
  duration_secs: number | null;
  distance_m: number | null;
  pace_s_per_m: number | null;
  hr_zone: HrZone | null;
  notes: string | null;
  sub_type: ActivitySubType | null;
  is_commute: boolean;
  is_race: boolean;
  fit_data: string | null;
  source: string | null;
  source_id: string | null;
  created_at: string;
  updated_at: string;
}

export interface ActivityFilter {
  activity_type?: ActivityType;
  sub_type?: ActivitySubType;
  date_from?: string;
  date_to?: string;
  limit?: number;
  offset?: number;
}

export interface WeeklySummary {
  week_start: string;
  activity_count: number;
  total_distance_m: number;
  total_duration_secs: number;
  activity_type: ActivityType | null;
}

export type ActivityEffect =
  | { type: "Created"; id: string }
  | { type: "Updated"; id: string }
  | { type: "Deleted"; id: string }
  | { type: "ValidationError"; reason: string };

export interface CreateActivityParams {
  activity_type: ActivityType;
  date: string;
  duration_secs?: number;
  distance_m?: number;
  pace_s_per_m?: number;
  hr_zone?: HrZone;
  notes?: string;
  sub_type?: ActivitySubType;
  is_commute?: boolean;
  is_race?: boolean;
}

export interface UpdateActivityParams {
  id: string;
  activity_type?: ActivityType;
  date?: string;
  duration_secs?: number | null;
  distance_m?: number | null;
  pace_s_per_m?: number | null;
  hr_zone?: HrZone | null;
  notes?: string | null;
  sub_type?: ActivitySubType | null;
  is_commute?: boolean;
  is_race?: boolean;
}

export async function createActivity(
  params: CreateActivityParams
): Promise<ActivityEffect> {
  return invoke("create_activity", { ...params });
}

export async function updateActivity(
  params: UpdateActivityParams
): Promise<ActivityEffect> {
  return invoke("update_activity", { ...params });
}

export async function deleteActivity(id: string): Promise<ActivityEffect> {
  return invoke("delete_activity", { id });
}

export async function listActivities(
  filter: ActivityFilter
): Promise<Activity[]> {
  return invoke("list_activities", { filter });
}

export async function getActivity(id: string): Promise<Activity> {
  return invoke("get_activity", { id });
}

export async function getWeeklySummary(
  activityType?: ActivityType
): Promise<WeeklySummary[]> {
  return invoke("weekly_summary", { activity_type: activityType ?? null });
}

export async function getPreference(key: string): Promise<string | null> {
  return invoke("get_preference", { key });
}

export async function setPreference(
  key: string,
  value: string
): Promise<void> {
  return invoke("set_preference", { key, value });
}

// Garmin sync

export interface SyncProgress {
  kind: "Started" | "Activity" | "Skipped" | "Finished";
  total?: number;
  current?: number;
  reason?: string;
  imported?: number;
  skipped?: number;
  errors?: number;
}

export async function garminStartLogin(): Promise<void> {
  return invoke("garmin_start_login");
}

export async function garminCheckAuth(): Promise<boolean> {
  return invoke("garmin_check_auth");
}

export async function garminStoreTokens(
  accessToken: string,
  refreshToken?: string,
  expiresAt?: number
): Promise<void> {
  return invoke("garmin_store_tokens", {
    access_token: accessToken,
    refresh_token: refreshToken ?? null,
    expires_at: expiresAt ?? null,
  });
}

export async function garminSyncActivities(
  startDate: string,
  endDate: string
): Promise<SyncProgress> {
  return invoke("garmin_sync_activities", {
    start_date: startDate,
    end_date: endDate,
  });
}

export async function garminDisconnect(): Promise<void> {
  return invoke("garmin_disconnect");
}
