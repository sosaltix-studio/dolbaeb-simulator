use crate::objects::Bullet;
use macroquad::prelude::*;
use miniquad::{BlendFactor, BlendState, BlendValue, Equation};

const PLAYER_LIGHT_RADIUS: f32 = 240.0;
const PLAYER_LIGHT_INTENSITY: f32 = 0.9;
const PLAYER_LIGHT_COLOR: Color = Color::new(1.0, 0.92, 0.78, 1.0);

const BULLET_LIGHT_RADIUS: f32 = 50.0;
const BULLET_LIGHT_INTENSITY: f32 = 0.5;
const BULLET_LIGHT_COLOR: Color = Color::new(1.0, 0.85, 0.4, 1.0);

pub const MAP_STATIC_LIGHT_COLOR: Color = Color::new(1.0, 0.92, 0.78, 1.0);
pub const MAP_STATIC_LIGHT_RADIUS: f32 = 500.0;
pub const MAP_STATIC_LIGHT_INTENSITY: f32 = 1.0;
pub const MAP_STATIC_LIGHT_FLICKER: f32 = 0.0;

#[derive(Clone, Copy)]
pub struct Light {
    pub pos: Vec2,
    pub radius: f32,
    pub color: Color,
    pub intensity: f32,
    pub flicker: f32,
}

pub const fn lamp(pos: Vec2, radius: f32, color: Color, intensity: f32, flicker: f32) -> Light {
    Light {
        pos,
        radius,
        color,
        intensity,
        flicker,
    }
}

const LIGHT_VERTEX_SHADER: &str = r#"#version 100
attribute vec3 position;
attribute vec2 texcoord;
attribute vec4 color0;

varying mediump vec2 uv;
varying mediump vec4 color;

uniform mat4 Model;
uniform mat4 Projection;

void main() {
    gl_Position = Projection * Model * vec4(position, 1.0);
    uv = texcoord;
    color = color0 / 255.0;
}
"#;

const BLOB_FRAGMENT_SHADER: &str = r#"#version 100
precision mediump float;

varying mediump vec2 uv;
varying mediump vec4 color;
uniform vec4 u_light;

void main() {
    float dist = length(uv - vec2(0.5)) * 2.0;
    float t = max(1.0 - dist, 0.0);
    float falloff = t * t * (3.0 - 2.0 * t);
    gl_FragColor = vec4(u_light.rgb * u_light.a * falloff, 1.0) * color;
}
"#;

const COMPOSITE_FRAGMENT_SHADER: &str = r#"#version 100
precision mediump float;

varying mediump vec2 uv;
varying mediump vec4 color;

uniform sampler2D Texture;

void main() {
    gl_FragColor = texture2D(Texture, uv) * color;
}
"#;

pub struct LightingManager {
    pub ambient_color: Color,
    pub enabled: bool,
    light_map: RenderTarget,
    blob_material: Material,
    composite_material: Material,
}

impl LightingManager {
    pub fn new() -> Self {
        let blob_material = load_material(
            ShaderSource::Glsl {
                vertex: LIGHT_VERTEX_SHADER,
                fragment: BLOB_FRAGMENT_SHADER,
            },
            MaterialParams {
                uniforms: vec![UniformDesc::new("u_light", UniformType::Float4)],
                pipeline_params: PipelineParams {
                    color_blend: Some(BlendState::new(
                        Equation::Add,
                        BlendFactor::One,
                        BlendFactor::One,
                    )),
                    ..Default::default()
                },
                ..Default::default()
            },
        )
        .unwrap();

        let composite_material = load_material(
            ShaderSource::Glsl {
                vertex: LIGHT_VERTEX_SHADER,
                fragment: COMPOSITE_FRAGMENT_SHADER,
            },
            MaterialParams {
                pipeline_params: PipelineParams {
                    color_blend: Some(BlendState::new(
                        Equation::Add,
                        BlendFactor::Value(BlendValue::DestinationColor),
                        BlendFactor::Zero,
                    )),
                    ..Default::default()
                },
                ..Default::default()
            },
        )
        .unwrap();

        let width = screen_width() as u32;
        let height = screen_height() as u32;

        Self {
            ambient_color: Color::new(0.16, 0.17, 0.24, 1.0),
            enabled: true,
            light_map: render_target(width, height),
            blob_material,
            composite_material,
        }
    }

    pub fn draw(
        &mut self,
        camera: &Camera2D,
        static_lights: &[Light],
        player_pos: Vec2,
        bullets: &[Bullet],
    ) {
        if !self.enabled {
            return;
        }

        self.sync_light_map_size();

        let view = ViewTransform::from_camera(camera);
        let time = get_time() as f32;

        set_camera(&self.light_map_camera());
        clear_background(self.ambient_color);

        for light in static_lights {
            self.draw_light(&view, *light, time);
        }
        self.draw_light(&view, player_light(player_pos), time);
        for bullet in bullets {
            self.draw_light(&view, bullet_light(bullet), time);
        }

        set_default_camera();
        gl_use_material(&self.composite_material);
        draw_texture_ex(
            &self.light_map.texture,
            0.0,
            0.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(screen_width(), screen_height())),
                flip_y: true,
                ..Default::default()
            },
        );
        gl_use_default_material();
    }

    fn draw_light(&self, view: &ViewTransform, light: Light, time: f32) {
        let center = view.to_screen(light.pos);
        let radius = view.radius_to_screen(light.radius);
        if !view.is_visible(center, radius) {
            return;
        }

        self.blob_material.set_uniform(
            "u_light",
            [
                light.color.r,
                light.color.g,
                light.color.b,
                light.intensity * flicker_multiplier(light, time),
            ],
        );
        gl_use_material(&self.blob_material);
        draw_rectangle(
            center.x - radius,
            center.y - radius,
            radius * 2.0,
            radius * 2.0,
            WHITE,
        );
        gl_use_default_material();
    }

    fn sync_light_map_size(&mut self) {
        let width = screen_width() as u32;
        let height = screen_height() as u32;

        if self.light_map.texture.width() as u32 != width
            || self.light_map.texture.height() as u32 != height
        {
            self.light_map = render_target(width, height);
        }
    }

    fn light_map_camera(&self) -> Camera2D {
        let mut camera =
            Camera2D::from_display_rect(Rect::new(0.0, 0.0, screen_width(), screen_height()));
        camera.render_target = Some(self.light_map.clone());
        camera
    }
}

fn player_light(pos: Vec2) -> Light {
    lamp(
        pos,
        PLAYER_LIGHT_RADIUS,
        PLAYER_LIGHT_COLOR,
        PLAYER_LIGHT_INTENSITY,
        0.0,
    )
}

fn bullet_light(bullet: &Bullet) -> Light {
    lamp(
        bullet.pos,
        BULLET_LIGHT_RADIUS,
        BULLET_LIGHT_COLOR,
        BULLET_LIGHT_INTENSITY,
        0.0,
    )
}

fn flicker_multiplier(light: Light, time: f32) -> f32 {
    if light.flicker <= 0.0 {
        return 1.0;
    }

    let phase = light.pos.x * 0.017 + light.pos.y * 0.011;
    let wave = (time * 7.0 + phase).sin() * 0.6 + (time * 19.0 + phase).sin() * 0.4;
    1.0 - light.flicker * (0.5 + 0.5 * wave)
}

struct ViewTransform {
    target: Vec2,
    zoom: Vec2,
    screen_size: Vec2,
}

impl ViewTransform {
    fn from_camera(camera: &Camera2D) -> Self {
        Self {
            target: camera.target,
            zoom: camera.zoom,
            screen_size: vec2(screen_width(), screen_height()),
        }
    }

    fn to_screen(&self, world_pos: Vec2) -> Vec2 {
        vec2(
            self.screen_size.x * (0.5 + 0.5 * self.zoom.x * (world_pos.x - self.target.x)),
            self.screen_size.y * (0.5 + 0.5 * self.zoom.y * (world_pos.y - self.target.y)),
        )
    }

    fn radius_to_screen(&self, world_radius: f32) -> f32 {
        world_radius * self.zoom.x * self.screen_size.x * 0.5
    }

    fn is_visible(&self, center: Vec2, radius: f32) -> bool {
        center.x + radius > 0.0
            && center.x - radius < self.screen_size.x
            && center.y + radius > 0.0
            && center.y - radius < self.screen_size.y
    }
}
