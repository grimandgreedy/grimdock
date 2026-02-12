use egui::{Align2, Popup, Pos2, Rect, Sense, Stroke, StrokeKind, Ui, vec2};

use crate::{
    AddTabEntry, OpenBehavior, PaneActionInvocation, PaneAnchor, PaneMenuAction,
    dnd::DragPayload,
    ids::{tab_button_id, tab_close_button_id},
    style::PanelStyle,
    tab::TabIcon,
    tree::{ChildSide, Node, PaneId, PanelTree, SplitDir},
};

#[derive(Clone)]
enum PendingAction<T: Clone + 'static> {
    Close(usize),
    CloseOthers(usize),
    CloseAll,
    RemovePane,
    MovePane(PaneAnchor),
    AddTab(AddTabEntry<T>),
    SplitWith(AddTabEntry<T>, SplitDir, ChildSide),
    EmitPaneAction(PaneActionInvocation),
}

#[derive(Clone, Copy)]
enum HeaderButtonIcon {
    Add,
    Overflow,
    PaneMenu,
    Collapse { collapsed: bool },
    Close,
}

/// Run the header pass over all leaf nodes.
pub(crate) fn header_pass<T: Clone + 'static>(
    ui: &mut Ui,
    tree: &mut PanelTree<T>,
    leaf_rects: &[(usize, Rect)],
    style: &PanelStyle,
    add_tab_entries: &[AddTabEntry<T>],
    add_tab_provider: Option<&dyn Fn(PaneId, &PanelTree<T>) -> Vec<AddTabEntry<T>>>,
    pane_menu_actions: &[PaneMenuAction],
    pane_menu_provider: Option<&dyn Fn(PaneId, &PanelTree<T>) -> Vec<PaneMenuAction>>,
    closed_tabs: &mut Vec<T>,
    pane_actions: &mut Vec<PaneActionInvocation>,
) where
    T: PartialEq,
{
    for &(leaf_idx, content_rect) in leaf_rects {
        let pane_style_override = match tree.node(leaf_idx) {
            Node::Leaf { options, .. } => options.style_override,
            _ => None,
        };
        let pane_border = style.pane_border_color(pane_style_override);
        let pane_content_bg = style.pane_content_bg(pane_style_override);
        let pane_header_bg = style.pane_header_bg(pane_style_override);

        if !tree.header_visible(leaf_idx) {
            if matches!(tree.node(leaf_idx), Node::Leaf { options, .. } if options.paint_content_bg) {
                ui.painter()
                    .rect_filled(content_rect, style.pane_rounding, pane_content_bg);
            }
            ui.painter().rect_stroke(
                content_rect,
                style.pane_rounding,
                Stroke::new(1.0, pane_border),
                StrokeKind::Inside,
            );
            continue;
        }

        let full_rect = match tree.node(leaf_idx) {
            Node::Leaf { rect, .. } => *rect,
            _ => continue,
        };

        let header_rect = Rect::from_min_size(full_rect.min, vec2(full_rect.width(), style.header_height));

        ui.painter()
            .rect_filled(header_rect, style.pane_rounding, pane_header_bg);

        if matches!(tree.node(leaf_idx), Node::Leaf { options, .. } if options.paint_content_bg) {
            ui.painter()
                .rect_filled(content_rect, style.pane_rounding, pane_content_bg);
        }

        ui.painter().rect_stroke(
            content_rect,
            style.pane_rounding,
            Stroke::new(1.0, pane_border),
            StrokeKind::Inside,
        );

        let pane_entries = if let Some(Node::Leaf { pane, .. }) = Some(tree.node(leaf_idx)) {
            if let Some(provider) = add_tab_provider {
                provider(*pane, tree)
            } else {
                add_tab_entries.to_vec()
            }
        } else {
            add_tab_entries.to_vec()
        };

        let pane_menu_entries = if let Some(Node::Leaf { pane, .. }) = Some(tree.node(leaf_idx)) {
            if let Some(provider) = pane_menu_provider {
                provider(*pane, tree)
            } else {
                pane_menu_actions.to_vec()
            }
        } else {
            pane_menu_actions.to_vec()
        };

        render_tabs(
            ui,
            tree,
            leaf_idx,
            header_rect,
            style,
            &pane_entries,
            &pane_menu_entries,
            closed_tabs,
            pane_actions,
        );
    }
}

fn render_tabs<T: Clone + 'static>(
    ui: &mut Ui,
    tree: &mut PanelTree<T>,
    leaf_idx: usize,
    header_rect: Rect,
    style: &PanelStyle,
    add_tab_entries: &[AddTabEntry<T>],
    pane_menu_actions: &[PaneMenuAction],
    closed_tabs: &mut Vec<T>,
    pane_actions: &mut Vec<PaneActionInvocation>,
) where
    T: PartialEq,
{
    let (pane_id, tab_count, focused, collapse_allowed, collapsed, pane_style_override, lock_layout) =
        match tree.node(leaf_idx) {
        Node::Leaf {
            pane,
            tabs,
            focused,
            options,
            collapsed,
            ..
        } => (
            *pane,
            tabs.len(),
            *focused,
            options.allow_collapse,
            *collapsed,
            options.style_override,
            options.lock_layout,
        ),
        _ => return,
    };

    let action_button_width = 18.0;
    let action_button_padding = 4.0;
    let mut right_edge = header_rect.max.x;

    let collapse_rect = if collapse_allowed {
        let rect = Rect::from_center_size(
            egui::pos2(
                right_edge - action_button_padding - action_button_width * 0.5,
                header_rect.center().y,
            ),
            vec2(action_button_width, action_button_width),
        );
        right_edge = rect.min.x - action_button_padding;
        Some(rect)
    } else {
        None
    };

    let pane_menu_rect = {
        let rect = Rect::from_center_size(
            egui::pos2(
                right_edge - action_button_padding - action_button_width * 0.5,
                header_rect.center().y,
            ),
            vec2(action_button_width, action_button_width),
        );
        right_edge = rect.min.x - action_button_padding;
        rect
    };

    let add_rect = if add_tab_entries.is_empty() {
        None
    } else {
        let rect = Rect::from_center_size(
            egui::pos2(
                right_edge - action_button_padding - action_button_width * 0.5,
                header_rect.center().y,
            ),
            vec2(action_button_width, action_button_width),
        );
        right_edge = rect.min.x - action_button_padding;
        Some(rect)
    };

    let mut tabs_rect = Rect::from_min_max(header_rect.min, egui::pos2(right_edge, header_rect.max.y));

    if collapsed {
        if let Some(collapse_rect) = collapse_rect {
            let collapse_response = ui.interact(
                collapse_rect,
                egui::Id::new("grimdock::tab_collapse").with(pane_id.into_raw()),
                Sense::click(),
            );
            paint_header_button(
                ui,
                collapse_rect,
                &collapse_response,
                style,
                HeaderButtonIcon::Collapse { collapsed: true },
            );
            if collapse_response.clicked() {
                let _ = tree.toggle_collapsed(leaf_idx);
            }
        }
        return;
    }

    let min_tab_width = 72.0;
    let overflow_button_width = action_button_width + action_button_padding * 2.0;
    let (visible_indices, hidden_indices, tab_width) =
        compute_visible_tabs(tab_count, focused, tabs_rect.width(), min_tab_width, overflow_button_width);

    let overflow_rect = if hidden_indices.is_empty() {
        None
    } else {
        let rect = Rect::from_min_max(
            egui::pos2(tabs_rect.max.x - overflow_button_width, tabs_rect.min.y),
            tabs_rect.max,
        );
        tabs_rect.max.x -= overflow_button_width;
        Some(rect)
    };

    let mut pending_action: Option<PendingAction<T>> = None;
    let mut new_focused = focused;
    let padding_x = 8.0;
    let leading_gap = 6.0;
    let close_button_width = 16.0;
    let close_button_padding = 4.0;

    for (slot, &tab_index) in visible_indices.iter().enumerate() {
        let Some((title, icon, draggable, closable, tab_style_override)) = (match tree.node(leaf_idx) {
            Node::Leaf { tabs, options, .. } => tabs.get(tab_index).map(|t| {
                (
                    t.title.clone(),
                    t.icon.clone(),
                    t.draggable
                        && !options.lock_layout
                        && (options.allow_tab_drag_out || options.allow_tab_reorder),
                    t.closable,
                    t.style_override,
                )
            }),
            _ => None,
        }) else {
            continue;
        };

        let tab_rect = Rect::from_min_size(
            egui::pos2(tabs_rect.min.x + slot as f32 * tab_width, tabs_rect.min.y),
            vec2(tab_width, style.header_height),
        );
        let painted_tab_rect = paint_tab_rect(tab_rect, slot, visible_indices.len());
        let is_focused = tab_index == focused;
        let hovered = ui.rect_contains_pointer(tab_rect);
        let tab_state = style.tab_state(is_focused, hovered, pane_style_override, tab_style_override);

        ui.painter()
            .rect_filled(painted_tab_rect, tab_rounding(style), tab_state.bg);

        let close_rect = Rect::from_center_size(
            egui::pos2(
                tab_rect.max.x - close_button_padding - close_button_width * 0.5,
                tab_rect.center().y,
            ),
            vec2(close_button_width, close_button_width),
        );

        if is_focused {
            let accent_rect = Rect::from_min_max(
                egui::pos2(painted_tab_rect.min.x + 2.0, painted_tab_rect.max.y - 2.0),
                egui::pos2(painted_tab_rect.max.x - 2.0, painted_tab_rect.max.y),
            );
            ui.painter().rect_filled(accent_rect, 0.0, tab_state.accent_color);
        }

        let mut text_x = tab_rect.min.x + padding_x;
        if let Some(icon) = &icon {
            let icon_tint = tab_style_override
                .and_then(|o| o.icon_color)
                .unwrap_or(tab_state.text_color);
            let icon_width = paint_tab_leading_icon(
                ui,
                icon,
                text_x,
                tab_rect,
                icon_tint,
                style,
            );
            text_x += icon_width + leading_gap;
        }

        let text_max_x = if closable {
            close_rect.min.x - padding_x
        } else {
            tab_rect.max.x - padding_x
        };
        let text_pos = egui::pos2(text_x, tab_rect.center().y);
        ui.painter().text(
            text_pos,
            Align2::LEFT_CENTER,
            &title,
            style.typography.tab_title_font.clone(),
            tab_state.text_color,
        );

        let mut response = ui.interact(tab_rect, tab_button_id(pane_id, tab_index), Sense::click_and_drag());
        if title_is_truncated(ui, &title, (text_max_x - text_pos.x).max(0.0), style) {
            response = response.on_hover_text(title.clone());
        }

        if response.clicked() {
            new_focused = tab_index;
        }
        if draggable && response.drag_started() {
            response.dnd_set_drag_payload(DragPayload {
                src_pane: pane_id,
                tab_pos: tab_index,
            });
        }

        response.context_menu(|ui| {
            populate_tab_context_menu(ui, add_tab_entries, closable, tab_index, &mut pending_action);
        });

        if closable {
            let close_response =
                ui.interact(close_rect, tab_close_button_id(pane_id, tab_index), Sense::click());
            let close_color = if close_response.hovered() {
                tab_state.accent_color
            } else {
                tab_state.text_color
            };
            paint_icon(ui, close_rect, HeaderButtonIcon::Close, close_color);
            if close_response.clicked() {
                pending_action = Some(PendingAction::Close(tab_index));
            }
        }
    }

    if let Some(overflow_rect) = overflow_rect {
        let overflow_response = header_menu_button(
            ui,
            overflow_rect,
            egui::Id::new("grimdock::overflow").with(pane_id.into_raw()),
            style,
            HeaderButtonIcon::Overflow,
        );
        Popup::menu(&overflow_response).show(|ui| {
            for &tab_index in &hidden_indices {
                let Some(label) = (match tree.node(leaf_idx) {
                    Node::Leaf { tabs, .. } => tabs.get(tab_index).map(|tab| {
                        overflow_tab_label(tab)
                    }),
                    _ => None,
                }) else {
                    continue;
                };
                if ui.selectable_label(tab_index == focused, label).clicked() {
                    new_focused = tab_index;
                    ui.close();
                }
            }
        });
    }

    if let Some(add_rect) = add_rect {
        let add_response = header_menu_button(
            ui,
            add_rect,
            egui::Id::new("grimdock::add_tab").with(pane_id.into_raw()),
            style,
            HeaderButtonIcon::Add,
        );
        Popup::menu(&add_response).show(|ui| {
            for entry in add_tab_entries {
                if ui.button(&entry.title).clicked() {
                    pending_action = Some(PendingAction::AddTab(entry.clone()));
                    ui.close();
                }
            }
        });
    }

    {
        let pane_response = header_menu_button(
            ui,
            pane_menu_rect,
            egui::Id::new("grimdock::pane_menu").with(pane_id.into_raw()),
            style,
            HeaderButtonIcon::PaneMenu,
        );
        Popup::menu(&pane_response).show(|ui| {
            if ui.button("Close all").clicked() {
                pending_action = Some(PendingAction::CloseAll);
                ui.close();
            }
            if ui
                .add_enabled(tree.can_remove_pane(pane_id), egui::Button::new("Remove pane"))
                .clicked()
            {
                pending_action = Some(PendingAction::RemovePane);
                ui.close();
            }
            if !lock_layout {
                ui.separator();
                populate_move_pane_menu(ui, &mut pending_action);
            }
            if !pane_menu_actions.is_empty() {
                ui.separator();
                for action in pane_menu_actions {
                    if ui.button(&action.title).clicked() {
                        pending_action = Some(PendingAction::EmitPaneAction(PaneActionInvocation {
                            pane_id,
                            action_id: action.id.clone(),
                        }));
                        ui.close();
                    }
                }
            }
            if !add_tab_entries.is_empty() {
                ui.separator();
                populate_split_menu(ui, add_tab_entries, &mut pending_action);
            }
        });
    }

    if let Some(collapse_rect) = collapse_rect {
        let collapse_response = ui.interact(
            collapse_rect,
            egui::Id::new("grimdock::tab_collapse").with(pane_id.into_raw()),
            Sense::click(),
        );
        paint_header_button(
            ui,
            collapse_rect,
            &collapse_response,
            style,
            HeaderButtonIcon::Collapse { collapsed: false },
        );
        if collapse_response.clicked() {
            let _ = tree.toggle_collapsed(leaf_idx);
        }
    }

    if new_focused != focused {
        if let Node::Leaf { focused: f, .. } = tree.node_mut(leaf_idx) {
            *f = new_focused;
        }
    }

    if let Some(action) = pending_action {
        apply_pending_action(tree, leaf_idx, action, closed_tabs, pane_actions);
    }
}

fn populate_tab_context_menu<T: Clone + 'static>(
    ui: &mut Ui,
    add_tab_entries: &[AddTabEntry<T>],
    closable: bool,
    tab_index: usize,
    pending_action: &mut Option<PendingAction<T>>,
) where
    T: PartialEq,
{
    if closable && ui.button("Close").clicked() {
        *pending_action = Some(PendingAction::Close(tab_index));
        ui.close();
    }
    if ui.button("Close others").clicked() {
        *pending_action = Some(PendingAction::CloseOthers(tab_index));
        ui.close();
    }
    if ui.button("Close all").clicked() {
        *pending_action = Some(PendingAction::CloseAll);
        ui.close();
    }
    if !add_tab_entries.is_empty() {
        ui.separator();
        populate_split_menu(ui, add_tab_entries, pending_action);
    }
}

fn populate_split_menu<T: Clone + 'static>(
    ui: &mut Ui,
    add_tab_entries: &[AddTabEntry<T>],
    pending_action: &mut Option<PendingAction<T>>,
) where
    T: PartialEq,
{
    for (label, dir, side) in [
        ("Split left", SplitDir::Horizontal, ChildSide::First),
        ("Split right", SplitDir::Horizontal, ChildSide::Second),
        ("Split top", SplitDir::Vertical, ChildSide::First),
        ("Split bottom", SplitDir::Vertical, ChildSide::Second),
    ] {
        ui.menu_button(label, |ui| {
            for entry in add_tab_entries {
                if ui.button(&entry.title).clicked() {
                    *pending_action = Some(PendingAction::SplitWith(entry.clone(), dir, side));
                    ui.close();
                }
            }
        });
    }
}

fn populate_move_pane_menu<T: Clone + 'static>(
    ui: &mut Ui,
    pending_action: &mut Option<PendingAction<T>>,
) {
    for (label, anchor) in [
        ("Move pane left", PaneAnchor::Left),
        ("Move pane right", PaneAnchor::Right),
        ("Move pane top", PaneAnchor::Top),
        ("Move pane bottom", PaneAnchor::Bottom),
        ("Move pane center", PaneAnchor::Center),
    ] {
        if ui.button(label).clicked() {
            *pending_action = Some(PendingAction::MovePane(anchor));
            ui.close();
        }
    }
}

fn apply_open_entry_to_leaf<T: Clone + 'static>(
    tree: &mut PanelTree<T>,
    leaf_idx: usize,
    entry: AddTabEntry<T>,
) where
    T: PartialEq,
{
    let AddTabEntry {
        tab,
        open_behavior,
        ..
    } = entry;

    match open_behavior {
        OpenBehavior::AllowDuplicate => tree.insert_tab_into_leaf(leaf_idx, tab),
        OpenBehavior::FocusExisting => {
            if !tree.focus_tab(&tab.id) {
                tree.insert_tab_into_leaf(leaf_idx, tab);
            }
        }
        OpenBehavior::FocusExistingInPane => {
            let exists_in_target = tree
                .find_tab(&tab.id)
                .map(|(existing_leaf, tab_pos)| {
                    if existing_leaf == leaf_idx {
                        if let Node::Leaf { focused, .. } = tree.node_mut(leaf_idx) {
                            *focused = tab_pos;
                        }
                        true
                    } else {
                        false
                    }
                })
                .unwrap_or(false);
            if !exists_in_target {
                tree.insert_tab_into_leaf(leaf_idx, tab);
            }
        }
    }
}

fn apply_pending_action<T: Clone + 'static>(
    tree: &mut PanelTree<T>,
    leaf_idx: usize,
    action: PendingAction<T>,
    closed_tabs: &mut Vec<T>,
    pane_actions: &mut Vec<PaneActionInvocation>,
) where
    T: PartialEq,
{
    match action {
        PendingAction::Close(tab_pos) => {
            if let Some(tab) = tree.remove_tab_at(leaf_idx, tab_pos) {
                closed_tabs.push(tab.id);
            }
        }
        PendingAction::CloseOthers(keep_pos) => {
            closed_tabs.extend(tree.close_other_tabs_in_leaf(leaf_idx, keep_pos));
        }
        PendingAction::CloseAll => {
            closed_tabs.extend(tree.close_all_tabs_in_leaf(leaf_idx));
        }
        PendingAction::RemovePane => {
            if let Some(pane_id) = tree.pane_id_at(leaf_idx) {
                if let Some(removed) = tree.remove_pane(pane_id) {
                    closed_tabs.extend(removed);
                }
            }
        }
        PendingAction::MovePane(anchor) => {
            if let Some(pane_id) = tree.pane_id_at(leaf_idx) {
                let _ = tree.move_pane_to_anchor(pane_id, anchor);
            }
        }
        PendingAction::AddTab(entry) => {
            apply_open_entry_to_leaf(tree, leaf_idx, entry);
        }
        PendingAction::SplitWith(entry, dir, side) => {
            let AddTabEntry {
                title: _,
                tab,
                open_behavior,
            } = entry;

            match open_behavior {
                OpenBehavior::AllowDuplicate => {
                    tree.split_leaf(leaf_idx, dir, tab, side);
                }
                OpenBehavior::FocusExisting => {
                    if !tree.focus_tab(&tab.id) {
                        tree.split_leaf(leaf_idx, dir, tab, side);
                    }
                }
                OpenBehavior::FocusExistingInPane => {
                    let exists_in_target = tree
                        .find_tab(&tab.id)
                        .map(|(existing_leaf, tab_pos)| {
                            if existing_leaf == leaf_idx {
                                if let Node::Leaf { focused, .. } = tree.node_mut(leaf_idx) {
                                    *focused = tab_pos;
                                }
                                true
                            } else {
                                false
                            }
                        })
                        .unwrap_or(false);
                    if !exists_in_target {
                        tree.split_leaf(leaf_idx, dir, tab, side);
                    }
                }
            }
        }
        PendingAction::EmitPaneAction(action) => {
            pane_actions.push(action);
        }
    }
}

fn compute_visible_tabs(
    tab_count: usize,
    focused: usize,
    available_width: f32,
    min_tab_width: f32,
    overflow_width: f32,
) -> (Vec<usize>, Vec<usize>, f32) {
    if tab_count == 0 {
        return (Vec::new(), Vec::new(), min_tab_width);
    }

    let mut needs_overflow = false;
    let mut visible_count;
    loop {
        let width = (available_width - if needs_overflow { overflow_width } else { 0.0 }).max(0.0);
        visible_count = ((width / min_tab_width).floor() as usize).max(1).min(tab_count);
        let next = visible_count < tab_count;
        if next == needs_overflow {
            break;
        }
        needs_overflow = next;
    }

    let mut visible: Vec<usize> = (0..visible_count).collect();
    if focused >= visible_count && !visible.is_empty() {
        let last = visible.len() - 1;
        visible[last] = focused;
        visible.sort_unstable();
        visible.dedup();
        while visible.len() < visible_count {
            let candidate = (0..tab_count).find(|idx| !visible.contains(idx)).unwrap_or(0);
            visible.push(candidate);
            visible.sort_unstable();
        }
    }

    let hidden: Vec<usize> = (0..tab_count).filter(|idx| !visible.contains(idx)).collect();
    let visible_width =
        (available_width - if hidden.is_empty() { 0.0 } else { overflow_width }).max(0.0);
    let tab_width = (visible_width / visible.len().max(1) as f32).min(120.0).max(40.0);
    (visible, hidden, tab_width)
}

fn header_menu_button(
    ui: &mut Ui,
    rect: Rect,
    id: egui::Id,
    style: &PanelStyle,
    icon: HeaderButtonIcon,
) -> egui::Response {
    let response = ui.interact(rect, id, Sense::click());
    paint_header_button(ui, rect, &response, style, icon);
    response
}

fn paint_header_button(
    ui: &Ui,
    rect: Rect,
    response: &egui::Response,
    style: &PanelStyle,
    icon: HeaderButtonIcon,
) {
    let bg = if response.hovered() {
        style.header.button.hover_bg
    } else {
        style.header.button.bg
    };
    let fg = if response.hovered() {
        style.header.button.hover_icon_color
    } else {
        style.header.button.icon_color
    };
    ui.painter()
        .rect_filled(rect, style.header.button.rounding, bg);
    ui.painter()
        .rect_stroke(
            rect,
            style.header.button.rounding,
            Stroke::new(1.0, style.header.button.stroke_color),
            StrokeKind::Inside,
        );
    paint_icon(ui, rect, icon, fg);
}

fn paint_icon(ui: &Ui, rect: Rect, icon: HeaderButtonIcon, color: egui::Color32) {
    match icon {
        HeaderButtonIcon::Add => {
            let c = rect.center();
            let r = rect.width().min(rect.height()) * 0.22;
            let stroke = Stroke::new(1.5, color);
            ui.painter().line_segment([egui::pos2(c.x - r, c.y), egui::pos2(c.x + r, c.y)], stroke);
            ui.painter().line_segment([egui::pos2(c.x, c.y - r), egui::pos2(c.x, c.y + r)], stroke);
        }
        HeaderButtonIcon::Overflow | HeaderButtonIcon::PaneMenu => {
            let c = rect.center();
            let spacing = rect.width().min(rect.height()) * 0.18;
            let radius = 1.5;
            for offset in [-spacing, 0.0, spacing] {
                ui.painter()
                    .circle_filled(egui::pos2(c.x + offset, c.y), radius, color);
            }
        }
        HeaderButtonIcon::Collapse { collapsed } => {
            let c = rect.center();
            let s = rect.width().min(rect.height()) * 0.22;
            let points: [Pos2; 3] = if collapsed {
                [
                    egui::pos2(c.x - s * 0.5, c.y - s),
                    egui::pos2(c.x + s * 0.9, c.y),
                    egui::pos2(c.x - s * 0.5, c.y + s),
                ]
            } else {
                [
                    egui::pos2(c.x - s, c.y - s * 0.5),
                    egui::pos2(c.x, c.y + s * 0.9),
                    egui::pos2(c.x + s, c.y - s * 0.5),
                ]
            };
            let stroke = Stroke::new(1.5, color);
            ui.painter().line_segment([points[0], points[1]], stroke);
            ui.painter().line_segment([points[1], points[2]], stroke);
        }
        HeaderButtonIcon::Close => {
            let c = rect.center();
            let s = rect.width().min(rect.height()) * 0.22;
            let stroke = Stroke::new(1.5, color);
            ui.painter().line_segment(
                [egui::pos2(c.x - s, c.y - s), egui::pos2(c.x + s, c.y + s)],
                stroke,
            );
            ui.painter().line_segment(
                [egui::pos2(c.x - s, c.y + s), egui::pos2(c.x + s, c.y - s)],
                stroke,
            );
        }
    }
}

fn paint_tab_leading_icon(
    ui: &Ui,
    icon: &TabIcon,
    min_x: f32,
    tab_rect: Rect,
    tint: egui::Color32,
    style: &PanelStyle,
) -> f32 {
    match icon {
        TabIcon::Text(text) => {
            ui.painter().text(
                egui::pos2(min_x, tab_rect.center().y),
                Align2::LEFT_CENTER,
                text,
                style.typography.tab_icon_text_font.clone(),
                tint,
            );
            ui.painter()
                .layout_no_wrap(
                    text.clone(),
                    style.typography.tab_icon_text_font.clone(),
                    tint,
                )
                .size()
                .x
        }
        TabIcon::Texture { texture_id, size } => {
            let max_side = (tab_rect.height() - 8.0).max(8.0);
            let scale = (max_side / size.y.max(1.0)).min(max_side / size.x.max(1.0));
            let draw_size = egui::vec2(size.x * scale, size.y * scale);
            let icon_rect = Rect::from_min_size(
                egui::pos2(min_x, tab_rect.center().y - draw_size.y * 0.5),
                draw_size,
            );
            ui.painter().image(
                *texture_id,
                icon_rect,
                Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                tint,
            );
            draw_size.x
        }
    }
}

fn overflow_tab_label<T: Clone + 'static>(tab: &crate::tab::Tab<T>) -> String {
    match &tab.icon {
        Some(TabIcon::Text(text)) => format!("{text} {}", tab.title),
        Some(TabIcon::Texture { .. }) => format!("[img] {}", tab.title),
        None => tab.title.clone(),
    }
}

fn title_is_truncated(ui: &Ui, title: &str, available_width: f32, style: &PanelStyle) -> bool {
    if available_width <= 0.0 {
        return true;
    }
    let galley = ui.painter().layout_no_wrap(
        title.to_owned(),
        style.typography.tab_title_font.clone(),
        style.tabs.inactive.text_color,
    );
    galley.size().x > available_width
}

fn paint_tab_rect(tab_rect: Rect, index: usize, tab_count: usize) -> Rect {
    let overlap = 1.0;
    let min_x = if index == 0 { tab_rect.min.x } else { tab_rect.min.x - overlap };
    let max_x = if index + 1 == tab_count {
        tab_rect.max.x
    } else {
        tab_rect.max.x + overlap
    };
    Rect::from_min_max(egui::pos2(min_x, tab_rect.min.y), egui::pos2(max_x, tab_rect.max.y))
}

fn tab_rounding(style: &PanelStyle) -> egui::CornerRadius {
    egui::CornerRadius {
        nw: style.tabs.rounding.nw,
        ne: style.tabs.rounding.ne,
        sw: 0,
        se: 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Tab;

    fn count_tab<T: Clone + PartialEq + 'static>(tree: &PanelTree<T>, id: &T) -> usize {
        tree.leaf_indices()
            .map(|leaf_idx| match tree.node(leaf_idx) {
                Node::Leaf { tabs, .. } => tabs.iter().filter(|tab| &tab.id == id).count(),
                _ => 0,
            })
            .sum()
    }

    #[test]
    fn focus_existing_open_behavior_prevents_duplicates() {
        let mut tree = PanelTree::new(vec![Tab::new("A", "a")]);
        let entry = AddTabEntry::new("A", Tab::new("A", "a"))
            .with_open_behavior(OpenBehavior::FocusExisting);

        apply_open_entry_to_leaf(&mut tree, 0, entry);

        assert_eq!(count_tab(&tree, &"a"), 1);
    }

    #[test]
    fn focus_existing_in_pane_allows_duplicate_in_other_pane() {
        let mut tree = PanelTree::new(vec![Tab::new("A", "a")]);
        tree.split_leaf(0, SplitDir::Horizontal, Tab::new("B", "b"), ChildSide::Second);

        let entry = AddTabEntry::new("A", Tab::new("A", "a"))
            .with_open_behavior(OpenBehavior::FocusExistingInPane);

        apply_open_entry_to_leaf(&mut tree, 2, entry);

        assert_eq!(count_tab(&tree, &"a"), 2);
    }
}
