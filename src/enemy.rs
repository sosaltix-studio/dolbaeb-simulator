use crate::assets::Assets;
use crate::audio::AudioManager;
use crate::entity::*;
use crate::objects::{Bullet, BulletOwner, DroppedWeapon, Weapon};
use crate::player::Player;
use crate::tilemap::{TilemapManager, WorldManager};
use macroquad::prelude::*;

const BACK_VISION_RADIUS: f32 = 300.0;
const RETREAT_DISTANCE: f32 = 180.0;
const REACTION_TIME: f32 = 0.2;
const ROTATION_SPEED: f32 = 30.0;

fn rotate_towards(current: f32, target: f32, max_delta: f32) -> f32 {
    let diff = (target - current + std::f32::consts::PI).rem_euclid(2.0 * std::f32::consts::PI)
        - std::f32::consts::PI;
    if diff.abs() <= max_delta {
        target
    } else {
        current + diff.signum() * max_delta
    }
}

// Состояния
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EnemyState {
    Patrol,
    MeleeChase,
    RangedChase,
}

// Структура врага
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Enemy {
    pub pos: Vec2,
    pub speed: f32,
    pub rotation: f32,
    pub legs_rotation: f32,
    pub torso_anim: AnimationState,
    pub legs_anim: AnimationState,
    pub weapon: Weapon,
    pub is_dead: bool,
    pub is_moving: bool,
    pub is_attacking: bool,
    pub is_knock: bool,
    pub state: EnemyState,
    pub patrol_dir: Vec2,
    pub shoot_cooldown: f32,
    pub reaction_timer: f32,
    pub last_known_pos: Option<Vec2>,
    pub attack_timer: f32,
    pub knock_timer: f32,
}

impl Enemy {
    pub fn new(pos: Vec2, weapon: Weapon, patrol_dir: Vec2) -> Self {
        let dir = patrol_dir.normalize_or_zero();
        let initial_rot = if dir != Vec2::ZERO {
            dir.y.atan2(dir.x)
        } else {
            0.0
        };

        Self {
            pos,
            speed: ENEMY_SPEED,
            rotation: initial_rot,
            legs_rotation: initial_rot,
            torso_anim: AnimationState::new(1, 1.0, PUNCH_ROW),
            legs_anim: AnimationState::new(LEGS_FRAMES, 14.0, LEGS_ROW),
            weapon,
            is_dead: false,
            is_moving: false,
            is_attacking: false,
            is_knock: false,
            state: EnemyState::Patrol,
            patrol_dir: dir,
            shoot_cooldown: 0.0,
            reaction_timer: REACTION_TIME,
            last_known_pos: None,
            attack_timer: 0.0,
            knock_timer: 0.0,
        }
    }

    pub fn collider(&self) -> Rect {
        get_damage_collider(&self.pos)
    }

    pub fn update(
        &mut self,
        player: &mut Player,
        world_manager: &WorldManager,
        dropped_weapons: &mut Vec<DroppedWeapon>,
        bullets: &mut Vec<Bullet>,
        dt: f32,
        audio: &AudioManager,
    ) {
        if self.is_dead {
            return;
        }

        let active_map = world_manager.get_active();
        let dist_to_player = self.pos.distance(player.pos);

        if self.handle_knock(dt, player) {
            return;
        }

        if self.handle_dead(player, dropped_weapons, audio) {
            return;
        }

        let sees_player = self.handle_see(dt, player, dist_to_player, active_map);

        self.handle_state(
            dt,
            active_map,
            sees_player,
            player,
            dist_to_player,
            audio,
            dropped_weapons,
            bullets,
        );

        if self.shoot_cooldown > 0.0 {
            self.shoot_cooldown -= dt;
        }

        self.handle_anim(dt);

        self.location_restriction(active_map);
    }

    fn handle_knock(&mut self, dt: f32, player: &Player) -> bool {
        if self.is_knock {
            if !player.is_executing {
                self.knock_timer -= dt;
            }
            self.is_moving = false;
            self.is_attacking = false;

            if self.knock_timer <= 0.0 {
                self.is_knock = false;
                let (row, frames, fps) = self.weapon.anim_info();
                self.torso_anim.set_state(row, frames, fps);
            }
            return true;
        }
        false
    }

    fn handle_dead(
        &mut self,
        player: &Player,
        dropped_weapons: &mut Vec<DroppedWeapon>,
        audio: &AudioManager,
    ) -> bool {
        let player_can_hit = player.is_attacking
            && is_in_attack_sector(player.pos, player.rotation, self.pos, ATTACK_RADIUS);

        if player_can_hit {
            match player.weapon {
                Weapon::Fists => {
                    if !self.is_knock {
                        self.is_knock = true;
                        self.knock_timer = 4.0;
                        self.torso_anim.set_state(STUNNED_ROW, STUNNED_FRAMES, 1.0);

                        if self.weapon != Weapon::Fists && self.weapon != Weapon::Dead {
                            let ammo = match self.weapon {
                                Weapon::Pistol => 12,
                                Weapon::Rifle => 30,
                                _ => 0,
                            };
                            dropped_weapons.push(DroppedWeapon::new(
                                self.pos,
                                self.weapon,
                                ammo,
                                self.rotation,
                            ));
                            self.weapon = Weapon::Fists;
                        }
                        return true;
                    }
                }
                Weapon::Knife => {
                    audio.play(&audio.sound_knife);
                    self.die(dropped_weapons);
                    return true;
                }
                Weapon::Pipe => {
                    audio.play(&audio.sound_pipe);
                    self.die(dropped_weapons);
                    return true;
                }
                _ => {}
            }
        }
        false
    }

    fn handle_see(
        &mut self,
        dt: f32,
        player: &Player,
        dist_to_player: f32,
        active_map: &TilemapManager,
    ) -> bool {
        let forward = vec2(self.rotation.cos(), self.rotation.sin());
        let to_player = player.pos - self.pos;

        let is_in_vision_sector = if dist_to_player > 0.001 {
            let dot = forward.dot(to_player / dist_to_player);
            if dot >= 0.0 {
                true
            } else {
                dist_to_player <= BACK_VISION_RADIUS
            }
        } else {
            true
        };

        let sees_player = !player.is_dead
            && is_in_vision_sector
            && active_map.has_line_of_sight(self.pos, player.pos);

        if sees_player {
            self.reaction_timer = (self.reaction_timer - dt).max(0.0);
            self.last_known_pos = Some(player.pos);
            self.chase_state();
        } else {
            self.reaction_timer = REACTION_TIME;
            if let Some(target) = self.last_known_pos {
                if self.pos.distance(target) < 20.0 {
                    self.last_known_pos = None;
                    self.state = EnemyState::Patrol;
                } else {
                    self.chase_state();
                }
            } else {
                self.state = EnemyState::Patrol;
            }
        }

        sees_player
    }

    fn handle_state(
        &mut self,
        dt: f32,
        active_map: &TilemapManager,
        sees_player: bool,
        player: &mut Player,
        dist_to_player: f32,
        audio: &AudioManager,
        dropped_weapons: &mut Vec<DroppedWeapon>,
        bullets: &mut Vec<Bullet>,
    ) {
        match self.state {
            EnemyState::Patrol => {
                if self.patrol_dir != Vec2::ZERO {
                    let move_vec = self.patrol_dir * self.speed * dt;
                    let old_pos = self.pos;

                    self.pos += move_vec;
                    self.is_moving = true;

                    let bounds = active_map.bounds();
                    let hit_obstacle = active_map.check_collision(self.collider())
                        || self.collider().x < 0.0
                        || self.collider().x + self.collider().w > bounds.w
                        || self.collider().y < 0.0
                        || self.collider().y + self.collider().h > bounds.h;

                    if hit_obstacle {
                        self.pos = old_pos;
                        self.patrol_dir = -self.patrol_dir;
                    }

                    let target_rot = self.patrol_dir.y.atan2(self.patrol_dir.x);
                    self.rotation = rotate_towards(self.rotation, target_rot, ROTATION_SPEED * dt);
                    self.legs_rotation = self.rotation;
                } else {
                    self.is_moving = false;
                }
            }

            EnemyState::MeleeChase => {
                let target = if sees_player {
                    player.pos
                } else {
                    self.last_known_pos.unwrap_or(player.pos)
                };

                let dir = if active_map.has_line_of_sight(self.pos, target) {
                    (target - self.pos).normalize_or_zero()
                } else {
                    active_map.get_flow_direction(self.pos)
                };

                if dir != Vec2::ZERO {
                    self.is_moving = true;
                    let move_vec = dir * self.speed * dt;
                    move_char(&mut self.pos, move_vec, active_map);

                    let target_rot = dir.y.atan2(dir.x);
                    self.rotation = rotate_towards(self.rotation, target_rot, ROTATION_SPEED * dt);
                    self.legs_rotation = self.rotation;
                } else {
                    self.is_moving = false;
                    let to_target = (target - self.pos).normalize_or_zero();
                    if to_target != Vec2::ZERO {
                        let target_rot = to_target.y.atan2(to_target.x);
                        self.rotation =
                            rotate_towards(self.rotation, target_rot, ROTATION_SPEED * dt);
                        self.legs_rotation = self.rotation;
                    }
                }

                if sees_player && dist_to_player < ATTACK_RADIUS && self.shoot_cooldown <= 0.0 {
                    if !self.is_attacking {
                        self.is_attacking = true;
                        self.attack_timer = MELEE_ATTACK_TIME;
                        self.shoot_cooldown = 0.5;
                        audio.play(&audio.sound_swosh);
                        let (row, frames, fps) = self.weapon.anim_info();
                        self.torso_anim.set_state(row, frames, fps);
                    }
                }

                if self.is_attacking && self.attack_timer > 0.0 {
                    self.attack_timer -= dt;
                    if self.attack_timer <= 0.0 {
                        let enemy_can_hit =
                            is_in_attack_sector(self.pos, self.rotation, player.pos, ATTACK_RADIUS);
                        if enemy_can_hit && !player.is_dead {
                            match self.weapon {
                                Weapon::Knife => audio.play(&audio.sound_knife),
                                Weapon::Pipe | Weapon::Fists => audio.play(&audio.sound_pipe),
                                _ => {}
                            }
                            player.die(dropped_weapons);
                        }
                    }
                }
            }

            EnemyState::RangedChase => {
                if sees_player {
                    let dir_to_player = (player.pos - self.pos).normalize_or_zero();
                    let mut is_aimed = false;

                    if dir_to_player != Vec2::ZERO {
                        let target_rot = dir_to_player.y.atan2(dir_to_player.x);
                        self.rotation =
                            rotate_towards(self.rotation, target_rot, ROTATION_SPEED * dt);
                        self.legs_rotation = self.rotation;

                        let angle_diff = (target_rot - self.rotation + std::f32::consts::PI)
                            .rem_euclid(2.0 * std::f32::consts::PI)
                            - std::f32::consts::PI;
                        if angle_diff.abs() < 0.3 {
                            is_aimed = true;
                        }
                    }

                    if dist_to_player <= RETREAT_DISTANCE {
                        self.is_moving = false;
                    } else {
                        self.is_moving = true;
                        let move_dir = active_map.get_flow_direction(self.pos);
                        let move_vec = move_dir * self.speed * dt;
                        move_char(&mut self.pos, move_vec, active_map);
                    }

                    if self.shoot_cooldown <= 0.0
                        && self.reaction_timer <= 0.0
                        && is_aimed
                        && active_map.has_line_of_sight(self.pos, player.pos)
                    {
                        self.is_attacking = true;
                        match self.weapon {
                            Weapon::Rifle => audio.play(&audio.sound_ak47),
                            Weapon::Pistol => audio.play(&audio.sound_pistol),
                            _ => {}
                        }
                        bullets.push(Bullet::new(self.pos, dir_to_player, BulletOwner::Enemy));

                        self.shoot_cooldown = match self.weapon {
                            Weapon::Rifle => RIFLE_CD,
                            _ => PISTOL_CD,
                        };
                    }
                } else if let Some(target_pos) = self.last_known_pos {
                    self.is_attacking = false;
                    let move_dir = if active_map.has_line_of_sight(self.pos, target_pos) {
                        (target_pos - self.pos).normalize_or_zero()
                    } else {
                        active_map.get_flow_direction(self.pos)
                    };

                    if move_dir != Vec2::ZERO {
                        self.is_moving = true;
                        let move_vec = move_dir * self.speed * dt;
                        move_char(&mut self.pos, move_vec, active_map);

                        let target_rot = move_dir.y.atan2(move_dir.x);
                        self.rotation =
                            rotate_towards(self.rotation, target_rot, ROTATION_SPEED * dt);
                        self.legs_rotation = self.rotation;
                    } else {
                        self.is_moving = false;
                    }
                }
            }
        }
    }

    fn handle_anim(&mut self, dt: f32) {
        if !self.is_attacking {
            let (row, frames, fps) = self.weapon.anim_info();
            self.torso_anim.set_state(row, frames, fps);
        }

        if self.is_attacking {
            let animation_finished = self.torso_anim.update(dt);

            if animation_finished {
                let keep_shooting =
                    self.weapon == Weapon::Rifle && self.state == EnemyState::RangedChase;

                if keep_shooting {
                    self.torso_anim.reset();
                } else {
                    self.is_attacking = false;
                    let row = self.weapon.anim_info().0;
                    self.torso_anim.set_state(row, 1, 1.0);
                }
            }
        }

        if self.is_moving {
            self.legs_anim.update(dt);
        } else {
            self.legs_anim.reset();
        }
    }

    fn location_restriction(&mut self, active_map: &TilemapManager) {
        let bounds = active_map.bounds();
        self.pos.x = self.pos.x.clamp(0.0, bounds.w);
        self.pos.y = self.pos.y.clamp(0.0, bounds.h);
    }

    pub fn chase_state(&mut self) {
        self.state = match self.weapon {
            Weapon::Rifle | Weapon::Pistol => EnemyState::RangedChase,
            _ => EnemyState::MeleeChase,
        }
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

    pub fn draw(&self, assets: &Assets) {
        draw_char(
            &assets.enemy,
            self.pos,
            self.rotation,
            self.legs_rotation,
            &self.torso_anim,
            &self.legs_anim,
            self.is_dead,
            self.is_knock,
        );
    }
}

pub fn alert_enemies(enemies: &mut [Enemy], shot_pos: &Vec2) {
    const SOUND_RADIUS: f32 = 1000.0;
    for enemy in enemies.iter_mut() {
        if !enemy.is_dead {
            if enemy.pos.distance(*shot_pos) <= SOUND_RADIUS {
                enemy.last_known_pos = Some(*shot_pos);
                if enemy.state == EnemyState::Patrol {
                    enemy.chase_state();
                }
            }
        }
    }
}
