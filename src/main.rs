#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::egui;
use egui::*;

fn main() -> Result<(), eframe::Error> {
    // Log to stdout (if you run with `RUST_LOG=debug`).
    tracing_subscriber::fmt::init();

    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Keyboard events",
        options,
        Box::new(|_cc| Box::new(Content::default())),
    )
}

fn bubble_ui(ui: &mut egui::Ui, color: Color32) -> egui::Response {
    let desired_size = ui.spacing().interact_size.y * egui::vec2(2.0, 2.0);
    let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::click());

    if ui.is_rect_visible(rect) {
        let visuals = ui.style().interact_selectable(&response, false);
        let rect = rect.expand(visuals.expansion);
        let radius = 0.5 * rect.height();
        let center = egui::pos2(rect.left() + radius, rect.center().y);
        ui.painter()
            .circle(center, 0.75 * radius, color, visuals.fg_stroke);
    }

    response
}

pub fn bubble(color: Color32) -> impl egui::Widget {
    move |ui: &mut egui::Ui| bubble_ui(ui, color)
}

pub const EMPTY: Color32 = Color32::from_rgba_premultiplied(0, 0, 0, 0);
struct GameState<const DEPTH: usize, const STACK_CNT: usize> {
    colors: [[Color32; DEPTH]; STACK_CNT],
}

struct Move {
    from: u8,
    to: u8,
}

impl<const DEPTH: usize, const STACK_CNT: usize> GameState<DEPTH, STACK_CNT> {
    fn empty_cnt(&self, stack: usize) -> usize {
        for d in 0..DEPTH {
            if self.colors[stack][d] != EMPTY {
                return d;
            }
        }
        DEPTH
    }

    fn apply_swap(&mut self, from: usize, to: usize) {
        assert!(from < STACK_CNT && to < STACK_CNT);

        let from_depth = self.empty_cnt(from);

        assert!(self.empty_cnt(to) >= 1);
        let to_depth = self.empty_cnt(to) - 1;

        let temp = self.colors[to][to_depth];
        self.colors[to][to_depth] = self.colors[from][from_depth];
        self.colors[from][from_depth] = temp;
    }

    fn is_valid_swap(&mut self, from: usize, to: usize) -> bool {
        assert!(from < STACK_CNT && to < STACK_CNT);

        if from == to {
            return false;
        }

        let src_empty_cnt = self.empty_cnt(from);
        let dst_empty_cnt = self.empty_cnt(to);

        if src_empty_cnt == DEPTH {
            return false;
        }

        if dst_empty_cnt == 0 {
            return false;
        }

        if dst_empty_cnt == DEPTH {
            return true;
        }

        self.colors[from][src_empty_cnt] == self.colors[to][dst_empty_cnt]
    }

    fn is_solved(&self) -> bool {
        for x in 0..STACK_CNT {
            let base_color = self.colors[x][DEPTH-1];
            if base_color == EMPTY {
                continue;
            }
            for y in 0..DEPTH-1 {
                if self.colors[x][y] != base_color {
                    return false;
                }
            }
        }

        return true;
    }

    fn apply(&mut self, m: &Move) {
        self.apply_swap(m.from as usize, m.to as usize)
    }

    fn revert(&mut self, m: &Move) {
        self.apply_swap(m.to as usize, m.from as usize)
    }
}

fn draw_field<const DEPTH: usize, const STACK_CNT: usize>(ui: &mut egui::Ui, field: GameState<DEPTH, STACK_CNT>) {
    ui.horizontal(|ui| {
        for x in 0..STACK_CNT {
            ui.vertical(|ui| {
                for y in 0..DEPTH {
                    ui.add(bubble(field.colors[x][y]));
                }
            });
        }
    });
}

#[derive(Default)]
struct Content {
    text: String,
}

impl eframe::App for Content {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Press/Hold/Release example. Press A to test.");

            let mut f = GameState::<2, 3>{
                colors: [
                    [Color32::RED, Color32::BLUE], 
                    [Color32::DARK_BLUE, Color32::GREEN], 
                    [EMPTY, EMPTY],
                ],
            };

            f.apply(&Move{from: 0, to: 2});
            f.revert(&Move{from: 0, to: 2});
            draw_field(ui, f);

            if ui.button("Clear").clicked() {
                self.text.clear();
            }
            ScrollArea::vertical()
                .auto_shrink([false; 2])
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    ui.label(&self.text);
                });

            if ctx.input(|i| i.key_pressed(Key::A)) {
                self.text.push_str("\nPressed");
            }
            if ctx.input(|i| i.key_down(Key::A)) {
                self.text.push_str("\nHeld");
                ui.ctx().request_repaint(); // make sure we note the holding.
            }
            if ctx.input(|i| i.key_released(Key::A)) {
                self.text.push_str("\nReleased");
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_move_validation() {
        let mut f = GameState::<2, 3>{
            colors: [
                [Color32::BLUE, Color32::RED], 
                [Color32::BLUE, Color32::GREEN], 
                [EMPTY, EMPTY],
            ],
        };

        assert!(!f.is_valid_swap(0, 0), "swapping element with itself must not be valid");

        assert!(f.is_valid_swap(0, 2), "stacking into empty columns must be valid");
        assert!(!f.is_valid_swap(0, 1), "stacking into full rows must not be valid");

        f.apply_swap(0, 2);
        assert!(f.is_valid_swap(1, 2), "stacking matching colors must be valid");
        assert!(!f.is_valid_swap(2, 0), "stacking mismatching colors must not be valid");
    }

    #[test]
    fn test_solved() {
        let mut f = GameState::<2, 3>{
            colors: [
                [Color32::BLUE, Color32::BLUE], 
                [Color32::BLUE, Color32::GREEN], 
                [EMPTY, EMPTY],
            ],
        };

        assert!(!f.is_solved(), "unsolved state");

        f.colors[1][1] = Color32::BLUE;
        assert!(f.is_solved(), "solved state");

        f.colors[2][1] = Color32::BLUE;
        assert!(!f.is_solved(), "partial empty stack");
    }
}