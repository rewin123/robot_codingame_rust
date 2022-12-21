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
    pub owner : TileOwner
}

impl Default for Tile {
    fn default() -> Self {
        Self { 
            scrap_amount: 0, 
            recycler: false, 
            units: 0, 
            owner: TileOwner::No
        }
    }
}

pub struct Map {
    pub data : Vec<Tile>,
    pub w : u32,
    pub h : u32,
    pub my_scrap : i32,
    pub enemy_scrap : i32
}

impl Map {
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
                let units = parse_input!(inputs[2], i32);
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

                let tile = Tile {
                    owner : tile_owner,
                    units,
                    scrap_amount,
                    recycler : (recycler == 1)
                };

                data[(y * width + x) as usize] = tile;
            }

        }

        Self {
            data,
            w : width,
            h : height,
            my_scrap,
            enemy_scrap
        }
    }

    pub fn load_file<P : AsRef<Path>>(path : P) -> Self {
        let mut file = std::fs::File::open(path).unwrap();
        let mut data = String::new();
        file.read_to_string(&mut data);
        Map::load(data)
    }
}