use std::path::PathBuf;
use std::rc::Rc;

use crate::types::Item;
use gpui::img;
use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::label::Label;
use gpui_component::{ActiveTheme, IconName, VirtualListScrollHandle, h_flex, v_virtual_list};

pub struct ToggleFavoriteEvent(pub usize);

pub struct LauncherList {
    pub items: Vec<Item>,
    pub filtered: Vec<Item>,
    pub selected_index: Option<usize>,
    item_sizes: Rc<Vec<Size<Pixels>>>,
    scroll_handle: VirtualListScrollHandle,
    pub favorite_ids: Vec<String>,
}

impl EventEmitter<ToggleFavoriteEvent> for LauncherList {}

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
            favorite_ids: Vec::new(),
        }
    }

    pub fn is_favorite(&self, item: &Item) -> bool {
        self.favorite_ids.iter().any(|id| id == &item.id)
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

                        let icon = if let Some(path) = &item.icon_path {
                            img(path.clone())
                        } else {
                            let themes_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                                .join("src/icons/placeHolderIcon.svg");
                            img(themes_dir)
                        };

                        Some(
                            h_flex()
                                .id(format!("item-{}", ix))
                                .gap_1()
                                .items_center()
                                .justify_between()
                                .w_full()
                                .h(px(56.))
                                .px_2()
                                .py_2()
                                .rounded_lg()
                                .when(view.selected_index == Some(ix), |this| {
                                    this.bg(cx.theme().background)
                                })
                                .hover(|this| this.bg(cx.theme().background.opacity(0.5)))
                                .cursor_pointer()
                                .on_click(cx.listener(move |list, _, window, _cx| {
                                    list.selected_index = Some(ix);
                                    if let Some(command) = list
                                        .filtered
                                        .get(ix)
                                        .and_then(|item| item.running_command.as_ref())
                                    {
                                        match std::process::Command::new(&command.command)
                                            .args(&command.args)
                                            .spawn()
                                        {
                                            Ok(_) => {
                                                window.remove_window();
                                            }
                                            Err(e) => {
                                                eprintln!("Failed to spawn command: {}", e);
                                            }
                                        }
                                    }
                                }))
                                .child(
                                    h_flex()
                                        .items_center()
                                        .gap_3()
                                        .child(icon)
                                        .child(Label::new(item.name.clone())),
                                )
                                .child(
                                    Button::new(format!("pin-{ix}"))
                                        .ghost()
                                        .icon(if view.is_favorite(item) {
                                            IconName::StarFill
                                        } else {
                                            IconName::Star
                                        })
                                        .on_click(cx.listener(move |_list, _, _, cx| {
                                            cx.stop_propagation();
                                            cx.emit(ToggleFavoriteEvent(ix));
                                        })),
                                ),
                        )
                    })
                    .collect()
            },
        )
        .bg(cx.theme().secondary)
        .p_1()
        .rounded_md()
        .track_scroll(&self.scroll_handle)
    }
}
