use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::label::Label;
use gpui_component::list::{List, ListDelegate, ListEvent, ListItem, ListSeparatorItem, ListState};
use gpui_component::{ActiveTheme, IndexPath, select};

use crate::types::Item;

pub struct LauncherDelegate {
    pub items: Vec<Item>,
    pub filtered: Vec<Item>,
    pub selected_index: Option<IndexPath>,
}

impl LauncherDelegate {
    pub fn new(items: Vec<Item>) -> Self {
        let filtered = items.clone();
        Self {
            items,
            filtered,
            selected_index: None,
        }
    }

    pub fn select_next(&self) -> Option<IndexPath> {
        let Some(selected_index) = self.selected_index else {
            return Some(IndexPath::new(0));
        };

        let next_index = selected_index.row.saturating_add(1);

        if self.filtered.get(next_index).is_some() {
            return Some(IndexPath::new(next_index));
        }
        Some(IndexPath::new(0))
    }
    pub fn select_prev(&self) -> Option<IndexPath> {
        let last = self.filtered.len().saturating_sub(1);

        let Some(selected_index) = self.selected_index else {
            return Some(IndexPath::new(last));
        };

        if selected_index.row == 0 {
            return Some(IndexPath::new(last));
        }

        Some(IndexPath::new(selected_index.row - 1))
    }
}

impl ListDelegate for LauncherDelegate {
    type Item = ListItem;

    fn items_count(&self, _section: usize, _cx: &App) -> usize {
        self.filtered.len()
    }

    fn render_item(
        &mut self,
        ix: IndexPath,
        _window: &mut Window,
        cx: &mut Context<ListState<Self>>,
    ) -> Option<Self::Item> {
        self.filtered.get(ix.row).map(|item| {
            ListItem::new(ix)
                .child(
                    Label::new(item.name.clone())
                        .text_3xl()
                        .text_color(gpui::white()),
                )
                .selected(Some(ix) == self.selected_index)
        })
    }

    fn set_selected_index(
        &mut self,
        ix: Option<IndexPath>,
        _window: &mut Window,
        cx: &mut Context<ListState<Self>>,
    ) {
        self.selected_index = ix;
        cx.notify();
    }
}
