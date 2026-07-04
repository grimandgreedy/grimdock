use egui::{Rect, Ui};

use crate::{
    ids::pane_content_id,
    style::PanelStyle,
    tree::{Node, PanelTree},
};

/// Run the content pass: invoke the caller's render callback for each leaf's
/// focused tab inside a clipped child UI.
pub(crate) fn content_pass<T: Clone + 'static>(
    ui: &mut Ui,
    tree: &PanelTree<T>,
    leaf_rects: &[(usize, Rect)],
    style: &PanelStyle,
    mut render: impl FnMut(&mut Ui, &T),
) {
    for &(leaf_idx, content_rect) in leaf_rects {
        if tree.is_collapsed(leaf_idx) {
            continue;
        }
        let (pane_id, tab_id, focused_idx, style_override) = match tree.node(leaf_idx) {
            Node::Leaf {
                pane,
                tabs,
                focused,
                options,
                ..
            } => {
                if let Some(tab) = tabs.get(*focused) {
                    (*pane, tab.id.clone(), *focused, options.style_override)
                } else {
                    continue;
                }
            }
            _ => continue,
        };

        // Use a stable ID per leaf + focused tab so widget state (scroll
        // position, text input, etc.) survives tab switches in other panes.
        let content_id = pane_content_id(pane_id).with(focused_idx);
        // Per-pane inset when set (edge-to-edge viewport), else the global.
        let inset_rect = content_rect.shrink(style.pane_content_inset(style_override));

        ui.push_id(content_id, |ui| {
            ui.scope_builder(egui::UiBuilder::new().max_rect(inset_rect), |ui| {
                ui.set_clip_rect(inset_rect);
                render(ui, &tab_id);
            });
        });
    }
}
