extern crate core;

use core::panicking::panic;
use std::fmt::{Debug, Formatter};
use egui::Context;
use rand::Rng;
use bot::{Action, Map, TileOwner};
use bot::net::{Layer, NetImage, SimpleNetwork};


pub trait Scene {
    fn update(&mut self, ctx: &egui::Context);
}

pub struct Agent {
    pub network : SimpleNetwork,
    pub output : NetImage,
    pub input : NetImage,
    pub fitness : f32,
    pub owner : TileOwner
}


pub struct GeneticAlgorithm {
    pub population : Vec<Agent>,
    pub game_count : usize,
    pub selection_rate : f32
}

impl GeneticAlgorithm {
    fn step(&mut self) {

    }
}

impl Default for GeneticAlgorithm {
    fn default() -> Self {
        GeneticAlgorithm {
            population : vec![],
            game_count : 3,
            selection_rate : 0.5
        }
    }
}

impl Agent {
    pub fn build() -> Agent {
        Agent {
            network : SimpleNetwork::simple_maker(5, 4, 16, 4, 2),
            fitness : 0.0,
            output : NetImage::new(1,1,1),
            input : NetImage::new(1,1,1),
            owner : TileOwner::No
        }
    }

    pub fn prepare(&mut self, map : &Map, side : TileOwner) {
        let input = NetImage::new(map.w, map.h, 4);
        self.output = self.network.allocate_output(&input);
        self.input = input;
        self.owner = side;
    }

    pub fn get_actions(&mut self, map : &Map) -> Vec<Action> {
        //fill data
        let unit_k = match self.owner {
            TileOwner::Me => {1.0}
            TileOwner::Enemy => {-1.0}
            TileOwner::No => {panic!("Unsupported owner")}
        };

        for y in 0..map.h {
            for x in 0..map.w {
                let idx = y * map.w + x;
                *self.input.get_mut(x, y, 0) = map.data[idx].units as f32 * unit_k;
                *self.input.get_mut(x, y, 1) = map.data[idx].scrap_amount as f32;

                let mut owner = 0.0;
                if map.data[idx].owner == TileOwner::Me {
                    owner = 1.0;
                } else if map.data[idx].owner == TileOwner::Enemy {
                    owner = -1.0;
                }
                *self.input.get_mut(x, y, 2) = owner;

                *self.input.get_mut(x, y, 3) = match map.data[idx].recycler {
                    true => {1.0}
                    false => {0.0}
                };
            }
        }

        self.network.process(&self.input, &mut self.output);



        vec![]
    }
}

pub struct GeneticScene {
    pub ga : GeneticAlgorithm
}



impl Default for GeneticScene {
    fn default() -> Self {
        Self {
            ga : GeneticAlgorithm::default()
        }
    }
}

impl Scene for GeneticScene {
    fn update(&mut self, ctx: &Context) {
        egui::SidePanel::left("Left").show(ctx, |ui| {
            ui.label(format!("Population size: {}", self.ga.population.len()));
            if ui.button("Fill population").clicked() {
                for idx in 0..100 {
                    self.ga.population.push(Agent::build());
                }
            }
        });
    }
}

