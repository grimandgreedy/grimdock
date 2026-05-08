//! Texture-backed tab icon example:
//! `cargo run --example 5_texture_icons`

use eframe::egui;
use grimdock::{PanelContext, PanelStyle, PanelTree, Tab};

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("grimdock 5_texture_icons")
            .with_inner_size([960.0, 560.0]),
        ..Default::default()
    };

    eframe::run_native(
        "grimdock 5_texture_icons",
        options,
        Box::new(|cc| Ok(Box::new(App::new(cc)))),
    )
}

#[derive(Clone, PartialEq, Eq, Debug)]
enum TabId {
    Rust,
    Search,
    Graph,
}

struct App {
    tree: PanelTree<TabId>,
    style: PanelStyle,
    _textures: Vec<egui::TextureHandle>,
}

impl App {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let textures = vec![
            make_icon(&cc.egui_ctx, "rust_icon", egui::Color32::from_rgb(208, 116, 61)),
            make_icon(&cc.egui_ctx, "search_icon", egui::Color32::from_rgb(84, 164, 218)),
            make_icon(&cc.egui_ctx, "graph_icon", egui::Color32::from_rgb(108, 196, 119)),
        ];

        let icon_size = egui::vec2(14.0, 14.0);
        let tree = PanelTree::new(vec![
            Tab::new("rust.rs", TabId::Rust)
                .with_icon_texture(textures[0].id(), icon_size)
                .with_closable(true),
            Tab::new("search", TabId::Search)
                .with_icon_texture(textures[1].id(), icon_size)
                .with_closable(true),
            Tab::new("graph", TabId::Graph)
                .with_icon_texture(textures[2].id(), icon_size)
                .with_closable(true),
        ]);

        Self {
            tree,
            style: PanelStyle::default(),
            _textures: textures,
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ctx, |ui| {
                PanelContext::new(ui, &mut self.tree, &self.style).show(|ui, tab_id| match tab_id {
                    TabId::Rust => {
                        ui.heading("Texture-backed tab icons");
                        ui.label("These tabs use TextureId + size instead of text markers.");
                    }
                    TabId::Search => {
                        ui.heading("Search");
                        ui.label("Use this pattern when your app already has texture handles.");
                    }
                    TabId::Graph => {
                        ui.heading("Graph");
                        ui.label("The example creates simple textures in memory at startup.");
                    }
                });
            });
    }
}

fn make_icon(
    ctx: &egui::Context,
    name: &str,
    accent: egui::Color32,
) -> egui::TextureHandle {
    let size = [16, 16];
    let mut image = egui::ColorImage::filled(size, egui::Color32::TRANSPARENT);

    for y in 2..14 {
        for x in 2..14 {
            let dx = x as i32 - 8;
            let dy = y as i32 - 8;
            if dx * dx + dy * dy <= 24 {
                image[(x, y)] = accent;
            }
        }
    }

    for i in 4..12 {
        image[(i, 8)] = egui::Color32::WHITE;
        image[(8, i)] = egui::Color32::WHITE;
    }

    ctx.load_texture(name.to_owned(), image, egui::TextureOptions::LINEAR)
}
