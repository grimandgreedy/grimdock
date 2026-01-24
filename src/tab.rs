use crate::{
    style::TabStyleOverride,
    tree::{PaneId, PaneRole},
};

/// Per-tab drop constraints evaluated against the destination pane.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TabDropPolicy {
    /// If set, this tab may only be dropped into the given pane.
    pub locked_to_pane: Option<PaneId>,
    /// If set, this tab may only be dropped into panes with the given role.
    pub locked_to_role: Option<PaneRole>,
    /// If set, this tab may only be dropped into panes in this allow-list.
    pub allowed_panes: Option<Vec<PaneId>>,
    /// If set, this tab may only be dropped into panes with roles in this allow-list.
    pub allowed_roles: Option<Vec<PaneRole>>,
    /// Panes in this block-list reject this tab even if otherwise allowed.
    pub blocked_panes: Vec<PaneId>,
    /// Roles in this block-list reject this tab even if otherwise allowed.
    pub blocked_roles: Vec<PaneRole>,
}

impl TabDropPolicy {
    /// Return whether this tab may be dropped into the given pane target.
    ///
    /// Pane identity and semantic role are evaluated together. This lets
    /// applications express either "only this pane instance" or "any pane with
    /// this role" style rules.
    pub fn allows_target(&self, pane_id: PaneId, role: Option<PaneRole>) -> bool {
        if let Some(locked_to_pane) = self.locked_to_pane {
            return locked_to_pane == pane_id;
        }

        if let Some(locked_to_role) = self.locked_to_role {
            return role == Some(locked_to_role);
        }

        if let Some(allowed_panes) = &self.allowed_panes {
            if !allowed_panes.contains(&pane_id) {
                return false;
            }
        }

        if let Some(allowed_roles) = &self.allowed_roles {
            if !role.is_some_and(|role| allowed_roles.contains(&role)) {
                return false;
            }
        }

        if self.blocked_panes.contains(&pane_id) {
            return false;
        }

        !role.is_some_and(|role| self.blocked_roles.contains(&role))
    }
}

impl Default for TabDropPolicy {
    fn default() -> Self {
        Self {
            locked_to_pane: None,
            locked_to_role: None,
            allowed_panes: None,
            allowed_roles: None,
            blocked_panes: Vec::new(),
            blocked_roles: Vec::new(),
        }
    }
}

/// Leading tab icon content rendered before the tab title.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TabIcon {
    /// Short text marker such as ASCII tokens or compact glyphs.
    Text(String),
    /// Texture-backed image icon with an explicit display size.
    Texture {
        texture_id: egui::TextureId,
        size: egui::Vec2,
    },
}

/// A single tab that lives inside a pane.
///
/// `T` is the caller-supplied identifier — must be `Clone + 'static`.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Tab<T: Clone + 'static> {
    pub title: String,
    pub id: T,
    /// Optional leading icon rendered before the title in tab headers.
    pub icon: Option<TabIcon>,
    /// Whether the tab may be dragged to another pane.
    pub draggable: bool,
    /// Per-tab destination constraints evaluated during drag-and-drop.
    pub drop_policy: TabDropPolicy,
    /// Whether the tab may be closed from the header UI.
    pub closable: bool,
    /// Optional visual overrides for this tab.
    pub style_override: Option<TabStyleOverride>,
}

impl<T: Clone + 'static> Tab<T> {
    /// Create a new tab with a title and caller-owned identifier.
    pub fn new(title: impl Into<String>, id: T) -> Self {
        Self {
            title: title.into(),
            id,
            icon: None,
            draggable: true,
            drop_policy: TabDropPolicy::default(),
            closable: false,
            style_override: None,
        }
    }

    /// Set a short text marker as the leading tab visual.
    pub fn with_leading_visual(mut self, leading_visual: impl Into<String>) -> Self {
        self.icon = Some(TabIcon::Text(leading_visual.into()));
        self
    }

    /// Set the leading icon payload directly.
    pub fn with_icon(mut self, icon: TabIcon) -> Self {
        self.icon = Some(icon);
        self
    }

    /// Set a texture-backed leading icon.
    pub fn with_icon_texture(mut self, texture_id: egui::TextureId, size: egui::Vec2) -> Self {
        self.icon = Some(TabIcon::Texture { texture_id, size });
        self
    }

    /// Control whether this tab may be dragged at all.
    pub fn with_draggable(mut self, draggable: bool) -> Self {
        self.draggable = draggable;
        self
    }

    /// Replace the full tab drop-policy payload.
    pub fn with_drop_policy(mut self, drop_policy: TabDropPolicy) -> Self {
        self.drop_policy = drop_policy;
        self
    }

    /// Lock the tab to a specific pane instance.
    pub fn with_locked_pane(mut self, pane_id: PaneId) -> Self {
        self.drop_policy.locked_to_pane = Some(pane_id);
        self
    }

    /// Lock the tab to panes with a specific semantic role.
    pub fn with_locked_role(mut self, role: PaneRole) -> Self {
        self.drop_policy.locked_to_role = Some(role);
        self
    }

    /// Restrict the tab to a set of pane identifiers.
    pub fn with_allowed_drop_panes(
        mut self,
        allowed_panes: impl IntoIterator<Item = PaneId>,
    ) -> Self {
        self.drop_policy.allowed_panes = Some(allowed_panes.into_iter().collect());
        self
    }

    /// Reject drops into a set of pane identifiers.
    pub fn with_blocked_drop_panes(
        mut self,
        blocked_panes: impl IntoIterator<Item = PaneId>,
    ) -> Self {
        self.drop_policy.blocked_panes = blocked_panes.into_iter().collect();
        self
    }

    /// Restrict the tab to a set of pane roles.
    pub fn with_allowed_drop_roles(
        mut self,
        allowed_roles: impl IntoIterator<Item = PaneRole>,
    ) -> Self {
        self.drop_policy.allowed_roles = Some(allowed_roles.into_iter().collect());
        self
    }

    /// Reject drops into panes with a set of semantic roles.
    pub fn with_blocked_drop_roles(
        mut self,
        blocked_roles: impl IntoIterator<Item = PaneRole>,
    ) -> Self {
        self.drop_policy.blocked_roles = blocked_roles.into_iter().collect();
        self
    }

    /// Control whether the tab may be closed from the built-in header UI.
    pub fn with_closable(mut self, closable: bool) -> Self {
        self.closable = closable;
        self
    }

    /// Apply a per-tab style override used by the built-in header renderer.
    pub fn with_style_override(mut self, style_override: TabStyleOverride) -> Self {
        self.style_override = Some(style_override);
        self
    }
}
