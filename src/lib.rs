//! # grimdock
//!
//! A dockable panel layout system for [egui](https://github.com/emilk/egui).
//!
//! Provides an IDE-style workspace where panels can be split, resized, and
//! rearranged by dragging tabs, all within egui's immediate-mode layer.
//!
//! ## Quick start
//!
//! ```rust,no_run
//! # use grimdock::{PanelTree, PanelStyle, PanelContext, Tab};
//! # fn show(ui: &mut egui::Ui, tree: &mut PanelTree<&'static str>) {
//! let style = PanelStyle::default();
//! PanelContext::new(ui, tree, &style).show(|ui, tab_id| {
//!     ui.label(*tab_id);
//! });
//! # }
//! ```

mod content;
mod header;
mod ids;
mod layout;
mod style;
mod tab;
mod tree;

pub use style::{
    ContentStyle, HandleStyle, HeaderButtonStyle, HeaderStyle, OverlayStyle, PaneStyleOverride,
    PanelStyle, TabStateStyle, TabStyle, TabStyleOverride, TypographyStyle,
};
pub use tab::{Tab, TabDropPolicy, TabIcon};
pub use tree::{
    ChildSide, DropPolicy, HeaderVisibility, Node, Pane, PaneAnchor, PaneBuilder, PaneId, PaneMut,
    PaneOptions, PaneRole, PanelTree, SplitDir,
};

use egui::Ui;

/// Built-in behavior for resolving add/open actions when a tab with the same
/// identifier may already exist.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OpenBehavior {
    AllowDuplicate,
    FocusExisting,
    FocusExistingInPane,
}

/// A caller-provided entry shown in built-in add-tab and split-here menus.
#[derive(Clone, Debug)]
pub struct AddTabEntry<T: Clone + 'static> {
    pub title: String,
    pub tab: Tab<T>,
    pub open_behavior: OpenBehavior,
}

impl<T: Clone + 'static> AddTabEntry<T> {
    pub fn new(title: impl Into<String>, tab: Tab<T>) -> Self {
        Self {
            title: title.into(),
            tab,
            open_behavior: OpenBehavior::FocusExisting,
        }
    }

    pub fn with_open_behavior(mut self, open_behavior: OpenBehavior) -> Self {
        self.open_behavior = open_behavior;
        self
    }
}

/// A caller-provided pane action shown in the built-in pane menu.
#[derive(Clone, Debug)]
pub struct PaneMenuAction {
    pub id: String,
    pub title: String,
}

impl PaneMenuAction {
    pub fn new(id: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
        }
    }
}

/// A custom pane action invoked from the built-in pane menu during this frame.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PaneActionInvocation {
    pub pane_id: PaneId,
    pub action_id: String,
}

/// Mutations emitted by a single [`PanelContext::show`] pass.
#[derive(Debug)]
pub struct PanelOutput<T> {
    pub closed_tabs: Vec<T>,
    pub pane_actions: Vec<PaneActionInvocation>,
}

impl<T> Default for PanelOutput<T> {
    fn default() -> Self {
        Self {
            closed_tabs: Vec::new(),
            pane_actions: Vec::new(),
        }
    }
}

/// Entry point for rendering the panel layout each frame.
pub struct PanelContext<'ui, T: Clone + 'static> {
    ui: &'ui mut Ui,
    tree: &'ui mut PanelTree<T>,
    style: &'ui PanelStyle,
    add_tab_entries: &'ui [AddTabEntry<T>],
    add_tab_provider: Option<&'ui dyn Fn(PaneId, &PanelTree<T>) -> Vec<AddTabEntry<T>>>,
    pane_menu_actions: &'ui [PaneMenuAction],
    pane_menu_provider: Option<&'ui dyn Fn(PaneId, &PanelTree<T>) -> Vec<PaneMenuAction>>,
}

impl<'ui, T: Clone + 'static> PanelContext<'ui, T> {
    pub fn new(
        ui: &'ui mut Ui,
        tree: &'ui mut PanelTree<T>,
        style: &'ui PanelStyle,
    ) -> Self {
        Self {
            ui,
            tree,
            style,
            add_tab_entries: &[],
            add_tab_provider: None,
            pane_menu_actions: &[],
            pane_menu_provider: None,
        }
    }

    pub fn with_add_tab_entries(mut self, add_tab_entries: &'ui [AddTabEntry<T>]) -> Self {
        self.add_tab_entries = add_tab_entries;
        self
    }

    pub fn with_add_tab_provider(
        mut self,
        add_tab_provider: &'ui dyn Fn(PaneId, &PanelTree<T>) -> Vec<AddTabEntry<T>>,
    ) -> Self {
        self.add_tab_provider = Some(add_tab_provider);
        self
    }

    pub fn with_pane_menu_actions(mut self, pane_menu_actions: &'ui [PaneMenuAction]) -> Self {
        self.pane_menu_actions = pane_menu_actions;
        self
    }

    pub fn with_pane_menu_provider(
        mut self,
        pane_menu_provider: &'ui dyn Fn(PaneId, &PanelTree<T>) -> Vec<PaneMenuAction>,
    ) -> Self {
        self.pane_menu_provider = Some(pane_menu_provider);
        self
    }

    /// Run the layout, header, and content passes for the current frame.
    pub fn show(self, render: impl FnMut(&mut Ui, &T)) -> PanelOutput<T>
    where
        T: PartialEq,
    {
        let Self {
            ui,
            tree,
            style,
            add_tab_entries,
            add_tab_provider,
            pane_menu_actions,
            pane_menu_provider,
        } = self;
        let mut output = PanelOutput::default();

        let leaf_rects = layout::layout_pass(ui, tree, style);
        header::header_pass(
            ui,
            tree,
            &leaf_rects,
            style,
            add_tab_entries,
            add_tab_provider,
            pane_menu_actions,
            pane_menu_provider,
            &mut output.closed_tabs,
            &mut output.pane_actions,
        );
        content::content_pass(ui, tree, &leaf_rects, style, render);
        output
    }
}
