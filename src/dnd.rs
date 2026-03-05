use egui::{DragAndDrop, LayerId, Order, Pos2, Rect, Ui};

use crate::{
    style::PanelStyle,
    tab::Tab,
    tree::{ChildSide, Node, PaneId, PaneOptions, PaneRole, PanelTree, SplitDir},
};

/// Payload carried through egui's drag-and-drop system.
#[derive(Clone, Debug)]
pub(crate) struct DragPayload {
    /// Stable pane identifier of the leaf that owns the dragged tab.
    pub src_pane: PaneId,
    /// Position of the dragged tab within the source leaf's tab list.
    pub tab_pos: usize,
}

/// Determines where relative to a pane's centre the cursor is sitting.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DropZone {
    Center,
    Left,
    Right,
    Top,
    Bottom,
}

fn drop_zone_allowed(options: PaneOptions, zone: DropZone) -> bool {
    if options.lock_layout {
        return false;
    }

    match zone {
        DropZone::Center => options.drop_policy.allow_merge,
        DropZone::Left => options.drop_policy.allows_split(SplitDir::Horizontal, ChildSide::First),
        DropZone::Right => {
            options
                .drop_policy
                .allows_split(SplitDir::Horizontal, ChildSide::Second)
        }
        DropZone::Top => options.drop_policy.allows_split(SplitDir::Vertical, ChildSide::First),
        DropZone::Bottom => {
            options
                .drop_policy
                .allows_split(SplitDir::Vertical, ChildSide::Second)
        }
    }
}

fn tab_allows_target_pane<T: Clone + 'static>(
    tab: &Tab<T>,
    pane_id: PaneId,
    role: Option<PaneRole>,
) -> bool {
    tab.drop_policy.allows_target(pane_id, role)
}

fn header_tab_drop_index<T: Clone + 'static>(
    tree: &PanelTree<T>,
    leaf_idx: usize,
    header_rect: Rect,
    cursor_pos: Pos2,
) -> Option<usize> {
    let tab_count = match tree.node(leaf_idx) {
        Node::Leaf { tabs, .. } => tabs.len(),
        _ => return None,
    };
    if tab_count == 0 {
        return None;
    }
    let tab_width = (header_rect.width() / tab_count.max(1) as f32)
        .min(120.0)
        .max(40.0);
    let relative_x = (cursor_pos.x - header_rect.min.x).clamp(0.0, header_rect.width());
    let raw_index = (relative_x / tab_width).floor() as usize;
    Some(raw_index.min(tab_count.saturating_sub(1)))
}

fn header_tab_rect<T: Clone + 'static>(
    tree: &PanelTree<T>,
    leaf_idx: usize,
    header_rect: Rect,
    tab_pos: usize,
) -> Option<Rect> {
    let tab_count = match tree.node(leaf_idx) {
        Node::Leaf { tabs, .. } => tabs.len(),
        _ => return None,
    };
    if tab_pos >= tab_count {
        return None;
    }
    let tab_width = (header_rect.width() / tab_count.max(1) as f32)
        .min(120.0)
        .max(40.0);
    Some(Rect::from_min_size(
        egui::pos2(
            header_rect.min.x + tab_pos as f32 * tab_width,
            header_rect.min.y,
        ),
        egui::vec2(tab_width, header_rect.height()),
    ))
}

fn root_edge_drop_zone(cursor: Pos2, root_rect: Rect) -> Option<DropZone> {
    if !root_rect.contains(cursor) {
        return None;
    }
    let edge_band = 40.0_f32.min(root_rect.width() * 0.25).min(root_rect.height() * 0.25);
    let left = cursor.x - root_rect.min.x;
    let right = root_rect.max.x - cursor.x;
    let top = cursor.y - root_rect.min.y;
    let bottom = root_rect.max.y - cursor.y;

    let mut best: Option<(f32, DropZone)> = None;
    for (dist, zone) in [
        (left, DropZone::Left),
        (right, DropZone::Right),
        (top, DropZone::Top),
        (bottom, DropZone::Bottom),
    ] {
        if dist <= edge_band {
            match best {
                Some((best_dist, _)) if dist >= best_dist => {}
                _ => best = Some((dist, zone)),
            }
        }
    }
    best.map(|(_, zone)| zone)
}

impl DropZone {
    /// Classify `cursor` against the five zones of `rect`.
    ///
    /// The central 40 % of each axis forms the `Center` zone. Outside that,
    /// the remaining area is divided into four triangular quadrants by the two
    /// diagonals of the rectangle.
    pub(crate) fn classify(cursor: Pos2, rect: Rect) -> Self {
        let center = rect.center();
        let dx = (cursor.x - center.x) / rect.width().max(1.0);
        let dy = (cursor.y - center.y) / rect.height().max(1.0);

        // Central region: both axes within ±20 % of the centre.
        if dx.abs() < 0.2 && dy.abs() < 0.2 {
            return DropZone::Center;
        }

        // Outside the centre: pick the dominant axis.
        if dx.abs() > dy.abs() {
            if dx > 0.0 {
                DropZone::Right
            } else {
                DropZone::Left
            }
        } else if dy > 0.0 {
            DropZone::Bottom
        } else {
            DropZone::Top
        }
    }

    /// Return the preview rectangle for this drop zone within `pane_rect`.
    pub(crate) fn preview_rect(self, pane_rect: Rect) -> Rect {
        match self {
            DropZone::Center => pane_rect,
            DropZone::Left => Rect::from_min_max(
                pane_rect.min,
                egui::pos2(pane_rect.center().x, pane_rect.max.y),
            ),
            DropZone::Right => Rect::from_min_max(
                egui::pos2(pane_rect.center().x, pane_rect.min.y),
                pane_rect.max,
            ),
            DropZone::Top => Rect::from_min_max(
                pane_rect.min,
                egui::pos2(pane_rect.max.x, pane_rect.center().y),
            ),
            DropZone::Bottom => Rect::from_min_max(
                egui::pos2(pane_rect.min.x, pane_rect.center().y),
                pane_rect.max,
            ),
        }
    }
}

/// Run the drag-and-drop resolution pass.
///
/// Draws the drop-target overlay while a drag is live, and mutates the tree
/// on pointer release.
pub(crate) fn dnd_pass<T: Clone + 'static>(
    ui: &mut Ui,
    tree: &mut PanelTree<T>,
    style: &PanelStyle,
) {
    let ctx = ui.ctx().clone();

    // Is a drag live?
    let Some(live_payload) = DragAndDrop::payload::<DragPayload>(&ctx) else {
        return;
    };

    let cursor_pos = match ctx.pointer_hover_pos() {
        Some(p) => p,
        None => return,
    };

    // Find which leaf the cursor is over.
    let target_leaf = tree.leaf_indices().find(|&idx| {
        if let Node::Leaf { rect, .. } = tree.node(idx) {
            rect.contains(cursor_pos)
        } else {
            false
        }
    });

    if let Some(target_leaf) = target_leaf {
        let root_rect = match tree.node(0) {
            Node::Split { rect, .. } => *rect,
            Node::Leaf { rect, .. } => *rect,
            Node::Empty => return,
        };
        let pane_rect = match tree.node(target_leaf) {
            Node::Leaf { rect, .. } => *rect,
            _ => return,
        };
        let Some(options) = tree.pane_options(target_leaf) else {
            return;
        };

        // Cursor in the header bar → always merge (Center).  Without this,
        // a cursor sitting on a tab button lands near the top of the pane
        // rect and gets classified as DropZone::Top, causing an unintended
        // split when the user just wanted to drop onto that pane's tab bar.
        let header_visible = tree.header_visible(target_leaf);
        let header_rect = egui::Rect::from_min_size(
            pane_rect.min,
            egui::vec2(pane_rect.width(), style.header_height),
        );
        let zone = if header_visible && header_rect.contains(cursor_pos) {
            DropZone::Center
        } else {
            // Classify relative to the content area only (below the header).
            let header_height = if header_visible {
                style.header_height
            } else {
                0.0
            };
            let content_rect = egui::Rect::from_min_max(
                egui::pos2(pane_rect.min.x, pane_rect.min.y + header_height),
                pane_rect.max,
            );
            DropZone::classify(cursor_pos, content_rect)
        };
        let Some(target_pane) = tree.pane_id_at(target_leaf) else {
            return;
        };
        let target_role = match tree.node(target_leaf) {
            Node::Leaf { options, .. } => options.role,
            _ => None,
        };
        let dragged_tab = tree
            .pane_index(live_payload.src_pane)
            .and_then(|src_leaf| match &tree.nodes[src_leaf] {
                Node::Leaf { tabs, .. } => tabs.get(live_payload.tab_pos),
                _ => None,
            });
        if !dragged_tab
            .map(|tab| tab_allows_target_pane(tab, target_pane, target_role))
            .unwrap_or(false)
        {
            return;
        }

        let mut zone_allowed = drop_zone_allowed(options, zone);
        let root_zone = root_edge_drop_zone(cursor_pos, root_rect)
            .filter(|&root_zone| drop_zone_allowed(options, root_zone));
        let effective_zone = root_zone.unwrap_or(zone);
        zone_allowed = root_zone.is_some() || zone_allowed;
        if !zone_allowed {
            return;
        }
        let reorder_preview = if target_pane == live_payload.src_pane
            && effective_zone == DropZone::Center
            && header_visible
            && header_rect.contains(cursor_pos)
            && !options.lock_layout
            && options.allow_tab_reorder
        {
            header_tab_drop_index(tree, target_leaf, header_rect, cursor_pos)
                .and_then(|target_pos| header_tab_rect(tree, target_leaf, header_rect, target_pos))
        } else {
            None
        };
        let preview = if let Some(reorder_rect) = reorder_preview {
            reorder_rect
        } else if root_zone.is_some() {
            effective_zone.preview_rect(root_rect)
        } else {
            effective_zone.preview_rect(pane_rect)
        };

        // Draw translucent overlay on a top-level layer.
        let overlay_painter = ctx.layer_painter(LayerId::new(
            Order::Tooltip,
            egui::Id::new("grimdock_dnd_overlay"),
        ));
        overlay_painter.rect_filled(preview, style.pane_rounding, style.overlay.fill);
        overlay_painter.rect_stroke(
            preview,
            style.pane_rounding,
            style.overlay.stroke,
            egui::StrokeKind::Inside,
        );

        // On release, apply the drop.
        let released = ctx.input(|i| i.pointer.any_released());
        if released {
            // Take the payload — this clears the drag state.
            if let Some(payload) = DragAndDrop::take_payload::<DragPayload>(&ctx) {
                let Some(src_leaf) = tree.pane_index(payload.src_pane) else {
                    return;
                };
                let tab_pos = payload.tab_pos;

                let draggable = match &tree.nodes[src_leaf] {
                    Node::Leaf { tabs, options, .. } => tabs
                        .get(tab_pos)
                        .map(|tab| {
                            tab.draggable
                                && tab_allows_target_pane(tab, target_pane, target_role)
                                && !options.lock_layout
                                && (options.allow_tab_drag_out || options.allow_tab_reorder)
                        }),
                    _ => None,
                };
                if draggable != Some(true) {
                    return;
                }

                // Retrieve the tab before mutating the tree.
                let tab = match &tree.nodes[src_leaf] {
                    Node::Leaf { tabs, .. } => tabs.get(tab_pos).cloned(),
                    _ => None,
                };

                if let Some(tab) = tab {
                    match effective_zone {
                        DropZone::Center => {
                            if src_leaf == target_leaf {
                                if header_visible && header_rect.contains(cursor_pos) && options.allow_tab_reorder {
                                    if let Some(target_pos) =
                                        header_tab_drop_index(tree, target_leaf, header_rect, cursor_pos)
                                    {
                                        let _ = tree.move_tab_within_leaf(target_leaf, tab_pos, target_pos);
                                    }
                                }
                            } else {
                                tree.merge_tab_into(src_leaf, tab_pos, target_leaf);
                                tree.collapse_empty_leaf(src_leaf);
                            }
                        }
                        DropZone::Left => {
                            if root_zone.is_some() {
                                apply_root_directional_drop(
                                    tree,
                                    src_leaf,
                                    tab_pos,
                                    tab,
                                    SplitDir::Horizontal,
                                    ChildSide::First,
                                );
                            } else {
                                apply_directional_drop(
                                    tree,
                                    src_leaf,
                                    tab_pos,
                                    target_leaf,
                                    tab,
                                    SplitDir::Horizontal,
                                    ChildSide::First,
                                );
                            }
                        }
                        DropZone::Right => {
                            if root_zone.is_some() {
                                apply_root_directional_drop(
                                    tree,
                                    src_leaf,
                                    tab_pos,
                                    tab,
                                    SplitDir::Horizontal,
                                    ChildSide::Second,
                                );
                            } else {
                                apply_directional_drop(
                                    tree,
                                    src_leaf,
                                    tab_pos,
                                    target_leaf,
                                    tab,
                                    SplitDir::Horizontal,
                                    ChildSide::Second,
                                );
                            }
                        }
                        DropZone::Top => {
                            if root_zone.is_some() {
                                apply_root_directional_drop(
                                    tree,
                                    src_leaf,
                                    tab_pos,
                                    tab,
                                    SplitDir::Vertical,
                                    ChildSide::First,
                                );
                            } else {
                                apply_directional_drop(
                                    tree,
                                    src_leaf,
                                    tab_pos,
                                    target_leaf,
                                    tab,
                                    SplitDir::Vertical,
                                    ChildSide::First,
                                );
                            }
                        }
                        DropZone::Bottom => {
                            if root_zone.is_some() {
                                apply_root_directional_drop(
                                    tree,
                                    src_leaf,
                                    tab_pos,
                                    tab,
                                    SplitDir::Vertical,
                                    ChildSide::Second,
                                );
                            } else {
                                apply_directional_drop(
                                    tree,
                                    src_leaf,
                                    tab_pos,
                                    target_leaf,
                                    tab,
                                    SplitDir::Vertical,
                                    ChildSide::Second,
                                );
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Remove the tab from its source and split the target pane to host it.
///
/// **Operation order**: split target first (no indices change), then remove
/// from source, then collapse if source is empty.  Doing it in this order
/// keeps `src_leaf` at a stable index throughout — collapsing before
/// splitting would shift the target's index in any layout where target is
/// inside the subtree that gets promoted.
fn apply_directional_drop<T: Clone + 'static>(
    tree: &mut PanelTree<T>,
    src_leaf: usize,
    tab_pos: usize,
    target_leaf: usize,
    tab: crate::tab::Tab<T>,
    dir: SplitDir,
    side: ChildSide,
) {
    if src_leaf == target_leaf {
        // Dragging a tab onto the edge of its own pane.
        // split_leaf keeps the existing tabs in one child and places `tab` in
        // the other.  We need to remove the original occurrence of the tab
        // from the "existing-tabs" child afterwards.
        tree.split_leaf(target_leaf, dir, tab, side);

        // After the split, src_leaf is now a Split node.  The child that
        // received the original tabs is the opposite of `side`.
        let old_child = match side {
            ChildSide::First => PanelTree::<T>::right_child(target_leaf),
            ChildSide::Second => PanelTree::<T>::left_child(target_leaf),
        };
        if let Node::Leaf { tabs, focused, .. } = &mut tree.nodes[old_child] {
            if tab_pos < tabs.len() {
                tabs.remove(tab_pos);
                if !tabs.is_empty() && *focused >= tabs.len() {
                    *focused = tabs.len() - 1;
                }
            }
        }
        // If the old child is now empty (sole-tab pane split onto its own
        // edge), collapse it so the layout stays consistent.
        tree.collapse_empty_leaf(old_child);
        return;
    }

    // Step 1: split the target.  `split_leaf` only adds child nodes — it
    // never moves any existing nodes — so `src_leaf` keeps its index.
    tree.split_leaf(target_leaf, dir, tab, side);

    // Step 2: remove the tab from the source.
    match &mut tree.nodes[src_leaf] {
        Node::Leaf { tabs, focused, .. } => {
            if tab_pos < tabs.len() {
                tabs.remove(tab_pos);
                if !tabs.is_empty() && *focused >= tabs.len() {
                    *focused = tabs.len() - 1;
                }
            }
        }
        _ => return,
    }

    // Step 3: collapse the source if it is now empty.
    let src_empty = matches!(&tree.nodes[src_leaf], Node::Leaf { tabs, .. } if tabs.is_empty());
    if src_empty {
        tree.collapse_empty_leaf(src_leaf);
    }
}

fn apply_root_directional_drop<T: Clone + 'static>(
    tree: &mut PanelTree<T>,
    src_leaf: usize,
    tab_pos: usize,
    tab: crate::tab::Tab<T>,
    dir: SplitDir,
    side: ChildSide,
) {
    tree.wrap_root_with_split(dir, tab, side);

    // After wrap_root_with_split the old subtree (rooted at old index 0) is
    // placed at `existing_child` in the new tree.  copy_subtree_with_offset
    // maps old index `i` → `existing_child * 2^depth(i) + i`, where
    // `depth(i) = floor(log2(i + 1))`.  The previously used formulas
    // `2*src_leaf+2` / `2*src_leaf+1` were only correct for the shallowest
    // nodes and caused an out-of-bounds panic for deeper layouts.
    let existing_child = match side {
        ChildSide::First => PanelTree::<T>::right_child(0),
        ChildSide::Second => PanelTree::<T>::left_child(0),
    };
    let depth = usize::BITS - (src_leaf + 1).leading_zeros() - 1;
    let shifted_src_leaf = existing_child * (1 << depth) + src_leaf;

    match &mut tree.nodes[shifted_src_leaf] {
        Node::Leaf { tabs, focused, .. } => {
            if tab_pos < tabs.len() {
                tabs.remove(tab_pos);
                if !tabs.is_empty() && *focused >= tabs.len() {
                    *focused = tabs.len() - 1;
                }
            }
        }
        _ => return,
    }

    let src_empty =
        matches!(&tree.nodes[shifted_src_leaf], Node::Leaf { tabs, .. } if tabs.is_empty());
    if src_empty {
        tree.collapse_empty_leaf(shifted_src_leaf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DropPolicy, HeaderVisibility, PanelTree, Tab, TabDropPolicy};

    #[test]
    fn root_edge_drop_zone_prefers_nearest_edge() {
        let root = Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(200.0, 100.0));

        assert_eq!(
            root_edge_drop_zone(egui::pos2(4.0, 50.0), root),
            Some(DropZone::Left)
        );
        assert_eq!(
            root_edge_drop_zone(egui::pos2(196.0, 50.0), root),
            Some(DropZone::Right)
        );
        assert_eq!(
            root_edge_drop_zone(egui::pos2(100.0, 4.0), root),
            Some(DropZone::Top)
        );
        assert_eq!(
            root_edge_drop_zone(egui::pos2(100.0, 96.0), root),
            Some(DropZone::Bottom)
        );
        assert_eq!(root_edge_drop_zone(egui::pos2(100.0, 50.0), root), None);
    }

    #[test]
    fn drop_zone_allowed_respects_non_droppable_and_locked_panes() {
        let mut options = crate::tree::PaneOptions::default();
        options.drop_policy = DropPolicy::none();
        assert!(!drop_zone_allowed(options, DropZone::Center));
        assert!(!drop_zone_allowed(options, DropZone::Left));

        options.drop_policy = DropPolicy::all();
        options.lock_layout = true;
        assert!(!drop_zone_allowed(options, DropZone::Center));
        assert!(!drop_zone_allowed(options, DropZone::Right));
    }

    #[test]
    fn apply_root_directional_drop_wraps_root_without_losing_tabs() {
        let mut tree = PanelTree::new(vec![Tab::new("A", "a")]);
        tree.insert_tab_into_leaf(0, Tab::new("B", "b"));

        let tab = match tree.node(0) {
            Node::Leaf { tabs, .. } => tabs[0].clone(),
            _ => panic!("expected root leaf"),
        };

        apply_root_directional_drop(
            &mut tree,
            0,
            0,
            tab,
            SplitDir::Horizontal,
            ChildSide::First,
        );

        assert!(matches!(tree.node(0), Node::Split { .. }));
        assert!(tree.find_tab(&"a").is_some());
        assert!(tree.find_tab(&"b").is_some());
    }

    /// Regression: dragging a tab from a deep leaf (depth ≥ 2) onto the root
    /// edge used to compute the shifted_src_leaf index as `right_child(src)`
    /// / `left_child(src)`, which gives the wrong node and panics with
    /// "index out of bounds" once the tree is tall enough.
    #[test]
    fn apply_root_directional_drop_deep_layout_no_panic() {
        // Build a tree: root → [A | [B | C]]
        // After two split_leaf calls the layout is:
        //   0 = Split
        //   1 = Leaf [A]        (left child of root)
        //   2 = Split
        //   5 = Leaf [B]        (left child of node 2)
        //   6 = Leaf [C]        (right child of node 2)  ← src_leaf = 6
        let mut tree = PanelTree::new(vec![Tab::new("A", "a")]);
        tree.split_leaf(0, SplitDir::Horizontal, Tab::new("B", "b"), ChildSide::Second);
        // node 0 = Split, node 1 = Leaf [A], node 2 = Leaf [B]
        tree.split_leaf(2, SplitDir::Horizontal, Tab::new("C", "c"), ChildSide::Second);
        // node 0 = Split, node 1 = Leaf [A], node 2 = Split, node 5 = Leaf [B], node 6 = Leaf [C]

        let src_leaf = 6;
        let tab = match tree.node(src_leaf) {
            Node::Leaf { tabs, .. } => tabs[0].clone(),
            _ => panic!("expected leaf at index {src_leaf}"),
        };

        // This used to panic with "index out of bounds" because shifted_src_leaf
        // was computed as left_child(6) = 13 but the tree only had 11 nodes.
        apply_root_directional_drop(
            &mut tree,
            src_leaf,
            0,
            tab,
            SplitDir::Horizontal,
            ChildSide::Second,
        );

        assert!(matches!(tree.node(0), Node::Split { .. }));
        assert!(tree.find_tab(&"a").is_some());
        assert!(tree.find_tab(&"b").is_some());
        // "c" was the dragged tab so it should be in the new root-edge pane
        assert!(tree.find_tab(&"c").is_some());
    }

    #[test]
    fn header_drop_index_handles_single_hidden_header_layouts() {
        let mut tree = PanelTree::new(vec![Tab::new("A", "a"), Tab::new("B", "b")]);
        let mut options = tree.pane_options(0).unwrap();
        options.header_visibility = HeaderVisibility::WhenMultipleTabs;
        tree.set_pane_options(0, options);

        let rect = Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(100.0, 20.0));
        assert_eq!(
            header_tab_drop_index(&tree, 0, rect, egui::pos2(5.0, 10.0)),
            Some(0)
        );
        assert_eq!(
            header_tab_drop_index(&tree, 0, rect, egui::pos2(95.0, 10.0)),
            Some(1)
        );
    }

    #[test]
    fn tab_drop_policy_blocks_non_whitelisted_target_panes() {
        let mut tree = PanelTree::new(vec![Tab::new("A", "a")]);
        let right_pane = tree.split_leaf(
            0,
            SplitDir::Horizontal,
            Tab::new("B", "b").with_drop_policy(TabDropPolicy {
                locked_to_pane: None,
                locked_to_role: None,
                allowed_panes: Some(vec![PaneId::from_raw(1)]),
                allowed_roles: None,
                blocked_panes: Vec::new(),
                blocked_roles: Vec::new(),
            }),
            ChildSide::Second,
        );

        let right_idx = tree.pane_index(right_pane).expect("right pane should exist");
        let tab = match tree.node(right_idx) {
            Node::Leaf { tabs, .. } => tabs.first().expect("right pane should hold a tab"),
            other => panic!("expected right leaf, got {:?}", other),
        };

        assert!(!tab_allows_target_pane(tab, right_pane, None));
    }

    #[test]
    fn tab_drop_policy_honors_blocked_pane_list() {
        let policy = TabDropPolicy {
            locked_to_pane: None,
            locked_to_role: None,
            allowed_panes: None,
            allowed_roles: None,
            blocked_panes: vec![PaneId::from_raw(7)],
            blocked_roles: Vec::new(),
        };

        assert!(policy.allows_target(PaneId::from_raw(3), None));
        assert!(!policy.allows_target(PaneId::from_raw(7), None));
    }

    #[test]
    fn tab_drop_policy_honors_role_filters() {
        let policy = TabDropPolicy {
            locked_to_pane: None,
            locked_to_role: Some(PaneRole::Terminal),
            allowed_panes: None,
            allowed_roles: None,
            blocked_panes: Vec::new(),
            blocked_roles: Vec::new(),
        };

        assert!(policy.allows_target(PaneId::from_raw(1), Some(PaneRole::Terminal)));
        assert!(!policy.allows_target(PaneId::from_raw(1), Some(PaneRole::Editor)));
    }
}
