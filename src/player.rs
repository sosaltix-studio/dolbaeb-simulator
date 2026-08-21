use crate::assets::Assets;
use crate::audio::AudioManager;
use crate::entity::*;
use crate::objects::{Bullet, BulletOwner, DroppedWeapon, Weapon};
use crate::tilemap::{TilemapManager, WorldManager};
use macroquad::prelude::*;

// Структура игрока
pub struct Player {
    pub pos: Vec2,
    pub speed: f32,
    pub rotation: f32,
    pub legs_rotation: f32,
    pub torso_anim: AnimationState,
    pub legs_anim: AnimationState,
    pub is_moving: bool,
    pub weapon: Weapon,
    pub ammo: u32,
    pub is_attacking: bool,
    pub is_dead: bool,
    pub shoot_cooldown: f32,
    pub atack_timer: f32,
}

impl Player {
    pub fn new(pos: Vec2, speed: f32) -> Self {
        Self {
            pos,
            speed,
            rotation: 0.0,
            legs_rotation: 0.0,
            torso_anim: AnimationState::new(1, 1.0, PUNCH_ROW),
            legs_anim: AnimationState::new(LEGS_FRAMES, 14.0, LEGS_ROW),
            is_moving: false,
            is_attacking: false,
            is_dead: false,
            weapon: Weapon::Fists,
            ammo: 0,
            shoot_cooldown: 0.0,
            atack_timer: 0.0,
        }
    }

    pub fn collider(&self) -> Rect {
        get_collider(&self.pos)
    }

    pub fn handle_input(
        &mut self,
        delta_time: f32,
        world_manager: &WorldManager,
        camera: &Camera2D,
        dropped_weapons: &mut Vec<DroppedWeapon>,
        bullets: &mut Vec<Bullet>,
        audio: &AudioManager,
        last_shot_pos: &mut Option<Vec2>,
    ) {
        if self.is_dead {
            return;
        }

        if is_mouse_button_pressed(MouseButton::Right) {
            if let Some(idx) = dropped_weapons
                .iter()
                .position(|w| w.collider().overlaps(&self.collider()))
            {
                let picked = dropped_weapons.remove(idx);

                if self.weapon != Weapon::Fists {
                    dropped_weapons.push(DroppedWeapon::new(
                        self.pos,
                        self.weapon,
                        self.ammo,
                        self.rotation,
                    ));
                }

                self.is_attacking = false;
                self.weapon = picked.weapon;
                self.ammo = picked.ammo;

                let row = self.weapon.anim_info().0;

                self.torso_anim.set_state(row, 1, 1.0);
            } else if self.weapon != Weapon::Fists {
                dropped_weapons.push(DroppedWeapon::new(
                    self.pos,
                    self.weapon,
                    self.ammo,
                    self.rotation,
                ));

                self.is_attacking = false;
                self.weapon = Weapon::Fists;
                self.ammo = 0;
                self.torso_anim.set_state(PUNCH_ROW, 1, 1.0);
            }
        }

        if !self.is_attacking {
            let old_weapon = self.weapon;

            if is_key_pressed(KeyCode::Key1) {
                self.weapon = Weapon::Fists;
                self.ammo = 0;
            }
            if is_key_pressed(KeyCode::Key2) {
                self.weapon = Weapon::Pipe;
                self.ammo = 0;
            }
            if is_key_pressed(KeyCode::Key3) {
                self.weapon = Weapon::Knife;
                self.ammo = 0;
            }
            if is_key_pressed(KeyCode::Key4) {
                self.weapon = Weapon::Pistol;
                self.ammo = 12;
            }
            if is_key_pressed(KeyCode::Key5) {
                self.weapon = Weapon::Rifle;
                self.ammo = 30;
            }

            if self.weapon != old_weapon {
                let row = self.weapon.anim_info().0;
                self.torso_anim.set_state(row, 1, 1.0);
            }
        }

        let mut move_vec = vec2(0.0, 0.0);
        if is_key_down(KeyCode::W) {
            move_vec.y -= 1.0;
        }
        if is_key_down(KeyCode::S) {
            move_vec.y += 1.0;
        }
        if is_key_down(KeyCode::A) {
            move_vec.x -= 1.0;
        }
        if is_key_down(KeyCode::D) {
            move_vec.x += 1.0;
        }

        self.is_moving = move_vec != vec2(0.0, 0.0);

        if self.is_moving {
            move_vec = move_vec.normalize();
            let delta = move_vec * self.speed * delta_time;
            move_char(&mut self.pos, delta, world_manager.get_active());

            self.legs_rotation = move_vec.y.atan2(move_vec.x);
            self.legs_anim.update(delta_time);
        } else {
            self.legs_anim.reset();
        }

        if self.shoot_cooldown > 0.0 {
            self.shoot_cooldown -= delta_time;
        }

        let is_firearm = self.weapon == Weapon::Pistol || self.weapon == Weapon::Rifle;
        let can_shoot = !is_firearm || self.ammo > 0;

        let attack_triggered = match self.weapon {
            Weapon::Rifle => is_mouse_button_down(MouseButton::Left),
            _ => is_mouse_button_pressed(MouseButton::Left),
        };

        if attack_triggered && !self.is_attacking && can_shoot {
            self.is_attacking = true;

            if !is_firearm {
                self.atack_timer = MELEE_ATACK_TIME;
                audio.play(&audio.sound_swosh);
            }

            let (row, frames, fps) = self.weapon.anim_info();
            self.torso_anim.set_state(row, frames, fps);
        }

        if self.is_attacking {
            let animation_finished = self.torso_anim.update(delta_time);

            if animation_finished {
                let keep_shooting =
                    is_firearm && self.ammo > 0 && is_mouse_button_down(MouseButton::Left);

                if is_firearm && self.ammo > 0 {
                    let mouse_world =
                        camera.screen_to_world(vec2(mouse_position().0, mouse_position().1));
                    let dir = mouse_world - self.pos;

                    bullets.push(Bullet::new(self.pos, dir, BulletOwner::Player));
                    *last_shot_pos = Some(self.pos);

                    self.shoot_cooldown = match self.weapon {
                        Weapon::Rifle => RIFLE_CD,
                        Weapon::Pistol => PISTOL_CD,
                        _ => 0.0,
                    };

                    self.ammo = self.ammo.saturating_sub(1);

                    match self.weapon {
                        Weapon::Rifle => audio.play(&audio.sound_ak47),
                        Weapon::Pistol => audio.play(&audio.sound_pistol),
                        _ => {}
                    }
                }

                if keep_shooting {
                    self.torso_anim.reset();
                } else {
                    self.is_attacking = false;
                    let row = self.weapon.anim_info().0;
                    self.torso_anim.set_state(row, 1, 1.0);
                }
            }
        }
    }

    pub fn update_rotation(&mut self, camera: &Camera2D) {
        if !self.is_dead {
            let mouse_screen = mouse_position();
            let mouse_world = camera.screen_to_world(vec2(mouse_screen.0, mouse_screen.1));
            let direction = mouse_world - self.pos;

            self.rotation = direction.y.atan2(direction.x);
        }
    }

    // Ограничение локаций
    pub fn location_restriction(&mut self, active_map: &TilemapManager) {
        let bounds = active_map.bounds();
        self.pos.x = self.pos.x.clamp(0.0, bounds.w);
        self.pos.y = self.pos.y.clamp(0.0, bounds.h);
    }

    pub fn restart(&mut self, pos: Vec2) {
        let weapon = Weapon::Fists;
        restart_char(
            &mut self.pos,
            pos,
            &mut self.is_dead,
            &mut self.weapon,
            weapon,
            &mut self.torso_anim,
        );
    }

    pub fn die(&mut self, dropped_weapons: &mut Vec<DroppedWeapon>) {
        char_die(
            self.pos,
            self.rotation,
            &mut self.is_attacking,
            &mut self.is_dead,
            &mut self.weapon,
            &mut self.torso_anim,
            dropped_weapons,
        );
    }

    // Отрисовка игрока
    pub fn draw(&mut self, assets: &Assets) {
        draw_char(
            &assets.player,
            self.pos,
            self.rotation,
            self.legs_rotation,
            &self.torso_anim,
            &self.legs_anim,
            self.is_dead,
        );
    }
}
