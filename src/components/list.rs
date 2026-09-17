use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;

use crate::components::search::SearchEngine;
use crate::consts;
use crate::types::{Item, Kind};
use crate::utils::asset_path;
use gpui::img;
use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::label::Label;
use gpui_component::{ActiveTheme, IconName, VirtualListScrollHandle, h_flex, v_virtual_list};
use webbrowser;
use widestring::u16cstr;
use windows::Win32::UI::WindowsAndMessaging::{FindWindowW, SW_HIDE, ShowWindow};
use windows::core::PCWSTR;

pub struct ToggleFavoriteEvent(pub usize);

pub struct LauncherList {
    pub items: Vec<Item>,
    pub filtered: Vec<Item>,
    pub selected_index: Option<usize>,
    item_sizes: Rc<Vec<Size<Pixels>>>,
    scroll_handle: VirtualListScrollHandle,
    pub favorite_ids: Vec<String>,
    pub search: SearchEngine,
}

impl EventEmitter<ToggleFavoriteEvent> for LauncherList {}

impl LauncherList {
    pub fn new(items: Vec<Item>) -> Self {
        let filtered = items.clone();
        let item_sizes = Rc::new(items.iter().map(|_| size(px(200.), px(56.))).collect());
        let mut search = SearchEngine::new();
        search.add(items.clone());
        Self {
            items,
            filtered,
            item_sizes,
            selected_index: None,
            scroll_handle: VirtualListScrollHandle::new(),
            favorite_ids: Vec::new(),
            search,
        }
    }

    pub fn is_favorite(&self, item: &Item) -> bool {
        self.favorite_ids.iter().any(|id| id == &item.id)
    }

    pub fn update_filtered(&mut self, input: &str, cx: &mut Context<Self>) {
        if let Some((bang, bang_query)) = crate::bangs::parse_bang(input) {
            let url = crate::bangs::search_url(bang, bang_query);
            self.filtered = vec![Item {
                id: url,
                name: format!("Search {} for: {}", bang.name, bang_query),
                kind: Kind::Search,
                icon_path: None,
                running_command: None,
            }];
            self.selected_index = Some(0);
            self.item_sizes = Rc::new(vec![size(px(200.), px(56.))]);
            cx.notify();
            return;
        }

        self.search.search(input);

        self.filtered = if input.trim().is_empty() {
            self.items.clone()
        } else {
            self.search
                .results()
                .into_iter()
                .map(|mut item| {
                    if let Some(src) = self.items.iter().find(|i| i.id == item.id) {
                        item.icon_path = src.icon_path.clone();
                    }
                    item
                })
                .collect()
        };

        self.selected_index = if self.filtered.is_empty() {
            None
        } else {
            Some(0)
        };

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

    /// Batch-update icon paths after background icon extraction completes.
    /// Both `items` (master list) and `filtered` (current view) are updated
    /// so icons appear immediately and persist through search re-filtering.
    pub fn apply_icons(&mut self, icons: &[(String, Option<PathBuf>)]) {
        for (id, icon_path) in icons {
            if let Some(item) = self.items.iter_mut().find(|i| i.id == *id) {
                item.icon_path = icon_path.clone();
            }
            if let Some(item) = self.filtered.iter_mut().find(|i| i.id == *id) {
                item.icon_path = icon_path.clone();
            }
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
                            let placeholder = asset_path("icons/placeHolderIcon.svg")
                                .filter(|p| p.exists())
                                .unwrap_or_else(|| {
                                    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                                        .join("src/icons/placeHolderIcon.svg")
                                });
                            img(placeholder)
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
                                .on_click(cx.listener(move |list, _, _window, _cx| {
                                    list.selected_index = Some(ix);
                                    let Some(item) = list.filtered.get(ix) else {
                                        return;
                                    };
                                    if item.kind == Kind::Search {
                                        let _ = webbrowser::open(&item.id);
                                        unsafe {
                                            if let Ok(hwnd) = FindWindowW(
                                                None,
                                                PCWSTR(u16cstr!(consts::APP_NAME).as_ptr()),
                                            ) {
                                                let _ = ShowWindow(hwnd, SW_HIDE);
                                            }
                                        }
                                    } else if let Some(command) = item.running_command.as_ref() {
                                        match std::process::Command::new(&command.command)
                                            .args(&command.args)
                                            .spawn()
                                        {
                                            Ok(_) => unsafe {
                                                if let Ok(hwnd) = FindWindowW(
                                                    None,
                                                    PCWSTR(u16cstr!(consts::APP_NAME).as_ptr()),
                                                ) {
                                                    let _ = ShowWindow(hwnd, SW_HIDE);
                                                }
                                            },
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
