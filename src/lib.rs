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

/// Entry point for rendering the panel layout each frame.
pub struct PanelContext<'ui, T: Clone + 'static> {
    ui: &'ui mut Ui,
    tree: &'ui mut PanelTree<T>,
    style: &'ui PanelStyle,
}

impl<'ui, T: Clone + 'static> PanelContext<'ui, T> {
    pub fn new(
        ui: &'ui mut Ui,
        tree: &'ui mut PanelTree<T>,
        style: &'ui PanelStyle,
    ) -> Self {
        Self { ui, tree, style }
    }

    /// Run the layout and content passes for the current frame.
    pub fn show(self, render: impl FnMut(&mut Ui, &T))
    where
        T: PartialEq,
    {
        let Self { ui, tree, style } = self;
        let leaf_rects = layout::layout_pass(ui, tree, style);
        content::content_pass(ui, tree, &leaf_rects, style, render);
    }
}
