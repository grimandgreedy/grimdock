# grimdock

`grimdock` is an [egui](https://github.com/emilk/egui) docking crate I use for my own projects.

It keeps to a single-surface IDE-style dock: split panes, draggable tabs, resizing, and rearranging without turning into a floating window system.

## Features

- split-pane dock layouts
- stable pane identity via `PaneId`
- pane-scoped policy and mutation APIs
- separate pane policy and tab drop policy
- built-in header interactions and pane actions
- pane-specific add/open menus
- pane anchors and semantic pane roles
- versioned persistence
- theme, typography, and icon support, including texture icons

## Why this exists

This crate has a few things I needed that other egui panel/docking options
didn't line up with:

- explicit stable pane identity
- pane-level policy separate from tab-level drop rules
- anchors and semantic pane roles built into the dock model
- pane-scoped add/open actions and persistence behavior
