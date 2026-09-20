use macroquad::prelude::*;

const VERTEX_SHADER: &str = r#"#version 100
attribute vec3 position;
attribute vec2 texcoord;
attribute vec4 color0;

varying lowp vec2 uv;
varying lowp vec4 color;

uniform mat4 Model;
uniform mat4 Projection;

void main() {
    gl_Position = Projection * Model * vec4(position, 1.0);
    uv = texcoord;
    color = color0;
}
"#;

const FRAGMENT_SHADER: &str = include_str!("../effects/da.glsl");

pub struct TripEffect {
    pub active: bool,
    pub intensity: f32,
    target_intensity: f32,
    material: Material,
    pub scene_target: RenderTarget,
    pub trail_target: RenderTarget,
    pub time: f32,
}

impl TripEffect {
    pub fn new() -> Self {
        let material = load_material(
            ShaderSource::Glsl {
                vertex: VERTEX_SHADER,
                fragment: FRAGMENT_SHADER,
            },
            MaterialParams {
                uniforms: vec![
                    UniformDesc::new("u_time", UniformType::Float1),
                    UniformDesc::new("u_intensity", UniformType::Float1),
                ],
                ..Default::default()
            },
        )
        .unwrap();

        let w = screen_width() as u32;
        let h = screen_height() as u32;

        let scene_target = render_target(w, h);
        let trail_target = render_target(w, h);

        scene_target.texture.set_filter(FilterMode::Nearest);
        trail_target.texture.set_filter(FilterMode::Nearest);

        Self {
            active: false,
            intensity: 0.0,
            target_intensity: 0.0,
            material,
            scene_target,
            trail_target,
            time: 0.0,
        }
    }

    pub fn toggle(&mut self, enable: bool) {
        self.active = enable;
        self.target_intensity = if enable { 1.0 } else { 0.0 };
    }

    pub fn update(&mut self, dt: f32) {
        self.time += dt;

        let speed = 2.5;
        if self.intensity < self.target_intensity {
            self.intensity = (self.intensity + dt * speed).min(self.target_intensity);
        } else if self.intensity > self.target_intensity {
            self.intensity = (self.intensity - dt * speed).max(self.target_intensity);
        }

        let w = screen_width() as u32;
        let h = screen_height() as u32;
        if self.scene_target.texture.width() as u32 != w
            || self.scene_target.texture.height() as u32 != h
        {
            self.scene_target = render_target(w, h);
            self.trail_target = render_target(w, h);
            self.scene_target.texture.set_filter(FilterMode::Nearest);
            self.trail_target.texture.set_filter(FilterMode::Nearest);
        }
    }

    pub fn render_to_screen(&self) {
        let mut trail_cam =
            Camera2D::from_display_rect(Rect::new(0.0, 0.0, screen_width(), screen_height()));
        trail_cam.render_target = Some(self.trail_target.clone());
        set_camera(&trail_cam);

        let fade_alpha = 0.5 - (0.96 * self.intensity);

        draw_rectangle(
            0.0,
            0.0,
            screen_width(),
            screen_height(),
            Color::new(0.0, 0.0, 0.0, fade_alpha),
        );

        draw_texture_ex(
            &self.scene_target.texture,
            0.0,
            0.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(screen_width(), screen_height())),
                flip_y: true,
                ..Default::default()
            },
        );

        set_default_camera();
        self.material.set_uniform("u_time", self.time);
        self.material.set_uniform("u_intensity", self.intensity);

        gl_use_material(&self.material);

        draw_texture_ex(
            &self.trail_target.texture,
            0.0,
            0.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(screen_width(), screen_height())),
                ..Default::default()
            },
        );

        gl_use_default_material();
    }
}
