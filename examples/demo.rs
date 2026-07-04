//! Runnable demo: `cargo run --example demo`
//!
//! Opens a window with a pre-built split layout. Tabs can be dragged between
//! panes, panes can be resized by dragging the handle between them.

use eframe::egui;
use grimdock::{
    AddTabEntry, ChildSide, DropPolicy, Node, PaneBuilder, PaneStyleOverride, PanelContext,
    PanelStyle, PanelTree, SplitDir, Tab, TabStyleOverride,
};

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("grimdock demo")
            .with_inner_size([1024.0, 680.0]),
        ..Default::default()
    };
    eframe::run_native("grimdock demo", options, Box::new(|_cc| Ok(Box::new(App::new()))))
}

/// A tab identifier. Using a simple enum so each variant renders unique content.
#[derive(Clone, PartialEq, Eq, Debug)]
enum TabId {
    Editor(u32),
    FileTree,
    Terminal,
    Search,
    Problems,
    Output,
}

impl std::fmt::Display for TabId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TabId::Editor(n) => write!(f, "editor_{n}"),
            TabId::FileTree => write!(f, "file_tree"),
            TabId::Terminal => write!(f, "terminal"),
            TabId::Search => write!(f, "search"),
            TabId::Problems => write!(f, "problems"),
            TabId::Output => write!(f, "output"),
        }
    }
}

struct App {
    tree: PanelTree<TabId>,
    /// Counter that ticks up each frame in the terminal pane.
    tick: u64,
}

impl App {
    fn new() -> Self {
        // Build an initial layout:
        //
        //  ┌───────────┬──────────────────┐
        //  │ File Tree │  Editor 1        │
        //  │           ├──────────────────┤
        //  │           │ Terminal │ Output│
        //  └───────────┴──────────────────┘
        //
        // We do this by starting with a single pane and splitting.

        let mut tree = PanelTree::new(vec![
            Tab::new("editor_1", TabId::Editor(1))
                .with_leading_visual(">")
                .with_style_override(TabStyleOverride {
                    active_bg: Some(egui::Color32::from_rgb(62, 73, 50)),
                    inactive_bg: Some(egui::Color32::from_rgb(44, 52, 38)),
                    hovered_bg: Some(egui::Color32::from_rgb(55, 65, 45)),
                    text_color: Some(egui::Color32::from_rgb(221, 231, 200)),
                    accent_color: Some(egui::Color32::from_rgb(164, 196, 92)),
                    icon_color: None,
                    max_width: None,
                }),
            Tab::new("editor_2", TabId::Editor(2)).with_leading_visual("+"),
        ]);

        // Split root (editors) horizontally: file tree on the left (First side).
        let _file_tree_pane = tree.split_leaf_with(
            0,
            SplitDir::Horizontal,
            PaneBuilder::new(Tab::new("file_tree", TabId::FileTree).with_leading_visual("#")).with_options(
                grimdock::PaneOptions {
                    style_override: Some(PaneStyleOverride {
                        header_bg: Some(egui::Color32::from_rgb(27, 47, 44)),
                        content_bg: Some(egui::Color32::from_rgb(21, 36, 34)),
                        border_color: Some(egui::Color32::from_rgb(52, 90, 84)),
                        accent_color: Some(egui::Color32::from_rgb(85, 188, 162)),
                        content_inset: None,
                    }),
                    drop_policy: DropPolicy::merge_only(),
                    ..Default::default()
                },
            ),
            ChildSide::First,
        );
        // Node 0 is now Horizontal split.
        // Node 1 (left/First): FileTree
        // Node 2 (right/Second): editors

        // Split the right pane (editors, node 2) vertically: bottom gets terminal.
        let editors_pane = tree
            .find_pane_containing(&TabId::Editor(1))
            .expect("editor pane should exist after initial split");
        let bottom_pane = {
            let mut pane = tree.pane_mut(editors_pane).expect("editor pane should exist");
            pane.split_with(
                SplitDir::Vertical,
                PaneBuilder::new(Tab::new("terminal", TabId::Terminal).with_leading_visual("$")),
                ChildSide::Second,
            )
        };
        // Node 2 is now Vertical split.
        // Node 5 (top/First): editors
        // Node 6 (bottom/Second): terminal

        // Add extra tabs to the bottom bar.
        // Nodes are: 5 = editors top pane, 6 = bottom pane (terminal).
        let mut bottom = tree.pane_mut(bottom_pane).expect("bottom pane should exist");
        bottom.push_tab(Tab::new("output", TabId::Output).with_leading_visual("!"));
        bottom.push_tab(Tab::new("problems", TabId::Problems).with_leading_visual("x"));
        bottom.push_tab(Tab::new("search", TabId::Search).with_leading_visual("?"));

        // Narrow the left (file tree) column — set the split ratio.
        if let Node::Split { ratio, .. } = tree.node_mut(0) {
            *ratio = 0.22;
        }

        // Make the bottom strip shorter.
        if let Node::Split { ratio, .. } = tree.node_mut(2) {
            *ratio = 0.65;
        }

        Self {
            tree,
            tick: 0,
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.tick += 1;

        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ctx, |ui| {
                let tree = &mut self.tree;
                let mut style = PanelStyle::from_egui_style(ui.style().as_ref());
                style.content_inset = 4.0;
                style.tabs.rounding = egui::CornerRadius::same(5);
                style.header.button.rounding = egui::CornerRadius::same(5);
                let tick = self.tick;
                let add_tab_entries = vec![
                    AddTabEntry::new(
                        "Editor 3",
                        Tab::new("editor_3", TabId::Editor(3))
                            .with_leading_visual("*")
                            .with_closable(true),
                    ),
                    AddTabEntry::new(
                        "Terminal",
                        Tab::new("terminal", TabId::Terminal)
                            .with_leading_visual("$")
                            .with_closable(true),
                    ),
                    AddTabEntry::new(
                        "Search",
                        Tab::new("search", TabId::Search)
                            .with_leading_visual("?")
                            .with_closable(true),
                    ),
                    AddTabEntry::new(
                        "Problems",
                        Tab::new("problems", TabId::Problems)
                            .with_leading_visual("x")
                            .with_closable(true),
                    ),
                ];

                PanelContext::new(ui, tree, &style)
                    .with_add_tab_entries(&add_tab_entries)
                    .show(|ui, tab_id| {
                        render_tab(ui, tab_id, tick);
                    });
            });

        // Repaint continuously so the terminal counter updates.
        ctx.request_repaint();
    }
}

fn render_tab(ui: &mut egui::Ui, tab_id: &TabId, tick: u64) {
    egui::ScrollArea::vertical().show(ui, |ui| {
        match tab_id {
            TabId::Editor(n) => {
                ui.add_space(8.0);
                ui.label(
                    egui::RichText::new(format!("// Editor {n}\n\nfn main() {{\n    println!(\"Hello, world!\");\n}}"))
                        .monospace()
                        .size(13.0),
                );
                ui.add_space(8.0);
                ui.label("Drag a tab to a different pane to rearrange.");
                ui.label("Drag the handle between panes to resize.");
            }
            TabId::FileTree => {
                ui.add_space(6.0);
                for item in &["src/", "  lib.rs", "  tree.rs", "  layout.rs", "  header.rs", "  dnd.rs", "Cargo.toml"] {
                    ui.label(egui::RichText::new(*item).monospace().size(12.0));
                }
            }
            TabId::Terminal => {
                ui.add_space(4.0);
                ui.label(egui::RichText::new(format!("$ frame {tick}")).monospace().size(12.0));
                ui.label(egui::RichText::new("grimdock running ✓").monospace().color(egui::Color32::from_rgb(100, 200, 120)).size(12.0));
            }
            TabId::Search => {
                ui.add_space(8.0);
                ui.label("Search");
                ui.text_edit_singleline(&mut String::new());
            }
            TabId::Problems => {
                ui.add_space(8.0);
                ui.label(egui::RichText::new("No problems detected.").color(egui::Color32::from_rgb(100, 200, 120)));
            }
            TabId::Output => {
                ui.add_space(4.0);
                ui.label(egui::RichText::new("[INFO] Build complete.").monospace().size(12.0));
            }
        }
    });
}
