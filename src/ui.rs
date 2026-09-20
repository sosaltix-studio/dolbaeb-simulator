use crate::assets::Assets;
use crate::audio::AudioManager;
use crate::input::GameInput;
use crate::logs::BOOT_LOGS;
use crate::objects::Weapon;
use crate::player::Player;
use macroquad::prelude::*;
use std::collections::VecDeque;

const MAX_LOG_HISTORY: usize = 100;
const PAUSE_MENU_OPTIONS: [&str; 3] = ["RESUME", "RESTART", "TO TERMINAL"];

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum GameState {
    Terminal,
    Playing,
    Pause,
}

pub struct TuiLine {
    pub text: String,
    pub color: Color,
}

impl TuiLine {
    pub fn new(text: impl Into<String>, color: Color) -> Self {
        Self {
            text: text.into(),
            color,
        }
    }
}

pub struct UiManager {
    pub game_state: GameState,
    pub previous_state: GameState,
    pub fullscreen: bool,
    pub restart_requested: bool,

    pub input_buffer: String,
    pub history: VecDeque<TuiLine>,
    pub cmd_history: Vec<String>,
    pub cmd_history_idx: Option<usize>,
    pub cursor_timer: f32,
    pub pause_selected_idx: usize,

    pub boot_logs_idx: usize,
    pub boot_timer: f32,
    pub boot_delay: f32,
}

impl Default for UiManager {
    fn default() -> Self {
        let mut ui = Self {
            game_state: GameState::Terminal,
            previous_state: GameState::Terminal,
            fullscreen: true,
            restart_requested: false,
            input_buffer: String::with_capacity(64),
            history: VecDeque::with_capacity(MAX_LOG_HISTORY),
            cmd_history: Vec::new(),
            cmd_history_idx: None,
            cursor_timer: 0.0,
            pause_selected_idx: 0,
            boot_logs_idx: 0,
            boot_timer: 0.0,
            boot_delay: 0.07,
        };

        ui.run_boot_sequence();
        ui
    }
}

impl UiManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn change_state(&mut self, new_state: GameState) {
        self.previous_state = self.game_state;
        self.game_state = new_state;
    }

    pub fn log(&mut self, text: impl Into<String>, color: Color) {
        if self.history.len() >= MAX_LOG_HISTORY {
            self.history.pop_front();
        }
        self.history.push_back(TuiLine::new(text, color));
    }

    pub fn run_boot_sequence(&mut self) {
        self.history.clear();
        self.boot_logs_idx = 0;
        self.boot_timer = 0.0;
    }

    pub fn update(&mut self, audio_manager: &mut AudioManager, input: &GameInput) {
        let dt = get_frame_time();
        self.cursor_timer += dt;

        self.update_boot_sequence(dt);

        if self.game_state != GameState::Terminal {
            while get_char_pressed().is_some() {}
        }

        if input.pause_pressed {
            match self.game_state {
                GameState::Playing => self.change_state(GameState::Pause),
                GameState::Pause => self.change_state(GameState::Playing),
                GameState::Terminal => {}
            }
        }

        match self.game_state {
            GameState::Pause => self.update_pause_menu(input),
            GameState::Terminal => self.update_terminal_input(input, audio_manager),
            GameState::Playing => {}
        }
    }

    pub fn draw(&self, assets: &Assets) {
        match self.game_state {
            GameState::Terminal => self.draw_terminal(assets),
            GameState::Pause => draw_pause_menu(assets, self.pause_selected_idx),
            GameState::Playing => {}
        }
    }

    fn update_boot_sequence(&mut self, dt: f32) {
        if self.boot_logs_idx < BOOT_LOGS.len() {
            self.boot_timer += dt;
            if self.boot_timer >= self.boot_delay {
                self.boot_timer = 0.0;
                let line = BOOT_LOGS[self.boot_logs_idx];
                self.log(line, LIGHTGRAY);
                self.boot_logs_idx += 1;
            }
        }
    }

    fn update_pause_menu(&mut self, input: &GameInput) {
        let count = PAUSE_MENU_OPTIONS.len();

        if input.menu_up {
            self.pause_selected_idx = (self.pause_selected_idx + count - 1) % count;
        }

        if input.menu_down {
            self.pause_selected_idx = (self.pause_selected_idx + 1) % count;
        }

        if input.menu_ent {
            match self.pause_selected_idx {
                0 => self.change_state(GameState::Playing),
                1 => {
                    self.restart_requested = true;
                    self.change_state(GameState::Playing);
                }
                2 => self.change_state(GameState::Terminal),
                _ => {}
            }
        }
    }

    fn update_terminal_input(&mut self, input: &GameInput, audio_manager: &mut AudioManager) {
        while let Some(c) = get_char_pressed() {
            if c >= ' ' && c != '`' && c != '~' {
                self.input_buffer.push(c);
            }
        }

        if input.menu_del {
            self.input_buffer.pop();
        }

        self.handle_history_navigation(input);

        if input.menu_ent {
            let cmd = self.input_buffer.trim().to_string();
            if !cmd.is_empty() {
                self.log(format!("user@zinux:~$ {cmd}"), GREEN);
                self.execute_command(&cmd, audio_manager);
                self.cmd_history.push(cmd);
                self.cmd_history_idx = None;
                self.input_buffer.clear();
            }
        }
    }

    fn handle_history_navigation(&mut self, input: &GameInput) {
        let up = is_key_pressed(KeyCode::Up) || input.menu_up;
        let down = is_key_pressed(KeyCode::Down) || input.menu_down;

        if up && !self.cmd_history.is_empty() {
            let idx = self
                .cmd_history_idx
                .map_or(self.cmd_history.len() - 1, |i| i.saturating_sub(1));
            self.cmd_history_idx = Some(idx);
            self.input_buffer = self.cmd_history[idx].clone();
        } else if down {
            if let Some(idx) = self.cmd_history_idx {
                if idx + 1 < self.cmd_history.len() {
                    self.cmd_history_idx = Some(idx + 1);
                    self.input_buffer = self.cmd_history[idx + 1].clone();
                } else {
                    self.cmd_history_idx = None;
                    self.input_buffer.clear();
                }
            }
        }
    }

    fn execute_command(&mut self, raw_cmd: &str, audio_manager: &mut AudioManager) {
        let mut parts = raw_cmd.split_whitespace();
        let Some(cmd) = parts.next() else { return };

        let cmd = cmd.to_lowercase();
        let args: Vec<&str> = parts.collect();

        match cmd.as_str() {
            "start" | "resume" => {
                self.change_state(GameState::Playing);
                self.log("Game started.", GREEN);
            }
            "pause" => {
                self.change_state(GameState::Pause);
                self.log("Game paused.", GREEN);
            }
            "restart" => {
                self.restart_requested = true;
                self.change_state(GameState::Playing);
                self.log("Restarting level...", GREEN);
            }
            "clear" => self.history.clear(),
            "exit" | "quit" => std::process::exit(0),
            "set-volume" => {
                if let Some(Ok(val)) = args.first().map(|s| s.parse::<f32>()) {
                    let clamped = val.clamp(0.0, 100.0);
                    audio_manager.set_volume(clamped / 100.0);
                    self.log(format!("Volume: {}%", clamped as u32), GREEN);
                } else {
                    self.log("Usage: set-volume <0-100>", RED);
                }
            }
            "set-fullscreen" => {
                self.fullscreen = !self.fullscreen;
                set_fullscreen(self.fullscreen);
                self.log(format!("Fullscreen: {}", self.fullscreen), GREEN);
            }
            "help" => {
                self.log(
                    "Commands: start, pause, resume, restart, status, set-volume <0-100>, set-fullscreen, clear, boot, exit",
                    LIGHTGRAY,
                );
            }
            _ => {
                self.log(format!("Unknown command: '{cmd}'"), RED);
            }
        }
    }

    fn draw_terminal(&self, assets: &Assets) {
        set_default_camera();
        clear_background(BLACK);

        let font_size = 24;
        let line_h = 24.0;

        draw_rectangle(
            0.0,
            0.0,
            screen_width(),
            30.0,
            Color::new(0.1, 0.1, 0.1, 1.0),
        );
        draw_ui_text(
            &assets.font,
            "ZINUX v14.88 Rust edition",
            20.0,
            21.0,
            18,
            YELLOW,
        );
        draw_line(0.0, 30.0, screen_width(), 30.0, 1.0, GREEN);

        let max_lines = ((screen_height() - 85.0) / line_h).max(1.0) as usize;
        let visible_lines = self.history.iter().rev().take(max_lines).rev();

        let mut cur_y = 50.0;
        for line in visible_lines {
            draw_ui_text(&assets.font, &line.text, 20.0, cur_y, font_size, line.color);
            cur_y += line_h;
        }

        let cursor = if (self.cursor_timer * 3.0) as usize % 2 == 0 {
            "_"
        } else {
            ""
        };
        let prompt_text = format!("user@zinux:~$ {}{cursor}", self.input_buffer);

        draw_ui_text(
            &assets.font,
            &prompt_text,
            20.0,
            screen_height() - 15.0,
            font_size,
            GREEN,
        );
    }
}

fn draw_ui_text(font: &Font, text: &str, x: f32, y: f32, font_size: u16, color: Color) {
    draw_text_ex(
        text,
        x,
        y,
        TextParams {
            font: Some(font),
            font_size,
            color,
            ..Default::default()
        },
    );
}

pub fn draw_ui(assets: &Assets, player: &Player) {
    if matches!(player.weapon, Weapon::Pistol | Weapon::Rifle) {
        set_default_camera();
        let ammo_text = format!("AMMO: {}", player.ammo);
        draw_ui_text(
            &assets.font,
            &ammo_text,
            screen_width() - 200.0,
            screen_height() - 50.0,
            32,
            YELLOW,
        );
    }
}

pub fn draw_pause_menu(assets: &Assets, selected_idx: usize) {
    set_default_camera();

    draw_rectangle(
        0.0,
        0.0,
        screen_width(),
        screen_height(),
        Color::new(0.0, 0.0, 0.0, 0.6),
    );
    draw_ui_text(
        &assets.font,
        "PAUSE",
        screen_width() / 2.0 - 65.0,
        screen_height() / 2.0 - 120.0,
        45,
        YELLOW,
    );

    for (i, &option) in PAUSE_MENU_OPTIONS.iter().enumerate() {
        let is_selected = i == selected_idx;
        let color = if is_selected { YELLOW } else { WHITE };
        let text = if is_selected {
            format!("> {option}")
        } else {
            option.to_string()
        };

        draw_ui_text(
            &assets.font,
            &text,
            screen_width() / 2.0 - 130.0,
            screen_height() / 2.0 + (i as f32 * 45.0),
            24,
            color,
        );
    }
}

pub fn draw_dead_menu(assets: &Assets) {
    set_default_camera();

    draw_rectangle(
        screen_width() / 2.0 - 200.0,
        screen_height() / 2.0 - 50.0,
        400.0,
        80.0,
        Color::new(0.0, 0.0, 0.0, 0.9),
    );

    draw_ui_text(
        &assets.font,
        "YOU DIED PRESS R TO RESTART",
        screen_width() / 2.0 - 180.0,
        screen_height() / 2.0,
        24,
        RED,
    );
}

pub fn draw_cursor(assets: &Assets, mouse_pos: Vec2) {
    draw_texture_ex(
        &assets.cursor,
        mouse_pos.x - 12.0,
        mouse_pos.y - 12.0,
        WHITE,
        DrawTextureParams::default(),
    );
}
