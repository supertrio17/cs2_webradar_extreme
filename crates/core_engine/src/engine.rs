use std::collections::HashMap;

use contracts::{
    BombState, BombStatus, DroppedWeaponState, EngineStatus, GrenadeState, MatchState, PlayerState,
    PROTOCOL_VERSION, RadarSnapshot, SnapshotDiagnostics, SurvivalBand, Team, ThreatBand, Vec2,
};
use dump_runtime::{ActiveDump, DumpSource};

use crate::model::{RawBomb, RawFrame, RawPlayer};

pub trait GameDataProvider {
    fn current_build_number(&self) -> Option<u32>;
    fn poll_frame(&mut self) -> Option<RawFrame>;
}

#[derive(Debug, Clone)]
pub struct EngineConfig {
    pub tick_interval_ms: u16,
    pub interpolation_window_ms: u16,
    pub invalid_frame_threshold: u16,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            tick_interval_ms: 40,
            interpolation_window_ms: 100,
            invalid_frame_threshold: 10,
        }
    }
}

pub struct Engine<P: GameDataProvider> {
    config: EngineConfig,
    provider: P,
    active_dump: ActiveDump,
    class_cache: HashMap<u32, String>,
    invalid_frame_count: u16,
}

impl<P: GameDataProvider> Engine<P> {
    pub fn new(config: EngineConfig, provider: P, active_dump: ActiveDump) -> Self {
        Self {
            config,
            provider,
            active_dump,
            class_cache: HashMap::new(),
            invalid_frame_count: 0,
        }
    }

    pub fn next_snapshot(&mut self) -> RadarSnapshot {
        let maybe_frame = self.provider.poll_frame();
        let Some(frame) = maybe_frame else {
            self.invalid_frame_count = self.invalid_frame_count.saturating_add(1);
            return self.empty_snapshot(EngineStatus::WaitingForProcess);
        };

        if frame.match_state.map_name.trim().is_empty() {
            self.invalid_frame_count = self.invalid_frame_count.saturating_add(1);
            return self.empty_snapshot(EngineStatus::Recovering);
        }

        self.invalid_frame_count = 0;

        let local_origin = frame
            .players
            .iter()
            .find(|player| player.is_local)
            .map(|player| player.position)
            .unwrap_or(Vec2 { x: 0.0, y: 0.0 });

        let players = frame
            .players
            .iter()
            .map(|raw| self.transform_player(raw, local_origin))
            .collect::<Vec<_>>();

        let stale_entities = players
            .iter()
            .filter(|player| player.staleness_ms > u64::from(self.config.tick_interval_ms * 3))
            .count() as u16;

        let bomb = frame
            .bomb
            .as_ref()
            .map(|raw_bomb| self.transform_bomb(raw_bomb, local_origin));

        let grenades = frame
            .grenades
            .iter()
            .map(|raw| GrenadeState {
                entity_id: raw.entity_id,
                kind: raw.kind,
                position: raw.position,
                remaining_ms: raw.remaining_ms,
                owner_team: raw.owner_team,
            })
            .collect::<Vec<_>>();

        let dropped_weapons = frame
            .dropped_weapons
            .iter()
            .map(|raw| DroppedWeaponState {
                entity_id: raw.entity_id,
                weapon_name: raw.weapon_name.clone(),
                position: raw.position,
                ammo_clip: raw.ammo_clip,
            })
            .collect::<Vec<_>>();

        let status = if self.invalid_frame_count >= self.config.invalid_frame_threshold {
            EngineStatus::Recovering
        } else {
            EngineStatus::Healthy
        };

        RadarSnapshot {
            protocol_version: PROTOCOL_VERSION,
            timestamp_ms: frame.timestamp_ms,
            tick: frame.tick,
            interpolation_window_ms: self.config.interpolation_window_ms,
            match_state: MatchState {
                map_name: frame.match_state.map_name,
                round: frame.match_state.round,
                phase: frame.match_state.phase,
                score_t: frame.match_state.score_t,
                score_ct: frame.match_state.score_ct,
                local_team: frame.match_state.local_team,
            },
            players,
            bomb,
            grenades,
            dropped_weapons,
            diagnostics: SnapshotDiagnostics {
                engine_status: status,
                offset_source: match self.active_dump.source {
                    DumpSource::User => "user".to_string(),
                    DumpSource::Embedded => "embedded".to_string(),
                },
                build_number: self.active_dump.pack.info.build_number,
                tick_interval_ms: self.config.tick_interval_ms,
                stale_entities,
            },
        }
    }

    fn empty_snapshot(&self, engine_status: EngineStatus) -> RadarSnapshot {
        RadarSnapshot {
            protocol_version: PROTOCOL_VERSION,
            timestamp_ms: 0,
            tick: 0,
            interpolation_window_ms: self.config.interpolation_window_ms,
            match_state: MatchState {
                map_name: "unknown".to_string(),
                round: 0,
                phase: "waiting".to_string(),
                score_t: 0,
                score_ct: 0,
                local_team: Team::Unknown,
            },
            players: Vec::new(),
            bomb: None,
            grenades: Vec::new(),
            dropped_weapons: Vec::new(),
            diagnostics: SnapshotDiagnostics {
                engine_status,
                offset_source: match self.active_dump.source {
                    DumpSource::User => "user".to_string(),
                    DumpSource::Embedded => "embedded".to_string(),
                },
                build_number: self.active_dump.pack.info.build_number,
                tick_interval_ms: self.config.tick_interval_ms,
                stale_entities: 0,
            },
        }
    }

    fn transform_player(&mut self, raw: &RawPlayer, local_origin: Vec2) -> PlayerState {
        if let Some(weapon) = &raw.weapon {
            self.class_cache.insert(raw.entity_id, weapon.clone());
        }

        let distance = ((raw.position.x - local_origin.x).powi(2)
            + (raw.position.y - local_origin.y).powi(2))
        .sqrt()
        .max(1.0);
        let weapon_factor = weapon_factor(raw.weapon.as_deref());
        let visibility_factor = if raw.visible_proxy || raw.spotted { 0.6 } else { 0.1 };
        let scoped_factor = if raw.is_scoped { 0.35 } else { 0.0 };
        let score = ((900.0 / distance) * weapon_factor
            + visibility_factor
            + scoped_factor
            + f32::from(raw.health) / 160.0)
            .clamp(0.0, 10.0);

        PlayerState {
            entity_id: raw.entity_id,
            steam_name: raw.steam_name.clone(),
            team: raw.team,
            alive: raw.alive,
            position: raw.position,
            z: raw.z,
            health: raw.health,
            armor: raw.armor,
            money: raw.money,
            weapon: raw.weapon.clone(),
            has_bomb: raw.has_bomb,
            is_scoped: raw.is_scoped,
            is_flashed: raw.is_flashed,
            ping_ms: raw.ping_ms,
            visible_proxy: raw.visible_proxy,
            spotted: raw.spotted,
            threat_score: score,
            threat_band: threat_band(score),
            staleness_ms: raw.staleness_ms,
        }
    }

    fn transform_bomb(&self, raw: &RawBomb, local_origin: Vec2) -> BombState {
        let (predicted_damage, survival_band) = match (raw.status, raw.position) {
            (BombStatus::Planted, Some(position)) => {
                let distance = ((position.x - local_origin.x).powi(2)
                    + (position.y - local_origin.y).powi(2))
                .sqrt()
                .max(1.0);
                let damage = (820.0 / distance).clamp(0.0, 100.0);
                (Some(damage), Some(survival_for_damage(damage)))
            }
            _ => (None, None),
        };

        BombState {
            status: raw.status,
            position: raw.position,
            site: raw.site.clone(),
            timer_remaining_ms: raw.timer_remaining_ms,
            defuse_remaining_ms: raw.defuse_remaining_ms,
            predicted_damage,
            survival_band,
        }
    }
}

fn threat_band(score: f32) -> ThreatBand {
    match score {
        s if s < 1.5 => ThreatBand::Low,
        s if s < 3.5 => ThreatBand::Medium,
        s if s < 6.0 => ThreatBand::High,
        _ => ThreatBand::Critical,
    }
}

fn survival_for_damage(damage: f32) -> SurvivalBand {
    match damage {
        d if d < 35.0 => SurvivalBand::Safe,
        d if d < 80.0 => SurvivalBand::Risky,
        _ => SurvivalBand::Lethal,
    }
}

fn weapon_factor(weapon: Option<&str>) -> f32 {
    let Some(weapon) = weapon else {
        return 1.0;
    };

    match weapon.to_ascii_lowercase().as_str() {
        "weapon_awp" => 2.0,
        "weapon_ssg08" => 1.7,
        "weapon_ak47" | "weapon_m4a1" | "weapon_m4a1_silencer" => 1.5,
        "weapon_deagle" | "weapon_revolver" => 1.3,
        "weapon_glock" | "weapon_usp_silencer" => 1.1,
        _ => 1.0,
    }
}

#[cfg(test)]
mod tests {
    use contracts::{BombStatus, Team, Vec2};
    use dump_runtime::{ActiveDump, DumpInfo, DumpPack, DumpSource};
    use serde_json::json;

    use super::*;
    use crate::model::{RawBomb, RawFrame, RawMatchState, RawPlayer};

    struct StaticProvider(Option<RawFrame>);

    impl GameDataProvider for StaticProvider {
        fn current_build_number(&self) -> Option<u32> {
            Some(15000)
        }

        fn poll_frame(&mut self) -> Option<RawFrame> {
            self.0.take()
        }
    }

    fn active_dump() -> ActiveDump {
        ActiveDump {
            source: DumpSource::Embedded,
            pack: DumpPack {
                root_dir: std::path::PathBuf::from("."),
                info: DumpInfo {
                    source: "embedded".to_string(),
                    build_number: Some(15000),
                    generated_at: None,
                    dumper_version: None,
                },
                offsets: json!({}),
                schema: json!({}),
            },
            warnings: Vec::new(),
        }
    }

    #[test]
    fn computes_bomb_survival_prediction() {
        let frame = RawFrame {
            tick: 10,
            timestamp_ms: 100,
            match_state: RawMatchState {
                map_name: "de_dust2".to_string(),
                round: 5,
                phase: "live".to_string(),
                score_t: 2,
                score_ct: 3,
                local_team: Team::CounterTerrorists,
            },
            players: vec![RawPlayer {
                entity_id: 1,
                steam_name: "local".to_string(),
                team: Team::CounterTerrorists,
                alive: true,
                position: Vec2 { x: 0.0, y: 0.0 },
                z: 0.0,
                health: 100,
                armor: 100,
                money: 1000,
                weapon: Some("weapon_m4a1".to_string()),
                has_bomb: false,
                is_scoped: false,
                is_flashed: false,
                ping_ms: 12,
                visible_proxy: true,
                spotted: true,
                staleness_ms: 0,
                is_local: true,
            }],
            bomb: Some(RawBomb {
                status: BombStatus::Planted,
                position: Some(Vec2 { x: 4.0, y: 0.0 }),
                site: Some("A".to_string()),
                timer_remaining_ms: Some(30000),
                defuse_remaining_ms: None,
            }),
            grenades: vec![],
            dropped_weapons: vec![],
        };

        let provider = StaticProvider(Some(frame));
        let mut engine = Engine::new(EngineConfig::default(), provider, active_dump());
        let snapshot = engine.next_snapshot();

        let bomb = snapshot.bomb.expect("bomb exists");
        assert!(bomb.predicted_damage.expect("damage") > 80.0);
    }
}
