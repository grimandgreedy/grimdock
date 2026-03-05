//! Policy controls example:
//! `cargo run --example 6_policy_controls`

use eframe::egui;
use grimdock::{
    DropPolicy, PaneAnchor, PaneBuilder, PaneId, PaneMenuAction, PaneOptions, PaneRole,
    PanelContext, PanelStyle, PanelTree, Tab, TabDropPolicy,
};

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("grimdock 6_policy_controls")
            .with_inner_size([1180.0, 720.0]),
        ..Default::default()
    };

    eframe::run_native(
        "grimdock 6_policy_controls",
        options,
        Box::new(|_cc| Ok(Box::new(App::new()))),
    )
}

#[derive(Clone, PartialEq, Eq, Debug)]
enum TabId {
    Files,
    Editor,
    Search,
    Terminal,
}

struct App {
    tree: PanelTree<TabId>,
    style: PanelStyle,
    files_home_pane: PaneId,
    editor_home_pane: PaneId,
    terminal_pane: PaneId,
    persist_files_pane: bool,
    lock_files_layout: bool,
    disable_files_drops: bool,
    disable_terminal_resize: bool,
    disable_editor_reorder: bool,
    disable_editor_drag_out: bool,
    freeze_search_tab: bool,
    lock_search_to_files_pane: bool,
    allow_search_only_files_and_editor: bool,
    block_search_from_terminal: bool,
    lock_search_to_sidebar_role: bool,
    allow_search_only_sidebar_and_editor_roles: bool,
    block_search_from_terminal_role: bool,
    pane_action_log: Vec<String>,
}

impl App {
    fn new() -> Self {
        let (tree, files_home_pane, editor_home_pane, terminal_pane) = make_tree();
        Self {
            tree,
            style: PanelStyle::default(),
            files_home_pane,
            editor_home_pane,
            terminal_pane,
            persist_files_pane: true,
            lock_files_layout: false,
            disable_files_drops: false,
            disable_terminal_resize: false,
            disable_editor_reorder: false,
            disable_editor_drag_out: false,
            freeze_search_tab: false,
            lock_search_to_files_pane: false,
            allow_search_only_files_and_editor: false,
            block_search_from_terminal: false,
            lock_search_to_sidebar_role: false,
            allow_search_only_sidebar_and_editor_roles: false,
            block_search_from_terminal_role: false,
            pane_action_log: Vec::new(),
        }
    }

    fn reset_layout(&mut self) {
        let (tree, files_home_pane, editor_home_pane, terminal_pane) = make_tree();
        self.tree = tree;
        self.files_home_pane = files_home_pane;
        self.editor_home_pane = editor_home_pane;
        self.terminal_pane = terminal_pane;
    }

    fn update_pane_options(
        &mut self,
        pane_id: PaneId,
        update: impl FnOnce(&mut PaneOptions),
    ) {
        if let Some(mut pane) = self.tree.pane_mut(pane_id) {
            let mut options = pane.options();
            update(&mut options);
            pane.set_options(options);
        }
    }

    fn apply_policy_toggles(&mut self) {
        let persist_files_pane = self.persist_files_pane;
        let lock_files_layout = self.lock_files_layout;
        let disable_files_drops = self.disable_files_drops;
        let disable_terminal_resize = self.disable_terminal_resize;
        let disable_editor_reorder = self.disable_editor_reorder;
        let disable_editor_drag_out = self.disable_editor_drag_out;
        let freeze_search_tab = self.freeze_search_tab;
        let lock_search_to_files_pane = self.lock_search_to_files_pane;
        let allow_search_only_files_and_editor = self.allow_search_only_files_and_editor;
        let block_search_from_terminal = self.block_search_from_terminal;
        let lock_search_to_sidebar_role = self.lock_search_to_sidebar_role;
        let allow_search_only_sidebar_and_editor_roles =
            self.allow_search_only_sidebar_and_editor_roles;
        let block_search_from_terminal_role = self.block_search_from_terminal_role;
        let files_home_pane = self.files_home_pane;
        let editor_home_pane = self.editor_home_pane;
        let terminal_pane = self.terminal_pane;

        self.update_pane_options(files_home_pane, |options| {
            options.persist_when_empty = persist_files_pane;
            options.lock_layout = lock_files_layout;
            options.drop_policy = if disable_files_drops {
                DropPolicy::none()
            } else {
                DropPolicy::all()
            };
        });

        self.update_pane_options(editor_home_pane, |options| {
            options.allow_tab_reorder = !disable_editor_reorder;
            options.allow_tab_drag_out = !disable_editor_drag_out;
        });

        self.update_pane_options(terminal_pane, |options| {
            options.allow_resize = !disable_terminal_resize;
        });

        if let Some(search_pane_id) = self.tree.find_pane_containing(&TabId::Search) {
            if let Some(mut pane) = self.tree.pane_mut(search_pane_id) {
                if let Some(tab) = pane.tabs_mut().iter_mut().find(|tab| tab.id == TabId::Search) {
                    tab.draggable = !freeze_search_tab;

                    let mut drop_policy = TabDropPolicy::default();
                    if lock_search_to_files_pane {
                        drop_policy.locked_to_pane = Some(files_home_pane);
                    } else if lock_search_to_sidebar_role {
                        drop_policy.locked_to_role = Some(PaneRole::Sidebar);
                    } else if allow_search_only_files_and_editor {
                        drop_policy.allowed_panes = Some(vec![files_home_pane, editor_home_pane]);
                    } else if allow_search_only_sidebar_and_editor_roles {
                        drop_policy.allowed_roles =
                            Some(vec![PaneRole::Sidebar, PaneRole::Editor]);
                    }
                    if block_search_from_terminal {
                        drop_policy.blocked_panes.push(terminal_pane);
                    }
                    if block_search_from_terminal_role {
                        drop_policy.blocked_roles.push(PaneRole::Terminal);
                    }
                    tab.drop_policy = drop_policy;
                }
            }
        }
    }

    fn pane_present(&self, pane_id: PaneId) -> bool {
        self.tree.pane(pane_id).is_some()
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.apply_policy_toggles();

        egui::SidePanel::left("policy_controls")
            .resizable(false)
            .default_width(320.0)
            .show(ctx, |ui| {
                ui.heading("Policy controls");
                ui.label("This example separates pane policy from tab policy.");
                ui.separator();

                ui.label("Pane policy");
                ui.checkbox(&mut self.persist_files_pane, "Persist Files pane when empty");
                ui.checkbox(&mut self.lock_files_layout, "Lock Files pane layout");
                ui.checkbox(&mut self.disable_files_drops, "Reject drops into Files pane");
                ui.checkbox(&mut self.disable_terminal_resize, "Disable Terminal resize");
                ui.checkbox(&mut self.disable_editor_reorder, "Disable Editor pane reorder");
                ui.checkbox(&mut self.disable_editor_drag_out, "Disable Editor pane drag-out");

                ui.separator();
                ui.label("Tab policy");
                ui.checkbox(&mut self.freeze_search_tab, "Make Search tab immovable");
                ui.checkbox(&mut self.lock_search_to_files_pane, "Lock Search tab to Files pane ID");
                ui.checkbox(&mut self.lock_search_to_sidebar_role, "Lock Search tab to Sidebar role");
                ui.checkbox(
                    &mut self.allow_search_only_files_and_editor,
                    "Allow Search only in Files and Editor pane IDs",
                );
                ui.checkbox(
                    &mut self.allow_search_only_sidebar_and_editor_roles,
                    "Allow Search only in Sidebar and Editor roles",
                );
                ui.checkbox(&mut self.block_search_from_terminal, "Block Search from Terminal pane ID");
                ui.checkbox(
                    &mut self.block_search_from_terminal_role,
                    "Block Search from Terminal role",
                );

                ui.separator();
                if ui.button("Reset layout").clicked() {
                    self.reset_layout();
                }

                ui.separator();
                ui.label(format!(
                    "Files home pane present: {}",
                    if self.pane_present(self.files_home_pane) { "yes" } else { "no" }
                ));
                ui.label(format!(
                    "Files anchor owner: {:?}",
                    self.tree.find_pane_with_anchor(PaneAnchor::Left)
                ));
                ui.label(format!(
                    "Terminal anchor owner: {:?}",
                    self.tree.find_pane_with_anchor(PaneAnchor::Bottom)
                ));
                ui.label(format!(
                    "Sidebar role owner: {:?}",
                    self.tree.find_pane_with_role(PaneRole::Sidebar)
                ));
                ui.label(format!(
                    "Terminal role owner: {:?}",
                    self.tree.find_pane_with_role(PaneRole::Terminal)
                ));
                ui.label(format!(
                    "Search tab currently in pane: {:?}",
                    self.tree.find_pane_containing(&TabId::Search)
                ));

                ui.separator();
                ui.label("What to try:");
                ui.label("1. Move Files out of its home pane. With persistence on, the anchored left pane stays as an empty drop target.");
                ui.label("2. Turn off Files persistence, empty that pane, and notice the pane can now disappear.");
                ui.label("3. Lock Search to the Files pane, then try dropping it into Editor or Terminal.");
                ui.label("4. Switch between pane-ID and pane-role targeting to compare semantic versus identity-based policy.");
                if let Some(last) = self.pane_action_log.last() {
                    ui.separator();
                    ui.label(format!("Last custom pane action: {last}"));
                }
            });

        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ctx, |ui| {
                let output = PanelContext::new(ui, &mut self.tree, &self.style)
                    .with_pane_menu_provider(&|pane_id, tree| {
                        let mut actions = vec![PaneMenuAction::new("inspect", "Inspect pane")];
                        if tree.find_pane_with_anchor(PaneAnchor::Left) == Some(pane_id) {
                            actions.push(PaneMenuAction::new("mark_left", "Mark left anchor"));
                        }
                        actions
                    })
                    .show(|ui, tab_id| match tab_id {
                    TabId::Files => {
                        ui.heading("Files");
                        ui.label("This pane demonstrates pane persistence, pane locking, and pane drop policy.");
                    }
                    TabId::Editor => {
                        ui.heading("Editor");
                        ui.label("This pane demonstrates pane-level reorder and drag-out controls.");
                    }
                    TabId::Search => {
                        ui.heading("Search");
                        ui.label("This tab demonstrates draggable state plus pane lock / allow-list / block-list targeting.");
                    }
                    TabId::Terminal => {
                        ui.heading("Terminal");
                        ui.label("This pane demonstrates resize locking and can also be blocked as a tab destination.");
                    }
                });

                for action in output.pane_actions {
                    self.pane_action_log
                        .push(format!("{} on pane {:?}", action.action_id, action.pane_id));
                }
            });
    }
}

fn make_tree() -> (PanelTree<TabId>, PaneId, PaneId, PaneId) {
    let mut tree = PanelTree::new(vec![
        Tab::new("editor", TabId::Editor).with_leading_visual(">"),
        Tab::new("search", TabId::Search).with_leading_visual("?"),
    ]);

    let files_home_pane = tree.ensure_pane_at_anchor(
        PaneAnchor::Left,
        PaneBuilder::new(Tab::new("files", TabId::Files).with_leading_visual("#")),
    );

    let editor_home_pane = tree
        .find_pane_containing(&TabId::Editor)
        .expect("editor pane should exist after initial split");

    let terminal_pane = tree.ensure_pane_at_anchor(
        PaneAnchor::Bottom,
        PaneBuilder::new(Tab::new("terminal", TabId::Terminal).with_leading_visual("$")),
    );

    if let Some(mut pane) = tree.pane_mut(files_home_pane) {
        let mut options = pane.options();
        options.persist_when_empty = true;
        options.role = Some(PaneRole::Sidebar);
        pane.set_options(options);
    }

    if let Some(mut pane) = tree.pane_mut(editor_home_pane) {
        let mut options = pane.options();
        options.role = Some(PaneRole::Editor);
        pane.set_options(options);
    }

    if let Some(mut pane) = tree.pane_mut(terminal_pane) {
        let mut options = pane.options();
        options.role = Some(PaneRole::Terminal);
        pane.set_options(options);
    }

    (tree, files_home_pane, editor_home_pane, terminal_pane)
}
