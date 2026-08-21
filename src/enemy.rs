use crate::assets::Assets;
use crate::audio::AudioManager;
use crate::enemy::EnemyState::RangedChase;
use crate::entity::*;
use crate::objects::{Bullet, BulletOwner, DroppedWeapon, Weapon};
use crate::player::Player;
use crate::tilemap::WorldManager;
use macroquad::prelude::*;

const VISION_RADIUS: f32 = 150.0;
const RETREAT_DISTANCE: f32 = 180.0;

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
    pub state: EnemyState,
    pub patrol_dir: Vec2,
    pub shoot_cooldown: f32,
    pub last_known_pos: Option<Vec2>,
    pub attack_timer: f32,
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
            speed: 180.0,
            rotation: initial_rot,
            legs_rotation: initial_rot,
            torso_anim: AnimationState::new(1, 1.0, PUNCH_ROW),
            legs_anim: AnimationState::new(LEGS_FRAMES, 14.0, LEGS_ROW),
            weapon,
            is_dead: false,
            is_moving: false,
            is_attacking: false,
            state: EnemyState::Patrol,
            patrol_dir: dir,
            shoot_cooldown: 0.0,
            last_known_pos: None,
            attack_timer: 0.0,
        }
    }

    pub fn collider(&self) -> Rect {
        get_collider(&self.pos)
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
        let map_bounds = active_map.bounds();

        let dist_to_player = self.pos.distance(player.pos);
        let is_player_melee = player.weapon != Weapon::Pistol
            && player.weapon != Weapon::Rifle
            && player.weapon != Weapon::Dead;

        if player.is_attacking
            && is_player_melee
            && dist_to_player < ATACK_RADIUS
            && is_player_melee
        {
            match player.weapon {
                Weapon::Knife => audio.play(&audio.sound_knife),
                Weapon::Pipe | Weapon::Fists => audio.play(&audio.sound_pipe),
                _ => {}
            }
            self.die(dropped_weapons);
            return;
        }

        let sees_player = (dist_to_player < VISION_RADIUS
            || (active_map.has_line_of_sight(self.pos, player.pos)))
            && !player.is_dead;

        if sees_player {
            self.last_known_pos = Some(player.pos);
            self.state = match self.weapon {
                Weapon::Pistol | Weapon::Rifle => EnemyState::RangedChase,
                _ => EnemyState::MeleeChase,
            };
        } else if let Some(target) = self.last_known_pos {
            if self.pos.distance(target) < 25.0 {
                self.last_known_pos = None;
                self.state = EnemyState::Patrol;
            } else {
                self.state = match self.weapon {
                    Weapon::Pistol | Weapon::Rifle => EnemyState::RangedChase,
                    _ => EnemyState::MeleeChase,
                };
            }
        } else {
            self.state = EnemyState::Patrol;
        }

        match self.state {
            EnemyState::Patrol => {
                if self.patrol_dir != Vec2::ZERO {
                    let move_vec = self.patrol_dir * self.speed * dt;
                    let old_pos = self.pos;

                    self.pos += move_vec;
                    self.is_moving = true;

                    let col = get_collider(&self.pos);

                    let hit_obstacle = active_map.check_collision(col)
                        || col.x < 0.0
                        || col.x + col.w > map_bounds.w
                        || col.y < 0.0
                        || col.y + col.h > map_bounds.h;

                    if hit_obstacle {
                        self.pos = old_pos;
                        self.patrol_dir = -self.patrol_dir;
                    }

                    self.rotation = self.patrol_dir.y.atan2(self.patrol_dir.x);
                    self.legs_rotation = self.rotation;
                }
            }

            EnemyState::MeleeChase => {
                self.is_moving = true;

                let dir = active_map.get_flow_direction(self.pos);

                if dir != Vec2::ZERO {
                    let move_vec = dir * self.speed * dt;
                    move_char(&mut self.pos, move_vec, active_map);
                    self.rotation = dir.y.atan2(dir.x);
                    self.legs_rotation = self.rotation;
                } else {
                    let to_player = (player.pos - self.pos).normalize_or_zero();
                    self.rotation = to_player.y.atan2(to_player.x);
                }

                if dist_to_player < ATACK_RADIUS {
                    if !self.is_attacking {
                        self.is_attacking = true;
                        self.attack_timer = MELEE_ATACK_TIME;
                        audio.play(&audio.sound_swosh);
                        let (row, frames, fps) = self.weapon.anim_info();
                        self.torso_anim.set_state(row, frames, fps);
                    }
                }

                if self.is_attacking {
                    if self.attack_timer > 0.0 {
                        self.attack_timer -= dt;
                        if self.attack_timer <= 0.0 {
                            if dist_to_player < ATACK_RADIUS && !player.is_dead {
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
            }

            EnemyState::RangedChase => {
                if sees_player {
                    let dir_to_player = (player.pos - self.pos).normalize_or_zero();
                    if dir_to_player != Vec2::ZERO {
                        self.rotation = dir_to_player.y.atan2(dir_to_player.x);
                        self.legs_rotation = self.rotation;
                    }

                    if dist_to_player <= RETREAT_DISTANCE {
                        self.is_moving = false;
                    } else {
                        self.is_moving = true;
                        let move_vec = dir_to_player * self.speed * dt;
                        move_char(&mut self.pos, move_vec, active_map);
                    }

                    if self.shoot_cooldown <= 0.0
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
                        self.rotation = move_dir.y.atan2(move_dir.x);
                        self.legs_rotation = self.rotation;
                    } else {
                        self.is_moving = false;
                    }
                }
            }
        }

        if self.shoot_cooldown > 0.0 {
            self.shoot_cooldown -= dt;
        }

        if !self.is_attacking {
            let (row, frames, fps) = self.weapon.anim_info();
            self.torso_anim.set_state(row, frames, fps);
        }

        // Обновление анимации торса
        if self.is_attacking {
            let animation_finished = self.torso_anim.update(dt);

            if animation_finished {
                let keep_shooting = self.weapon == Weapon::Rifle && self.state == RangedChase;

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

    pub fn restart(&mut self, pos: Vec2, weapon: Weapon) {
        restart_char(
            &mut self.pos,
            pos,
            &mut self.is_dead,
            &mut self.weapon,
            weapon,
            &mut self.torso_anim,
        );
        self.last_known_pos = None;
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
                    enemy.state = match enemy.weapon {
                        Weapon::Pistol | Weapon::Rifle => EnemyState::RangedChase,
                        _ => EnemyState::MeleeChase,
                    };
                }
            }
        }
    }
}
