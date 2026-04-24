export type Team = "unknown" | "terrorists" | "counter_terrorists" | "spectator";

export type ThreatBand = "low" | "medium" | "high" | "critical";

export type EngineStatus = "healthy" | "recovering" | "waiting_for_process" | "dump_mismatch";

export interface Vec2 {
  x: number;
  y: number;
}

export interface PlayerState {
  entity_id: number;
  steam_name: string;
  team: Team;
  alive: boolean;
  position: Vec2;
  z: number;
  health: number;
  armor: number;
  money: number;
  weapon: string | null;
  has_bomb: boolean;
  is_scoped: boolean;
  is_flashed: boolean;
  ping_ms: number;
  visible_proxy: boolean;
  spotted: boolean;
  threat_score: number;
  threat_band: ThreatBand;
  staleness_ms: number;
}

export interface BombState {
  status: "carried" | "planted" | "defused" | "exploded";
  position: Vec2 | null;
  site: string | null;
  timer_remaining_ms: number | null;
  defuse_remaining_ms: number | null;
  predicted_damage: number | null;
  survival_band: "safe" | "risky" | "lethal" | null;
}

export interface GrenadeState {
  entity_id: number;
  kind: "smoke" | "flash" | "molotov" | "incendiary" | "he" | "decoy" | "unknown";
  position: Vec2;
  remaining_ms: number;
  owner_team: Team;
}

export interface DroppedWeaponState {
  entity_id: number;
  weapon_name: string;
  position: Vec2;
  ammo_clip: number | null;
}

export interface MatchState {
  map_name: string;
  round: number;
  phase: string;
  score_t: number;
  score_ct: number;
  local_team: Team;
}

export interface SnapshotDiagnostics {
  engine_status: EngineStatus;
  offset_source: string;
  build_number: number | null;
  tick_interval_ms: number;
  stale_entities: number;
}

export interface RadarSnapshot {
  protocol_version: number;
  timestamp_ms: number;
  tick: number;
  interpolation_window_ms: number;
  match_state: MatchState;
  players: PlayerState[];
  bomb: BombState | null;
  grenades: GrenadeState[];
  dropped_weapons: DroppedWeaponState[];
  diagnostics: SnapshotDiagnostics;
}

export interface RadarEnvelope {
  protocol_version: number;
  emitted_at_ms: number;
  snapshot: RadarSnapshot;
}
