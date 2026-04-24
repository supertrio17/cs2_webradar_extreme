import { RadarEnvelope, RadarSnapshot } from "../types/contracts";

const mockNames = ["Apex", "Nexa", "Fury", "Rogue", "Shade", "Echo", "Nova", "Pulse", "Zen", "Viper"];

function mockSnapshot(tick: number): RadarSnapshot {
  const players = mockNames.map((name, index) => {
    const angle = tick / 20 + index;
    const radius = 280 + index * 20;
    return {
      entity_id: index + 1,
      steam_name: name,
      team: index < 5 ? "counter_terrorists" : "terrorists",
      alive: true,
      position: { x: Math.cos(angle) * radius, y: Math.sin(angle) * radius },
      z: 0,
      health: 100,
      armor: 80,
      money: 3400,
      weapon: index % 2 ? "weapon_m4a1" : "weapon_ak47",
      has_bomb: index === 7,
      is_scoped: index % 3 === 0,
      is_flashed: false,
      ping_ms: 16 + index,
      visible_proxy: index % 2 === 0,
      spotted: true,
      threat_score: 1 + index / 2,
      threat_band: index > 7 ? "critical" : index > 5 ? "high" : "medium",
      staleness_ms: index % 4 === 0 ? 80 : 20
    };
  });

  return {
    protocol_version: 1,
    timestamp_ms: Date.now(),
    tick,
    interpolation_window_ms: 100,
    match_state: {
      map_name: "de_dust2",
      round: Math.floor((tick % 150) / 5),
      phase: "live",
      score_t: 7,
      score_ct: 8,
      local_team: "counter_terrorists"
    },
    players,
    bomb:
      tick % 120 > 70
        ? {
            status: "planted",
            position: { x: 20, y: -40 },
            site: tick % 2 ? "A" : "B",
            timer_remaining_ms: 27000 - ((tick % 120) - 70) * 200,
            defuse_remaining_ms: 8000,
            predicted_damage: 54,
            survival_band: "risky"
          }
        : {
            status: "carried",
            position: null,
            site: null,
            timer_remaining_ms: null,
            defuse_remaining_ms: null,
            predicted_damage: null,
            survival_band: null
          },
    grenades: [
      {
        entity_id: tick,
        kind: tick % 2 ? "smoke" : "flash",
        position: { x: -120 + (tick % 30) * 4, y: 40 },
        remaining_ms: 2000,
        owner_team: "counter_terrorists"
      }
    ],
    dropped_weapons: [
      {
        entity_id: 900,
        weapon_name: "weapon_awp",
        position: { x: 80, y: 60 },
        ammo_clip: 5
      }
    ],
    diagnostics: {
      engine_status: "healthy",
      offset_source: "embedded",
      build_number: 15000,
      tick_interval_ms: 40,
      stale_entities: 2
    }
  };
}

export async function startSnapshotStream(onSnapshot: (snapshot: RadarSnapshot) => void) {
  const tauri = (window as Window & { __TAURI__?: { event?: { listen: Function } } }).__TAURI__;

  if (tauri?.event?.listen) {
    const unlisten = await tauri.event.listen("radar_snapshot", (event: { payload: RadarEnvelope }) => {
      onSnapshot(event.payload.snapshot);
    });
    return () => {
      unlisten();
    };
  }

  let tick = 0;
  const interval = window.setInterval(() => {
    tick += 1;
    onSnapshot(mockSnapshot(tick));
  }, 50);

  return () => window.clearInterval(interval);
}
