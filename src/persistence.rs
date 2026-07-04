use std::collections::HashSet;

use crate::{
    tab::Tab,
    tree::{Node, PaneId, PaneOptions, PanelTree, SplitDir},
};

/// Current persisted layout format version.
pub const PANEL_TREE_FORMAT_VERSION: u32 = 1;

/// Versioned, runtime-independent layout payload for persistence.
///
/// This format is the intended stable save/load contract for `grimdock`.
/// It avoids runtime-only fields such as layout rects and carries an explicit
/// version number so future structural changes can be migrated intentionally.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PersistedPanelTree<T: Clone + 'static> {
    pub version: u32,
    pub next_pane_id: u64,
    pub nodes: Vec<PersistedNode<T>>,
}

/// Persisted node payload for [`PersistedPanelTree`].
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum PersistedNode<T: Clone + 'static> {
    Empty,
    Split {
        dir: SplitDir,
        ratio: f32,
    },
    Leaf {
        pane: PaneId,
        tabs: Vec<Tab<T>>,
        focused: usize,
        options: PaneOptions,
        collapsed: bool,
    },
}

/// Legacy unversioned persistence shape that mirrors direct serde of
/// [`PanelTree`]. This is only used for migration into the explicit versioned
/// format.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LegacyPersistedPanelTree<T: Clone + 'static> {
    pub nodes: Vec<LegacyPersistedNode<T>>,
    pub next_pane_id: u64,
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum LegacyPersistedNode<T: Clone + 'static> {
    Empty,
    Split {
        dir: SplitDir,
        ratio: f32,
    },
    Leaf {
        pane: PaneId,
        tabs: Vec<Tab<T>>,
        focused: usize,
        options: PaneOptions,
        collapsed: bool,
    },
}

/// Deserialization entry point that accepts both the current versioned format
/// and the legacy unversioned format.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
pub enum PersistedPanelTreeFile<T: Clone + 'static> {
    Versioned(PersistedPanelTree<T>),
    Legacy(LegacyPersistedPanelTree<T>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PersistError {
    UnsupportedVersion(u32),
    EmptyTree,
    EmptyLeaf { pane: PaneId },
    FocusOutOfRange {
        pane: PaneId,
        focused: usize,
        tab_count: usize,
    },
    DuplicatePaneId(PaneId),
    NextPaneIdTooLow {
        next_pane_id: u64,
        required_minimum: u64,
    },
}

impl<T: Clone + 'static> PersistedPanelTree<T> {
    /// Export a runtime tree into the current versioned persistence format.
    pub fn current(tree: &PanelTree<T>) -> Self {
        Self {
            version: PANEL_TREE_FORMAT_VERSION,
            next_pane_id: tree.next_pane_id,
            nodes: tree
                .nodes
                .iter()
                .map(|node| match node {
                    Node::Empty => PersistedNode::Empty,
                    Node::Split { dir, ratio, .. } => PersistedNode::Split {
                        dir: *dir,
                        ratio: *ratio,
                    },
                    Node::Leaf {
                        pane,
                        tabs,
                        focused,
                        options,
                        collapsed,
                        ..
                    } => PersistedNode::Leaf {
                        pane: *pane,
                        tabs: tabs.clone(),
                        focused: *focused,
                        options: *options,
                        collapsed: *collapsed,
                    },
                })
                .collect(),
        }
    }

    /// Validate the persisted payload before attempting to build a runtime tree.
    pub fn validate(&self) -> Result<(), PersistError> {
        if self.nodes.is_empty() {
            return Err(PersistError::EmptyTree);
        }

        let mut panes = HashSet::new();
        let mut max_pane = 0_u64;

        for node in &self.nodes {
            if let PersistedNode::Leaf {
                pane,
                tabs,
                focused,
                options,
                ..
            } = node
            {
                if !panes.insert(*pane) {
                    return Err(PersistError::DuplicatePaneId(*pane));
                }
                max_pane = max_pane.max(pane.into_raw());
                if tabs.is_empty() {
                    if options.persist_when_empty {
                        if *focused != 0 {
                            return Err(PersistError::FocusOutOfRange {
                                pane: *pane,
                                focused: *focused,
                                tab_count: 0,
                            });
                        }
                        continue;
                    }
                    return Err(PersistError::EmptyLeaf { pane: *pane });
                }
                if *focused >= tabs.len() {
                    return Err(PersistError::FocusOutOfRange {
                        pane: *pane,
                        focused: *focused,
                        tab_count: tabs.len(),
                    });
                }
            }
        }

        let required_minimum = max_pane.saturating_add(1).max(1);
        if self.next_pane_id < required_minimum {
            return Err(PersistError::NextPaneIdTooLow {
                next_pane_id: self.next_pane_id,
                required_minimum,
            });
        }

        Ok(())
    }

    /// Convert the versioned persisted payload into a runtime tree.
    pub fn into_panel_tree(self) -> Result<PanelTree<T>, PersistError> {
        if self.version != PANEL_TREE_FORMAT_VERSION {
            return Err(PersistError::UnsupportedVersion(self.version));
        }
        self.validate()?;

        Ok(PanelTree {
            nodes: self
                .nodes
                .into_iter()
                .map(|node| match node {
                    PersistedNode::Empty => Node::Empty,
                    PersistedNode::Split { dir, ratio } => Node::Split {
                        dir,
                        ratio,
                        rect: egui::Rect::NOTHING,
                    },
                    PersistedNode::Leaf {
                        pane,
                        tabs,
                        focused,
                        options,
                        collapsed,
                    } => Node::Leaf {
                        pane,
                        tabs,
                        focused,
                        options,
                        collapsed,
                        rect: egui::Rect::NOTHING,
                    },
                })
                .collect(),
            next_pane_id: self.next_pane_id,
        })
    }
}

impl<T: Clone + 'static> From<&PanelTree<T>> for PersistedPanelTree<T> {
    fn from(value: &PanelTree<T>) -> Self {
        Self::current(value)
    }
}

impl<T: Clone + 'static> From<&PanelTree<T>> for LegacyPersistedPanelTree<T> {
    fn from(value: &PanelTree<T>) -> Self {
        Self {
            next_pane_id: value.next_pane_id,
            nodes: value
                .nodes
                .iter()
                .map(|node| match node {
                    Node::Empty => LegacyPersistedNode::Empty,
                    Node::Split { dir, ratio, .. } => LegacyPersistedNode::Split {
                        dir: *dir,
                        ratio: *ratio,
                    },
                    Node::Leaf {
                        pane,
                        tabs,
                        focused,
                        options,
                        collapsed,
                        ..
                    } => LegacyPersistedNode::Leaf {
                        pane: *pane,
                        tabs: tabs.clone(),
                        focused: *focused,
                        options: *options,
                        collapsed: *collapsed,
                    },
                })
                .collect(),
        }
    }
}

impl<T: Clone + 'static> From<LegacyPersistedPanelTree<T>> for PersistedPanelTree<T> {
    fn from(value: LegacyPersistedPanelTree<T>) -> Self {
        Self {
            version: PANEL_TREE_FORMAT_VERSION,
            next_pane_id: value.next_pane_id,
            nodes: value
                .nodes
                .into_iter()
                .map(|node| match node {
                    LegacyPersistedNode::Empty => PersistedNode::Empty,
                    LegacyPersistedNode::Split { dir, ratio } => PersistedNode::Split { dir, ratio },
                    LegacyPersistedNode::Leaf {
                        pane,
                        tabs,
                        focused,
                        options,
                        collapsed,
                    } => PersistedNode::Leaf {
                        pane,
                        tabs,
                        focused,
                        options,
                        collapsed,
                    },
                })
                .collect(),
        }
    }
}

impl<T: Clone + 'static> PersistedPanelTreeFile<T> {
    /// Migrate either the current versioned payload or the legacy payload into
    /// the current versioned format.
    pub fn migrate(self) -> Result<PersistedPanelTree<T>, PersistError> {
        match self {
            PersistedPanelTreeFile::Versioned(layout) => {
                if layout.version != PANEL_TREE_FORMAT_VERSION {
                    return Err(PersistError::UnsupportedVersion(layout.version));
                }
                layout.validate()?;
                Ok(layout)
            }
            PersistedPanelTreeFile::Legacy(layout) => {
                let layout = PersistedPanelTree::from(layout);
                layout.validate()?;
                Ok(layout)
            }
        }
    }

    /// Convert either the current or legacy persisted payload into a runtime tree.
    pub fn into_panel_tree(self) -> Result<PanelTree<T>, PersistError> {
        self.migrate()?.into_panel_tree()
    }
}

impl<T: Clone + 'static> PanelTree<T> {
    /// Export the current layout into the versioned persisted format.
    pub fn to_persisted(&self) -> PersistedPanelTree<T> {
        PersistedPanelTree::from(self)
    }

    /// Reconstruct a runtime tree from the versioned persisted format.
    pub fn from_persisted(layout: PersistedPanelTree<T>) -> Result<Self, PersistError> {
        layout.into_panel_tree()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DropPolicy, HeaderVisibility, PaneBuilder, PaneRole, TabDropPolicy, TabStyleOverride};

    fn sample_tree() -> PanelTree<&'static str> {
        let mut options = PaneOptions::default();
        options.header_visibility = HeaderVisibility::WhenMultipleTabs;
        options.drop_policy = DropPolicy::merge_only();
        options.style_override = Some(crate::style::PaneStyleOverride {
            header_bg: Some(egui::Color32::from_rgb(1, 2, 3)),
            content_bg: None,
            border_color: None,
            accent_color: None,
            content_inset: None,
        });

        let mut tree = PanelTree::from_pane(
            PaneBuilder::new(
                Tab::new("A", "a").with_style_override(TabStyleOverride {
                    active_bg: Some(egui::Color32::from_rgb(4, 5, 6)),
                    inactive_bg: None,
                    hovered_bg: None,
                    text_color: None,
                    accent_color: None,
                    icon_color: None,
                    max_width: Some(120.0),
                }),
            )
            .push_tab(Tab::new("B", "b"))
            .with_options(options),
        );
        tree.split_leaf(0, SplitDir::Horizontal, Tab::new("C", "c"), crate::ChildSide::Second);
        tree
    }

    #[test]
    fn persisted_round_trip_restores_tree() {
        let tree = sample_tree();
        let persisted = tree.to_persisted();
        let restored = PanelTree::from_persisted(persisted).expect("persistence should round-trip");

        assert_eq!(restored.next_pane_id, tree.next_pane_id);
        assert_eq!(restored.nodes.len(), tree.nodes.len());
        assert_eq!(restored.find_pane_containing(&"a"), tree.find_pane_containing(&"a"));
        assert_eq!(restored.find_pane_containing(&"c"), tree.find_pane_containing(&"c"));
    }

    #[test]
    fn persisted_validation_rejects_invalid_focus() {
        let mut layout = sample_tree().to_persisted();
        layout.nodes[1] = PersistedNode::Leaf {
            pane: PaneId::from_raw(1),
            tabs: vec![Tab::new("A", "a")],
            focused: 3,
            options: PaneOptions::default(),
            collapsed: false,
        };

        let err = layout.validate().expect_err("layout should be invalid");
        assert!(matches!(err, PersistError::FocusOutOfRange { .. }));
    }

    #[test]
    fn legacy_layout_migrates_to_current_format() {
        let legacy = LegacyPersistedPanelTree::from(&sample_tree());
        let migrated = PersistedPanelTreeFile::Legacy(legacy)
            .migrate()
            .expect("legacy layout should migrate");

        assert_eq!(migrated.version, PANEL_TREE_FORMAT_VERSION);
        assert!(migrated.validate().is_ok());
    }

    #[test]
    fn persisted_validation_allows_persistent_empty_leaf() {
        let mut options = PaneOptions::default();
        options.persist_when_empty = true;
        let layout: PersistedPanelTree<&'static str> = PersistedPanelTree {
            version: PANEL_TREE_FORMAT_VERSION,
            next_pane_id: 2,
            nodes: vec![PersistedNode::Leaf {
                pane: PaneId::from_raw(1),
                tabs: Vec::new(),
                focused: 0,
                options,
                collapsed: false,
            }],
        };

        assert!(layout.validate().is_ok());
    }

    #[test]
    fn persisted_round_trip_preserves_tab_drop_policy() {
        let tree = PanelTree::new(vec![
            Tab::new("A", "a").with_drop_policy(TabDropPolicy {
                locked_to_pane: Some(PaneId::from_raw(9)),
                locked_to_role: Some(PaneRole::Terminal),
                allowed_panes: Some(vec![PaneId::from_raw(9), PaneId::from_raw(10)]),
                allowed_roles: Some(vec![PaneRole::Editor, PaneRole::Sidebar]),
                blocked_panes: vec![PaneId::from_raw(10)],
                blocked_roles: vec![PaneRole::Inspector],
            }),
        ]);
        let persisted = tree.to_persisted();
        let restored = PanelTree::from_persisted(persisted).expect("persistence should round-trip");

        match restored.node(0) {
            Node::Leaf { tabs, .. } => {
                assert_eq!(tabs[0].drop_policy.locked_to_pane, Some(PaneId::from_raw(9)));
                assert_eq!(tabs[0].drop_policy.locked_to_role, Some(PaneRole::Terminal));
                assert_eq!(
                    tabs[0].drop_policy.allowed_panes,
                    Some(vec![PaneId::from_raw(9), PaneId::from_raw(10)])
                );
                assert_eq!(
                    tabs[0].drop_policy.allowed_roles,
                    Some(vec![PaneRole::Editor, PaneRole::Sidebar])
                );
                assert_eq!(tabs[0].drop_policy.blocked_panes, vec![PaneId::from_raw(10)]);
                assert_eq!(tabs[0].drop_policy.blocked_roles, vec![PaneRole::Inspector]);
            }
            other => panic!("expected restored root leaf, got {:?}", other),
        }
    }
}
