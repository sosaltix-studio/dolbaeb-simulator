use crate::{
    objects::{DroppedWeapon, Weapon},
    tilemap::TilemapManager,
};
use macroquad::prelude::*;

// Настройка спрайтов и смещения центра
pub const SPRITE_SIZE: f32 = 48.0;
pub const SCALE: f32 = 2.4;
pub const SCALED_SIZE: f32 = SPRITE_SIZE * SCALE;

// Сдвиг текстур (тайлсет кривой что пиздец)
pub const SPRITE_OFFSET_X: f32 = 4.0;
pub const SPRITE_OFFSET_Y: f32 = 0.0;

// Константы строк и кадров для тайлсета
// Кулаки
pub const PUNCH_ROW: usize = 0;
pub const PUNCH_FRAMES: usize = 7;

// Труба
pub const PIPE_ROW: usize = 1;
pub const PIPE_FRAMES: usize = 7;

// Нож
pub const KNIFE_ROW: usize = 2;
pub const KNIFE_FRAMES: usize = 5;

// Пистолет
pub const PISTOL_ROW: usize = 3;
pub const PISTOL_FRAMES: usize = 2;

// Автомат
pub const RIFLE_ROW: usize = 4;
pub const RIFLE_FRAMES: usize = 2;

// Ноги
pub const LEGS_ROW: usize = 5;
pub const LEGS_FRAMES: usize = 7;

pub const DEAD_ROW: usize = 6;
pub const DEAD_FRAMES: usize = 1;

pub const ATACK_RADIUS: f32 = 100.0;
pub const RIFLE_CD: f32 = 0.2;
pub const PISTOL_CD: f32 = 0.6;
pub const MELEE_ATACK_TIME: f32 = 0.2;

// Структура анимаций
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AnimationState {
    pub current_frame: usize,
    frame_timer: f32,
    frame_delay: f32,
    pub num_frames: usize,
    pub row_index: f32,
}

impl AnimationState {
    pub fn new(num_frames: usize, fps: f32, row_index: usize) -> Self {
        Self {
            current_frame: 0,
            frame_timer: 0.0,
            frame_delay: 1.0 / fps,
            num_frames,
            row_index: row_index as f32,
        }
    }

    pub fn update(&mut self, dt: f32) -> bool {
        if self.num_frames <= 1 {
            return false;
        }

        self.frame_timer += dt;
        if self.frame_timer >= self.frame_delay {
            self.frame_timer -= self.frame_delay;
            let next_frame = self.current_frame + 1;

            if next_frame >= self.num_frames {
                self.current_frame = 0;
                return true;
            } else {
                self.current_frame = next_frame;
            }
        }
        false
    }

    pub fn set_state(&mut self, row: usize, num_frames: usize, fps: f32) {
        self.row_index = row as f32;
        self.num_frames = num_frames;
        self.frame_delay = 1.0 / fps;
        self.current_frame = 0;
        self.frame_timer = 0.0;
    }

    pub fn reset(&mut self) {
        self.current_frame = 0;
        self.frame_timer = 0.0;
    }
}

pub fn get_collider(pos: &Vec2) -> Rect {
    let width = 36.0;
    let height = 36.0;
    Rect::new(pos.x - width / 2.0, pos.y, width, height)
}

pub fn move_char(pos: &mut Vec2, delta: Vec2, active_map: &TilemapManager) {
    if delta == Vec2::ZERO {
        return;
    }
    let old_x = pos.x;
    pos.x += delta.x;
    if active_map.check_collision(get_collider(pos)) {
        pos.x = old_x;
    }

    let old_y = pos.y;
    pos.y += delta.y;
    if active_map.check_collision(get_collider(pos)) {
        pos.y = old_y;
    }
}

pub fn restart_char(
    current_pos: &mut Vec2,
    pos: Vec2,
    is_dead: &mut bool,
    current_weapon: &mut Weapon,
    weapon: Weapon,
    torso_anim: &mut AnimationState,
) {
    *current_pos = pos;
    *is_dead = false;
    *current_weapon = weapon;
    let (row, frames, fps) = current_weapon.anim_info();
    torso_anim.set_state(row, frames, fps);
}

pub fn char_die(
    pos: Vec2,
    rotation: f32,
    is_atacking: &mut bool,
    is_dead: &mut bool,
    weapon: &mut Weapon,
    torso_anim: &mut AnimationState,
    dropped_weapons: &mut Vec<DroppedWeapon>,
) {
    if *is_dead {
        return;
    }
    *is_atacking = false;
    *is_dead = true;

    if *weapon != Weapon::Fists && *weapon != Weapon::Dead {
        let ammo = match weapon {
            Weapon::Pistol => 12,
            Weapon::Rifle => 30,
            _ => 0,
        };

        dropped_weapons.push(DroppedWeapon::new(pos, *weapon, ammo, rotation));
    }

    *weapon = Weapon::Dead;

    let (row, frames, fps) = weapon.anim_info();
    torso_anim.set_state(row, frames, fps);
}

pub fn draw_char(
    texture: &Texture2D,
    pos: Vec2,
    rotation: f32,
    legs_rotation: f32,
    torso_anim: &AnimationState,
    legs_anim: &AnimationState,
    is_dead: bool,
) {
    let half_scaled_size = SCALED_SIZE / 2.0;
    let visual_offset = vec2(SPRITE_OFFSET_X * SCALE, SPRITE_OFFSET_Y * SCALE);

    // Ноги
    if !is_dead {
        let legs_src_x = legs_anim.current_frame as f32 * SPRITE_SIZE;
        let legs_src_y = legs_anim.row_index * SPRITE_SIZE;

        draw_texture_ex(
            texture,
            pos.x - half_scaled_size,
            pos.y - half_scaled_size,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(SCALED_SIZE, SCALED_SIZE)),
                source: Some(Rect::new(legs_src_x, legs_src_y, SPRITE_SIZE, SPRITE_SIZE)),
                rotation: legs_rotation,
                pivot: Some(pos),
                ..Default::default()
            },
        );
    }

    // Торс
    let torso_src_x = torso_anim.current_frame as f32 * SPRITE_SIZE;
    let torso_src_y = torso_anim.row_index * SPRITE_SIZE;

    draw_texture_ex(
        texture,
        pos.x - half_scaled_size + visual_offset.x,
        pos.y - half_scaled_size + visual_offset.y,
        WHITE,
        DrawTextureParams {
            dest_size: Some(vec2(SCALED_SIZE, SCALED_SIZE)),
            source: Some(Rect::new(
                torso_src_x,
                torso_src_y,
                SPRITE_SIZE,
                SPRITE_SIZE,
            )),
            rotation,
            pivot: Some(pos),
            ..Default::default()
        },
    );
}
