//! Split layout and resize example:
//! `cargo run --example 2_split_layout`

use eframe::egui;
use grimdock::{ChildSide, PanelContext, PanelStyle, PanelTree, SplitDir, Tab};

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("grimdock 2_split_layout")
            .with_inner_size([960.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "grimdock 2_split_layout",
        options,
        Box::new(|_cc| Ok(Box::new(App::new()))),
    )
}

#[derive(Clone, PartialEq, Eq, Debug)]
enum TabId {
    Files,
    Editor,
    Terminal,
}

struct App {
    tree: PanelTree<TabId>,
    style: PanelStyle,
}

impl App {
    fn new() -> Self {
        let mut tree = PanelTree::new(vec![Tab::new("editor", TabId::Editor)]);
        tree.split_leaf(
            0,
            SplitDir::Horizontal,
            Tab::new("files", TabId::Files),
            ChildSide::First,
        );

        let editor_pane = tree
            .find_pane_containing(&TabId::Editor)
            .expect("editor pane should exist");
        tree.pane_mut(editor_pane)
            .expect("editor pane should exist")
            .split(
                SplitDir::Vertical,
                Tab::new("terminal", TabId::Terminal),
                ChildSide::Second,
            );

        Self {
            tree,
            style: PanelStyle::default(),
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ctx, |ui| {
                PanelContext::new(ui, &mut self.tree, &self.style).show(|ui, tab_id| match tab_id {
                    TabId::Files => {
                        ui.heading("Files");
                        ui.label("Try dragging the resize handles between panes.");
                    }
                    TabId::Editor => {
                        ui.heading("Editor");
                        ui.label("This pane started as the root.");
                    }
                    TabId::Terminal => {
                        ui.heading("Terminal");
                        ui.label("This pane was added by splitting the editor vertically.");
                    }
                });
            });
    }
}
