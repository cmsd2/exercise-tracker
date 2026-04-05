import { create } from "zustand";
import * as api from "../lib/tauri";

interface GarminState {
  connected: boolean;
  syncing: boolean;
  syncProgress: api.SyncProgress | null;
  lastSyncResult: api.SyncProgress | null;
  error: string | null;

  startLogin: () => Promise<void>;
  checkAuth: () => Promise<void>;
  disconnect: () => Promise<void>;
  syncActivities: (startDate: string, endDate: string) => Promise<void>;
  clearError: () => void;
  clearResult: () => void;
}

export const useGarminStore = create<GarminState>((set) => ({
  connected: false,
  syncing: false,
  syncProgress: null,
  lastSyncResult: null,
  error: null,

  startLogin: async () => {
    try {
      await api.garminStartLogin();
    } catch (e) {
      set({ error: String(e) });
    }
  },

  checkAuth: async () => {
    try {
      const connected = await api.garminCheckAuth();
      set({ connected });
    } catch (e) {
      set({ error: String(e) });
    }
  },

  disconnect: async () => {
    try {
      await api.garminDisconnect();
      set({ connected: false, lastSyncResult: null });
    } catch (e) {
      set({ error: String(e) });
    }
  },

  syncActivities: async (startDate, endDate) => {
    set({ syncing: true, syncProgress: null, lastSyncResult: null, error: null });
    try {
      const result = await api.garminSyncActivities(startDate, endDate);
      set({ syncing: false, lastSyncResult: result, syncProgress: null });
    } catch (e) {
      set({ syncing: false, error: String(e) });
    }
  },

  clearError: () => set({ error: null }),
  clearResult: () => set({ lastSyncResult: null }),
}));
