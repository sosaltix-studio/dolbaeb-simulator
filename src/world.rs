use crate::SCALE;
use crate::assets::Assets;
use crate::audio::AudioManager;
use crate::camera::CameraManager;
use crate::enemy::{Enemy, alert_enemies};
use crate::input::{GameInput, VirtualCursor, collect_input};
use crate::level::{Level, handle_location_switch};
use crate::lighting::LightingManager;
use crate::objects::{Bullet, DroppedWeapon, update_bullets};
use crate::player::Player;
use crate::tilemap::{MapId, WorldManager};
use crate::ui::{GameState, UiManager, draw_cursor, draw_dead_menu, draw_ui};
use macroquad::prelude::*;

pub struct Game {
    pub assets: Assets,
    pub world_manager: WorldManager,
    pub audio_manager: AudioManager,
    pub lighting_manager: LightingManager,
    pub player: Player,
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
            lighting_manager: LightingManager::new(),
            player,
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
                if self.input.debug_toggle_effect {
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

            self.lighting_manager.draw(&self.camera_manager.camera);

            self.camera_manager.end_render();

            set_default_camera();

            draw_ui(&self.assets, &self.player);

            self.ui_manager.draw(&self.assets);

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
