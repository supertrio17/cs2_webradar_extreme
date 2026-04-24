import { create } from "zustand";

interface SettingsState {
  showThreatRings: boolean;
  showGrenadeTimers: boolean;
  overlayMode: boolean;
  language: string;
  set<K extends keyof Omit<SettingsState, "set" | "hydrate">>(key: K, value: SettingsState[K]): void;
  hydrate(): void;
}

const storageKey = "cs2_webradar_extreme_settings";

export const useSettingsStore = create<SettingsState>((set, get) => ({
  showThreatRings: true,
  showGrenadeTimers: true,
  overlayMode: false,
  language: "en",
  set: (key, value) => {
    set({ [key]: value } as Partial<SettingsState>);
    const next = get();
    localStorage.setItem(
      storageKey,
      JSON.stringify({
        showThreatRings: next.showThreatRings,
        showGrenadeTimers: next.showGrenadeTimers,
        overlayMode: next.overlayMode,
        language: next.language
      })
    );
  },
  hydrate: () => {
    const raw = localStorage.getItem(storageKey);
    if (!raw) return;
    try {
      const parsed = JSON.parse(raw) as Partial<SettingsState>;
      set({
        showThreatRings: parsed.showThreatRings ?? true,
        showGrenadeTimers: parsed.showGrenadeTimers ?? true,
        overlayMode: parsed.overlayMode ?? false,
        language: parsed.language ?? "en"
      });
    } catch {
      localStorage.removeItem(storageKey);
    }
  }
}));
