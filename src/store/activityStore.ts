import { create } from "zustand";
import * as api from "../lib/tauri";

interface ActivityState {
  activities: api.Activity[];
  weeklySummaries: api.WeeklySummary[];
  filter: api.ActivityFilter;
  loading: boolean;
  error: string | null;

  fetchActivities: () => Promise<void>;
  fetchWeeklySummaries: (activityType?: api.ActivityType) => Promise<void>;
  createActivity: (
    params: api.CreateActivityParams
  ) => Promise<api.ActivityEffect>;
  updateActivity: (
    params: api.UpdateActivityParams
  ) => Promise<api.ActivityEffect>;
  deleteActivity: (id: string) => Promise<api.ActivityEffect>;
  setFilter: (filter: Partial<api.ActivityFilter>) => void;
  clearError: () => void;
}

export const useActivityStore = create<ActivityState>((set, get) => ({
  activities: [],
  weeklySummaries: [],
  filter: {},
  loading: false,
  error: null,

  fetchActivities: async () => {
    set({ loading: true, error: null });
    try {
      const activities = await api.listActivities(get().filter);
      set({ activities, loading: false });
    } catch (e) {
      set({ error: String(e), loading: false });
    }
  },

  fetchWeeklySummaries: async (activityType?: api.ActivityType) => {
    try {
      const weeklySummaries = await api.getWeeklySummary(activityType);
      set({ weeklySummaries });
    } catch (e) {
      set({ error: String(e) });
    }
  },

  createActivity: async (params) => {
    const effect = await api.createActivity(params);
    if (effect.type === "Created") {
      await get().fetchActivities();
      await get().fetchWeeklySummaries();
    }
    return effect;
  },

  updateActivity: async (params) => {
    const effect = await api.updateActivity(params);
    if (effect.type === "Updated") {
      await get().fetchActivities();
      await get().fetchWeeklySummaries();
    }
    return effect;
  },

  deleteActivity: async (id) => {
    const effect = await api.deleteActivity(id);
    if (effect.type === "Deleted") {
      await get().fetchActivities();
      await get().fetchWeeklySummaries();
    }
    return effect;
  },

  setFilter: (filter) => {
    set({ filter: { ...get().filter, ...filter } });
    get().fetchActivities();
  },

  clearError: () => set({ error: null }),
}));
