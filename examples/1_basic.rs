//! Smallest runnable UI example:
//! `cargo run --example 1_basic`

use eframe::egui;
use grimdock::{PanelContext, PanelStyle, PanelTree, Tab};

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("grimdock 1_basic")
            .with_inner_size([800.0, 480.0]),
        ..Default::default()
    };

    eframe::run_native(
        "grimdock 1_basic",
        options,
        Box::new(|_cc| Ok(Box::new(App::new()))),
    )
}

struct App {
    tree: PanelTree<&'static str>,
    style: PanelStyle,
}

impl App {
    fn new() -> Self {
        Self {
            tree: PanelTree::new(vec![Tab::new("welcome", "welcome")]),
            style: PanelStyle::default(),
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ctx, |ui| {
                PanelContext::new(ui, &mut self.tree, &self.style).show(|ui, tab_id| {
                    ui.add_space(12.0);
                    ui.heading(*tab_id);
                    ui.label("This is the smallest useful grimdock setup.");
                    ui.label("There is one pane and one tab.");
                });
            });
    }
}
