use eframe::egui;
use egui::*;
use bot::*;
fn main() {

    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Keyboard events",
        options,
        Box::new(|_cc| Box::new(Content::default())),
    );
}

struct Content {
    map : Map
}

fn tile_to_color(tile : &Tile) -> Color32 {
    if tile.scrap_amount == 0 {
        Color32::DARK_GREEN
    } else if tile.owner == TileOwner::Me {
        Color32::DARK_BLUE
    } else if tile.owner == TileOwner::No {
        Color32::DARK_GRAY
    } else if tile.owner == TileOwner::Enemy {
        Color32::DARK_RED
    } else {
        Color32::BLACK
    }
}

fn draw_map(map : &Map, ui : &egui::Ui) {
    let painter = ui.painter();
    let aval_width = ui.available_width();
    let tile_size = aval_width / map.w as f32;

    let shift = Vec2::new(tile_size / 2.0, tile_size / 2.0);

    for y in 0..map.h {
        for x in 0..map.w {
            let tile = &map.data[(y * map.w + x) as usize];
            let pos = Pos2::new(x as f32 * tile_size, y as f32 * tile_size) + shift;
            let size = Vec2::new(tile_size - 2.0, tile_size - 2.0);
            let rect = Rect::from_center_size(pos, size);
            let color = tile_to_color(tile);
            painter.rect_filled(rect, 1.0, color);

            if tile.units > 0 {
                let bot_color;
                if tile.owner == TileOwner::Me {
                    bot_color = Color32::LIGHT_BLUE;
                } else {
                    bot_color = Color32::LIGHT_RED;
                }
                painter.circle_filled(pos, tile_size / 2.0 * 0.8, bot_color);
                painter.debug_text(pos, Align2::CENTER_CENTER, Color32::BLACK, format!("{}", tile.units));
            }
        }
    }
}

impl Default for Content {
    fn default() -> Self {
        let map = Map::load_file("start_map.txt");

        Self { 
            map
        }
    }
}

impl eframe::App for Content {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            draw_map(&self.map, ui);
        });
    }
}