use egui::Rect;

use crate::{
    style::PaneStyleOverride,
    tab::Tab,
};

#[cfg(feature = "serde")]
fn serde_rect_nothing() -> Rect {
    Rect::NOTHING
}

/// Stable identifier for a pane leaf.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PaneId(u64);

impl PaneId {
    /// Create a pane identifier from its raw integer value.
    pub const fn from_raw(raw: u64) -> Self {
        Self(raw)
    }

    /// Return the raw integer value of this identifier.
    pub const fn into_raw(self) -> u64 {
        self.0
    }
}

/// Builder for constructing a pane with explicit tabs, focus, and options.
#[derive(Clone, Debug)]
pub struct PaneBuilder<T: Clone + 'static> {
    tabs: Vec<Tab<T>>,
    focused: usize,
    options: PaneOptions,
    collapsed: bool,
}

impl<T: Clone + 'static> PaneBuilder<T> {
    /// Start a pane builder with a single initial tab.
    pub fn new(tab: Tab<T>) -> Self {
        Self {
            tabs: vec![tab],
            focused: 0,
            options: PaneOptions::default(),
            collapsed: false,
        }
    }

    /// Replace the builder's tab list.
    pub fn with_tabs(mut self, tabs: Vec<Tab<T>>) -> Self {
        assert!(!tabs.is_empty(), "PaneBuilder requires at least one tab");
        self.tabs = tabs;
        self.focused = self.focused.min(self.tabs.len().saturating_sub(1));
        self
    }

    /// Append a tab to the pane being built.
    pub fn push_tab(mut self, tab: Tab<T>) -> Self {
        self.tabs.push(tab);
        self
    }

    /// Set the initially focused tab index.
    pub fn with_focused(mut self, focused: usize) -> Self {
        assert!(
            focused < self.tabs.len(),
            "focused tab index must be in bounds for PaneBuilder",
        );
        self.focused = focused;
        self
    }

    /// Replace the full options payload for the pane being built.
    pub fn with_options(mut self, options: PaneOptions) -> Self {
        self.options = options;
        self
    }

    /// Assign a root-edge anchor to the pane being built.
    pub fn with_anchor(mut self, anchor: PaneAnchor) -> Self {
        self.options.anchor = Some(anchor);
        self
    }

    /// Assign a semantic role to the pane being built.
    pub fn with_role(mut self, role: PaneRole) -> Self {
        self.options.role = Some(role);
        self
    }

    /// Set the initial collapsed state for the pane being built.
    pub fn with_collapsed(mut self, collapsed: bool) -> Self {
        self.collapsed = collapsed;
        self
    }

    fn build(self, pane: PaneId) -> Node<T> {
        assert!(!self.tabs.is_empty(), "PaneBuilder requires at least one tab");
        assert!(
            self.focused < self.tabs.len(),
            "focused tab index must be in bounds for PaneBuilder",
        );
        Node::Leaf {
            pane,
            tabs: self.tabs,
            focused: self.focused,
            options: self.options,
            collapsed: self.collapsed,
            rect: Rect::NOTHING,
        }
    }
}

/// Controls when a pane's header/tab bar is shown.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum HeaderVisibility {
    /// Always show the header, even for a single tab.
    Always,
    /// Show the header only when the pane holds multiple tabs.
    WhenMultipleTabs,
    /// Never show the header.
    Hidden,
}

/// Fine-grained permissions for tab merge and directional split drops.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DropPolicy {
    /// Whether tabs may be merged into this pane.
    pub allow_merge: bool,
    /// Whether a drop may create a left split relative to this pane.
    pub allow_split_left: bool,
    /// Whether a drop may create a right split relative to this pane.
    pub allow_split_right: bool,
    /// Whether a drop may create a top split relative to this pane.
    pub allow_split_top: bool,
    /// Whether a drop may create a bottom split relative to this pane.
    pub allow_split_bottom: bool,
}

impl DropPolicy {
    pub const fn all() -> Self {
        Self {
            allow_merge: true,
            allow_split_left: true,
            allow_split_right: true,
            allow_split_top: true,
            allow_split_bottom: true,
        }
    }

    pub const fn merge_only() -> Self {
        Self {
            allow_merge: true,
            allow_split_left: false,
            allow_split_right: false,
            allow_split_top: false,
            allow_split_bottom: false,
        }
    }

    pub const fn none() -> Self {
        Self {
            allow_merge: false,
            allow_split_left: false,
            allow_split_right: false,
            allow_split_top: false,
            allow_split_bottom: false,
        }
    }

    pub const fn allows_split(self, dir: SplitDir, side: ChildSide) -> bool {
        match (dir, side) {
            (SplitDir::Horizontal, ChildSide::First) => self.allow_split_left,
            (SplitDir::Horizontal, ChildSide::Second) => self.allow_split_right,
            (SplitDir::Vertical, ChildSide::First) => self.allow_split_top,
            (SplitDir::Vertical, ChildSide::Second) => self.allow_split_bottom,
        }
    }
}

impl Default for DropPolicy {
    fn default() -> Self {
        Self::all()
    }
}

/// Stable root-edge placement for panes that should keep a semantic location.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum PaneAnchor {
    Left,
    Right,
    Top,
    Bottom,
    Center,
}

impl PaneAnchor {
    fn split_dir_and_side(self) -> Option<(SplitDir, ChildSide)> {
        match self {
            PaneAnchor::Left => Some((SplitDir::Horizontal, ChildSide::First)),
            PaneAnchor::Right => Some((SplitDir::Horizontal, ChildSide::Second)),
            PaneAnchor::Top => Some((SplitDir::Vertical, ChildSide::First)),
            PaneAnchor::Bottom => Some((SplitDir::Vertical, ChildSide::Second)),
            PaneAnchor::Center => None,
        }
    }
}

/// Semantic pane role for higher-level policy targeting.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum PaneRole {
    Sidebar,
    Editor,
    Inspector,
    Terminal,
    BottomPanel,
    Custom(u64),
}

/// Behaviour flags for a pane leaf.
// Not `Eq`: `style_override` carries an `Option<f32>` inset, and `f32` is not
// `Eq`. `PartialEq` is retained.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PaneOptions {
    /// Header/tab bar visibility policy.
    pub header_visibility: HeaderVisibility,
    /// Whether the pane may be collapsed/expanded from the header UI.
    pub allow_collapse: bool,
    /// Whether tabs may be reordered within this pane.
    pub allow_tab_reorder: bool,
    /// Whether tabs in this pane may be dragged out of it.
    pub allow_tab_drag_out: bool,
    /// Whether resize-handle drags that affect this pane are allowed.
    pub allow_resize: bool,
    /// Whether drag/drop layout mutations are allowed for this pane.
    pub drop_policy: DropPolicy,
    /// Whether structural layout mutations should be frozen for this pane.
    pub lock_layout: bool,
    /// Optional semantic root-edge placement for this pane.
    pub anchor: Option<PaneAnchor>,
    /// Optional semantic role for higher-level app policy and role-based tab targeting.
    pub role: Option<PaneRole>,
    /// Whether an empty pane should stay in the layout instead of collapsing.
    pub persist_when_empty: bool,
    /// Whether the pane content background should be painted by the dock system.
    pub paint_content_bg: bool,
    /// Optional visual overrides for this pane.
    pub style_override: Option<PaneStyleOverride>,
}

impl Default for PaneOptions {
    fn default() -> Self {
        Self {
            header_visibility: HeaderVisibility::Always,
            allow_collapse: true,
            allow_tab_reorder: true,
            allow_tab_drag_out: true,
            allow_resize: true,
            drop_policy: DropPolicy::all(),
            lock_layout: false,
            anchor: None,
            role: None,
            persist_when_empty: false,
            paint_content_bg: true,
            style_override: None,
        }
    }
}

/// Direction of a split between two child panes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SplitDir {
    /// The two children are placed side-by-side (left | right).
    Horizontal,
    /// The two children are stacked (top / bottom).
    Vertical,
}

/// Which child slot a newly created pane should land in when calling
/// [`PanelTree::split_leaf`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChildSide {
    /// Left child (horizontal) or top child (vertical).
    First,
    /// Right child (horizontal) or bottom child (vertical).
    Second,
}

/// A single node in the flat-array binary tree.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Node<T: Clone + 'static> {
    /// Padding slot — exists only to keep the array a complete binary tree shape.
    Empty,
    /// An internal node that divides its rectangle between two children.
    Split {
        dir: SplitDir,
        /// Fraction `[0, 1]` of the rectangle given to the *first* child
        /// (left for horizontal, top for vertical).
        ratio: f32,
        /// Rectangle assigned during the last layout pass.
        #[cfg_attr(feature = "serde", serde(skip, default = "serde_rect_nothing"))]
        rect: Rect,
    },
    /// A leaf node containing one or more tabs.
    Leaf {
        /// Stable identifier for this pane.
        pane: PaneId,
        tabs: Vec<Tab<T>>,
        /// Index into `tabs` of the currently focused tab.
        focused: usize,
        /// Behaviour flags for this pane.
        options: PaneOptions,
        /// Whether the pane is collapsed to a strip along its split axis.
        collapsed: bool,
        /// Full pane rectangle (including header) assigned during last layout pass.
        #[cfg_attr(feature = "serde", serde(skip, default = "serde_rect_nothing"))]
        rect: Rect,
    },
}


/// The panel layout tree — a binary tree stored as a flat heap-indexed array.
///
/// Index arithmetic (same as a binary heap):
/// - Root: `0`
/// - Left child of `i`: `2*i + 1`
/// - Right child of `i`: `2*i + 2`
/// - Parent of `i` (for `i > 0`): `(i - 1) / 2`
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PanelTree<T: Clone + 'static> {
    pub(crate) nodes: Vec<Node<T>>,
    pub(crate) next_pane_id: u64,
}

/// Shared read-only access to a pane.
pub struct Pane<'a, T: Clone + 'static> {
    tree: &'a PanelTree<T>,
    leaf_idx: usize,
}

impl<'a, T: Clone + 'static> Pane<'a, T> {
    pub fn id(&self) -> PaneId {
        self.tree
            .pane_id_at(self.leaf_idx)
            .expect("Pane handle must point at a leaf")
    }

    pub fn index(&self) -> usize {
        self.leaf_idx
    }

    pub fn options(&self) -> PaneOptions {
        self.tree
            .pane_options(self.leaf_idx)
            .expect("Pane handle must point at a leaf")
    }

    pub fn is_collapsed(&self) -> bool {
        self.tree.is_collapsed(self.leaf_idx)
    }

    pub fn tabs(&self) -> &[Tab<T>] {
        match self.tree.node(self.leaf_idx) {
            Node::Leaf { tabs, .. } => tabs,
            _ => unreachable!("Pane handle must point at a leaf"),
        }
    }

    pub fn focused_index(&self) -> usize {
        match self.tree.node(self.leaf_idx) {
            Node::Leaf { focused, .. } => *focused,
            _ => unreachable!("Pane handle must point at a leaf"),
        }
    }

    pub fn focused_tab(&self) -> Option<&Tab<T>> {
        let focused = self.focused_index();
        self.tabs().get(focused)
    }
}

/// Mutable access to a pane.
pub struct PaneMut<'a, T: Clone + 'static> {
    tree: &'a mut PanelTree<T>,
    leaf_idx: usize,
}

impl<'a, T: Clone + 'static> PaneMut<'a, T> {
    pub fn id(&self) -> PaneId {
        self.tree
            .pane_id_at(self.leaf_idx)
            .expect("Pane handle must point at a leaf")
    }

    pub fn index(&self) -> usize {
        self.leaf_idx
    }

    pub fn options(&self) -> PaneOptions {
        self.tree
            .pane_options(self.leaf_idx)
            .expect("Pane handle must point at a leaf")
    }

    pub fn set_options(&mut self, options: PaneOptions) {
        let changed = self.tree.set_pane_options(self.leaf_idx, options);
        debug_assert!(changed, "Pane handle must point at a leaf");
    }

    pub fn is_collapsed(&self) -> bool {
        self.tree.is_collapsed(self.leaf_idx)
    }

    pub fn set_collapsed(&mut self, collapsed: bool) -> bool {
        self.tree.set_collapsed(self.leaf_idx, collapsed)
    }

    pub fn toggle_collapsed(&mut self) -> bool {
        self.tree.toggle_collapsed(self.leaf_idx)
    }

    pub fn tabs(&self) -> &[Tab<T>] {
        match self.tree.node(self.leaf_idx) {
            Node::Leaf { tabs, .. } => tabs,
            _ => unreachable!("Pane handle must point at a leaf"),
        }
    }

    pub fn tabs_mut(&mut self) -> &mut Vec<Tab<T>> {
        match self.tree.node_mut(self.leaf_idx) {
            Node::Leaf { tabs, .. } => tabs,
            _ => unreachable!("Pane handle must point at a leaf"),
        }
    }

    pub fn focused_index(&self) -> usize {
        match self.tree.node(self.leaf_idx) {
            Node::Leaf { focused, .. } => *focused,
            _ => unreachable!("Pane handle must point at a leaf"),
        }
    }

    pub fn focused_tab(&self) -> Option<&Tab<T>> {
        let focused = self.focused_index();
        self.tabs().get(focused)
    }

    pub fn set_focused_index(&mut self, focused: usize) -> bool {
        match self.tree.node_mut(self.leaf_idx) {
            Node::Leaf {
                tabs,
                focused: current,
                ..
            } if focused < tabs.len() => {
                *current = focused;
                true
            }
            Node::Leaf { .. } => false,
            _ => unreachable!("Pane handle must point at a leaf"),
        }
    }

    pub fn push_tab(&mut self, tab: Tab<T>) {
        self.tree.insert_tab_into_leaf(self.leaf_idx, tab);
    }

    pub fn close_all_tabs(&mut self) -> Vec<T> {
        self.tree.close_all_tabs_in_leaf(self.leaf_idx)
    }

    pub fn close_other_tabs(&mut self, keep_pos: usize) -> Vec<T> {
        self.tree.close_other_tabs_in_leaf(self.leaf_idx, keep_pos)
    }

    pub fn remove_tab_at(&mut self, tab_pos: usize) -> Option<Tab<T>> {
        self.tree.remove_tab_at(self.leaf_idx, tab_pos)
    }

    pub fn split(&mut self, dir: SplitDir, new_tab: Tab<T>, side: ChildSide) -> PaneId {
        self.tree.split_leaf(self.leaf_idx, dir, new_tab, side)
    }

    pub fn split_with(
        &mut self,
        dir: SplitDir,
        new_pane: PaneBuilder<T>,
        side: ChildSide,
    ) -> PaneId {
        self.tree.split_leaf_with(self.leaf_idx, dir, new_pane, side)
    }
}

impl<T: Clone + 'static> PanelTree<T> {
    /// Create a tree containing a single pane from a builder.
    pub fn from_pane(root: PaneBuilder<T>) -> Self {
        Self {
            nodes: vec![root.build(PaneId(1))],
            next_pane_id: 2,
        }
    }

    /// Create a tree containing a single pane with the given tabs.
    pub fn new(tabs: Vec<Tab<T>>) -> Self {
        assert!(!tabs.is_empty(), "PanelTree requires at least one tab");
        Self::from_pane(PaneBuilder {
            tabs,
            focused: 0,
            options: PaneOptions::default(),
            collapsed: false,
        })
    }

    fn alloc_pane_id(&mut self) -> PaneId {
        let pane = PaneId(self.next_pane_id);
        self.next_pane_id += 1;
        pane
    }

    // ── Index helpers ────────────────────────────────────────────────────────

    pub(crate) fn left_child(i: usize) -> usize {
        2 * i + 1
    }

    pub(crate) fn right_child(i: usize) -> usize {
        2 * i + 2
    }

    pub(crate) fn parent(i: usize) -> Option<usize> {
        if i == 0 {
            None
        } else {
            Some((i - 1) / 2)
        }
    }

    pub(crate) fn sibling(i: usize) -> Option<usize> {
        let parent = Self::parent(i)?;
        let left = Self::left_child(parent);
        let right = Self::right_child(parent);
        if i == left {
            Some(right)
        } else {
            Some(left)
        }
    }

    // ── Node access ─────────────────────────────────────────────────────────

    pub(crate) fn node(&self, idx: usize) -> &Node<T> {
        self.nodes.get(idx).unwrap_or(&Node::Empty)
    }

    pub fn node_mut(&mut self, idx: usize) -> &mut Node<T> {
        self.ensure_capacity(idx);
        &mut self.nodes[idx]
    }

    /// Return the stable pane identifier for a leaf index.
    pub fn pane_id_at(&self, leaf_idx: usize) -> Option<PaneId> {
        match self.node(leaf_idx) {
            Node::Leaf { pane, .. } => Some(*pane),
            _ => None,
        }
    }

    /// Return the current leaf index for a pane identifier.
    pub fn pane_index(&self, pane_id: PaneId) -> Option<usize> {
        self.leaf_indices()
            .find(|&leaf_idx| self.pane_id_at(leaf_idx) == Some(pane_id))
    }

    /// Return the pane currently assigned to the given anchor, if any.
    pub fn find_pane_with_anchor(&self, anchor: PaneAnchor) -> Option<PaneId> {
        self.leaf_indices().find_map(|leaf_idx| match self.node(leaf_idx) {
            Node::Leaf { pane, options, .. } if options.anchor == Some(anchor) => Some(*pane),
            _ => None,
        })
    }

    /// Return the pane currently assigned to the given role, if any.
    pub fn find_pane_with_role(&self, role: PaneRole) -> Option<PaneId> {
        self.leaf_indices().find_map(|leaf_idx| match self.node(leaf_idx) {
            Node::Leaf { pane, options, .. } if options.role == Some(role) => Some(*pane),
            _ => None,
        })
    }

    /// Borrow a pane by stable pane identifier.
    pub fn pane(&self, pane_id: PaneId) -> Option<Pane<'_, T>> {
        let leaf_idx = self.pane_index(pane_id)?;
        Some(Pane { tree: self, leaf_idx })
    }

    /// Mutably borrow a pane by stable pane identifier.
    pub fn pane_mut(&mut self, pane_id: PaneId) -> Option<PaneMut<'_, T>> {
        let leaf_idx = self.pane_index(pane_id)?;
        Some(PaneMut {
            tree: self,
            leaf_idx,
        })
    }

    /// Return the current pane options for a leaf.
    ///
    /// This is the low-level leaf-indexed access path. Prefer `pane(...)` or
    /// `pane_mut(...)` when you already have a `PaneId`.
    pub fn pane_options(&self, leaf_idx: usize) -> Option<PaneOptions> {
        match self.node(leaf_idx) {
            Node::Leaf { options, .. } => Some(*options),
            _ => None,
        }
    }

    /// Mutate the pane options for a leaf node.
    ///
    /// This is the low-level leaf-indexed access path. Prefer `pane_mut(...)`
    /// for typical pane-centric code.
    pub fn set_pane_options(&mut self, leaf_idx: usize, options: PaneOptions) -> bool {
        match self.node_mut(leaf_idx) {
            Node::Leaf { options: current, .. } => {
                *current = options;
                true
            }
            _ => false,
        }
    }

    fn clear_anchor_owner(&mut self, anchor: PaneAnchor, except: Option<PaneId>) {
        let pane_ids = self
            .leaf_indices()
            .filter_map(|leaf_idx| match self.node(leaf_idx) {
                Node::Leaf { pane, options, .. }
                    if options.anchor == Some(anchor) && Some(*pane) != except =>
                {
                    Some(*pane)
                }
                _ => None,
            })
            .collect::<Vec<_>>();

        for pane_id in pane_ids {
            if let Some(mut pane) = self.pane_mut(pane_id) {
                let mut options = pane.options();
                options.anchor = None;
                pane.set_options(options);
            }
        }
    }

    /// Return whether the given pane may currently be removed from the layout.
    pub fn can_remove_pane(&self, pane_id: PaneId) -> bool {
        let Some(leaf_idx) = self.pane_index(pane_id) else {
            return false;
        };
        match self.node(leaf_idx) {
            Node::Leaf { options, .. } if options.lock_layout => false,
            Node::Leaf { options, .. } if leaf_idx == 0 && !options.persist_when_empty => false,
            Node::Leaf { .. } => true,
            _ => false,
        }
    }

    /// Return whether the header should be visible for the given leaf.
    pub fn header_visible(&self, leaf_idx: usize) -> bool {
        match self.node(leaf_idx) {
            Node::Leaf {
                tabs,
                options,
                collapsed,
                ..
            } => {
                if *collapsed {
                    true
                } else if tabs.is_empty() && options.persist_when_empty {
                    true
                } else {
                    match options.header_visibility {
                        HeaderVisibility::Always => true,
                        HeaderVisibility::WhenMultipleTabs => tabs.len() > 1,
                        HeaderVisibility::Hidden => false,
                    }
                }
            }
            _ => false,
        }
    }

    /// Return whether a leaf is currently collapsed.
    pub fn is_collapsed(&self, leaf_idx: usize) -> bool {
        matches!(self.node(leaf_idx), Node::Leaf { collapsed: true, .. })
    }

    /// Update the collapsed state for a leaf.
    pub fn set_collapsed(&mut self, leaf_idx: usize, collapsed: bool) -> bool {
        match self.node_mut(leaf_idx) {
            Node::Leaf {
                collapsed: current,
                options,
                ..
            } => {
                if options.lock_layout {
                    return false;
                }
                if collapsed && !options.allow_collapse {
                    return false;
                }
                *current = collapsed;
                true
            }
            _ => false,
        }
    }

    /// Toggle the collapsed state for a leaf.
    pub fn toggle_collapsed(&mut self, leaf_idx: usize) -> bool {
        let collapsed = self.is_collapsed(leaf_idx);
        self.set_collapsed(leaf_idx, !collapsed)
    }

    fn ensure_capacity(&mut self, idx: usize) {
        if idx >= self.nodes.len() {
            self.nodes.resize_with(idx + 1, || Node::Empty);
        }
    }

    // ── Iteration ────────────────────────────────────────────────────────────

    /// Iterate over all indices that hold a `Leaf` node.
    pub(crate) fn leaf_indices(&self) -> impl Iterator<Item = usize> + '_ {
        self.nodes.iter().enumerate().filter_map(|(i, n)| {
            if matches!(n, Node::Leaf { .. }) {
                Some(i)
            } else {
                None
            }
        })
    }

    pub(crate) fn subtree_resize_locked(&self, idx: usize) -> bool {
        match self.node(idx) {
            Node::Empty => false,
            Node::Split { .. } => {
                self.subtree_resize_locked(Self::left_child(idx))
                    || self.subtree_resize_locked(Self::right_child(idx))
            }
            Node::Leaf { options, .. } => options.lock_layout || !options.allow_resize,
        }
    }

    /// Return whether the resize handle for a split node is locked.
    ///
    /// The ownership model is subtree-based: a split handle is locked if
    /// either child subtree contains a pane that is layout-locked or marked
    /// non-resizable.
    pub fn split_resize_locked(&self, split_idx: usize) -> bool {
        match self.node(split_idx) {
            Node::Split { .. } => {
                self.subtree_resize_locked(Self::left_child(split_idx))
                    || self.subtree_resize_locked(Self::right_child(split_idx))
            }
            _ => false,
        }
    }

    /// Find the leaf and tab position for a tab identifier.
    pub fn find_tab(&self, id: &T) -> Option<(usize, usize)>
    where
        T: PartialEq,
    {
        self.leaf_indices().find_map(|leaf_idx| match self.node(leaf_idx) {
            Node::Leaf { tabs, .. } => tabs
                .iter()
                .position(|tab| &tab.id == id)
                .map(|tab_pos| (leaf_idx, tab_pos)),
            _ => None,
        })
    }

    /// Return the leaf index that contains the given tab identifier.
    pub fn find_leaf_containing(&self, id: &T) -> Option<usize>
    where
        T: PartialEq,
    {
        self.find_tab(id).map(|(leaf_idx, _)| leaf_idx)
    }

    /// Return the stable pane identifier that contains the given tab.
    pub fn find_pane_containing(&self, id: &T) -> Option<PaneId>
    where
        T: PartialEq,
    {
        let leaf_idx = self.find_leaf_containing(id)?;
        self.pane_id_at(leaf_idx)
    }

    /// Focus the tab with the given identifier if it exists.
    pub fn focus_tab(&mut self, id: &T) -> bool
    where
        T: PartialEq,
    {
        let Some((leaf_idx, tab_pos)) = self.find_tab(id) else {
            return false;
        };
        if let Node::Leaf { focused, .. } = self.node_mut(leaf_idx) {
            *focused = tab_pos;
            true
        } else {
            false
        }
    }

    /// Insert a tab into an existing leaf, focusing the inserted tab.
    pub fn insert_tab_into_leaf(&mut self, leaf_idx: usize, tab: Tab<T>) {
        match self.node_mut(leaf_idx) {
            Node::Leaf { tabs, focused, .. } => {
                tabs.push(tab);
                *focused = tabs.len() - 1;
            }
            _ => panic!("insert_tab_into_leaf called on non-Leaf node"),
        }
    }

    /// Ensure a tab exists in the tree, inserting it into `leaf_idx` if missing.
    pub fn ensure_tab_in_leaf(&mut self, leaf_idx: usize, tab: Tab<T>) -> bool
    where
        T: PartialEq,
    {
        if self.find_tab(&tab.id).is_some() {
            return false;
        }
        self.insert_tab_into_leaf(leaf_idx, tab);
        true
    }

    /// Remove a tab by leaf index and position, returning the removed tab.
    pub fn remove_tab_at(&mut self, leaf_idx: usize, tab_pos: usize) -> Option<Tab<T>> {
        let removed = self.extract_tab_at(leaf_idx, tab_pos)?;
        self.collapse_empty_leaf(leaf_idx);
        Some(removed)
    }

    fn extract_tab_at(&mut self, leaf_idx: usize, tab_pos: usize) -> Option<Tab<T>> {
        match self.node_mut(leaf_idx) {
            Node::Leaf { tabs, focused, .. } => {
                if tab_pos >= tabs.len() {
                    return None;
                }
                let removed = tabs.remove(tab_pos);
                if !tabs.is_empty() && *focused >= tabs.len() {
                    *focused = tabs.len() - 1;
                }
                Some(removed)
            }
            _ => return None,
        }
    }

    /// Remove a tab by identifier, returning the removed tab.
    pub fn remove_tab(&mut self, id: &T) -> Option<Tab<T>>
    where
        T: PartialEq,
    {
        let (leaf_idx, tab_pos) = self.find_tab(id)?;
        self.remove_tab_at(leaf_idx, tab_pos)
    }

    /// Close all closable tabs in a leaf and return their identifiers.
    pub fn close_all_tabs_in_leaf(&mut self, leaf_idx: usize) -> Vec<T> {
        self.close_tabs_in_leaf_where(leaf_idx, |_, tab| tab.closable)
    }

    /// Close all closable tabs except the kept position and return their identifiers.
    pub fn close_other_tabs_in_leaf(&mut self, leaf_idx: usize, keep_pos: usize) -> Vec<T> {
        self.close_tabs_in_leaf_where(leaf_idx, |idx, tab| idx != keep_pos && tab.closable)
    }

    fn close_tabs_in_leaf_where(
        &mut self,
        leaf_idx: usize,
        mut should_close: impl FnMut(usize, &Tab<T>) -> bool,
    ) -> Vec<T> {
        let removed = match self.node_mut(leaf_idx) {
            Node::Leaf { tabs, focused, .. } => {
                let mut kept = Vec::with_capacity(tabs.len());
                let mut removed = Vec::new();
                for (idx, tab) in std::mem::take(tabs).into_iter().enumerate() {
                    if should_close(idx, &tab) {
                        removed.push(tab.id);
                    } else {
                        kept.push(tab);
                    }
                }
                *tabs = kept;
                if !tabs.is_empty() {
                    *focused = (*focused).min(tabs.len() - 1);
                }
                removed
            }
            _ => Vec::new(),
        };

        self.collapse_empty_leaf(leaf_idx);
        removed
    }

    /// Move a tab to a new position within the same leaf, focusing it.
    pub fn move_tab_within_leaf(
        &mut self,
        leaf_idx: usize,
        from_pos: usize,
        to_pos: usize,
    ) -> bool {
        match self.node_mut(leaf_idx) {
            Node::Leaf { tabs, focused, .. } => {
                if from_pos >= tabs.len() || to_pos >= tabs.len() {
                    return false;
                }
                if from_pos == to_pos {
                    *focused = to_pos;
                    return true;
                }
                let tab = tabs.remove(from_pos);
                tabs.insert(to_pos, tab);
                *focused = to_pos;
                true
            }
            _ => false,
        }
    }

    // ── Mutation operations ──────────────────────────────────────────────────

    /// Convert the leaf at `leaf_idx` into a `Split` node, placing the
    /// existing leaf content on one side and a new single-tab leaf on the
    /// other.
    pub fn split_leaf(
        &mut self,
        leaf_idx: usize,
        dir: SplitDir,
        new_tab: Tab<T>,
        side: ChildSide,
    ) -> PaneId {
        self.split_leaf_with(leaf_idx, dir, PaneBuilder::new(new_tab), side)
    }

    /// Convert the leaf at `leaf_idx` into a `Split` node, placing the
    /// existing leaf content on one side and a caller-configured pane on the
    /// other.
    pub fn split_leaf_with(
        &mut self,
        leaf_idx: usize,
        dir: SplitDir,
        new_pane: PaneBuilder<T>,
        side: ChildSide,
    ) -> PaneId {
        let left = Self::left_child(leaf_idx);
        let right = Self::right_child(leaf_idx);

        // Ensure space for both children.
        self.ensure_capacity(right);

        // Extract the existing leaf's content before mutating.
        let existing = match self.nodes[leaf_idx].clone() {
            Node::Leaf {
                pane,
                tabs,
                focused,
                options,
                ..
            } => (pane, tabs, focused, options),
            _ => panic!("split_leaf called on non-Leaf node"),
        };

        // Build the two child leaves.
        let existing_leaf = Node::Leaf {
            pane: existing.0,
            tabs: existing.1,
            focused: existing.2,
            options: existing.3,
            collapsed: false,
            rect: Rect::NOTHING,
        };
        let new_pane_id = self.alloc_pane_id();
        let new_leaf = new_pane.build(new_pane_id);

        let (first_leaf, second_leaf) = match side {
            ChildSide::First => (new_leaf, existing_leaf),
            ChildSide::Second => (existing_leaf, new_leaf),
        };

        // Promote current node to a Split, then place children.
        self.nodes[leaf_idx] = Node::Split {
            dir,
            ratio: 0.5,
            rect: Rect::NOTHING,
        };
        // Recursively copy any existing subtrees at left/right out of the way
        // (they should be Empty for a former leaf, but be safe).
        self.nodes[left] = first_leaf;
        self.nodes[right] = second_leaf;
        new_pane_id
    }

    /// Wrap the current root in a new split, placing `new_tab` on one side
    /// and the existing layout subtree on the other.
    pub fn wrap_root_with_split(
        &mut self,
        dir: SplitDir,
        new_tab: Tab<T>,
        side: ChildSide,
    ) -> PaneId {
        self.wrap_root_with_split_with(dir, PaneBuilder::new(new_tab), side)
    }

    /// Wrap the current root in a new split, placing a caller-configured pane
    /// on one side and the existing layout subtree on the other.
    pub fn wrap_root_with_split_with(
        &mut self,
        dir: SplitDir,
        new_pane: PaneBuilder<T>,
        side: ChildSide,
    ) -> PaneId {
        let old_nodes = std::mem::take(&mut self.nodes);
        let existing_child = match side {
            ChildSide::First => Self::right_child(0),
            ChildSide::Second => Self::left_child(0),
        };
        let new_tab_child = match side {
            ChildSide::First => Self::left_child(0),
            ChildSide::Second => Self::right_child(0),
        };

        self.nodes = vec![Node::Split {
            dir,
            ratio: 0.5,
            rect: Rect::NOTHING,
        }];

        self.ensure_capacity(new_tab_child);
        let new_pane_id = self.alloc_pane_id();
        self.nodes[new_tab_child] = new_pane.build(new_pane_id);

        copy_subtree_with_offset(&old_nodes, 0, &mut self.nodes, existing_child);
        new_pane_id
    }

    fn wrap_root_with_existing_leaf(
        &mut self,
        dir: SplitDir,
        new_leaf: Node<T>,
        side: ChildSide,
    ) -> PaneId {
        let old_nodes = std::mem::take(&mut self.nodes);
        let existing_child = match side {
            ChildSide::First => Self::right_child(0),
            ChildSide::Second => Self::left_child(0),
        };
        let new_leaf_child = match side {
            ChildSide::First => Self::left_child(0),
            ChildSide::Second => Self::right_child(0),
        };

        let pane_id = match &new_leaf {
            Node::Leaf { pane, .. } => *pane,
            _ => panic!("wrap_root_with_existing_leaf requires a leaf"),
        };

        self.nodes = vec![Node::Split {
            dir,
            ratio: 0.5,
            rect: Rect::NOTHING,
        }];

        self.ensure_capacity(new_leaf_child);
        self.nodes[new_leaf_child] = new_leaf;

        copy_subtree_with_offset(&old_nodes, 0, &mut self.nodes, existing_child);
        pane_id
    }

    /// Ensure a pane exists at the requested root-edge anchor.
    ///
    /// If a pane already owns the anchor, it is returned unchanged. Otherwise
    /// the current root is wrapped and the new pane is placed on the requested
    /// edge. Non-center anchors default to `persist_when_empty = true` so the
    /// anchored location remains available after its tabs are moved away.
    pub fn ensure_pane_at_anchor(
        &mut self,
        anchor: PaneAnchor,
        mut pane: PaneBuilder<T>,
    ) -> PaneId {
        if let Some(existing) = self.find_pane_with_anchor(anchor) {
            return existing;
        }

        let mut options = pane.options;
        options.anchor = Some(anchor);
        if anchor != PaneAnchor::Center {
            options.persist_when_empty = true;
        }
        pane.options = options;

        match anchor.split_dir_and_side() {
            Some((dir, side)) => self.wrap_root_with_split_with(dir, pane, side),
            None => {
                if let Node::Leaf { pane, options, .. } = self.node_mut(0) {
                    options.anchor = Some(PaneAnchor::Center);
                    *pane
                } else {
                    self.wrap_root_with_split_with(SplitDir::Horizontal, pane, ChildSide::Second)
                }
            }
        }
    }

    /// Remove a pane from the layout, returning the tab identifiers it held.
    ///
    /// If the pane is persistent it is cleared and left in place as an empty
    /// placeholder. If it is the sole root pane and not persistent, removal is
    /// rejected.
    pub fn remove_pane(&mut self, pane_id: PaneId) -> Option<Vec<T>> {
        if !self.can_remove_pane(pane_id) {
            return None;
        }

        let leaf_idx = self.pane_index(pane_id)?;
        let mut removed = Vec::new();
        let persist_when_empty = match self.node_mut(leaf_idx) {
            Node::Leaf {
                tabs,
                focused,
                options,
                ..
            } => {
                removed.extend(tabs.drain(..).map(|tab| tab.id));
                *focused = 0;
                options.persist_when_empty
            }
            _ => return None,
        };

        if !persist_when_empty {
            self.collapse_empty_leaf(leaf_idx);
        }

        Some(removed)
    }

    /// Move an existing pane to a stable root-edge anchor, preserving its
    /// pane identifier and contents.
    ///
    /// This is the semantic location-oriented path. It complements
    /// `PaneId`-based identity and `PaneRole`-based policy targeting.
    pub fn move_pane_to_anchor(&mut self, pane_id: PaneId, anchor: PaneAnchor) -> Option<PaneId> {
        let leaf_idx = self.pane_index(pane_id)?;
        if matches!(self.node(leaf_idx), Node::Leaf { options, .. } if options.lock_layout) {
            return None;
        }

        self.clear_anchor_owner(anchor, Some(pane_id));

        if leaf_idx == 0 {
            if let Node::Leaf { options, .. } = self.node_mut(0) {
                options.anchor = Some(anchor);
            }
            return Some(pane_id);
        }

        let mut relocated_leaf = match self.node(leaf_idx).clone() {
            Node::Leaf { pane, tabs, focused, mut options, .. } => {
                options.anchor = Some(anchor);
                if anchor != PaneAnchor::Center {
                    options.persist_when_empty = true;
                }
                Node::Leaf {
                    pane,
                    tabs,
                    focused,
                    options,
                    collapsed: false,
                    rect: Rect::NOTHING,
                }
            }
            _ => return None,
        };

        if let Node::Leaf { options, .. } = self.node_mut(leaf_idx) {
            options.persist_when_empty = false;
        }
        if let Node::Leaf { tabs, focused, .. } = self.node_mut(leaf_idx) {
            tabs.clear();
            *focused = 0;
        }
        self.collapse_empty_leaf(leaf_idx);

        match anchor.split_dir_and_side() {
            Some((dir, side)) => Some(self.wrap_root_with_existing_leaf(dir, relocated_leaf, side)),
            None => {
                if let Node::Leaf { options, .. } = &mut relocated_leaf {
                    options.anchor = Some(PaneAnchor::Center);
                }
                Some(self.wrap_root_with_existing_leaf(
                    SplitDir::Horizontal,
                    relocated_leaf,
                    ChildSide::Second,
                ))
            }
        }
    }

    /// Move the tab at position `tab_pos` in `src_leaf` into `dst_leaf`.
    ///
    /// The tab is appended at the end of the destination's tab list and focused.
    pub(crate) fn merge_tab_into(&mut self, src_leaf: usize, tab_pos: usize, dst_leaf: usize) {
        let tab = self
            .extract_tab_at(src_leaf, tab_pos)
            .expect("merge_tab_into: src is not a Leaf or tab index is invalid");

        // Insert into destination.
        match &mut self.nodes[dst_leaf] {
            Node::Leaf { tabs, focused, .. } => {
                tabs.push(tab);
                *focused = tabs.len() - 1;
            }
            _ => panic!("merge_tab_into: dst is not a Leaf"),
        }
    }

    /// Collapse an empty leaf node, promoting its sibling subtree one level up.
    ///
    /// If the leaf is not empty this is a no-op.  If the leaf is the root, the
    /// tree is left in its current state (cannot collapse sole root).
    pub(crate) fn collapse_empty_leaf(&mut self, leaf_idx: usize) {
        // Check that the leaf is actually empty.
        let (is_empty, persist_when_empty) = match &self.nodes[leaf_idx] {
            Node::Leaf { tabs, options, .. } => (tabs.is_empty(), options.persist_when_empty),
            _ => return,
        };
        if !is_empty {
            return;
        }
        if persist_when_empty {
            return;
        }

        let parent_idx = match Self::parent(leaf_idx) {
            Some(p) => p,
            None => return, // root — cannot collapse
        };

        let sibling_idx = Self::sibling(leaf_idx).unwrap();

        // Copy the sibling's entire subtree into the parent's position.
        copy_subtree(&mut self.nodes, sibling_idx, parent_idx);

        // Clear the former empty leaf — but only if copy_subtree didn't
        // already write new content there.  When the promoted sibling was a
        // Split, copy_subtree places the sibling's children at the parent's
        // child slots (left_child(parent) and right_child(parent)).  leaf_idx
        // IS one of those slots, so clearing it would destroy the
        // newly-promoted content.
        //
        // After copy_subtree the node at parent_idx holds what was in sibling.
        // If that is a Split, leaf_idx was overwritten — skip the clear.
        let sibling_became_split = matches!(&self.nodes[parent_idx], Node::Split { .. });
        if !sibling_became_split && leaf_idx < self.nodes.len() {
            self.nodes[leaf_idx] = Node::Empty;
        }
    }
}

/// Copy the subtree rooted at `from` into the slot at `to`, then clear the
/// original source slots.
///
/// Uses a two-phase approach to avoid aliasing: first collect all
/// `(src_idx, cloned_node, dst_idx)` triples, then write them all, then
/// clear any source slot that was not itself used as a destination.  The
/// naive recursive approach has a subtle bug when the source subtree and
/// destination subtree overlap (which always happens when `from` is a child
/// of `to`), because clearing `nodes[from]` at the end of the recursion
/// overwrites a node that one of the recursive writes just placed there.
fn copy_subtree<T: Clone + 'static>(nodes: &mut Vec<Node<T>>, from: usize, to: usize) {
    // Phase 1: collect all (src, cloned_node, dst) triples.
    let mut moves: Vec<(usize, Node<T>, usize)> = Vec::new();
    collect_moves(nodes, from, to, &mut moves);

    if moves.is_empty() {
        return;
    }

    // Ensure the vec is large enough for all destination indices.
    let max_dst = moves.iter().map(|m| m.2).max().unwrap_or(0);
    if max_dst >= nodes.len() {
        nodes.resize_with(max_dst + 1, || Node::Empty);
    }

    let dst_set: std::collections::HashSet<usize> = moves.iter().map(|m| m.2).collect();

    // Phase 2: write all nodes to their destinations.
    for (_, node, dst) in moves.iter().cloned() {
        nodes[dst] = node;
    }

    // Phase 3: clear source slots that were not also used as destinations.
    for (src, _, _) in &moves {
        if !dst_set.contains(src) && *src < nodes.len() {
            nodes[*src] = Node::Empty;
        }
    }
}

/// Recursively collect `(src_idx, cloned_node, dst_idx)` triples for an
/// entire subtree, preserving the heap-index mapping.
fn collect_moves<T: Clone + 'static>(
    nodes: &[Node<T>],
    from: usize,
    to: usize,
    moves: &mut Vec<(usize, Node<T>, usize)>,
) {
    if from >= nodes.len() || matches!(nodes[from], Node::Empty) {
        return;
    }
    let node = nodes[from].clone();
    let is_split = matches!(node, Node::Split { .. });
    moves.push((from, node, to));
    if is_split {
        collect_moves(nodes, 2 * from + 1, 2 * to + 1, moves);
        collect_moves(nodes, 2 * from + 2, 2 * to + 2, moves);
    }
}

fn copy_subtree_with_offset<T: Clone + 'static>(
    src: &[Node<T>],
    from: usize,
    dst: &mut Vec<Node<T>>,
    to: usize,
) {
    if from >= src.len() || matches!(src[from], Node::Empty) {
        return;
    }
    if to >= dst.len() {
        dst.resize_with(to + 1, || Node::Empty);
    }
    let node = src[from].clone();
    let is_split = matches!(node, Node::Split { .. });
    dst[to] = node;
    if is_split {
        copy_subtree_with_offset(src, 2 * from + 1, dst, 2 * to + 1);
        copy_subtree_with_offset(src, 2 * from + 2, dst, 2 * to + 2);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tab::Tab;

    fn make_tree() -> PanelTree<&'static str> {
        PanelTree::new(vec![
            Tab::new("A", "a"),
            Tab::new("B", "b"),
        ])
    }

    #[test]
    fn index_arithmetic() {
        assert_eq!(PanelTree::<()>::left_child(0), 1);
        assert_eq!(PanelTree::<()>::right_child(0), 2);
        assert_eq!(PanelTree::<()>::left_child(1), 3);
        assert_eq!(PanelTree::<()>::right_child(1), 4);
        assert_eq!(PanelTree::<()>::parent(1), Some(0));
        assert_eq!(PanelTree::<()>::parent(2), Some(0));
        assert_eq!(PanelTree::<()>::parent(0), None);
    }

    #[test]
    fn sibling() {
        assert_eq!(PanelTree::<()>::sibling(1), Some(2));
        assert_eq!(PanelTree::<()>::sibling(2), Some(1));
        assert_eq!(PanelTree::<()>::sibling(3), Some(4));
    }

    #[test]
    fn split_creates_children() {
        let mut tree = make_tree();
        tree.split_leaf(0, SplitDir::Horizontal, Tab::new("C", "c"), ChildSide::Second);

        assert!(matches!(tree.node(0), Node::Split { dir: SplitDir::Horizontal, .. }));
        assert!(matches!(tree.node(1), Node::Leaf { .. }));
        assert!(matches!(tree.node(2), Node::Leaf { .. }));

        // The existing tabs should be in the first child (ChildSide::Second puts new in right).
        if let Node::Leaf { tabs, .. } = tree.node(1) {
            assert_eq!(tabs.len(), 2);
        } else {
            panic!("expected Leaf at index 1");
        }
        if let Node::Leaf { tabs, .. } = tree.node(2) {
            assert_eq!(tabs[0].id, "c");
        } else {
            panic!("expected Leaf at index 2");
        }
    }

    #[test]
    fn merge_tab_then_collapse() {
        // Single-tab tree: root has only "a".
        let mut tree = PanelTree::new(vec![Tab::new("A", "a")]);

        // Split root: new tab "b" on the right (Second), existing "a" stays on left (First).
        tree.split_leaf(0, SplitDir::Horizontal, Tab::new("B", "b"), ChildSide::Second);
        // node 1 (left)  = ["a"]
        // node 2 (right) = ["b"]

        // Merge "b" (node 2, pos 0) into node 1.
        tree.merge_tab_into(2, 0, 1);

        // Node 2 must now be empty.
        if let Node::Leaf { tabs, .. } = tree.node(2) {
            assert!(tabs.is_empty(), "node 2 should be empty after merge");
        } else {
            panic!("expected Leaf at index 2");
        }

        // Node 1 should now hold both tabs.
        if let Node::Leaf { tabs, .. } = tree.node(1) {
            assert_eq!(tabs.len(), 2);
        }

        // Collapse empty node 2: node 1 (sibling) gets promoted to root (node 0).
        tree.collapse_empty_leaf(2);

        if let Node::Leaf { tabs, .. } = tree.node(0) {
            assert_eq!(tabs.len(), 2, "root should hold both tabs after collapse");
        } else {
            panic!("expected root to be a Leaf after collapse");
        }

        // Nodes 1 and 2 should be Empty after promotion.
        assert!(matches!(tree.node(1), Node::Empty));
        assert!(matches!(tree.node(2), Node::Empty));
    }

    /// Regression: collapse a leaf whose sibling is a Split.
    ///
    /// Before the fix, copy_subtree would place the sibling's children at
    /// left_child(parent) and right_child(parent).  leaf_idx coincides with
    /// one of those slots (it IS a child of parent), so the subsequent
    /// `nodes[leaf_idx] = Empty` erased that newly-promoted content, causing
    /// a tab to silently disappear.
    #[test]
    fn collapse_leaf_with_split_sibling_preserves_content() {
        // Build:
        //   node 0: Split
        //   node 1: Leaf ["solo"]   ← will be emptied and collapsed
        //   node 2: Split           ← sibling, its children must survive
        //   node 5: Leaf ["left"]
        //   node 6: Leaf ["right"]

        let mut tree = PanelTree::new(vec![Tab::new("Solo", "solo")]);

        // Split root horizontally: solo stays left (First side), new tab right.
        tree.split_leaf(0, SplitDir::Horizontal, Tab::new("Right", "right"), ChildSide::Second);
        // node 1 = ["solo"], node 2 = ["right"]

        // Split node 2 vertically so it becomes a Split with two leaf children.
        tree.split_leaf(2, SplitDir::Vertical, Tab::new("Left", "left"), ChildSide::First);
        // node 2 = Split, node 5 = ["left"], node 6 = ["right"]

        // Empty node 1 by merging its tab into node 5.
        tree.merge_tab_into(1, 0, 5);
        // node 1 = Leaf[], node 5 = ["left", "solo"]

        assert!(matches!(tree.node(1), Node::Leaf { tabs, .. } if tabs.is_empty()));

        // Collapse empty node 1.  Sibling is node 2 (a Split).
        // After collapse:
        //   node 0 = Split (was node 2)
        //   node 1 = ["left", "solo"]   (promoted from node 5)
        //   node 2 = ["right"]          (promoted from node 6)
        tree.collapse_empty_leaf(1);

        // node 0 must be a Split.
        assert!(matches!(tree.node(0), Node::Split { .. }), "root should be Split");

        // node 1 must have the two tabs from old node 5 — not be Empty.
        if let Node::Leaf { tabs, .. } = tree.node(1) {
            assert_eq!(tabs.len(), 2, "promoted left child should have 2 tabs, got {:?}", tabs.iter().map(|t| t.id).collect::<Vec<_>>());
        } else {
            panic!("node 1 should be a Leaf after collapse, got {:?}", tree.node(1));
        }

        // node 2 must have the one tab from old node 6.
        if let Node::Leaf { tabs, .. } = tree.node(2) {
            assert_eq!(tabs.len(), 1, "promoted right child should have 1 tab");
            assert_eq!(tabs[0].id, "right");
        } else {
            panic!("node 2 should be a Leaf after collapse");
        }
    }

    #[test]
    fn pane_ids_stay_stable_across_split_and_collapse() {
        let mut tree = PanelTree::new(vec![Tab::new("Solo", "solo")]);
        let solo_pane = tree.find_pane_containing(&"solo").expect("solo pane should exist");

        let right_pane =
            tree.split_leaf(0, SplitDir::Horizontal, Tab::new("Right", "right"), ChildSide::Second);

        assert_eq!(tree.find_pane_containing(&"solo"), Some(solo_pane));
        assert_eq!(tree.find_pane_containing(&"right"), Some(right_pane));
        assert_ne!(solo_pane, right_pane);

        tree.merge_tab_into(tree.pane_index(right_pane).unwrap(), 0, tree.pane_index(solo_pane).unwrap());
        tree.collapse_empty_leaf(tree.pane_index(right_pane).unwrap());

        assert_eq!(tree.find_pane_containing(&"solo"), Some(solo_pane));
        assert_eq!(tree.find_pane_containing(&"right"), Some(solo_pane));
        assert!(tree.pane_index(right_pane).is_none());
    }

    #[test]
    fn pane_mut_supports_option_and_tab_updates() {
        let mut tree = PanelTree::new(vec![Tab::new("A", "a")]);
        let pane_id = tree.find_pane_containing(&"a").expect("root pane should exist");

        {
            let mut pane = tree.pane_mut(pane_id).expect("pane should exist");
            let mut options = pane.options();
            options.allow_collapse = false;
            pane.set_options(options);
            pane.push_tab(Tab::new("B", "b"));
            assert!(pane.set_focused_index(1));
            assert_eq!(pane.focused_tab().map(|tab| tab.id), Some("b"));
        }

        let pane = tree.pane(pane_id).expect("pane should still exist");
        assert!(!pane.options().allow_collapse);
        assert_eq!(pane.tabs().len(), 2);
        assert_eq!(pane.focused_tab().map(|tab| tab.id), Some("b"));
    }

    #[test]
    fn pane_builder_preserves_options_focus_and_tabs() {
        let mut options = PaneOptions::default();
        options.drop_policy = DropPolicy::merge_only();
        options.allow_resize = false;

        let tree = PanelTree::from_pane(
            PaneBuilder::new(Tab::new("A", "a"))
                .push_tab(Tab::new("B", "b"))
                .with_focused(1)
                .with_options(options),
        );

        let pane_id = tree.find_pane_containing(&"b").expect("pane should exist");
        let pane = tree.pane(pane_id).expect("pane should exist");
        assert_eq!(pane.tabs().len(), 2);
        assert_eq!(pane.focused_tab().map(|tab| tab.id), Some("b"));
        assert_eq!(pane.options().drop_policy, DropPolicy::merge_only());
        assert!(!pane.options().allow_resize);
    }

    #[test]
    fn header_visibility_respects_hidden_and_collapsed_flows() {
        let mut tree = PanelTree::new(vec![Tab::new("A", "a")]);
        let mut options = tree.pane_options(0).unwrap();
        options.header_visibility = HeaderVisibility::Hidden;
        assert!(tree.set_pane_options(0, options));

        assert!(!tree.header_visible(0), "hidden header should stay hidden");

        assert!(tree.set_collapsed(0, true));
        assert!(
            tree.header_visible(0),
            "collapsed panes should still show a header affordance"
        );
    }

    #[test]
    fn header_visibility_when_multiple_tabs_only() {
        let mut tree = PanelTree::new(vec![Tab::new("A", "a")]);
        let mut options = tree.pane_options(0).unwrap();
        options.header_visibility = HeaderVisibility::WhenMultipleTabs;
        assert!(tree.set_pane_options(0, options));

        assert!(!tree.header_visible(0));

        tree.insert_tab_into_leaf(0, Tab::new("B", "b"));
        assert!(tree.header_visible(0));
    }

    #[test]
    fn lock_layout_blocks_collapse() {
        let mut tree = PanelTree::new(vec![Tab::new("A", "a")]);
        let pane_id = tree.find_pane_containing(&"a").expect("root pane should exist");

        let mut options = tree.pane(pane_id).unwrap().options();
        options.lock_layout = true;
        assert!(tree.set_pane_options(0, options));

        assert!(!tree.set_collapsed(0, true));
        assert!(!tree.is_collapsed(0));
    }

    #[test]
    fn move_tab_within_leaf_reorders_and_refocuses() {
        let mut tree = PanelTree::new(vec![
            Tab::new("A", "a"),
            Tab::new("B", "b"),
            Tab::new("C", "c"),
        ]);

        assert!(tree.move_tab_within_leaf(0, 0, 2));

        if let Node::Leaf { tabs, focused, .. } = tree.node(0) {
            assert_eq!(tabs.iter().map(|tab| tab.id).collect::<Vec<_>>(), vec!["b", "c", "a"]);
            assert_eq!(*focused, 2);
        } else {
            panic!("expected root to remain a leaf");
        }
    }

    #[test]
    fn split_resize_locked_tracks_descendant_pane_policy() {
        let mut tree = PanelTree::new(vec![Tab::new("A", "a")]);
        let right_pane =
            tree.split_leaf(0, SplitDir::Horizontal, Tab::new("B", "b"), ChildSide::Second);

        assert!(!tree.split_resize_locked(0));

        let right_idx = tree.pane_index(right_pane).expect("right pane should exist");
        let mut options = tree.pane_options(right_idx).unwrap();
        options.allow_resize = false;
        assert!(tree.set_pane_options(right_idx, options));

        assert!(tree.split_resize_locked(0));
        assert!(tree.subtree_resize_locked(right_idx));
    }

    #[test]
    fn split_with_builder_creates_configured_pane() {
        let mut tree = PanelTree::new(vec![Tab::new("A", "a")]);
        let mut options = PaneOptions::default();
        options.lock_layout = true;
        options.drop_policy = DropPolicy::none();

        let new_pane = tree.split_leaf_with(
            0,
            SplitDir::Horizontal,
            PaneBuilder::new(Tab::new("B", "b"))
                .push_tab(Tab::new("C", "c"))
                .with_focused(1)
                .with_options(options),
            ChildSide::Second,
        );

        let pane = tree.pane(new_pane).expect("split pane should exist");
        assert_eq!(pane.tabs().len(), 2);
        assert_eq!(pane.focused_tab().map(|tab| tab.id), Some("c"));
        assert!(pane.options().lock_layout);
        assert_eq!(pane.options().drop_policy, DropPolicy::none());
    }

    #[test]
    fn persist_when_empty_keeps_leaf_in_layout() {
        let mut tree = PanelTree::new(vec![Tab::new("solo", "solo")]);
        let mut options = tree.pane_options(0).expect("root pane should exist");
        options.persist_when_empty = true;
        assert!(tree.set_pane_options(0, options));

        let removed = tree.remove_tab_at(0, 0).expect("tab should be removed");
        assert_eq!(removed.id, "solo");

        match tree.node(0) {
            Node::Leaf { tabs, .. } => assert!(tabs.is_empty(), "persistent pane should remain empty"),
            other => panic!("expected persistent empty leaf at root, got {:?}", other),
        }
    }

    #[test]
    fn persistent_empty_pane_keeps_header_visible() {
        let mut tree = PanelTree::new(vec![Tab::new("solo", "solo")]);
        let mut options = tree.pane_options(0).expect("root pane should exist");
        options.header_visibility = HeaderVisibility::Hidden;
        options.persist_when_empty = true;
        assert!(tree.set_pane_options(0, options));

        assert!(tree.remove_tab_at(0, 0).is_some());
        assert!(tree.header_visible(0));
    }

    #[test]
    fn ensure_pane_at_anchor_reuses_existing_anchor_owner() {
        let mut tree = PanelTree::new(vec![Tab::new("center", "center")]);
        let left = tree.ensure_pane_at_anchor(
            PaneAnchor::Left,
            PaneBuilder::new(Tab::new("files", "files")),
        );
        let left_again = tree.ensure_pane_at_anchor(
            PaneAnchor::Left,
            PaneBuilder::new(Tab::new("other", "other")),
        );

        assert_eq!(left, left_again);
        assert_eq!(tree.find_pane_with_anchor(PaneAnchor::Left), Some(left));
        assert_eq!(tree.find_tab(&"other"), None);
    }

    #[test]
    fn ensure_pane_at_anchor_marks_edge_panes_persistent() {
        let mut tree = PanelTree::new(vec![Tab::new("center", "center")]);
        let bottom = tree.ensure_pane_at_anchor(
            PaneAnchor::Bottom,
            PaneBuilder::new(Tab::new("terminal", "terminal")),
        );

        let pane = tree.pane(bottom).expect("bottom anchor pane should exist");
        assert_eq!(pane.options().anchor, Some(PaneAnchor::Bottom));
        assert!(pane.options().persist_when_empty);
    }

    #[test]
    fn remove_pane_clears_persistent_root_and_returns_tabs() {
        let mut tree = PanelTree::new(vec![Tab::new("solo", "solo")]);
        let pane_id = tree.find_pane_containing(&"solo").expect("root pane should exist");
        let mut options = tree.pane(pane_id).unwrap().options();
        options.persist_when_empty = true;
        assert!(tree.set_pane_options(0, options));

        let removed = tree.remove_pane(pane_id).expect("persistent root pane should be removable");
        assert_eq!(removed, vec!["solo"]);
        match tree.node(0) {
            Node::Leaf { tabs, .. } => assert!(tabs.is_empty()),
            other => panic!("expected empty root leaf, got {:?}", other),
        }
    }

    #[test]
    fn move_pane_to_anchor_preserves_identity_and_content() {
        let mut tree = PanelTree::new(vec![Tab::new("editor", "editor")]);
        let files = tree.ensure_pane_at_anchor(
            PaneAnchor::Left,
            PaneBuilder::new(Tab::new("files", "files")),
        );

        let moved = tree
            .move_pane_to_anchor(files, PaneAnchor::Bottom)
            .expect("pane should move to bottom");
        assert_eq!(moved, files);
        assert_eq!(tree.find_pane_with_anchor(PaneAnchor::Bottom), Some(files));
        assert_eq!(tree.find_tab(&"files").map(|(leaf, _)| tree.pane_id_at(leaf)), Some(Some(files)));
    }

    #[test]
    fn find_pane_with_role_tracks_semantic_owner() {
        let tree = PanelTree::from_pane(
            PaneBuilder::new(Tab::new("files", "files")).with_role(PaneRole::Sidebar),
        );

        assert_eq!(
            tree.find_pane_with_role(PaneRole::Sidebar),
            tree.find_pane_containing(&"files")
        );
    }
}
