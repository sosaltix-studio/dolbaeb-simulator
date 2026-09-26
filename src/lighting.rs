use macroquad::prelude::*;

pub struct LightingManager {
    pub ambient_color: Color,
    pub enabled: bool,
}

impl LightingManager {
    pub fn new() -> Self {
        Self {
            ambient_color: Color::new(0.0, 0.0, 0.0, 0.7),
            enabled: true,
        }
    }

    pub fn draw(&self, camera: &Camera2D) {
        if !self.enabled {
            return;
        }

        let target = camera.target;
        let screen_size = vec2(
            screen_width() / camera.zoom.x * 2.0,
            screen_height() / camera.zoom.y * 2.0,
        );
        let top_left = target - screen_size * 0.5;

        draw_rectangle(
            top_left.x,
            top_left.y,
            screen_size.x,
            screen_size.y,
            self.ambient_color,
        );
    }
}
