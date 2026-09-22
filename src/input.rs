use macroquad::prelude::*;

use crate::enemy::Enemy;

pub const LOCK_RADIUS: f32 = 100.0;

pub struct VirtualCursor {
    pub world_pos: Vec2,
    pub locked_enemy_idx: Option<usize>,
}

impl VirtualCursor {
    pub fn new() -> Self {
        Self {
            world_pos: Vec2::ZERO,
            locked_enemy_idx: None,
        }
    }

    pub fn update(
        &mut self,
        player_is_dead: bool,
        raw_world_pos: Vec2,
        enemies: &[Enemy],
        toggle_lock_on: bool,
    ) {
        if player_is_dead {
            self.locked_enemy_idx = None;
            self.world_pos = raw_world_pos;
            return;
        }

        if let Some(idx) = self.locked_enemy_idx {
            if idx >= enemies.len() || enemies[idx].is_dead {
                self.locked_enemy_idx = None;
            }
        }

        if toggle_lock_on {
            if self.locked_enemy_idx.is_some() {
                self.locked_enemy_idx = None;
            } else {
                let mut closest_idx = None;
                let mut min_dist = LOCK_RADIUS;

                for (idx, enemy) in enemies.iter().enumerate() {
                    if !enemy.is_dead {
                        let dist = raw_world_pos.distance(enemy.pos);
                        if dist < min_dist {
                            min_dist = dist;
                            closest_idx = Some(idx);
                        }
                    }
                }
                self.locked_enemy_idx = closest_idx;
            }
        }

        if let Some(idx) = self.locked_enemy_idx {
            self.world_pos = enemies[idx].pos;
        } else {
            self.world_pos = raw_world_pos;
        }
    }

    pub fn is_locked(&self) -> bool {
        self.locked_enemy_idx.is_some()
    }
}

#[derive(Default, Debug, Clone, Copy)]
pub struct GameInput {
    pub move_dir: Vec2,
    pub raw_aim_world_pos: Vec2,
    pub aim_world_pos: Vec2,
    pub attack_pressed: bool,
    pub attack_held: bool,
    pub action_pick_drop: bool,
    pub execute_pressed: bool,
    pub toggle_lock_on: bool,
    pub menu_up: bool,
    pub menu_down: bool,
    pub menu_del: bool,
    pub menu_ent: bool,
    pub pause_pressed: bool,
    pub look_around: bool,
    pub restart_pressed: bool,
    pub debug_toggle_effect: bool,
}

pub fn collect_input(camera: &Camera2D) -> GameInput {
    collect_pc_input(camera)
}

fn collect_pc_input(camera: &Camera2D) -> GameInput {
    let mut move_dir = Vec2::ZERO;

    if is_key_down(KeyCode::W) {
        move_dir.y -= 1.0;
    }
    if is_key_down(KeyCode::S) {
        move_dir.y += 1.0;
    }
    if is_key_down(KeyCode::A) {
        move_dir.x -= 1.0;
    }
    if is_key_down(KeyCode::D) {
        move_dir.x += 1.0;
    }

    let mouse_screen = vec2(mouse_position().0, mouse_position().1);
    let raw_aim_world_pos = camera.screen_to_world(mouse_screen);

    GameInput {
        move_dir,
        raw_aim_world_pos,
        aim_world_pos: raw_aim_world_pos,
        attack_pressed: is_mouse_button_pressed(MouseButton::Left)
            || is_key_pressed(KeyCode::Enter),
        attack_held: is_mouse_button_down(MouseButton::Left),
        action_pick_drop: is_mouse_button_pressed(MouseButton::Right),
        execute_pressed: is_key_pressed(KeyCode::Space),
        toggle_lock_on: is_mouse_button_pressed(MouseButton::Middle),
        menu_up: is_key_pressed(KeyCode::Up),
        menu_down: is_key_pressed(KeyCode::Down),
        menu_del: is_key_pressed(KeyCode::Backspace),
        menu_ent: is_key_pressed(KeyCode::Enter),
        pause_pressed: is_key_pressed(KeyCode::Escape),
        look_around: is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift),
        restart_pressed: is_key_pressed(KeyCode::R),
        debug_toggle_effect: is_key_pressed(KeyCode::T),
    }
}
