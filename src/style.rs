use egui::{Color32, CornerRadius, FontFamily, FontId, Stroke, Style, TextStyle, Visuals};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HeaderButtonStyle {
    pub bg: Color32,
    pub hover_bg: Color32,
    pub stroke_color: Color32,
    pub icon_color: Color32,
    pub hover_icon_color: Color32,
    pub rounding: CornerRadius,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HeaderStyle {
    pub bg: Color32,
    pub border_color: Color32,
    pub button: HeaderButtonStyle,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ContentStyle {
    pub bg: Color32,
    pub border_color: Color32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HandleStyle {
    pub color: Color32,
    pub hover_color: Color32,
    pub locked_color: Color32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OverlayStyle {
    pub fill: Color32,
    pub stroke: Stroke,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TabStateStyle {
    pub bg: Color32,
    pub text_color: Color32,
    pub accent_color: Color32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TabStyle {
    pub active: TabStateStyle,
    pub inactive: TabStateStyle,
    pub hovered: TabStateStyle,
    pub rounding: CornerRadius,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TypographyStyle {
    pub tab_title_font: FontId,
    pub tab_icon_text_font: FontId,
}

// `Eq` is intentionally not derived: `content_inset` holds an `Option<f32>`, and
// `f32` is not `Eq`. `PartialEq` still covers equality comparisons callers need.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PaneStyleOverride {
    pub header_bg: Option<Color32>,
    pub content_bg: Option<Color32>,
    pub border_color: Option<Color32>,
    pub accent_color: Option<Color32>,
    /// Overrides [`PanelStyle::content_inset`] for this pane. `None` uses the
    /// global inset. Set to `Some(0.0)` for an edge-to-edge pane (a 3D viewport
    /// whose render and gizmo overlay should fill the content area) while other
    /// panes keep the global padding.
    #[cfg_attr(feature = "serde", serde(default))]
    pub content_inset: Option<f32>,
}

impl PaneStyleOverride {
    pub const fn none() -> Self {
        Self {
            header_bg: None,
            content_bg: None,
            border_color: None,
            accent_color: None,
            content_inset: None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TabStyleOverride {
    pub active_bg: Option<Color32>,
    pub inactive_bg: Option<Color32>,
    pub hovered_bg: Option<Color32>,
    pub text_color: Option<Color32>,
    pub accent_color: Option<Color32>,
    /// Override colour applied only to the leading icon, not the title text.
    pub icon_color: Option<Color32>,
    /// Maximum header width for this tab when there is spare room.
    /// `None` means the tab may stretch to fill the available width.
    pub max_width: Option<f32>,
}

impl TabStyleOverride {
    pub const fn none() -> Self {
        Self {
            active_bg: None,
            inactive_bg: None,
            hovered_bg: None,
            text_color: None,
            accent_color: None,
            icon_color: None,
            max_width: None,
        }
    }
}

/// All visual parameters for the panel layout system.
#[derive(Clone, Debug)]
pub struct PanelStyle {
    pub header_height: f32,
    pub collapsed_pane_thickness: f32,
    pub handle_width: f32,
    pub min_pane_size: f32,
    pub content_inset: f32,
    pub pane_rounding: CornerRadius,
    pub typography: TypographyStyle,
    pub header: HeaderStyle,
    pub content: ContentStyle,
    pub tabs: TabStyle,
    pub handle: HandleStyle,
    pub overlay: OverlayStyle,
}

impl PanelStyle {
    pub fn from_egui_style(style: &Style) -> Self {
        let mut panel_style = Self::from_visuals(&style.visuals);
        let button_font = style
            .text_styles
            .get(&TextStyle::Button)
            .cloned()
            .unwrap_or_else(|| FontId::proportional(12.0));
        let small_font = style
            .text_styles
            .get(&TextStyle::Small)
            .cloned()
            .unwrap_or_else(|| FontId::monospace(11.0));
        panel_style.typography = TypographyStyle {
            tab_title_font: button_font,
            tab_icon_text_font: small_font,
        };
        panel_style
    }

    pub fn from_visuals(visuals: &Visuals) -> Self {
        let widgets = &visuals.widgets;
        let inactive_fill = widgets.inactive.weak_bg_fill;
        let hovered_fill = widgets.hovered.weak_bg_fill;
        let active_fill = widgets.active.weak_bg_fill;
        let accent = widgets.active.bg_fill;
        let border = widgets.noninteractive.bg_stroke.color;

        Self {
            header_height: 26.0,
            collapsed_pane_thickness: 32.0,
            handle_width: 5.0,
            min_pane_size: 60.0,
            content_inset: 0.0,
            pane_rounding: visuals.window_corner_radius,
            typography: TypographyStyle {
                tab_title_font: FontId::new(12.0, FontFamily::Proportional),
                tab_icon_text_font: FontId::new(11.0, FontFamily::Monospace),
            },
            header: HeaderStyle {
                bg: widgets.noninteractive.bg_fill,
                border_color: border,
                button: HeaderButtonStyle {
                    bg: inactive_fill,
                    hover_bg: hovered_fill,
                    stroke_color: border,
                    icon_color: widgets.inactive.fg_stroke.color,
                    hover_icon_color: widgets.hovered.fg_stroke.color,
                    rounding: CornerRadius::same(3),
                },
            },
            content: ContentStyle {
                bg: visuals.panel_fill,
                border_color: border,
            },
            tabs: TabStyle {
                active: TabStateStyle {
                    bg: active_fill,
                    text_color: widgets.active.fg_stroke.color,
                    accent_color: accent,
                },
                inactive: TabStateStyle {
                    bg: inactive_fill,
                    text_color: widgets.inactive.fg_stroke.color,
                    accent_color: accent,
                },
                hovered: TabStateStyle {
                    bg: hovered_fill,
                    text_color: widgets.hovered.fg_stroke.color,
                    accent_color: accent,
                },
                rounding: CornerRadius::same(3),
            },
            handle: HandleStyle {
                color: widgets.noninteractive.bg_fill,
                hover_color: widgets.hovered.bg_fill,
                locked_color: widgets.noninteractive.bg_fill.gamma_multiply(0.6),
            },
            overlay: OverlayStyle {
                fill: accent.gamma_multiply(0.35),
                stroke: Stroke::new(1.0, accent),
            },
        }
    }

    pub fn pane_header_bg(&self, pane: Option<PaneStyleOverride>) -> Color32 {
        pane.and_then(|pane| pane.header_bg).unwrap_or(self.header.bg)
    }

    pub fn pane_content_bg(&self, pane: Option<PaneStyleOverride>) -> Color32 {
        pane.and_then(|pane| pane.content_bg).unwrap_or(self.content.bg)
    }

    pub fn pane_content_inset(&self, pane: Option<PaneStyleOverride>) -> f32 {
        pane.and_then(|pane| pane.content_inset)
            .unwrap_or(self.content_inset)
    }

    pub fn pane_border_color(&self, pane: Option<PaneStyleOverride>) -> Color32 {
        pane.and_then(|pane| pane.border_color)
            .unwrap_or(self.content.border_color)
    }

    pub fn pane_accent_color(&self, pane: Option<PaneStyleOverride>) -> Color32 {
        pane.and_then(|pane| pane.accent_color)
            .unwrap_or(self.tabs.active.accent_color)
    }

    pub fn tab_state(
        &self,
        active: bool,
        hovered: bool,
        pane: Option<PaneStyleOverride>,
        tab: Option<TabStyleOverride>,
    ) -> TabStateStyle {
        let mut state = if active {
            self.tabs.active
        } else if hovered {
            self.tabs.hovered
        } else {
            self.tabs.inactive
        };

        if let Some(tab) = tab {
            state.bg = if active {
                tab.active_bg.unwrap_or(state.bg)
            } else if hovered {
                tab.hovered_bg.unwrap_or(state.bg)
            } else {
                tab.inactive_bg.unwrap_or(state.bg)
            };
            state.text_color = tab.text_color.unwrap_or(state.text_color);
            state.accent_color = tab.accent_color.unwrap_or(state.accent_color);
        }

        if let Some(pane) = pane {
            state.accent_color = pane.accent_color.unwrap_or(state.accent_color);
        }

        state
    }
}

impl Default for PanelStyle {
    fn default() -> Self {
        Self {
            header_height: 26.0,
            collapsed_pane_thickness: 32.0,
            handle_width: 5.0,
            min_pane_size: 60.0,
            content_inset: 0.0,
            pane_rounding: CornerRadius::same(2),
            typography: TypographyStyle {
                tab_title_font: FontId::new(12.0, FontFamily::Proportional),
                tab_icon_text_font: FontId::new(11.0, FontFamily::Monospace),
            },
            header: HeaderStyle {
                bg: Color32::from_rgb(30, 30, 35),
                border_color: Color32::from_rgb(45, 45, 55),
                button: HeaderButtonStyle {
                    bg: Color32::from_rgb(35, 35, 42),
                    hover_bg: Color32::from_rgb(50, 50, 60),
                    stroke_color: Color32::from_rgb(45, 45, 55),
                    icon_color: Color32::from_rgb(210, 210, 220),
                    hover_icon_color: Color32::from_rgb(80, 130, 220),
                    rounding: CornerRadius::same(3),
                },
            },
            content: ContentStyle {
                bg: Color32::from_rgb(27, 27, 31),
                border_color: Color32::from_rgb(45, 45, 55),
            },
            tabs: TabStyle {
                active: TabStateStyle {
                    bg: Color32::from_rgb(50, 50, 60),
                    text_color: Color32::from_rgb(210, 210, 220),
                    accent_color: Color32::from_rgb(80, 130, 220),
                },
                inactive: TabStateStyle {
                    bg: Color32::from_rgb(35, 35, 42),
                    text_color: Color32::from_rgb(210, 210, 220),
                    accent_color: Color32::from_rgb(80, 130, 220),
                },
                hovered: TabStateStyle {
                    bg: Color32::from_rgb(42, 42, 50),
                    text_color: Color32::from_rgb(220, 220, 228),
                    accent_color: Color32::from_rgb(80, 130, 220),
                },
                rounding: CornerRadius::same(3),
            },
            handle: HandleStyle {
                color: Color32::from_rgb(45, 45, 55),
                hover_color: Color32::from_rgb(80, 110, 180),
                locked_color: Color32::from_rgb(34, 34, 40),
            },
            overlay: OverlayStyle {
                fill: Color32::from_rgba_premultiplied(80, 110, 180, 60),
                stroke: Stroke::new(1.0, Color32::from_rgb(80, 130, 220)),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tab_state_prefers_overrides() {
        let style = PanelStyle::default();
        let pane = PaneStyleOverride {
            accent_color: Some(Color32::from_rgb(1, 2, 3)),
            ..PaneStyleOverride::none()
        };
        let tab = TabStyleOverride {
            active_bg: Some(Color32::from_rgb(4, 5, 6)),
            text_color: Some(Color32::from_rgb(7, 8, 9)),
            max_width: Some(120.0),
            ..TabStyleOverride::none()
        };

        let state = style.tab_state(true, false, Some(pane), Some(tab));
        assert_eq!(state.bg, Color32::from_rgb(4, 5, 6));
        assert_eq!(state.text_color, Color32::from_rgb(7, 8, 9));
        assert_eq!(state.accent_color, Color32::from_rgb(1, 2, 3));
    }

    #[test]
    fn pane_content_inset_prefers_override_then_global() {
        let style = PanelStyle {
            content_inset: 8.0,
            ..PanelStyle::default()
        };

        // No override falls back to the global inset.
        assert_eq!(style.pane_content_inset(None), 8.0);
        assert_eq!(
            style.pane_content_inset(Some(PaneStyleOverride::none())),
            8.0
        );

        // An explicit per-pane inset wins, including an edge-to-edge zero.
        let edge_to_edge = PaneStyleOverride {
            content_inset: Some(0.0),
            ..PaneStyleOverride::none()
        };
        assert_eq!(style.pane_content_inset(Some(edge_to_edge)), 0.0);
    }

    #[test]
    fn from_visuals_uses_widget_palette() {
        let visuals = Visuals::dark();
        let style = PanelStyle::from_visuals(&visuals);
        assert_eq!(style.content.bg, visuals.panel_fill);
        assert_eq!(
            style.header.button.icon_color,
            visuals.widgets.inactive.fg_stroke.color
        );
        assert_eq!(style.overlay.stroke.color, visuals.widgets.active.bg_fill);
        assert_eq!(style.typography.tab_title_font, FontId::proportional(12.0));
        assert_eq!(style.typography.tab_icon_text_font, FontId::monospace(11.0));
    }

    #[test]
    fn from_egui_style_uses_button_and_small_text_styles() {
        let mut egui_style = Style::default();
        egui_style
            .text_styles
            .insert(TextStyle::Button, FontId::proportional(16.0));
        egui_style
            .text_styles
            .insert(TextStyle::Small, FontId::monospace(13.0));

        let style = PanelStyle::from_egui_style(&egui_style);
        assert_eq!(style.typography.tab_title_font, FontId::proportional(16.0));
        assert_eq!(style.typography.tab_icon_text_font, FontId::monospace(13.0));
    }
}
