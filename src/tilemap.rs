use crate::SCALE;
use macroquad::prelude::*;
use std::cmp::Reverse;
use std::collections::BinaryHeap;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum MapId {
    House = 0,
    Level1_1 = 1,
    Level1_2 = 2,
    Level2_1 = 3,
    Level2_2 = 4,
}

struct MapConfig {
    json_str: &'static str,
    atlas_bytes: &'static [u8],
    texture_bytes: &'static [u8],
    width: usize,
    height: usize,
    tile_w: f32,
    tile_h: f32,
}

pub struct TilemapManager {
    bg_texture: Texture2D,
    map_rect: Rect,
    collision_grid: Box<[bool]>,
    cost_map: Vec<u16>,
    distance_map: Vec<u16>,
    flow_field: Vec<Vec2>,
    grid_width: usize,
    grid_height: usize,
    inv_cell_w: f32,
    inv_cell_h: f32,
    last_target_tile: (usize, usize),
}

impl TilemapManager {
    fn load(config: &MapConfig, scale: f32) -> Self {
        let bg_texture = Texture2D::from_file_with_format(config.texture_bytes, None);
        bg_texture.set_filter(FilterMode::Nearest);

        let texture = Texture2D::from_file_with_format(config.atlas_bytes, None);
        texture.set_filter(FilterMode::Nearest);

        let map_w = config.width as f32 * config.tile_w * scale;
        let map_h = config.height as f32 * config.tile_h * scale;
        let map_rect = Rect::new(0.0, 0.0, map_w, map_h);

        let inv_cell_w = 1.0 / (config.tile_w * scale);
        let inv_cell_h = 1.0 / (config.tile_h * scale);

        let grid_len = config.width * config.height;
        let mut collision_grid = vec![false; grid_len];

        if let Ok(tiled_json) = serde_json::from_str::<serde_json::Value>(config.json_str) {
            if let Some(layers) = tiled_json["layers"].as_array() {
                for layer in layers {
                    if layer["name"] == "Collisions" {
                        if let Some(data) = layer["data"].as_array() {
                            for (index, val) in data.iter().enumerate() {
                                if val.as_u64().unwrap_or(0) != 0 && index < collision_grid.len() {
                                    collision_grid[index] = true;
                                }
                            }
                        }
                    }
                }
            }
        }

        let mut cost_map = vec![1u16; grid_len];
        for y in 0..config.height {
            for x in 0..config.width {
                let idx = y * config.width + x;

                if collision_grid[idx] {
                    cost_map[idx] = u16::MAX;
                    continue;
                }

                let mut is_near_wall = false;
                for dx in -1..=1 {
                    for dy in -1..=1 {
                        if dx == 0 && dy == 0 {
                            continue;
                        }
                        let nx = x as isize + dx;
                        let ny = y as isize + dy;

                        if nx >= 0
                            && nx < config.width as isize
                            && ny >= 0
                            && ny < config.height as isize
                        {
                            let n_idx = (ny as usize) * config.width + (nx as usize);
                            if collision_grid[n_idx] {
                                is_near_wall = true;
                                break;
                            }
                        }
                    }
                    if is_near_wall {
                        break;
                    }
                }

                if is_near_wall {
                    cost_map[idx] = 2;
                }
            }
        }

        Self {
            bg_texture,
            map_rect,
            collision_grid: collision_grid.into_boxed_slice(),
            cost_map,
            distance_map: vec![u16::MAX; grid_len],
            flow_field: vec![Vec2::ZERO; grid_len],
            grid_width: config.width,
            grid_height: config.height,
            inv_cell_w,
            inv_cell_h,
            last_target_tile: (usize::MAX, usize::MAX),
        }
    }

    pub fn update_flow_field(&mut self, target_pos: Vec2) {
        if self.collision_grid.is_empty() {
            return;
        }

        let target_x = ((target_pos.x * self.inv_cell_w) as usize).clamp(0, self.grid_width - 1);
        let target_y = ((target_pos.y * self.inv_cell_h) as usize).clamp(0, self.grid_height - 1);
        if self.last_target_tile == (target_x, target_y) {
            return;
        }
        self.last_target_tile = (target_x, target_y);

        self.distance_map.fill(u16::MAX);
        self.flow_field.fill(Vec2::ZERO);

        let mut heap = BinaryHeap::new();
        let start_idx = target_y * self.grid_width + target_x;

        self.distance_map[start_idx] = 0;
        heap.push(Reverse((0u32, target_x, target_y)));

        let neighbors: [(isize, isize, u32); 8] = [
            (-1, 0, 10),
            (1, 0, 10),
            (0, -1, 10),
            (0, 1, 10),
            (-1, -1, 14),
            (1, -1, 14),
            (-1, 1, 14),
            (1, 1, 14),
        ];

        while let Some(Reverse((dist, x, y))) = heap.pop() {
            let curr_idx = y * self.grid_width + x;
            if dist > self.distance_map[curr_idx] as u32 {
                continue;
            }

            for (dx, dy, move_cost) in neighbors {
                let nx = x as isize + dx;
                let ny = y as isize + dy;

                if nx >= 0
                    && nx < self.grid_width as isize
                    && ny >= 0
                    && ny < self.grid_height as isize
                {
                    let unx = nx as usize;
                    let uny = ny as usize;

                    if dx != 0 && dy != 0 {
                        let wall1 = (y as usize) * self.grid_width + unx;
                        let wall2 = uny * self.grid_width + (x as usize);
                        if self.collision_grid[wall1] || self.collision_grid[wall2] {
                            continue;
                        }
                    }

                    let n_idx = uny * self.grid_width + unx;
                    let tile_cost = self.cost_map[n_idx] as u32;

                    if tile_cost != u16::MAX as u32 {
                        let new_dist = dist.saturating_add(tile_cost * move_cost);
                        if new_dist < self.distance_map[n_idx] as u32 {
                            self.distance_map[n_idx] = new_dist.min(u16::MAX as u32) as u16;
                            heap.push(Reverse((new_dist, unx, uny)));
                        }
                    }
                }
            }
        }

        for y in 0..self.grid_height {
            for x in 0..self.grid_width {
                let idx = y * self.grid_width + x;
                if self.collision_grid[idx] || self.distance_map[idx] == u16::MAX {
                    continue;
                }

                let mut min_dist = self.distance_map[idx];
                let mut best_dir = Vec2::ZERO;

                for dx in -1..=1 {
                    for dy in -1..=1 {
                        if dx == 0 && dy == 0 {
                            continue;
                        }
                        let nx = x as isize + dx;
                        let ny = y as isize + dy;

                        if nx >= 0
                            && nx < self.grid_width as isize
                            && ny >= 0
                            && ny < self.grid_height as isize
                        {
                            if dx != 0 && dy != 0 {
                                let wall_x = (y as isize) * (self.grid_width as isize) + nx;
                                let wall_y = ny * (self.grid_width as isize) + (x as isize);
                                if self.collision_grid[wall_x as usize]
                                    || self.collision_grid[wall_y as usize]
                                {
                                    continue;
                                }
                            }

                            let n_idx = (ny as usize) * self.grid_width + (nx as usize);
                            let dist = self.distance_map[n_idx];

                            if dist < min_dist {
                                min_dist = dist;
                                best_dir = Vec2::new(dx as f32, dy as f32);
                            }
                        }
                    }
                }

                self.flow_field[idx] = best_dir.normalize_or_zero();
            }
        }
    }

    pub fn get_flow_direction(&self, world_pos: Vec2) -> Vec2 {
        if self.collision_grid.is_empty() {
            return Vec2::ZERO;
        }

        let x = ((world_pos.x * self.inv_cell_w) as usize).clamp(0, self.grid_width - 1);
        let y = ((world_pos.y * self.inv_cell_h) as usize).clamp(0, self.grid_height - 1);

        self.flow_field[y * self.grid_width + x]
    }

    pub fn check_collision(&self, entity_rect: Rect) -> bool {
        if self.collision_grid.is_empty() {
            return false;
        }

        let min_x = (entity_rect.x * self.inv_cell_w).floor() as i32;
        let max_x = ((entity_rect.x + entity_rect.w) * self.inv_cell_w).floor() as i32;
        let min_y = (entity_rect.y * self.inv_cell_h).floor() as i32;
        let max_y = ((entity_rect.y + entity_rect.h) * self.inv_cell_h).floor() as i32;

        if max_x < 0
            || min_x >= self.grid_width as i32
            || max_y < 0
            || min_y >= self.grid_height as i32
        {
            return false;
        }

        let min_x = min_x.max(0) as usize;
        let max_x = (max_x as usize).min(self.grid_width - 1);
        let min_y = min_y.max(0) as usize;
        let max_y = (max_y as usize).min(self.grid_height - 1);

        for y in min_y..=max_y {
            let row_offset = y * self.grid_width;
            for x in min_x..=max_x {
                if self.collision_grid[row_offset + x] {
                    return true;
                }
            }
        }
        false
    }

    pub fn bounds(&self) -> Rect {
        self.map_rect
    }

    pub fn draw(&self) {
        draw_texture_ex(
            &self.bg_texture,
            0.0,
            0.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(self.map_rect.w, self.map_rect.h)),
                ..Default::default()
            },
        );
    }

    pub fn has_line_of_sight(&self, start: Vec2, end: Vec2) -> bool {
        if self.collision_grid.is_empty() {
            return true;
        }

        let dir = end - start;
        let dist = dir.length();
        if dist < 0.001 {
            return true;
        }

        let step_size = 8.0;
        let steps = (dist / step_size).ceil() as usize;
        let step_vec = dir / (steps as f32);

        let mut current_pos = start;
        for _ in 0..steps {
            current_pos += step_vec;

            let x = (current_pos.x * self.inv_cell_w) as i32;
            let y = (current_pos.y * self.inv_cell_h) as i32;

            if x < 0 || x >= self.grid_width as i32 || y < 0 || y >= self.grid_height as i32 {
                return false;
            }

            let idx = (y as usize) * self.grid_width + (x as usize);
            if self.collision_grid[idx] {
                return false;
            }
        }

        true
    }
}

pub struct WorldManager {
    maps: [Option<TilemapManager>; 2],
    pub current_map: MapId,
}

const CONFIGS: [MapConfig; 2] = [
    MapConfig {
        json_str: include_str!("../assets/house.json"),
        atlas_bytes: include_bytes!("../assets/tileset.png"),
        texture_bytes: include_bytes!("../assets/house.png"),
        width: 20,
        height: 20,
        tile_w: 16.0,
        tile_h: 16.0,
    },
    MapConfig {
        json_str: include_str!("../assets/level1_1.json"),
        atlas_bytes: include_bytes!("../assets/tileset.png"),
        texture_bytes: include_bytes!("../assets/level1_1.png"),
        width: 60,
        height: 30,
        tile_w: 16.0,
        tile_h: 16.0,
    },
];

impl WorldManager {
    pub fn init(scale: f32) -> Self {
        let house_map = TilemapManager::load(&CONFIGS[0], scale);

        Self {
            maps: [Some(house_map), None],
            current_map: MapId::House,
        }
    }

    pub fn switch_to(&mut self, map_id: MapId) {
        let idx = map_id as usize;
        if self.maps[idx].is_none() {
            self.maps[idx] = Some(TilemapManager::load(&CONFIGS[idx], SCALE));
        }

        self.current_map = map_id;
        self.get_active_mut().last_target_tile = (usize::MAX, usize::MAX);
    }

    pub fn get_active(&self) -> &TilemapManager {
        self.maps[self.current_map as usize].as_ref().unwrap()
    }

    pub fn get_active_mut(&mut self) -> &mut TilemapManager {
        self.maps[self.current_map as usize].as_mut().unwrap()
    }

    pub fn update_flow_field(&mut self, target_pos: Vec2) {
        self.get_active_mut().update_flow_field(target_pos);
    }

    pub fn draw(&self) {
        let active_map = self.get_active();
        active_map.draw();
    }
}
