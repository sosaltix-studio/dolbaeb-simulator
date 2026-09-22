use crate::effect::TripEffect;
use macroquad::prelude::*;

pub const MAX_LOOK_OFFSET: f32 = 300.0;
pub const CAMERA_SMOOTH_SPEED: f32 = 4.0;

pub struct CameraManager {
    pub camera: Camera2D,

    target_pos: Vec2,

    shake_timer: f32,
    shake_duration: f32,
    shake_intensity: f32,

    pub story_effect_active: bool,
    pub story_effect_timer: f32,
    pub trip_effect: TripEffect,
}

impl CameraManager {
    pub fn new() -> Self {
        let camera = Camera2D::default();

        Self {
            camera,
            target_pos: Vec2::ZERO,
            shake_timer: 0.0,
            shake_duration: 0.0,
            shake_intensity: 0.0,
            story_effect_active: false,
            story_effect_timer: 0.0,
            trip_effect: TripEffect::new(),
        }
    }

    pub fn set_story_effect(&mut self, active: bool) {
        self.story_effect_active = active;
        self.trip_effect.toggle(active);
        if !active {
            self.story_effect_timer = 0.0;
        }
    }

    pub fn toggle_story_effect(&mut self) {
        self.set_story_effect(!self.story_effect_active);
    }

    pub fn update(&mut self, player_pos: Vec2, aim_world_pos: Vec2, look_around: bool, dt: f32) {
        let desired_target = if look_around {
            let offset = aim_world_pos - player_pos;
            let clamped_offset = vec2(
                offset.x.clamp(-MAX_LOOK_OFFSET, MAX_LOOK_OFFSET),
                offset.y.clamp(-MAX_LOOK_OFFSET, MAX_LOOK_OFFSET),
            );
            player_pos + clamped_offset
        } else {
            player_pos
        };

        let factor = 1.0 - (-CAMERA_SMOOTH_SPEED * dt).exp();
        self.target_pos = self.target_pos.lerp(desired_target, factor);

        let mut shake_offset = Vec2::ZERO;
        if self.shake_timer > 0.0 {
            self.shake_timer -= dt;
            let progress = (self.shake_timer / self.shake_duration).max(0.0);
            let current_intensity = self.shake_intensity * progress;

            shake_offset = vec2(
                rand::gen_range(-1.0, 1.0) * current_intensity,
                rand::gen_range(-1.0, 1.0) * current_intensity,
            );
        }

        if self.story_effect_active || self.trip_effect.intensity > 0.0 {
            self.story_effect_timer += dt;
            let wave_x = (self.story_effect_timer * 2.0).sin() * 4.0 * self.trip_effect.intensity;
            let wave_y = (self.story_effect_timer * 2.0).cos() * 4.0 * self.trip_effect.intensity;
            shake_offset += vec2(wave_x, wave_y);
        }

        self.trip_effect.update(dt);

        self.camera.zoom = vec2(2.4 / screen_width(), 2.4 / screen_height());
        self.camera.target = self.target_pos + shake_offset;
    }

    pub fn begin_render(&mut self) {
        if self.trip_effect.intensity > 0.0 {
            self.camera.render_target = Some(self.trip_effect.scene_target.clone());
        } else {
            self.camera.render_target = None;
        }

        set_camera(&self.camera);
        clear_background(BLACK);
    }

    pub fn end_render(&mut self) {
        set_default_camera();

        if self.trip_effect.intensity > 0.0 {
            self.trip_effect.render_to_screen();
        }

        self.camera.render_target = None;
    }
}
