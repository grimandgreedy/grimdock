//! Persistence example with a runnable UI:
//! `cargo run --example 4_persistence`

use eframe::egui;
use grimdock::{
    ChildSide, DropPolicy, PaneBuilder, PaneOptions, PanelContext, PanelStyle, PanelTree,
    PersistedPanelTree, SplitDir, Tab,
};

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("grimdock 4_persistence")
            .with_inner_size([960.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "grimdock 4_persistence",
        options,
        Box::new(|_cc| Ok(Box::new(App::new()))),
    )
}

struct App {
    tree: PanelTree<&'static str>,
    saved: Option<PersistedPanelTree<&'static str>>,
    style: PanelStyle,
}

impl App {
    fn new() -> Self {
        Self {
            tree: make_tree(),
            saved: None,
            style: PanelStyle::default(),
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("controls").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Save layout").clicked() {
                    self.saved = Some(self.tree.to_persisted());
                }

                if ui.button("Restore saved layout").clicked() {
                    if let Some(saved) = self.saved.clone() {
                        self.tree =
                            PanelTree::from_persisted(saved).expect("saved layout should restore");
                    }
                }

                if ui.button("Reset demo layout").clicked() {
                    self.tree = make_tree();
                }
            });

            if let Some(saved) = &self.saved {
                ui.label(format!(
                    "Saved version {} with {} persisted nodes",
                    saved.version,
                    saved.nodes.len()
                ));
            } else {
                ui.label("No saved layout yet.");
            }
        });

        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ctx, |ui| {
                PanelContext::new(ui, &mut self.tree, &self.style).show(|ui, tab_id| {
                    ui.heading(*tab_id);
                    ui.label("Drag tabs, resize panes, then press Save layout.");
                    ui.label("Restore saved layout to rebuild the tree from the persisted format.");
                });
            });
    }
}

fn make_tree() -> PanelTree<&'static str> {
    let mut options = PaneOptions::default();
    options.drop_policy = DropPolicy::merge_only();
    options.allow_resize = false;

    let mut tree = PanelTree::from_pane(
        PaneBuilder::new(Tab::new("editor", "editor").with_leading_visual(">"))
            .push_tab(Tab::new("search", "search").with_leading_visual("?"))
            .with_options(options),
    );

    tree.split_leaf(
        0,
        SplitDir::Vertical,
        Tab::new("terminal", "terminal").with_leading_visual("$"),
        ChildSide::Second,
    );

    tree
}
