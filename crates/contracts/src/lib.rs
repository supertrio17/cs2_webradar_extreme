use serde::{Deserialize, Serialize};

pub const PROTOCOL_VERSION: u16 = 1;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Team {
    Unknown,
    Terrorists,
    CounterTerrorists,
    Spectator,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ThreatBand {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BombStatus {
    Carried,
    Planted,
    Defused,
    Exploded,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SurvivalBand {
    Safe,
    Risky,
    Lethal,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GrenadeKind {
    Smoke,
    Flash,
    Molotov,
    Incendiary,
    He,
    Decoy,
    Unknown,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PlayerState {
    pub entity_id: u32,
    pub steam_name: String,
    pub team: Team,
    pub alive: bool,
    pub position: Vec2,
    pub z: f32,
    pub health: u16,
    pub armor: u16,
    pub money: u32,
    pub weapon: Option<String>,
    pub has_bomb: bool,
    pub is_scoped: bool,
    pub is_flashed: bool,
    pub ping_ms: u32,
    pub visible_proxy: bool,
    pub spotted: bool,
    pub threat_score: f32,
    pub threat_band: ThreatBand,
    pub staleness_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BombState {
    pub status: BombStatus,
    pub position: Option<Vec2>,
    pub site: Option<String>,
    pub timer_remaining_ms: Option<u32>,
    pub defuse_remaining_ms: Option<u32>,
    pub predicted_damage: Option<f32>,
    pub survival_band: Option<SurvivalBand>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GrenadeState {
    pub entity_id: u32,
    pub kind: GrenadeKind,
    pub position: Vec2,
    pub remaining_ms: u32,
    pub owner_team: Team,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DroppedWeaponState {
    pub entity_id: u32,
    pub weapon_name: String,
    pub position: Vec2,
    pub ammo_clip: Option<u16>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MatchState {
    pub map_name: String,
    pub round: u16,
    pub phase: String,
    pub score_t: u8,
    pub score_ct: u8,
    pub local_team: Team,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EngineStatus {
    Healthy,
    Recovering,
    WaitingForProcess,
    DumpMismatch,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SnapshotDiagnostics {
    pub engine_status: EngineStatus,
    pub offset_source: String,
    pub build_number: Option<u32>,
    pub tick_interval_ms: u16,
    pub stale_entities: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RadarSnapshot {
    pub protocol_version: u16,
    pub timestamp_ms: u64,
    pub tick: u64,
    pub interpolation_window_ms: u16,
    pub match_state: MatchState,
    pub players: Vec<PlayerState>,
    pub bomb: Option<BombState>,
    pub grenades: Vec<GrenadeState>,
    pub dropped_weapons: Vec<DroppedWeaponState>,
    pub diagnostics: SnapshotDiagnostics,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RadarEnvelope {
    pub protocol_version: u16,
    pub emitted_at_ms: u64,
    pub snapshot: RadarSnapshot,
}
