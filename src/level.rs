use crate::enemy::Enemy;
use crate::objects::{Bullet, DroppedWeapon, Weapon};
use crate::player::Player;
use crate::tilemap::{MapId, WorldManager};
use macroquad::prelude::*;

#[derive(Clone)]
pub struct Level {
    pub map_id: MapId,
    pub player_start: Vec2,
    pub enemies: Vec<Enemy>,
    pub dropped_weapons: Vec<DroppedWeapon>,
    pub exit_trigger: Rect,
    pub next_map: Option<MapId>,
    pub next_player_pos: Vec2,
}

impl Level {
    pub fn get(map_id: MapId) -> Self {
        match map_id {
            MapId::House => Level {
                map_id: MapId::House,
                player_start: vec2(600.0, 600.0),
                enemies: vec![],
                dropped_weapons: vec![],
                exit_trigger: Rect::new(0.0, 0.0, 350.0, 50.0),
                next_map: Some(MapId::Level1_1),
                next_player_pos: vec2(88.0, 528.0),
            },
            MapId::Level1_1 => Level {
                map_id: MapId::Level1_1,
                player_start: vec2(88.0, 528.0),
                enemies: vec![
                    Enemy::new(vec2(342.0, 900.0), Weapon::Pipe, vec2(1.0, 0.0)),
                    Enemy::new(vec2(540.0, 360.0), Weapon::Knife, vec2(0.0, 1.0)),
                    Enemy::new(vec2(990.0, 220.0), Weapon::Pistol, vec2(0.0, 0.0)),
                    Enemy::new(vec2(990.0, 855.0), Weapon::Pipe, vec2(1.0, 0.0)),
                    Enemy::new(vec2(1620.0, 108.0), Weapon::Knife, vec2(0.0, 1.0)),
                    Enemy::new(vec2(1440.0, 540.0), Weapon::Rifle, vec2(0.0, 0.0)),
                    Enemy::new(vec2(1950.0, 540.0), Weapon::Pistol, vec2(1.0, 0.0)),
                    Enemy::new(vec2(1890.0, 405.0), Weapon::Pipe, vec2(0.0, 0.0)),
                    Enemy::new(vec2(1890.0, 900.0), Weapon::Knife, vec2(0.0, 0.0)),
                ],
                dropped_weapons: vec![DroppedWeapon::new(
                    vec2(250.0, 200.0),
                    Weapon::Rifle,
                    30,
                    0.0,
                )],
                exit_trigger: Rect::new(750.0, 100.0, 80.0, 80.0),
                next_map: Some(MapId::Level1_2),
                next_player_pos: vec2(91.0, 91.0),
            },
            MapId::Level1_2 => Level {
                map_id: MapId::Level1_2,
                player_start: vec2(88.0, 528.0),
                enemies: vec![
                    Enemy::new(vec2(300.0, 200.0), Weapon::Pistol, vec2(0.0, 1.0)),
                    Enemy::new(vec2(500.0, 200.0), Weapon::Pipe, vec2(-1.0, 0.0)),
                ],
                dropped_weapons: vec![],
                exit_trigger: Rect::new(80.0, 80.0, 80.0, 80.0),
                next_map: Some(MapId::Level2_1),
                next_player_pos: vec2(100.0, 100.0),
            },
            MapId::Level2_1 => Level {
                map_id: MapId::Level2_1,
                player_start: vec2(100.0, 100.0),
                enemies: vec![
                    Enemy::new(vec2(250.0, 150.0), Weapon::Pipe, vec2(-1.0, 0.0)),
                    Enemy::new(vec2(450.0, 250.0), Weapon::Pistol, vec2(0.0, -1.0)),
                ],
                dropped_weapons: vec![DroppedWeapon::new(
                    vec2(200.0, 150.0),
                    Weapon::Knife,
                    0,
                    0.0,
                )],
                exit_trigger: Rect::new(750.0, 100.0, 80.0, 80.0),
                next_map: Some(MapId::Level2_2),
                next_player_pos: vec2(100.0, 100.0),
            },
            MapId::Level2_2 => Level {
                map_id: MapId::Level2_2,
                player_start: vec2(100.0, 100.0),
                enemies: vec![
                    Enemy::new(vec2(300.0, 150.0), Weapon::Rifle, vec2(0.0, 1.0)),
                    Enemy::new(vec2(500.0, 300.0), Weapon::Pistol, vec2(-1.0, 0.0)),
                    Enemy::new(vec2(600.0, 150.0), Weapon::Rifle, vec2(-1.0, 0.0)),
                ],
                dropped_weapons: vec![],
                exit_trigger: Rect::new(80.0, 80.0, 80.0, 80.0),
                next_map: None,
                next_player_pos: vec2(0.0, 0.0),
            },
        }
    }
}

pub fn handle_location_switch(
    current_level: &mut Level,
    world_manager: &mut WorldManager,
    player: &mut Player,
    enemies: &mut Vec<Enemy>,
    dropped_weapons: &mut Vec<DroppedWeapon>,
    bullets: &mut Vec<Bullet>,
) {
    if current_level.exit_trigger.contains(player.pos) {
        if let Some(next_map) = current_level.next_map {
            let target_pos = current_level.next_player_pos;
            *current_level = Level::get(next_map);

            world_manager.switch_to(current_level.map_id);
            player.pos = target_pos;
            *enemies = current_level.enemies.clone();
            *dropped_weapons = current_level.dropped_weapons.clone();
            bullets.clear();
        }
    }
}
