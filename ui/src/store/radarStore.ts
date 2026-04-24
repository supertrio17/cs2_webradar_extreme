import { create } from "zustand";
import { RadarSnapshot } from "../types/contracts";

interface RadarState {
  latest: RadarSnapshot;
  previous: RadarSnapshot;
  update(snapshot: RadarSnapshot): void;
}

const emptySnapshot: RadarSnapshot = {
  protocol_version: 1,
  timestamp_ms: Date.now(),
  tick: 0,
  interpolation_window_ms: 100,
  match_state: {
    map_name: "loading",
    round: 0,
    phase: "waiting",
    score_t: 0,
    score_ct: 0,
    local_team: "unknown"
  },
  players: [],
  bomb: null,
  grenades: [],
  dropped_weapons: [],
  diagnostics: {
    engine_status: "waiting_for_process",
    offset_source: "embedded",
    build_number: null,
    tick_interval_ms: 40,
    stale_entities: 0
  }
};

export const useRadarStore = create<RadarState>((set, get) => ({
  latest: emptySnapshot,
  previous: emptySnapshot,
  update: (snapshot) => {
    set({ previous: get().latest, latest: snapshot });
  }
}));
