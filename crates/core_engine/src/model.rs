use contracts::{BombStatus, GrenadeKind, Team, Vec2};

#[derive(Debug, Clone)]
pub struct RawPlayer {
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
    pub staleness_ms: u64,
    pub is_local: bool,
}

#[derive(Debug, Clone)]
pub struct RawBomb {
    pub status: BombStatus,
    pub position: Option<Vec2>,
    pub site: Option<String>,
    pub timer_remaining_ms: Option<u32>,
    pub defuse_remaining_ms: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct RawGrenade {
    pub entity_id: u32,
    pub kind: GrenadeKind,
    pub position: Vec2,
    pub remaining_ms: u32,
    pub owner_team: Team,
}

#[derive(Debug, Clone)]
pub struct RawDroppedWeapon {
    pub entity_id: u32,
    pub weapon_name: String,
    pub position: Vec2,
    pub ammo_clip: Option<u16>,
}

#[derive(Debug, Clone)]
pub struct RawMatchState {
    pub map_name: String,
    pub round: u16,
    pub phase: String,
    pub score_t: u8,
    pub score_ct: u8,
    pub local_team: Team,
}

#[derive(Debug, Clone)]
pub struct RawFrame {
    pub tick: u64,
    pub timestamp_ms: u64,
    pub match_state: RawMatchState,
    pub players: Vec<RawPlayer>,
    pub bomb: Option<RawBomb>,
    pub grenades: Vec<RawGrenade>,
    pub dropped_weapons: Vec<RawDroppedWeapon>,
}
