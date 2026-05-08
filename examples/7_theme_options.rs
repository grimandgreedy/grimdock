//! Theme and colour options example:
//! `cargo run --example 7_theme_options`

use eframe::egui;
use grimdock::{
    ChildSide, PaneId, PaneStyleOverride, PanelContext, PanelStyle, PanelTree, SplitDir, Tab,
    TabStyleOverride,
};

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("grimdock 7_theme_options")
            .with_inner_size([1260.0, 760.0]),
        ..Default::default()
    };

    eframe::run_native(
        "grimdock 7_theme_options",
        options,
        Box::new(|cc| Ok(Box::new(App::new(&cc.egui_ctx)))),
    )
}

#[derive(Clone, PartialEq, Eq, Debug)]
enum TabId {
    Files,
    Editor,
    Search,
    Preview,
    Terminal,
}

struct App {
    tree: PanelTree<TabId>,
    style: PanelStyle,
    terminal_pane: PaneId,
    terminal_override_enabled: bool,
    terminal_override: PaneStyleOverride,
    search_override_enabled: bool,
    search_override: TabStyleOverride,
}

impl App {
    fn new(ctx: &egui::Context) -> Self {
        let (tree, terminal_pane) = make_tree();
        Self {
            tree,
            style: PanelStyle::from_visuals(&ctx.style().visuals),
            terminal_pane,
            terminal_override_enabled: true,
            terminal_override: PaneStyleOverride {
                header_bg: Some(egui::Color32::from_rgb(45, 28, 20)),
                content_bg: Some(egui::Color32::from_rgb(34, 22, 16)),
                border_color: Some(egui::Color32::from_rgb(168, 99, 56)),
                accent_color: Some(egui::Color32::from_rgb(230, 142, 78)),
            },
            search_override_enabled: true,
            search_override: TabStyleOverride {
                active_bg: Some(egui::Color32::from_rgb(30, 74, 66)),
                inactive_bg: Some(egui::Color32::from_rgb(24, 52, 48)),
                hovered_bg: Some(egui::Color32::from_rgb(36, 90, 80)),
                text_color: Some(egui::Color32::from_rgb(214, 245, 238)),
                accent_color: Some(egui::Color32::from_rgb(52, 204, 168)),
                icon_color: None,
            },
        }
    }

    fn sync_overrides(&mut self) {
        if let Some(mut pane) = self.tree.pane_mut(self.terminal_pane) {
            let mut options = pane.options();
            options.style_override = self.terminal_override_enabled.then_some(self.terminal_override);
            pane.set_options(options);
        }

        if let Some(search_pane_id) = self.tree.find_pane_containing(&TabId::Search) {
            if let Some(mut pane) = self.tree.pane_mut(search_pane_id) {
                if let Some(tab) = pane.tabs_mut().iter_mut().find(|tab| tab.id == TabId::Search) {
                    tab.style_override = self.search_override_enabled.then_some(self.search_override);
                }
            }
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.sync_overrides();

        egui::SidePanel::left("theme_controls")
            .resizable(true)
            .default_width(360.0)
            .show(ctx, |ui| {
                ui.heading("Theme workbench");
                ui.label("This example exposes the global PanelStyle plus pane and tab color overrides.");

                ui.horizontal(|ui| {
                    if ui.button("Use egui visuals").clicked() {
                        self.style = PanelStyle::from_visuals(&ctx.style().visuals);
                    }
                    if ui.button("Use crate defaults").clicked() {
                        self.style = PanelStyle::default();
                    }
                });

                ui.separator();

                ui.collapsing("Layout metrics", |ui| {
                    ui.add(egui::Slider::new(&mut self.style.header_height, 20.0..=40.0).text("Header height"));
                    ui.add(
                        egui::Slider::new(&mut self.style.collapsed_pane_thickness, 20.0..=60.0)
                            .text("Collapsed thickness"),
                    );
                    ui.add(egui::Slider::new(&mut self.style.handle_width, 2.0..=12.0).text("Handle width"));
                    ui.add(egui::Slider::new(&mut self.style.min_pane_size, 40.0..=160.0).text("Min pane size"));
                    ui.add(egui::Slider::new(&mut self.style.content_inset, 0.0..=20.0).text("Content inset"));
                    corner_radius_slider(ui, "Pane rounding", &mut self.style.pane_rounding);
                    corner_radius_slider(ui, "Tab rounding", &mut self.style.tabs.rounding);
                    corner_radius_slider(ui, "Header button rounding", &mut self.style.header.button.rounding);
                });

                ui.collapsing("Typography", |ui| {
                    ui.label("Dock chrome text is painted directly by grimdock.");
                    ui.label("Font weight comes from whichever font family your app has loaded.");
                    font_controls(ui, "Tab title", &mut self.style.typography.tab_title_font);
                    font_controls(
                        ui,
                        "Leading text icon",
                        &mut self.style.typography.tab_icon_text_font,
                    );
                });

                ui.collapsing("Header", |ui| {
                    color_row(ui, "Header bg", &mut self.style.header.bg);
                    color_row(ui, "Header border", &mut self.style.header.border_color);
                    color_row(ui, "Button bg", &mut self.style.header.button.bg);
                    color_row(ui, "Button hover bg", &mut self.style.header.button.hover_bg);
                    color_row(ui, "Button stroke", &mut self.style.header.button.stroke_color);
                    color_row(ui, "Button icon", &mut self.style.header.button.icon_color);
                    color_row(ui, "Button hover icon", &mut self.style.header.button.hover_icon_color);
                });

                ui.collapsing("Content", |ui| {
                    color_row(ui, "Content bg", &mut self.style.content.bg);
                    color_row(ui, "Content border", &mut self.style.content.border_color);
                });

                ui.collapsing("Tabs", |ui| {
                    ui.label("Active");
                    color_row(ui, "Active bg", &mut self.style.tabs.active.bg);
                    color_row(ui, "Active text", &mut self.style.tabs.active.text_color);
                    color_row(ui, "Active accent", &mut self.style.tabs.active.accent_color);

                    ui.separator();
                    ui.label("Inactive");
                    color_row(ui, "Inactive bg", &mut self.style.tabs.inactive.bg);
                    color_row(ui, "Inactive text", &mut self.style.tabs.inactive.text_color);
                    color_row(ui, "Inactive accent", &mut self.style.tabs.inactive.accent_color);

                    ui.separator();
                    ui.label("Hovered");
                    color_row(ui, "Hovered bg", &mut self.style.tabs.hovered.bg);
                    color_row(ui, "Hovered text", &mut self.style.tabs.hovered.text_color);
                    color_row(ui, "Hovered accent", &mut self.style.tabs.hovered.accent_color);
                });

                ui.collapsing("Handles and overlay", |ui| {
                    color_row(ui, "Handle", &mut self.style.handle.color);
                    color_row(ui, "Handle hover", &mut self.style.handle.hover_color);
                    color_row(ui, "Handle locked", &mut self.style.handle.locked_color);
                    color_row(ui, "Overlay fill", &mut self.style.overlay.fill);
                    color_row(ui, "Overlay stroke", &mut self.style.overlay.stroke.color);
                    ui.add(
                        egui::Slider::new(&mut self.style.overlay.stroke.width, 0.5..=4.0)
                            .text("Overlay stroke width"),
                    );
                });

                ui.collapsing("Pane override", |ui| {
                    ui.checkbox(&mut self.terminal_override_enabled, "Enable Terminal pane override");
                    color_option_row(ui, "Header bg", &mut self.terminal_override.header_bg);
                    color_option_row(ui, "Content bg", &mut self.terminal_override.content_bg);
                    color_option_row(ui, "Border", &mut self.terminal_override.border_color);
                    color_option_row(ui, "Accent", &mut self.terminal_override.accent_color);
                });

                ui.collapsing("Tab override", |ui| {
                    ui.checkbox(&mut self.search_override_enabled, "Enable Search tab override");
                    color_option_row(ui, "Active bg", &mut self.search_override.active_bg);
                    color_option_row(ui, "Inactive bg", &mut self.search_override.inactive_bg);
                    color_option_row(ui, "Hovered bg", &mut self.search_override.hovered_bg);
                    color_option_row(ui, "Text", &mut self.search_override.text_color);
                    color_option_row(ui, "Accent", &mut self.search_override.accent_color);
                });
            });

        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ctx, |ui| {
                PanelContext::new(ui, &mut self.tree, &self.style).show(|ui, tab_id| match tab_id {
                    TabId::Files => {
                        ui.heading("Files");
                        ui.label("Use this workbench to tune every main color and size in PanelStyle.");
                    }
                    TabId::Editor => {
                        ui.heading("Editor");
                        ui.label("Drag tabs and resize panes while adjusting the theme controls.");
                    }
                    TabId::Search => {
                        ui.heading("Search");
                        ui.label("This tab can demonstrate per-tab color overrides.");
                    }
                    TabId::Preview => {
                        ui.heading("Preview");
                        ui.label("The dock updates live as you change colors, rounding, and handle styles.");
                    }
                    TabId::Terminal => {
                        ui.heading("Terminal");
                        ui.label("This pane can demonstrate per-pane header/content/accent overrides.");
                    }
                });
            });
    }
}

fn color_row(ui: &mut egui::Ui, label: &str, color: &mut egui::Color32) {
    ui.horizontal(|ui| {
        ui.label(label);
        ui.color_edit_button_srgba(color);
    });
}

fn color_option_row(ui: &mut egui::Ui, label: &str, color: &mut Option<egui::Color32>) {
    let mut enabled = color.is_some();
    ui.horizontal(|ui| {
        ui.checkbox(&mut enabled, label);
        if !enabled {
            *color = None;
            return;
        }

        let value = color.get_or_insert(egui::Color32::WHITE);
        ui.color_edit_button_srgba(value);
    });
}

fn corner_radius_slider(ui: &mut egui::Ui, label: &str, radius: &mut egui::CornerRadius) {
    let mut value = radius.nw;
    ui.add(egui::Slider::new(&mut value, 0..=16).text(label));
    *radius = egui::CornerRadius::same(value);
}

fn font_controls(ui: &mut egui::Ui, label: &str, font: &mut egui::FontId) {
    ui.group(|ui| {
        ui.label(label);
        ui.add(egui::Slider::new(&mut font.size, 8.0..=24.0).text("Size"));

        let mut family = match &font.family {
            egui::FontFamily::Proportional => 0,
            egui::FontFamily::Monospace => 1,
            egui::FontFamily::Name(_) => 2,
        };
        egui::ComboBox::from_label("Family")
            .selected_text(match family {
                0 => "Proportional",
                1 => "Monospace",
                _ => "Custom",
            })
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut family, 0, "Proportional");
                ui.selectable_value(&mut family, 1, "Monospace");
                ui.selectable_value(&mut family, 2, "Custom");
            });

        if family == 2 {
            let mut name = match &font.family {
                egui::FontFamily::Name(name) => name.to_string(),
                _ => String::from("bold"),
            };
            ui.text_edit_singleline(&mut name);
            font.family = egui::FontFamily::Name(name.into());
        } else {
            font.family = if family == 0 {
                egui::FontFamily::Proportional
            } else {
                egui::FontFamily::Monospace
            };
        }
    });
}

fn make_tree() -> (PanelTree<TabId>, PaneId) {
    let mut tree = PanelTree::new(vec![
        Tab::new("editor", TabId::Editor).with_leading_visual(">"),
        Tab::new("search", TabId::Search).with_leading_visual("?"),
    ]);

    tree.split_leaf(
        0,
        SplitDir::Horizontal,
        Tab::new("files", TabId::Files).with_leading_visual("#"),
        ChildSide::First,
    );

    let editor_pane = tree
        .find_pane_containing(&TabId::Editor)
        .expect("editor pane should exist");
    let terminal_pane = tree
        .pane_mut(editor_pane)
        .expect("editor pane should exist")
        .split(
            SplitDir::Vertical,
            Tab::new("terminal", TabId::Terminal).with_leading_visual("$"),
            ChildSide::Second,
        );

    if let Some(mut pane) = tree.pane_mut(editor_pane) {
        pane.push_tab(Tab::new("preview", TabId::Preview).with_leading_visual("*"));
    }

    (tree, terminal_pane)
}
