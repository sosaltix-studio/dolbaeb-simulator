use crate::world::Game;
use macroquad::prelude::*;

// Игрок
mod player;

// Мир
mod world;

// Интерфейс
mod ui;

// Текстуры
mod assets;

// Объекты
mod objects;

// NPC
mod enemy;

// Тайл карта
mod tilemap;

mod entity;

mod audio;

mod effect;

mod input;

mod camera;

mod logs;

mod level;

mod lighting;

const SCALE: f32 = 2.2;

fn window_conf() -> Conf {
    Conf {
        window_title: "Dolbaeb Simulator".to_string(),
        platform: miniquad::conf::Platform {
            linux_backend: miniquad::conf::LinuxBackend::WaylandWithX11Fallback,
            ..Default::default()
        },
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut game = Game::new().await;

    show_mouse(false);

    set_fullscreen(game.ui_manager.fullscreen);

    // Главный игровой цикл
    loop {
        // Дельта времени (чтобы игра работала одинаково при разном фпс)
        let dt = get_frame_time();

        game.update(dt);

        next_frame().await;
    }
}
