use std::rc::Rc;

use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::label::Label;
use gpui_component::{
    ActiveTheme, Icon, IconName, VirtualListScrollHandle, h_flex, v_virtual_list,
};

use crate::types::Item;

pub struct LauncherList {
    pub items: Vec<Item>,
    pub filtered: Vec<Item>,
    pub selected_index: Option<usize>,
    item_sizes: Rc<Vec<Size<Pixels>>>,
    scroll_handle: VirtualListScrollHandle,
}

impl LauncherList {
    pub fn new(items: Vec<Item>) -> Self {
        let filtered = items.clone();
        let item_sizes = Rc::new(items.iter().map(|_| size(px(200.), px(56.))).collect());
        Self {
            items,
            filtered,
            item_sizes,
            selected_index: None,
            scroll_handle: VirtualListScrollHandle::new(),
        }
    }

    pub fn update_filtered(&mut self, input: &str, cx: &mut Context<Self>) {
        let query = input.trim().to_lowercase();

        self.filtered = if query.is_empty() {
            self.items.clone()
        } else {
            self.items
                .iter()
                .filter(|item| item.name.to_lowercase().contains(&query))
                .cloned()
                .collect()
        };

        self.selected_index = None;
        self.item_sizes = Rc::new(
            self.filtered
                .iter()
                .map(|_| size(px(200.), px(56.)))
                .collect(),
        );

        cx.notify();
    }

    pub fn select_next(&mut self) {
        if self.filtered.is_empty() {
            self.selected_index = None;
            return;
        }
        let next = match self.selected_index {
            Some(ix) if ix + 1 < self.filtered.len() => ix + 1,
            Some(_) => 0,
            None => 0,
        };

        self.selected_index = Some(next);
        self.scroll_to_selected();
    }

    pub fn select_prev(&mut self) {
        if self.filtered.is_empty() {
            self.selected_index = None;
            return;
        }

        let last = self.filtered.len().saturating_sub(1);

        let prev = match self.selected_index {
            Some(0) => last,
            Some(ix) => ix - 1,
            None => last,
        };

        self.selected_index = Some(prev);
        self.scroll_to_selected();
    }

    fn scroll_to_selected(&self) {
        if let Some(ix) = self.selected_index {
            self.scroll_handle
                .scroll_to_item(ix, ScrollStrategy::Center);
        }
    }
}

impl Render for LauncherList {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_virtual_list(
            cx.entity().clone(),
            "List",
            self.item_sizes.clone(),
            |view, visible_range, _, cx| {
                visible_range
                    .filter_map(|ix| {
                        let item = view.filtered.get(ix)?;

                        Some(
                            h_flex()
                                .gap_1()
                                .items_center()
                                .justify_between()
                                .w_full()
                                .h(px(56.))
                                .px_4()
                                .py_2()
                                .rounded_lg()
                                .when(view.selected_index == Some(ix), |this| {
                                    this.border_1().border_color(cx.theme().list_active_border)
                                })
                                .child(
                                    h_flex()
                                        .items_center()
                                        .gap_3()
                                        .child(Icon::new(IconName::Star))
                                        .child(Label::new(item.name.clone())),
                                )
                                .child(
                                    Button::new(format!("pin-{ix}"))
                                        .ghost()
                                        .icon(IconName::Star),
                                ),
                        )
                    })
                    .collect()
            },
        )
        .track_scroll(&self.scroll_handle)
    }
}
