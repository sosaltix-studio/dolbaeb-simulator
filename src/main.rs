use crate::enemy::Enemy;
use crate::objects::{Bullet, DroppedWeapon, Weapon};
use crate::player::Player;
use crate::tilemap::{MapId, WorldManager};
use crate::world::{GameState, Level};
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
    // Загрузка текстур
    let assets = assets::Assets::load();
    let mut world_manager = tilemap::WorldManager::init(SCALE);

    let mut audio = audio::AudioManager::load().await;

    // Камера
    let mut camera = Camera2D::default();
    camera.zoom = vec2(2.0 / screen_width(), 2.0 / screen_height());

    // Инициализация игрока
    let player_start_pos = vec2(0.0, 0.0);
    let mut player = Player::new(player_start_pos, 300.0);

    // Спавн телефона
    let mut phone = objects::Phone {
        charge: 100.0,
        is_get: false,
    };

    let mut current_level = Level::get(MapId::House);

    let mut enemies: Vec<Enemy> = current_level.enemies.clone();
    let mut bullets: Vec<Bullet> = Vec::with_capacity(128);

    let mut dropped_weapons: Vec<DroppedWeapon> = Vec::new();

    let mut last_shot_pos: Option<Vec2> = None;

    let mut phone_read = false;

    // Состояния меню и настроек
    let mut state = GameState::MainMenu;
    let mut previous_state = GameState::MainMenu;
    let mut is_paused = false;

    // Индексы выбранных пунктов для каждого меню
    let mut menu_idx = 0;
    let mut pause_idx = 0;
    let mut settings_idx = 0;

    // Текущие настройки
    let mut fullscreen = false;
    let mut font_idx = 0;

    let restart = |current_level: &mut Level,
                   player: &mut Player,
                   enemies: &mut Vec<Enemy>,
                   bullets: &mut Vec<Bullet>,
                   dropped_weapons: &mut Vec<DroppedWeapon>,
                   world_manager: &mut WorldManager,
                   last_shot_pos: &mut Option<Vec2>,
                   phone_read: &mut bool| {
        //*current_level = Level::get(MapId::House);
        world_manager.switch_to(current_level.map_id);
        player.restart(current_level.player_start);
        player.weapon = Weapon::Fists;
        *phone_read = false;
        *enemies = current_level.enemies.clone();
        *dropped_weapons = current_level.dropped_weapons.clone();
        bullets.clear();
        *last_shot_pos = None;
    };

    // Главный игровой цикл
    loop {
        // Дельта времени (чтобы игра работала одинаково при разном фпс)
        let delta_time = get_frame_time();

        match state {
            GameState::MainMenu => {
                // Отрисовка меню
                ui::draw_main_menu(&assets, menu_idx, font_idx);

                // Навигация (W - вверх S - вниз)
                if is_key_pressed(KeyCode::W) {
                    menu_idx = if menu_idx == 0 { 2 } else { menu_idx - 1 };
                }
                if is_key_pressed(KeyCode::S) {
                    menu_idx = if menu_idx == 2 { 0 } else { menu_idx + 1 };
                }

                // Подтверждение
                if is_key_pressed(KeyCode::Enter) {
                    match menu_idx {
                        0 => {
                            // Старт игры
                            state = GameState::Game;
                            restart(
                                &mut current_level,
                                &mut player,
                                &mut enemies,
                                &mut bullets,
                                &mut dropped_weapons,
                                &mut world_manager,
                                &mut last_shot_pos,
                                &mut phone_read,
                            );
                        }
                        1 => {
                            // В настройки
                            previous_state = GameState::MainMenu;
                            state = GameState::Settings;
                            settings_idx = 0;
                        }
                        2 => {
                            // Выход
                            break;
                        }
                        _ => {}
                    }
                }
            }

            GameState::Settings => {
                // Отрисовка настроек
                ui::draw_settings_menu(
                    &assets,
                    settings_idx,
                    font_idx,
                    fullscreen,
                    audio.volume_percent(),
                );

                // Навигация в настройках
                if is_key_pressed(KeyCode::W) {
                    settings_idx = if settings_idx == 0 {
                        3
                    } else {
                        settings_idx - 1
                    };
                }
                if is_key_pressed(KeyCode::S) {
                    settings_idx = if settings_idx == 3 {
                        0
                    } else {
                        settings_idx + 1
                    };
                }

                // Изменение громкости
                if settings_idx == 2 {
                    if is_key_pressed(KeyCode::A) || is_key_pressed(KeyCode::Left) {
                        audio.change_volume(-0.1);
                    }
                    if is_key_pressed(KeyCode::D) || is_key_pressed(KeyCode::Right) {
                        audio.change_volume(0.1);
                    }
                }

                // Подтверждение
                if is_key_pressed(KeyCode::Enter) {
                    match settings_idx {
                        0 => {
                            // Переключение на фулл скрин
                            fullscreen = !fullscreen;
                            set_fullscreen(fullscreen);
                        }
                        1 => {
                            // Переключение шрифта
                            font_idx = if font_idx == 3 { 0 } else { font_idx + 1 };
                        }
                        3 => {
                            // Возвращение туда откуда вызванно
                            state = previous_state;
                        }
                        _ => {}
                    }
                }

                // Возвращение туда откуда вызванно (на esc)
                if is_key_pressed(KeyCode::Escape) {
                    state = previous_state;
                }
            }

            _ => {
                // Нажатие ESC вызывает или закрывает паузу
                if is_key_pressed(KeyCode::Escape) {
                    is_paused = !is_paused;
                    pause_idx = 0;
                }

                // Отработка паузы
                if is_paused {
                    // Навигация в паузе
                    if is_key_pressed(KeyCode::W) {
                        pause_idx = if pause_idx == 0 { 3 } else { pause_idx - 1 };
                    }
                    if is_key_pressed(KeyCode::S) {
                        pause_idx = if pause_idx == 3 { 0 } else { pause_idx + 1 };
                    }

                    // Подтверждение
                    if is_key_pressed(KeyCode::Enter) {
                        match pause_idx {
                            0 => {
                                // Продолжить
                                is_paused = false;
                            }
                            1 => {
                                // Рестарт
                                restart(
                                    &mut current_level,
                                    &mut player,
                                    &mut enemies,
                                    &mut bullets,
                                    &mut dropped_weapons,
                                    &mut world_manager,
                                    &mut last_shot_pos,
                                    &mut phone_read,
                                );
                                is_paused = false;
                            }
                            2 => {
                                // В настройки
                                previous_state = state; // Текущая локация
                                state = GameState::Settings; // Переключение на настройки
                                settings_idx = 0;
                            }
                            3 => {
                                // Выход в главное меню
                                state = GameState::MainMenu;
                                is_paused = false;
                                menu_idx = 0;
                            }
                            _ => {}
                        }
                    }
                }

                // Обновление игры
                if !is_paused {
                    player.handle_input(
                        delta_time,
                        &world_manager,
                        &camera,
                        &mut dropped_weapons,
                        &mut bullets,
                        &audio,
                        &mut last_shot_pos,
                        &mut enemies,
                    );

                    player.update_rotation(&camera);
                    player.location_restriction(world_manager.get_active());
                    world_manager.update_flow_field(player.pos);

                    objects::update_bullets(
                        &mut bullets,
                        &mut player,
                        &mut enemies,
                        &world_manager,
                        &mut dropped_weapons,
                        delta_time,
                    );

                    phone.update(delta_time);

                    for enemy in &mut enemies {
                        enemy.update(
                            &mut player,
                            &world_manager,
                            &mut dropped_weapons,
                            &mut bullets,
                            delta_time,
                            &audio,
                        );
                    }

                    if let Some(shot_pos) = last_shot_pos.take() {
                        enemy::alert_enemies(&mut enemies, &shot_pos);
                    }

                    world::handle_location_switch(
                        &mut current_level,
                        &mut world_manager,
                        &mut player,
                        &mut enemies,
                        &mut dropped_weapons,
                        &mut bullets,
                        //phone_read,
                        //&mut state,
                        //&mut status_msg,
                    );
                    camera.target = player.pos;
                }

                // Отрисовка мира
                let target_visible_height = 600.0;
                let zoom_y = 2.0 / target_visible_height;
                let zoom_x = zoom_y * (screen_height() / screen_width());
                camera.zoom = vec2(zoom_x, zoom_y);

                clear_background(Color::new(0.0, 0.0, 0.0, 0.0));

                set_camera(&camera);

                world_manager.draw();
                for bullet in &bullets {
                    bullet.draw();
                }
                for enemy in &enemies {
                    enemy.draw(&assets);
                }
                for item in &dropped_weapons {
                    item.draw(&assets);
                }
                player.draw(&assets);

                // Статичный интерфейс
                ui::draw_ui(&assets, font_idx, &player);
                phone.draw(&assets, font_idx);

                if player.is_dead {
                    ui::draw_dead_menu(&assets, font_idx);
                    if is_key_pressed(KeyCode::R) {
                        restart(
                            &mut current_level,
                            &mut player,
                            &mut enemies,
                            &mut bullets,
                            &mut dropped_weapons,
                            &mut world_manager,
                            &mut last_shot_pos,
                            &mut phone_read,
                        );
                    }
                }

                // Повторыный вызов паузы чтобы она перекрывала интерфейс
                if is_paused {
                    ui::draw_pause_menu(&assets, pause_idx, font_idx);
                }
            }
        }
        next_frame().await
    }
}
