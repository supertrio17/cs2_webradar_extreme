use std::time::{SystemTime, UNIX_EPOCH};

use rand::Rng;

use contracts::{BombStatus, GrenadeKind, Team, Vec2};

use crate::engine::GameDataProvider;
use crate::model::{
    RawBomb, RawDroppedWeapon, RawFrame, RawGrenade, RawMatchState, RawPlayer,
};

pub struct MockProvider {
    tick: u64,
    rng: rand::rngs::ThreadRng,
}

impl MockProvider {
    pub fn new() -> Self {
        Self {
            tick: 0,
            rng: rand::rng(),
        }
    }
}

impl Default for MockProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl GameDataProvider for MockProvider {
    fn current_build_number(&self) -> Option<u32> {
        Some(15000)
    }

    fn poll_frame(&mut self) -> Option<RawFrame> {
        self.tick += 1;

        let timestamp_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .ok()
            .map_or(0, |d| d.as_millis() as u64);

        let mut players = Vec::with_capacity(10);
        for i in 0..10 {
            let is_ct = i < 5;
            let angle = (self.tick as f32 / 20.0) + i as f32;
            let radius = 350.0 + (i as f32 * 10.0);
            let x = angle.cos() * radius;
            let y = angle.sin() * radius;
            let team = if is_ct {
                Team::CounterTerrorists
            } else {
                Team::Terrorists
            };
            players.push(RawPlayer {
                entity_id: i as u32 + 1,
                steam_name: format!("Player{}", i + 1),
                team,
                alive: true,
                position: Vec2 { x, y },
                z: 0.0,
                health: self.rng.random_range(55..=100),
                armor: self.rng.random_range(0..=100),
                money: self.rng.random_range(0..=8500),
                weapon: Some(if i % 3 == 0 {
                    "weapon_ak47".to_string()
                } else if i % 2 == 0 {
                    "weapon_awp".to_string()
                } else {
                    "weapon_m4a1".to_string()
                }),
                has_bomb: i == 7,
                is_scoped: i % 2 == 0,
                is_flashed: i % 5 == 0,
                ping_ms: self.rng.random_range(8..=70),
                visible_proxy: i % 2 == 1,
                spotted: i % 3 != 0,
                staleness_ms: self.rng.random_range(0..=90),
                is_local: i == 0,
            });
        }

        let bomb = if self.tick % 200 < 100 {
            Some(RawBomb {
                status: BombStatus::Carried,
                position: None,
                site: None,
                timer_remaining_ms: None,
                defuse_remaining_ms: None,
            })
        } else {
            Some(RawBomb {
                status: BombStatus::Planted,
                position: Some(Vec2 { x: 20.0, y: -40.0 }),
                site: Some(if self.tick % 2 == 0 {
                    "A".to_string()
                } else {
                    "B".to_string()
                }),
                timer_remaining_ms: Some(((40_000_i64 - (self.tick as i64 * 120)).max(0)) as u32),
                defuse_remaining_ms: Some(9_500),
            })
        };

        let grenades = vec![RawGrenade {
            entity_id: 1000 + self.tick as u32,
            kind: if self.tick % 2 == 0 {
                GrenadeKind::Smoke
            } else {
                GrenadeKind::Flash
            },
            position: Vec2 {
                x: self.rng.random_range(-500.0..500.0),
                y: self.rng.random_range(-500.0..500.0),
            },
            remaining_ms: self.rng.random_range(300..2600),
            owner_team: if self.tick % 2 == 0 {
                Team::CounterTerrorists
            } else {
                Team::Terrorists
            },
        }];

        let dropped_weapons = vec![RawDroppedWeapon {
            entity_id: 2000,
            weapon_name: "weapon_ak47".to_string(),
            position: Vec2 { x: 100.0, y: 60.0 },
            ammo_clip: Some(18),
        }];

        Some(RawFrame {
            tick: self.tick,
            timestamp_ms,
            match_state: RawMatchState {
                map_name: "de_dust2".to_string(),
                round: ((self.tick / 200) % 30) as u16,
                phase: "live".to_string(),
                score_t: ((self.tick / 400) % 13) as u8,
                score_ct: ((self.tick / 450) % 13) as u8,
                local_team: Team::CounterTerrorists,
            },
            players,
            bomb,
            grenades,
            dropped_weapons,
        })
    }
}
