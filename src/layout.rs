use egui::{Rect, Sense, Ui, vec2};

use crate::{
    ids::resize_handle_id,
    style::PanelStyle,
    tree::{Node, PanelTree, SplitDir},
};

/// Run the layout pass over the entire tree.
///
/// Assigns a rectangle to every node starting from the root, renders resize
/// handles for split nodes, and handles ratio updates from handle drags.
///
/// Returns a list of `(leaf_index, content_rect)` pairs for consumption by
/// the header and content passes.
pub(crate) fn layout_pass<T: Clone + 'static>(
    ui: &mut Ui,
    tree: &mut PanelTree<T>,
    style: &PanelStyle,
) -> Vec<(usize, Rect)> {
    let root_rect = ui.available_rect_before_wrap();
    let mut leaf_rects: Vec<(usize, Rect)> = Vec::new();
    layout_node(ui, tree, 0, root_rect, style, &mut leaf_rects);
    leaf_rects
}

fn layout_node<T: Clone + 'static>(
    ui: &mut Ui,
    tree: &mut PanelTree<T>,
    idx: usize,
    rect: Rect,
    style: &PanelStyle,
    leaf_rects: &mut Vec<(usize, Rect)>,
) {
    // Store the rect in the node for DnD hit-testing later.
    match tree.node_mut(idx) {
        Node::Split { rect: r, .. } => *r = rect,
        Node::Leaf { rect: r, .. } => *r = rect,
        Node::Empty => return,
    }

    match tree.node(idx).clone() {
        Node::Split { dir, mut ratio, .. } => {
            let handle_w = style.handle_width;
            let first_idx = PanelTree::<T>::left_child(idx);
            let second_idx = PanelTree::<T>::right_child(idx);
            let first_collapsed = tree.is_collapsed(first_idx);
            let second_collapsed = tree.is_collapsed(second_idx);
            let resize_locked = tree.split_resize_locked(idx);

            let (_first_rect, handle_rect, _second_rect) = split_rect(
                rect,
                dir,
                ratio,
                handle_w,
                first_collapsed,
                second_collapsed,
                style.collapsed_pane_thickness,
            );

            // Draw and interact with the resize handle.
            let handle_id = resize_handle_id(idx);
            let handle_response = ui.interact(
                handle_rect,
                handle_id,
                if resize_locked {
                    Sense::hover()
                } else {
                    Sense::drag()
                },
            );

            let is_active = handle_response.dragged() || handle_response.hovered();
            let color = if resize_locked {
                style.handle.locked_color
            } else if is_active {
                style.handle.hover_color
            } else {
                style.handle.color
            };
            ui.painter().rect_filled(handle_rect, 0.0, color);

            if handle_response.dragged()
                && !resize_locked
                && !first_collapsed
                && !second_collapsed
            {
                let delta = handle_response.drag_delta();
                let delta_ratio = match dir {
                    SplitDir::Horizontal => delta.x / rect.width().max(1.0),
                    SplitDir::Vertical => delta.y / rect.height().max(1.0),
                };

                let min_ratio = style.min_pane_size
                    / match dir {
                        SplitDir::Horizontal => rect.width().max(1.0),
                        SplitDir::Vertical => rect.height().max(1.0),
                    };
                let max_ratio = 1.0 - min_ratio;
                ratio = (ratio + delta_ratio).clamp(min_ratio.max(0.0), max_ratio.min(1.0));

                if let Node::Split { ratio: r, .. } = tree.node_mut(idx) {
                    *r = ratio;
                }
            }

            // Recalculate rects with potentially updated ratio.
            let (first_rect, _, second_rect) = split_rect(
                rect,
                dir,
                ratio,
                handle_w,
                first_collapsed,
                second_collapsed,
                style.collapsed_pane_thickness,
            );

            layout_node(
                ui,
                tree,
                first_idx,
                first_rect,
                style,
                leaf_rects,
            );
            layout_node(
                ui,
                tree,
                second_idx,
                second_rect,
                style,
                leaf_rects,
            );
        }
        Node::Leaf { .. } => {
            let header_height = if tree.header_visible(idx) {
                style.header_height
            } else {
                0.0
            };
            let content_rect = Rect::from_min_size(
                rect.min + vec2(0.0, header_height),
                vec2(rect.width(), (rect.height() - header_height).max(0.0)),
            );
            leaf_rects.push((idx, content_rect));
        }
        Node::Empty => {}
    }
}

/// Split `rect` into two sub-rects with a handle strip between them.
///
/// Returns `(first, handle, second)`.
fn split_rect(
    rect: Rect,
    dir: SplitDir,
    ratio: f32,
    handle_w: f32,
    first_collapsed: bool,
    second_collapsed: bool,
    collapsed_thickness: f32,
) -> (Rect, Rect, Rect) {
    let total_extent = match dir {
        SplitDir::Horizontal => rect.width(),
        SplitDir::Vertical => rect.height(),
    }
    .max(0.0);
    let collapsed_extent = collapsed_thickness.min((total_extent - handle_w).max(0.0));

    let first_extent = if first_collapsed && !second_collapsed {
        collapsed_extent
    } else if second_collapsed && !first_collapsed {
        (total_extent - handle_w - collapsed_extent).max(0.0)
    } else {
        total_extent * ratio
    };

    match dir {
        SplitDir::Horizontal => {
            let split_x = rect.min.x + first_extent;
            let first = Rect::from_min_max(rect.min, egui::pos2(split_x - handle_w / 2.0, rect.max.y));
            let handle = Rect::from_min_max(
                egui::pos2(split_x - handle_w / 2.0, rect.min.y),
                egui::pos2(split_x + handle_w / 2.0, rect.max.y),
            );
            let second = Rect::from_min_max(egui::pos2(split_x + handle_w / 2.0, rect.min.y), rect.max);
            (first, handle, second)
        }
        SplitDir::Vertical => {
            let split_y = rect.min.y + first_extent;
            let first = Rect::from_min_max(rect.min, egui::pos2(rect.max.x, split_y - handle_w / 2.0));
            let handle = Rect::from_min_max(
                egui::pos2(rect.min.x, split_y - handle_w / 2.0),
                egui::pos2(rect.max.x, split_y + handle_w / 2.0),
            );
            let second = Rect::from_min_max(egui::pos2(rect.min.x, split_y + handle_w / 2.0), rect.max);
            (first, handle, second)
        }
    }
}
