use std::collections::VecDeque;
use std::fmt::Debug;
use std::{path::Path, io::Read};
use std::str::FromStr;

macro_rules! parse_input {
    ($x:expr, $t:ident) => ($x.trim().parse::<$t>().unwrap())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TileOwner {
    Me,
    Enemy,
    No
}

#[derive(Debug, Clone)]
pub struct Tile {
    pub scrap_amount : i32,
    pub recycler : bool,
    pub units : i32,
    pub delta_units : i32,
    pub owner : TileOwner
}

impl Default for Tile {
    fn default() -> Self {
        Self { 
            scrap_amount: 0, 
            recycler: false, 
            units: 0, 
            owner: TileOwner::No,
            delta_units : 0
        }
    }
}

pub struct MoveAction {
    pub amount : u32,
    pub fromX : usize,
    pub fromY : usize,
    pub toX : usize,
    pub toY : usize
}

pub struct SpawnAction {
    pub amount : u32,
    pub x : usize,
    pub y : usize
}

pub struct BuildAction {
    pub x : usize,
    pub y : usize
}

pub enum Action {
    Move(MoveAction),
    Spawn(SpawnAction),
    Build(BuildAction)
}

#[derive(Default, Clone, PartialEq, Eq, Debug)]
pub struct TVec2<T : Default + Clone + PartialEq + Eq + Debug> {
    pub x : T,
    pub y : T
}

impl<T : Default + Clone + PartialEq + Eq + Debug> TVec2<T> {
    pub fn new(x : T, y : T) -> TVec2<T> {
        TVec2 {
            x, 
            y
        }
    }
}

impl TVec2<usize> {
    fn dist(&self, other : &TVec2<usize>) -> f32 {
        let dx = self.x as f32 - other.x as f32;
        let dy = self.y as f32 - other.y as f32;
        let dist = (dx * dx + dy * dy).sqrt();
        dist
    }
}

pub struct Pathfinder {
    pub data : Vec<i32>,
    pub walls : Vec<bool>,
    pub prev_point : Vec<TVec2<usize>>,
    pub w : usize,
    pub h : usize
}

struct GrowPoint {
    pub p : TVec2<usize>,
    pub prev_p : TVec2<usize>,
    pub val : i32
}

impl Pathfinder {

    #[inline(always)]
    pub fn get_idx(&self, x : usize, y : usize) -> usize {
        y * self.w + x
    }

    pub fn find_path(&mut self, src : &TVec2<usize>, dst : &TVec2<usize>) -> TVec2<usize> {

        self.data.fill(0);

        let mut raise_points = VecDeque::new();
        raise_points.push_back(GrowPoint { p : src.clone(), prev_p : src.clone(), val : 1 });

        let d_w = self.w - 1;
        let d_h = self.h - 1;

        let mut nearest_point = src.clone();
        let mut nearest_dist = 100000000.0_f32;

        while raise_points.len() > 0 && self.data[dst.y * self.w + dst.x] == 0 {
            let gp = raise_points.pop_front().unwrap();
            let idx = gp.p.y * self.w + gp.p.x;
            
            if self.data[idx] == 0 && self.walls[idx] == false {
                self.data[idx] = gp.val;
                self.prev_point[idx] = gp.prev_p.clone();

                let dist = gp.p.dist(dst);
                if dist < nearest_dist {
                    nearest_dist = dist;
                    nearest_point = gp.p.clone();
                }

                if gp.p.x > 0 {
                    raise_points.push_back(
                        GrowPoint {
                            p : TVec2 {x : gp.p.x - 1, y : gp.p.y},
                            prev_p : gp.p.clone(),
                            val : gp.val + 1
                        }
                    )
                }

                if gp.p.y > 0 {
                    raise_points.push_back(
                        GrowPoint {
                            p : TVec2 {x : gp.p.x, y : gp.p.y - 1},
                            prev_p : gp.p.clone(),
                            val : gp.val + 1
                        }
                    )
                }

                if gp.p.x < d_w {
                    raise_points.push_back(
                        GrowPoint {
                            p : TVec2 {x : gp.p.x + 1, y : gp.p.y},
                            prev_p : gp.p.clone(),
                            val : gp.val + 1
                        }
                    )
                }

                if gp.p.y < d_h {
                    raise_points.push_back(
                        GrowPoint {
                            p : TVec2 {x : gp.p.x, y : gp.p.y + 1},
                            prev_p : gp.p.clone(),
                            val : gp.val + 1
                        }
                    )
                }
            }
        }

        //move back
        
        let mut tmp_point = nearest_point;
        let mut cur_val = self.data[tmp_point.y * self.w + tmp_point.x];

        while cur_val > 2 {
            tmp_point = self.prev_point[tmp_point.y * self.w + tmp_point.x].clone();
            cur_val = self.data[tmp_point.y * self.w + tmp_point.x];
        }
        
        tmp_point
    }

    pub fn new(w : usize, h : usize) -> Self {
        Self {
            w,
            h,
            data : vec![0; w * h],
            walls : vec![false; w * h],
            prev_point : vec![TVec2::default(); w * h]
        }
    }
}

pub struct Map {
    pub data : Vec<Tile>,
    pub recycle_me : Vec<bool>,
    pub recycle_enemy : Vec<bool>,
    pub w : usize,
    pub h : usize,
    pub my_scrap : i32,
    pub enemy_scrap : i32,
    pub pathfinder : Pathfinder
}


impl Map {

    pub fn next_turn(&mut self, my_actions : &Vec<Action>, enemy_actions : &Vec<Action>) {
        //build
        self.build(my_actions, enemy_actions);
        self.move_spawn(my_actions, enemy_actions);
        self.tile_process();
        self.recycler_process();

        //perturn scrap increase
        self.my_scrap += 10;
        self.enemy_scrap += 10;
    }

    fn recycler_process(&mut self) {
        self.recycle_me.fill(false);
        self.recycle_enemy.fill(false);

        let dw = self.w - 1;
        let dh = self.h - 1;

        for y in 0..self.h {
            for x in 0..self.w {
                let idx = y * self.w + x;
                if self.data[idx].recycler {
                    if self.data[idx].owner == TileOwner::Me {
                        self.recycle_me[idx] = true;
                        if x > 0 {
                            self.recycle_me[idx - 1] = true;
                        }
                        if x < dw {
                            self.recycle_me[idx + 1] = true;
                        }
                        if y > 0 {
                            self.recycle_me[idx - self.w] = true;
                        }
                        if y < dh {
                            self.recycle_me[idx + self.w] = true;
                        }
                    } else {
                        self.recycle_enemy[idx] = true;
                        if x > 0 {
                            self.recycle_enemy[idx - 1] = true;
                        }
                        if x < dw {
                            self.recycle_enemy[idx + 1] = true;
                        }
                        if y > 0 {
                            self.recycle_enemy[idx - self.w] = true;
                        }
                        if y < dh {
                            self.recycle_enemy[idx + self.w] = true;
                        }
                    }
                }
            }
        }

        for idx in 0..self.data.len() {
            let mut tile = &mut self.data[idx];
            let r_me = self.recycle_me[idx];
            let r_enemy = self.recycle_enemy[idx];
            if r_me || r_enemy {
                if tile.scrap_amount > 0 {
                    tile.scrap_amount -= 1;
                    if r_me {
                        self.my_scrap += 1;
                    }
                    if r_enemt {
                        self.enemy_scrap += 1;
                    }
                    if tile.scrap_amount == 0 {
                        tile.owner = TileOwner::No;
                        tile.units = 0;
                        tile.delta_units = 0;
                        tile.recycler = false;
                    }
                }

            }
        }
    }


    fn tile_process(&mut self) {
        for idx in 0..self.data.len() {
            let mut tile = &mut self.data[idx];
            tile.units += tile.delta_units;
            tile.delta_units = 0;

            if tile.units > 0 {
                tile.owner = TileOwner::Me;
            } else if tile.units < 0 {
                tile.owner = TileOwner::Enemy;
            }
        }
    }

    fn build(&mut self, my_actions : &Vec<Action>, enemy_actions : &Vec<Action>) {
        for a in my_actions.iter() {
            if self.my_scrap < 10 {
                break;
            }
            if let Action::Build(build) = a {
                let mut tile = &mut self.data[(build.y * self.w + build.x) as usize];
                if tile.owner == TileOwner::Me && tile.units == 0 {
                    tile.recycler = true;
                    self.my_scrap -= 10;
                }
            }
        }

        for a in enemy_actions.iter() {
            if self.enemy_scrap < 10 {
                break;
            }
            if let Action::Build(build) = a {
                let mut tile = &mut self.data[(build.y * self.w + build.x) as usize];
                if tile.owner == TileOwner::Enemy && tile.units == 0 {
                    tile.recycler = true;
                    self.enemy_scrap -= 10;
                }
            }
        }
    }

    fn move_spawn(&mut self, my_actions : &Vec<Action>, enemy_actions : &Vec<Action>) {
        //setup walls
        for idx in 0..self.data.len() {
            let tile = &self.data[idx];
            let wall = tile.recycler || tile.scrap_amount == 0;
            self.pathfinder.walls[idx] = wall;
        }

        //move
        for a in my_actions.iter() {
            if let Action::Move(mv) = a {
                let idx = (mv.fromX * self.w + mv.fromY) as usize;

                if self.data[idx].owner == TileOwner::Me && self.data[idx].units > 0 {
                    let move_amount = self.data[idx].units.min(mv.amount as i32);
                    let dst = self.pathfinder.find_path(
                        &TVec2::new(mv.fromX as usize, mv.fromY as usize),
                        &TVec2::new(mv.toX as usize, mv.toY as usize));
                    self.data[dst.y * self.w as usize + dst.x].delta_units += move_amount;
                    self.data[idx].units -= move_amount;
                }
            } else if let Action::Spawn(sp) = a {
                if sp.amount > 0 {
                    let cost = sp.amount as i32 * 10;
                    if self.my_scrap >= cost {
                        let tile = &mut self.data[(sp.x * self.w + sp.y) as usize];
                        tile.delta_units += sp.amount as i32;
                        self.my_scrap -= cost;
                    }
                }
            }
        }

        for a in enemy_actions.iter() {
            if let Action::Move(mv) = a {
                let idx = (mv.fromX * self.w + mv.fromY) as usize;

                if self.data[idx].owner == TileOwner::Enemy && self.data[idx].units < 0 {
                    let move_amount = (-self.data[idx].units).min(mv.amount as i32);
                    let dst = self.pathfinder.find_path(
                        &TVec2::new(mv.fromX as usize, mv.fromY as usize),
                        &TVec2::new(mv.toX as usize, mv.toY as usize));
                    self.data[dst.y * self.w as usize + dst.x].delta_units -= move_amount;
                    self.data[idx].units += move_amount;
                }
            } else if let Action::Spawn(sp) = a {
                if sp.amount > 0 {
                    let cost = sp.amount as i32 * 10;
                    if self.enemy_scrap >= cost {
                        let tile = &mut self.data[(sp.x * self.w + sp.y) as usize];
                        tile.delta_units -= sp.amount as i32;
                        self.enemy_scrap -= cost;
                    }
                }
            }
        }
    }

    pub fn load(data : String) -> Self {
        let mut lines = data.split(';');

        let first_line : Vec<&str> = lines.next().unwrap().split(' ').collect();
        let width = u32::from_str(first_line[0]).unwrap();
        let height = u32::from_str(first_line[1]).unwrap();

        let scrap_line = lines.next().unwrap().split(' ').collect::<Vec<&str>>();
        let my_scrap = parse_input!(scrap_line[0], i32);
        let enemy_scrap = parse_input!(scrap_line[1], i32);

        let mut data = vec![Tile::default(); (width * height) as usize];
        for y in 0..height {
            for x in 0..width {
                let inputs = lines.next().unwrap().split(' ').collect::<Vec<&str>>();
                let scrap_amount = parse_input!(inputs[0], i32);
                let owner = parse_input!(inputs[1], i32); // 1 = me, 0 = foe, -1 = neutral
                let mut units = parse_input!(inputs[2], i32);
                let recycler = parse_input!(inputs[3], i32);
                let can_build = parse_input!(inputs[4], i32);
                let can_spawn = parse_input!(inputs[5], i32);
                let in_range_of_recycler = parse_input!(inputs[6], i32);

                let tile_owner;
                if owner == 1 {
                    tile_owner = TileOwner::Me;
                } else if owner == -1 {
                    tile_owner = TileOwner::No;
                } else  {
                    tile_owner = TileOwner::Enemy;
                }

                if tile_owner ==  TileOwner::Enemy {
                    units *= -1;
                }

                let tile = Tile {
                    owner : tile_owner,
                    units,
                    scrap_amount,
                    recycler : (recycler == 1),
                    delta_units : 0
                };

                data[(y * width + x) as usize] = tile;
            }

        }

        let data_len = data.len();

        Self {
            data,
            recycle_me : vec![false; data_len],
            recycle_enemy : vec![false; data_len],
            w : width as usize,
            h : height as usize,
            my_scrap,
            enemy_scrap,
            pathfinder : Pathfinder::new(width as usize, height as usize)
        }
    }

    pub fn load_file<P : AsRef<Path>>(path : P) -> Self {
        let mut file = std::fs::File::open(path).unwrap();
        let mut data = String::new();
        file.read_to_string(&mut data);
        Map::load(data)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_load_map() {
        Map::load(String::from_str("14 7;10 10;9 -1 0 0 0 0 0;6 -1 0 0 0 0 0;8 -1 0 0 0 0 0;8 -1 0 0 0 0 0;4 -1 0 0 0 0 0;4 -1 0 0 0 0 0;8 -1 0 0 0 0 0;10 -1 0 0 0 0 0;6 -1 0 0 0 0 0;6 -1 0 0 0 0 0;10 -1 0 0 0 0 0;4 -1 0 0 0 0 0;8 -1 0 0 0 0 0;6 -1 0 0 0 0 0;8 -1 0 0 0 0 0;0 -1 0 0 0 0 0;6 -1 0 0 0 0 0;9 -1 0 0 0 0 0;6 -1 0 0 0 0 0;8 -1 0 0 0 0 0;8 -1 0 0 0 0 0;8 -1 0 0 0 0 0;4 -1 0 0 0 0 0;9 0 1 0 0 0 0;4 -1 0 0 0 0 0;6 -1 0 0 0 0 0;8 -1 0 0 0 0 0;9 -1 0 0 0 0 0;0 -1 0 0 0 0 0;0 -1 0 0 0 0 0;9 -1 0 0 0 0 0;8 -1 0 0 0 0 0;6 -1 0 0 0 0 0;9 -1 0 0 0 0 0;9 -1 0 0 0 0 0;9 -1 0 0 0 0 0;8 0 1 0 0 0 0;10 0 0 0 0 0 0;10 0 1 0 0 0 0;9 -1 0 0 0 0 0;4 -1 0 0 0 0 0;8 -1 0 0 0 0 0;8 -1 0 0 0 0 0;6 -1 0 0 0 0 0;4 -1 0 0 0 0 0;8 -1 0 0 0 0 0;8 1 1 0 0 1 0;8 -1 0 0 0 0 0;8 -1 0 0 0 0 0;8 -1 0 0 0 0 0;8 -1 0 0 0 0 0;8 0 1 0 0 0 0;8 -1 0 0 0 0 0;4 -1 0 0 0 0 0;6 -1 0 0 0 0 0;8 -1 0 0 0 0 0;8 -1 0 0 0 0 0;4 -1 0 0 0 0 0;9 -1 0 0 0 0 0;10 1 1 0 0 1 0;10 1 0 0 1 1 0;8 1 1 0 0 1 0;9 -1 0 0 0 0 0;9 -1 0 0 0 0 0;9 -1 0 0 0 0 0;6 -1 0 0 0 0 0;8 -1 0 0 0 0 0;9 -1 0 0 0 0 0;0 -1 0 0 0 0 0;0 -1 0 0 0 0 0;9 -1 0 0 0 0 0;8 -1 0 0 0 0 0;6 -1 0 0 0 0 0;4 -1 0 0 0 0 0;9 1 1 0 0 1 0;4 -1 0 0 0 0 0;8 -1 0 0 0 0 0;8 -1 0 0 0 0 0;8 -1 0 0 0 0 0;6 -1 0 0 0 0 0;9 -1 0 0 0 0 0;6 -1 0 0 0 0 0;0 -1 0 0 0 0 0;8 -1 0 0 0 0 0;6 -1 0 0 0 0 0;8 -1 0 0 0 0 0;4 -1 0 0 0 0 0;10 -1 0 0 0 0 0;6 -1 0 0 0 0 0;6 -1 0 0 0 0 0;10 -1 0 0 0 0 0;8 -1 0 0 0 0 0;4 -1 0 0 0 0 0;4 -1 0 0 0 0 0;8 -1 0 0 0 0 0;8 -1 0 0 0 0 0;6 -1 0 0 0 0 0;9 -1 0 0 0 0 0").unwrap());
    }

    #[test]
    fn test_dummy_pathfinder() {
        let mut pathfinder = Pathfinder::new(32, 32);
        let res = pathfinder.find_path(&TVec2::new(0, 0) , &TVec2::new(0, 10));

        assert_eq!(res, TVec2::new(0, 1));
    }
}