use egui::Id;

use crate::tree::PaneId;

/// Stable [`Id`] for the resize handle between the two children of a split node.
pub(crate) fn resize_handle_id(tree_node_idx: usize) -> Id {
    Id::new("grimdock::resize").with(tree_node_idx)
}

/// Stable [`Id`] for a tab button inside a leaf node.
pub(crate) fn tab_button_id(pane_id: PaneId, tab_pos: usize) -> Id {
    Id::new("grimdock::tab")
        .with(pane_id.into_raw())
        .with(tab_pos)
}

/// Stable [`Id`] for a tab close button inside a leaf node.
pub(crate) fn tab_close_button_id(pane_id: PaneId, tab_pos: usize) -> Id {
    Id::new("grimdock::tab_close")
        .with(pane_id.into_raw())
        .with(tab_pos)
}

/// Stable [`Id`] for the content child-UI of a leaf node.
pub(crate) fn pane_content_id(pane_id: PaneId) -> Id {
    Id::new("grimdock::content").with(pane_id.into_raw())
}
