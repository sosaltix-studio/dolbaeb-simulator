use crate::assets::Assets;
use crate::enemy::Enemy;
use crate::entity::*;
use crate::player::Player;
use crate::tilemap::WorldManager;
use macroquad::prelude::*;

// Всё оружие
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Weapon {
    Fists,
    Pipe,
    Knife,
    Pistol,
    Rifle,
    Dead,
}

impl Weapon {
    pub fn anim_info(&self) -> (usize, usize, f32) {
        match self {
            Weapon::Fists => (PUNCH_ROW, PUNCH_FRAMES, 14.0),
            Weapon::Pipe => (PIPE_ROW, PIPE_FRAMES, 14.0),
            Weapon::Knife => (KNIFE_ROW, KNIFE_FRAMES, 12.0),
            Weapon::Pistol => (PISTOL_ROW, PISTOL_FRAMES, 12.0),
            Weapon::Rifle => (RIFLE_ROW, RIFLE_FRAMES, 14.0),
            Weapon::Dead => (DEAD_ROW, DEAD_FRAMES, 14.0),
        }
    }

    pub fn default_ammo(&self) -> u32 {
        match *self {
            Weapon::Rifle => 30,
            Weapon::Pistol => 12,
            _ => 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BulletOwner {
    Player,
    Enemy,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bullet {
    pub pos: Vec2,
    pub dir: Vec2,
    pub speed: f32,
    pub lifetime: f32,
    pub owner: BulletOwner,
}

impl Bullet {
    pub fn new(pos: Vec2, dir: Vec2, owner: BulletOwner) -> Self {
        Self {
            pos,
            dir: dir.normalize_or_zero(),
            speed: 3000.0,
            lifetime: 2.0,
            owner,
        }
    }

    pub fn collider(&self) -> Rect {
        Rect::new(self.pos.x - 2.0, self.pos.y - 2.0, 3.0, 3.0)
    }

    pub fn draw(&self) {
        draw_circle(self.pos.x, self.pos.y, 2.0, YELLOW);
    }
}

pub fn update_bullets(
    bullets: &mut Vec<Bullet>,
    player: &mut Player,
    enemies: &mut [Enemy],
    world_manager: &WorldManager,
    dropped_weapons: &mut Vec<DroppedWeapon>,
    dt: f32,
) {
    let active_map = world_manager.get_active();
    let map_bounds = active_map.bounds();

    bullets.retain_mut(|bullet| {
        bullet.lifetime -= dt;
        if bullet.lifetime <= 0.0 {
            return false;
        }

        let total_move = bullet.dir * bullet.speed * dt;
        let dist = total_move.length();

        let step_size = 8.0;
        let steps = (dist / step_size).ceil().max(1.0) as usize;
        let step_vec = total_move / (steps as f32);

        for _ in 0..steps {
            bullet.pos += step_vec;

            if !map_bounds.contains(bullet.pos) {
                return false;
            }

            if active_map.check_collision(bullet.collider()) {
                return false;
            }

            match bullet.owner {
                BulletOwner::Player => {
                    for enemy in enemies.iter_mut() {
                        if !enemy.is_dead && bullet.collider().overlaps(&enemy.collider()) {
                            enemy.die(dropped_weapons);
                            return false;
                        }
                    }
                }
                BulletOwner::Enemy => {
                    if !player.is_dead && bullet.collider().overlaps(&player.collider()) {
                        player.die(dropped_weapons);
                        return false;
                    }
                }
            }
        }

        true
    });
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DroppedWeapon {
    pub pos: Vec2,
    pub weapon: Weapon,
    pub ammo: u32,
    pub rotation: f32,
}

impl DroppedWeapon {
    pub fn new(pos: Vec2, weapon: Weapon, ammo: u32, rotation: f32) -> Self {
        Self {
            pos,
            weapon,
            ammo,
            rotation,
        }
    }

    pub fn collider(&self) -> Rect {
        Rect::new(self.pos.x - 45.0, self.pos.y - 45.0, 90.0, 90.0)
    }

    pub fn draw(&self, assets: &Assets) {
        let sprite_idx = match self.weapon {
            Weapon::Pipe => 0.0,
            Weapon::Knife => 1.0,
            Weapon::Pistol => 2.0,
            Weapon::Rifle => 3.0,
            _ => return,
        };

        let frame_size = 48.0;
        let scale = 2.4;
        let scaled_size = frame_size * scale;

        draw_texture_ex(
            &assets.weapons,
            self.pos.x - scaled_size / 2.0,
            self.pos.y - scaled_size / 2.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(scaled_size, scaled_size)),
                source: Some(Rect::new(
                    sprite_idx * frame_size,
                    0.0,
                    frame_size,
                    frame_size,
                )),
                rotation: self.rotation,
                pivot: Some(self.pos),
                ..Default::default()
            },
        );
    }
}
