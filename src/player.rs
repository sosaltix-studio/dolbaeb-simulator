use crate::assets::Assets;
use crate::audio::AudioManager;
use crate::enemy::Enemy;
use crate::entity::*;
use crate::input::GameInput;
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
    pub attack_timer: f32,
    pub is_executing: bool,
    pub execution_orig_pos: Vec2,
    pub executing_enemy_idx: Option<usize>,
}

impl Player {
    pub fn new() -> Self {
        Self {
            pos: Vec2::new(0.0, 0.0),
            speed: PLAYER_SPEED,
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
            attack_timer: 0.0,
            is_executing: false,
            execution_orig_pos: Vec2::ZERO,
            executing_enemy_idx: None,
        }
    }

    pub fn collider(&self) -> Rect {
        get_damage_collider(&self.pos)
    }

    pub fn update(
        &mut self,
        input: &GameInput,
        dt: f32,
        world_manager: &WorldManager,
        dropped_weapons: &mut Vec<DroppedWeapon>,
        bullets: &mut Vec<Bullet>,
        audio: &AudioManager,
        last_shot_pos: &mut Option<Vec2>,
        enemies: &mut [Enemy],
        aim_world_pos: Vec2,
    ) {
        if self.is_dead {
            return;
        }

        self.handle_execute(input, dt, audio, dropped_weapons, enemies);

        if self.is_executing {
            return;
        }

        self.handle_pickup(input, dropped_weapons);

        self.handle_move(input, dt, world_manager);

        self.handle_attack(input, dt, bullets, audio, last_shot_pos);

        self.update_rotation(aim_world_pos);

        self.location_restriction(world_manager.get_active());
    }

    fn handle_execute(
        &mut self,
        input: &GameInput,
        dt: f32,
        audio: &AudioManager,
        dropped_weapons: &mut Vec<DroppedWeapon>,
        enemies: &mut [Enemy],
    ) {
        if self.is_executing {
            let animation_finished = self.torso_anim.update(dt);

            if animation_finished {
                if let Some(idx) = self.executing_enemy_idx {
                    if idx < enemies.len() && !enemies[idx].is_dead {
                        match self.weapon {
                            Weapon::Knife => audio.play(&audio.sound_knife),
                            Weapon::Pipe | Weapon::Fists => audio.play(&audio.sound_pipe),
                            _ => {}
                        }
                        enemies[idx].die(dropped_weapons);
                    }
                }

                self.pos = self.execution_orig_pos;
                self.is_executing = false;
                self.executing_enemy_idx = None;

                let row = self.weapon.anim_info().0;
                self.torso_anim.set_state(row, 1, 1.0);
            }
            return;
        }

        if input.execute_pressed && !self.is_attacking {
            let can_execute = matches!(self.weapon, Weapon::Fists | Weapon::Pipe | Weapon::Knife);
            if can_execute {
                if let Some(idx) = enemies.iter().position(|e| {
                    !e.is_dead && e.is_knock && self.pos.distance(e.pos) <= ATTACK_RADIUS
                }) {
                    self.is_executing = true;
                    self.execution_orig_pos = self.pos;
                    self.executing_enemy_idx = Some(idx);

                    self.pos = enemies[idx].pos;
                    self.rotation = enemies[idx].rotation;

                    let (exec_row, exec_frames, exec_fps) = match self.weapon {
                        Weapon::Pipe => (EXECUTE_PIPE_ROW, EXECUTE_PIPE_FRAMES, 10.0),
                        Weapon::Knife => (EXECUTE_KNIFE_ROW, EXECUTE_KNIFE_FRAMES, 10.0),
                        Weapon::Fists => (EXECUTE_FISTS_ROW, EXECUTE_FISTS_FRAMES, 12.0),
                        _ => (PUNCH_ROW, 1, 1.0),
                    };

                    self.torso_anim.set_state(exec_row, exec_frames, exec_fps);
                    return;
                }
            }
        }
    }

    fn handle_pickup(&mut self, input: &GameInput, dropped_weapons: &mut Vec<DroppedWeapon>) {
        if input.action_pick_drop {
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
    }

    fn handle_move(&mut self, input: &GameInput, dt: f32, world_manager: &WorldManager) {
        self.is_moving = input.move_dir != Vec2::ZERO;

        if self.is_moving {
            let move_vec = input.move_dir.normalize();
            let delta = move_vec * self.speed * dt;
            move_char(&mut self.pos, delta, world_manager.get_active());

            self.legs_rotation = move_vec.y.atan2(move_vec.x);
            self.legs_anim.update(dt);
        } else {
            self.legs_anim.reset();
        }
    }

    fn handle_attack(
        &mut self,
        input: &GameInput,
        dt: f32,
        bullets: &mut Vec<Bullet>,
        audio: &AudioManager,
        last_shot_pos: &mut Option<Vec2>,
    ) {
        if self.shoot_cooldown > 0.0 {
            self.shoot_cooldown -= dt;
        }
        if self.attack_timer > 0.0 {
            self.attack_timer -= dt;
        }

        let is_firearm = self.weapon == Weapon::Pistol || self.weapon == Weapon::Rifle;
        let can_shoot = !is_firearm || self.ammo > 0;
        let cooldown_ready = self.shoot_cooldown <= 0.0 && self.attack_timer <= 0.0;

        let attack_triggered = match self.weapon {
            Weapon::Rifle => input.attack_held,
            _ => input.attack_pressed,
        };

        if attack_triggered && !self.is_attacking && can_shoot && cooldown_ready {
            self.is_attacking = true;

            if is_firearm && self.ammo > 0 {
                let dir = input.aim_world_pos - self.pos;
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
            } else if !is_firearm {
                self.attack_timer = MELEE_ATTACK_TIME;
                audio.play(&audio.sound_swosh);
            }

            let (row, frames, fps) = self.weapon.anim_info();
            self.torso_anim.set_state(row, frames, fps);
        }

        if self.is_attacking {
            let animation_finished = self.torso_anim.update(dt);

            if animation_finished {
                self.is_attacking = false;
                let row = self.weapon.anim_info().0;
                self.torso_anim.set_state(row, 1, 1.0);
            }
        }
    }

    fn update_rotation(&mut self, aim_world_pos: Vec2) {
        if self.is_dead {
            return;
        }

        let direction = aim_world_pos - self.pos;
        if direction != Vec2::ZERO {
            self.rotation = direction.y.atan2(direction.x);
        }
    }

    // Ограничение локаций
    fn location_restriction(&mut self, active_map: &TilemapManager) {
        let bounds = active_map.bounds();
        self.pos.x = self.pos.x.clamp(0.0, bounds.w);
        self.pos.y = self.pos.y.clamp(0.0, bounds.h);
    }

    pub fn restart(&mut self, pos: Vec2) {
        self.pos = pos;
        self.rotation = 0.0;
        self.legs_rotation = 0.0;
        self.legs_anim.reset();
        self.is_moving = false;
        self.weapon = Weapon::Fists;
        self.torso_anim.set_state(PUNCH_ROW, PUNCH_FRAMES, 1.0);
        self.ammo = 0;
        self.is_attacking = false;
        self.is_dead = false;
        self.shoot_cooldown = 0.0;
        self.attack_timer = 0.0;
        self.is_executing = false;
        self.execution_orig_pos = pos;
        self.executing_enemy_idx = None;
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
            false,
        );
    }
}
