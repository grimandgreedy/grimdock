//! Header actions, add-tab menus, and tab markers:
//! `cargo run --example 3_header_features`

use eframe::egui;
use grimdock::{
    AddTabEntry, HeaderVisibility, OpenBehavior, PanelContext, PanelStyle, PanelTree,
    PersistedNode, Tab,
};

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("grimdock 3_header_features")
            .with_inner_size([960.0, 540.0]),
        ..Default::default()
    };

    eframe::run_native(
        "grimdock 3_header_features",
        options,
        Box::new(|_cc| Ok(Box::new(App::new()))),
    )
}

#[derive(Clone, PartialEq, Eq, Debug)]
enum TabId {
    Notes,
    Search,
    Problems,
    Terminal,
    Output,
}

struct App {
    tree: PanelTree<TabId>,
    style: PanelStyle,
    hide_headers: bool,
}

impl App {
    fn new() -> Self {
        Self {
            tree: PanelTree::new(vec![
                Tab::new("notes", TabId::Notes)
                    .with_leading_visual(">")
                    .with_closable(true),
                Tab::new("search", TabId::Search)
                    .with_leading_visual("?")
                    .with_closable(true),
                Tab::new("problems", TabId::Problems)
                    .with_leading_visual("x")
                    .with_closable(true),
            ]),
            style: PanelStyle::default(),
            hide_headers: false,
        }
    }

    fn sync_header_visibility(&mut self) {
        let header_visibility = if self.hide_headers {
            HeaderVisibility::Hidden
        } else {
            HeaderVisibility::Always
        };

        let pane_ids = self
            .tree
            .to_persisted()
            .nodes
            .into_iter()
            .filter_map(|node| match node {
                PersistedNode::Leaf { pane, .. } => Some(pane),
                _ => None,
            })
            .collect::<Vec<_>>();

        for pane_id in pane_ids {
            if let Some(mut pane) = self.tree.pane_mut(pane_id) {
                let mut options = pane.options();
                options.header_visibility = header_visibility;
                pane.set_options(options);
            }
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.sync_header_visibility();

        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ctx, |ui| {
                ui.checkbox(&mut self.hide_headers, "Hide pane headers");
                ui.separator();

                let output = PanelContext::new(ui, &mut self.tree, &self.style)
                    .with_add_tab_provider(&|pane_id, tree| {
                        let mut entries = vec![
                            AddTabEntry::new(
                                "Output",
                                Tab::new("output", TabId::Output)
                                    .with_leading_visual("!")
                                    .with_closable(true),
                            ),
                        ];

                        if tree.find_pane_containing(&TabId::Notes) == Some(pane_id) {
                            entries.push(
                                AddTabEntry::new(
                                    "Terminal",
                                    Tab::new("terminal", TabId::Terminal)
                                        .with_leading_visual("$")
                                        .with_closable(true),
                                )
                                .with_open_behavior(OpenBehavior::FocusExisting),
                            );
                        }

                        entries
                    })
                    .show(|ui, tab_id| match tab_id {
                        TabId::Notes => {
                            ui.heading("Notes");
                            ui.label("This pane gets both Output and Terminal in its + menu.");
                            ui.label("Right-click a tab for Close / Close others / Split actions.");
                        }
                        TabId::Search => {
                            ui.heading("Search");
                            ui.text_edit_singleline(&mut String::new());
                        }
                        TabId::Problems => {
                            ui.heading("Problems");
                            ui.label("No problems detected.");
                        }
                        TabId::Terminal => {
                            ui.heading("Terminal");
                            ui.label("$ cargo run");
                        }
                        TabId::Output => {
                            ui.heading("Output");
                            ui.label("[INFO] pane-specific add menus are enabled");
                        }
                    });

                if !output.closed_tabs.is_empty() {
                    ui.separator();
                    ui.label(format!("Closed this frame: {:?}", output.closed_tabs));
                }
            });
    }
}
