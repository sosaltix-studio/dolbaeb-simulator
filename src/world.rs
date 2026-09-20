use crate::SCALE;
use crate::assets::Assets;
use crate::audio::AudioManager;
use crate::camera::CameraManager;
use crate::enemy::{Enemy, alert_enemies};
use crate::input::{GameInput, VirtualCursor, collect_input};
use crate::objects::{Bullet, DroppedWeapon, Phone, Weapon, update_bullets};
use crate::player::Player;
use crate::tilemap::{MapId, WorldManager};
use crate::ui::{GameState, UiManager, draw_cursor, draw_dead_menu, draw_ui};
use macroquad::prelude::*;

pub struct Game {
    pub assets: Assets,
    pub world_manager: WorldManager,
    pub audio_manager: AudioManager,
    pub player: Player,
    pub phone: Phone,
    pub enemies: Vec<Enemy>,
    pub current_level: Level,
    pub bullets: Vec<Bullet>,
    pub dropped_weapons: Vec<DroppedWeapon>,
    pub last_shot_pos: Option<Vec2>,
    pub ui_manager: UiManager,
    pub camera_manager: CameraManager,
    pub virtual_cursor: VirtualCursor,
    pub input: GameInput,
}

impl Game {
    pub async fn new() -> Self {
        let current_level = Level::get(MapId::House);
        let mut player = Player::new();
        player.pos = current_level.player_start;

        Self {
            assets: Assets::load().await,
            world_manager: WorldManager::init(SCALE),
            audio_manager: AudioManager::load().await,
            player,
            phone: Phone::new(),
            enemies: current_level.enemies.clone(),
            current_level,
            bullets: Vec::with_capacity(128),
            dropped_weapons: Vec::new(),
            last_shot_pos: None,
            ui_manager: UiManager::new(),
            camera_manager: CameraManager::new(),
            virtual_cursor: VirtualCursor::new(),
            input: GameInput::default(),
        }
    }

    pub fn restart(&mut self) {
        self.world_manager.switch_to(self.current_level.map_id);
        self.player.restart(self.current_level.player_start);
        self.enemies = self.current_level.enemies.clone();
        self.dropped_weapons = self.current_level.dropped_weapons.clone();
        self.bullets.clear();
        self.last_shot_pos = None;
    }

    pub fn update(&mut self, dt: f32) {
        self.input = collect_input(&self.camera_manager.camera);

        self.virtual_cursor.update(
            self.player.is_dead,
            self.input.raw_aim_world_pos,
            &self.enemies,
            self.input.toggle_lock_on,
        );

        if self.virtual_cursor.is_locked() {
            self.input.aim_world_pos = self.virtual_cursor.world_pos;
        } else {
            self.input.aim_world_pos = self.input.raw_aim_world_pos;
        }

        if self.ui_manager.game_state == GameState::Playing
            && self.ui_manager.previous_state == GameState::Terminal
        {
            self.restart();
            self.ui_manager.previous_state = GameState::Playing;
        }

        if self.ui_manager.restart_requested {
            self.restart();
            self.ui_manager.restart_requested = false;
        }

        if self.ui_manager.game_state == GameState::Terminal {
            self.ui_manager.update(&mut self.audio_manager, &self.input);
            self.ui_manager.draw(&self.assets);
        } else {
            if self.ui_manager.game_state == GameState::Playing {
                if self.input.debug_toggl_effect {
                    self.camera_manager.toggle_story_effect();
                }

                self.player.update(
                    &self.input,
                    dt,
                    &self.world_manager,
                    &mut self.dropped_weapons,
                    &mut self.bullets,
                    &self.audio_manager,
                    &mut self.last_shot_pos,
                    &mut self.enemies,
                    self.input.aim_world_pos,
                );

                let camera_focus = self.input.look_around || self.virtual_cursor.is_locked();
                self.camera_manager.update(
                    self.player.pos,
                    self.input.aim_world_pos,
                    camera_focus,
                    dt,
                );

                self.world_manager.update_flow_field(self.player.pos);

                update_bullets(
                    &mut self.bullets,
                    &mut self.player,
                    &mut self.enemies,
                    &self.world_manager,
                    &mut self.dropped_weapons,
                    dt,
                );

                self.phone.update(dt);

                for enemy in &mut self.enemies {
                    enemy.update(
                        &mut self.player,
                        &self.world_manager,
                        &mut self.dropped_weapons,
                        &mut self.bullets,
                        dt,
                        &self.audio_manager,
                    );
                }

                if let Some(shot_pos) = self.last_shot_pos.take() {
                    alert_enemies(&mut self.enemies, &shot_pos);
                }

                handle_location_switch(
                    &mut self.current_level,
                    &mut self.world_manager,
                    &mut self.player,
                    &mut self.enemies,
                    &mut self.dropped_weapons,
                    &mut self.bullets,
                );
            }

            clear_background(BLACK);

            set_camera(&self.camera_manager.camera);

            self.camera_manager.begin_render();

            self.world_manager.draw();
            for bullet in &self.bullets {
                bullet.draw();
            }
            for enemy in &self.enemies {
                enemy.draw(&self.assets);
            }
            for item in &self.dropped_weapons {
                item.draw(&self.assets);
            }
            self.player.draw(&self.assets);

            self.camera_manager.end_render();

            set_default_camera();

            draw_ui(&self.assets, &self.player);
            self.phone.draw(&self.assets);

            if self.player.is_dead {
                draw_dead_menu(&self.assets);
                if self.input.restart_pressed {
                    self.restart();
                }
            }
            self.ui_manager.update(&mut self.audio_manager, &self.input);
        }

        if self.virtual_cursor.is_locked() {
            let cursor_screen_pos = self
                .camera_manager
                .camera
                .world_to_screen(self.virtual_cursor.world_pos);
            draw_cursor(&self.assets, cursor_screen_pos);
        } else {
            draw_cursor(&self.assets, mouse_position().into());
        }
    }
}

#[derive(Clone)]
pub struct Level {
    pub map_id: MapId,
    pub player_start: Vec2,
    pub enemies: Vec<Enemy>,
    pub dropped_weapons: Vec<DroppedWeapon>,
    pub exit_trigger: Rect,
    pub next_map: Option<MapId>,
    pub next_player_pos: Vec2,
    //pub requires_all_enemies_dead: bool,
    //pub requires_phone_read: bool,
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
                //requires_all_enemies_dead: false,
                //requires_phone_read: true,
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
                //requires_all_enemies_dead: true,
                //requires_phone_read: false,
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
                //requires_all_enemies_dead: true,
                //requires_phone_read: false,
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
                //requires_all_enemies_dead: true,
                //requires_phone_read: false,
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
                //requires_all_enemies_dead: true,
                //requires_phone_read: false,
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
    //phone_read: bool,
    //state: &mut GameState,
    //status_msg: &mut Option<(&'static str, f32)>,
) {
    //if is_key_pressed(KeyCode::Space) {
    if current_level.exit_trigger.contains(player.pos) {
        /*
        if current_level.requires_phone_read && !phone_read {
            *status_msg = Some(("Неготово пока нихуя...", 2.0));
            return;
        }

        if current_level.requires_all_enemies_dead && !enemies.is_empty() {
            *status_msg = Some(("Зачисти уровень!", 2.0));
            return;
        }

        if current_level.map_id == MapId::AutoService_2 {
            *state = GameState::ArrestCutscene;
            return;
        }

        if current_level.map_id == MapId::PoliceStation_2 {
            *state = GameState::DemoCompleted;
            return;
        }
        */

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
    //}
}
