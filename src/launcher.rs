use std::sync::Arc;
use std::time::Duration;

use crate::components::list::ToggleFavoriteEvent;
use crate::components::{fav::Fav, list::LauncherList};
use crate::scanner::scan_apps_fast;
use crate::types::{Item, Kind};
use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::{
    ActiveTheme, Icon, IconName,
    input::{Input, InputEvent, InputState},
    kbd,
};
use widestring::u16cstr;
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::{
    FindWindowW, SW_HIDE, SW_SHOW, SetForegroundWindow, ShowWindow,
};
use windows::core::PCWSTR;

pub struct LauncherState {
    input: Entity<InputState>,
    list: Entity<LauncherList>,
    fav: Entity<Fav>,
    hwnd: HWND,
    is_visible: bool,
}

actions!(
    launcher,
    [
        SelectNext,
        SelectPrev,
        Confirm,
        Cancel,
        ToggleFavorite,
        FocusSearch
    ]
);

impl LauncherState {
    pub fn new(
        window: &mut Window,
        cx: &mut Context<Self>,
        hotkey_rx: crossbeam_channel::Receiver<crate::hotkey::HotkeyEvent>,
    ) -> Self {
        let input = cx.new(|cx| {
            let input = InputState::new(window, cx).placeholder("Search...");

            input.focus(window, cx);

            input
        });
        let hwnd = unsafe { FindWindowW(None, PCWSTR(u16cstr!("PopMax").as_ptr())) }
            .expect("Failed to find PopMax window");
        let is_visible = true;

        // Phase 1 — fast: scan app names only, show list immediately with placeholder icons.
        let scan_entries = Arc::new(scan_apps_fast());
        let items: Vec<Item> = scan_entries.iter().map(|e| Item {
            icon_path: None,
            ..e.item.clone()
        }).collect();

        let list = cx.new(|_cx| LauncherList::new(items));

        // Phase 2 — background: extract all icons in parallel, then batch-update the list.
        let bg_entries = scan_entries.clone();
        cx.spawn_in(window, async move |this, cx| {
            let icons = cx
                .background_executor()
                .spawn(async move {
                    crate::scanner::extract_icons_batch(&bg_entries)
                })
                .await;
            this.update_in(cx, |this, _window, cx| {
                this.list.update(cx, |list, cx| {
                    list.apply_icons(&icons);
                    cx.notify();
                });

                let fresh_items: Vec<Item> = this.list.read(cx).items.clone();
                this.fav.update(cx, |fav, cx| {
                    let mut changed = false;
                    for f in &mut fav.favorites {
                        if let Some(matched) = fresh_items.iter().find(|i| i.id == f.id) {
                            if f.icon_path != matched.icon_path {
                                f.icon_path = matched.icon_path.clone();
                                changed = true;
                            }
                        }
                    }
                    if changed {
                        fav.save_favorites_to_disk();
                        cx.notify();
                    }
                });
            })
            .ok();
        })
        .detach();

        cx.subscribe_in::<_, InputEvent>(&input, window, |view, state, _event, _window, cx| {
            let input = state.read(cx).value();
            view.list.update(cx, |list, cx| {
                list.update_filtered(&input, cx);
            });
        })
        .detach();

        let fav = cx.new(|_cx| Fav::new());
        cx.observe(&fav, |this, _fav, cx| {
            let ids: Vec<String> = this
                .fav
                .read(cx)
                .favorites
                .iter()
                .map(|f| f.id.clone())
                .collect();
            this.list.update(cx, |list, cx| {
                list.favorite_ids = ids;
                cx.notify();
            });
        })
        .detach();

        // Subscribe to ToggleFavoriteEvent and update favorite status accordingly
        cx.subscribe_in::<LauncherList, ToggleFavoriteEvent>(
            &list,
            window,
            |view, _, event, _window, cx| {
                let ix = event.0;
                let Some(item) = view.get_item(ix, cx) else {
                    return;
                };

                let is_fav = view.fav.read(cx).is_favorite(&item.id);
                if is_fav {
                    view.fav
                        .update(cx, |fav, cx| fav.remove_favorite(&item.id, cx));
                } else {
                    view.fav.update(cx, |fav, cx| fav.add_favorite(item, cx));
                }

                let ids: Vec<String> = view
                    .fav
                    .read(cx)
                    .favorites
                    .iter()
                    .map(|f| f.id.clone())
                    .collect();
                view.list.update(cx, |list, cx| {
                    list.favorite_ids = ids;
                    cx.notify();
                })
            },
        )
        .detach();

        cx.spawn_in(window, async move |this, cx: &mut AsyncWindowContext| {
            loop {
                while let Ok(_event) = hotkey_rx.try_recv() {
                    this.update_in(cx, |this, window, cx| {
                        if this.is_visible {
                            unsafe {
                                let _ = ShowWindow(this.hwnd, SW_HIDE);
                            }
                            this.is_visible = false;
                        } else {
                            unsafe {
                                let _ = ShowWindow(this.hwnd, SW_SHOW);
                                let _ = SetForegroundWindow(this.hwnd);
                            }
                            window.activate_window();
                            this.is_visible = true;
                            this.input.update(cx, |input, cx| {
                                input.focus(window, cx);
                            });
                        }
                    })
                    .ok();
                }
                cx.background_executor()
                    .timer(Duration::from_millis(16))
                    .await;
            }
        })
        .detach();

        Self {
            input,
            list,
            fav,
            hwnd,
            is_visible,
        }
    }

    fn get_item(&self, ix: usize, cx: &Context<Self>) -> Option<Item> {
        self.list.read(cx).filtered.get(ix).cloned()
    }

    fn select_next(&mut self, _: &SelectNext, _window: &mut Window, cx: &mut Context<Self>) {
        self.list.update(cx, |list, cx| {
            list.select_next();
            cx.notify();
        });
    }

    fn select_prev(&mut self, _: &SelectPrev, _window: &mut Window, cx: &mut Context<Self>) {
        self.list.update(cx, |list, cx| {
            list.select_prev();
            cx.notify();
        });
    }

    fn launch_item(&self, item: &Item) -> bool {
        let Some(command) = &item.running_command else {
            return false;
        };
        match std::process::Command::new(&command.command)
            .args(&command.args)
            .spawn()
        {
            Ok(_) => true,
            Err(e) => {
                eprintln!("Failed to spawn command: {}", e);
                false
            }
        }
    }

    fn confirm(&mut self, _: &Confirm, _window: &mut Window, cx: &mut Context<Self>) {
        let list = self.list.read(cx);
        let Some(ix) = list.selected_index else {
            // No item selected — check if the input is a bang shortcut
            let input = self.input.read(cx).value();
            if let Some((bang, query)) = crate::bangs::parse_bang(&input) {
                let url = crate::bangs::search_url(bang, query);
                let _ = webbrowser::open(&url);
                unsafe {
                    let _ = ShowWindow(self.hwnd, SW_HIDE);
                }
                self.is_visible = false;
            }
            return;
        };
        let Some(item) = list.filtered.get(ix) else {
            return;
        };

        if item.kind == Kind::Search {
            let _ = webbrowser::open(&item.id);
            unsafe {
                let _ = ShowWindow(self.hwnd, SW_HIDE);
            }
            self.is_visible = false;
            return;
        }

        if self.launch_item(item) {
            unsafe {
                let _ = ShowWindow(self.hwnd, SW_HIDE);
            }
            self.is_visible = false;
        }
    }

    fn cancel(&mut self, _: &Cancel, _window: &mut Window, _cx: &mut Context<Self>) {
        unsafe {
            let _ = ShowWindow(self.hwnd, SW_HIDE);
        }
        self.is_visible = false;
    }

    fn toggle_favorite(
        &mut self,
        _: &ToggleFavorite,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(selected_index) = self.list.read(cx).selected_index else {
            return;
        };

        let Some(item) = self.list.read(cx).filtered.get(selected_index) else {
            return;
        };

        let id = item.id.clone();

        if !self.fav.read(cx).is_favorite(&id) {
            self.add_to_favorite_by_index(selected_index, cx);
        } else {
            self.remove_from_favorite_by_id(&id, cx);
        }
    }

    fn add_to_favorite_by_index(&self, ix: usize, cx: &mut Context<Self>) {
        let item = {
            let list = self.list.read(cx);
            match list.filtered.get(ix) {
                Some(item) => item.clone(),
                None => return,
            }
        };

        self.fav.update(cx, |fav, cx| {
            fav.add_favorite(item, cx);
        });
    }

    fn remove_from_favorite_by_id(&mut self, id: &str, cx: &mut Context<Self>) {
        self.fav.update(cx, |fav, cx| {
            fav.remove_favorite(id, cx);
        });
    }

    fn focus_search(&mut self, _: &FocusSearch, window: &mut Window, cx: &mut Context<Self>) {
        self.input.update(cx, |input, cx| {
            input.focus(window, cx);
        });
    }
}

impl Render for LauncherState {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let has_favorites = !self.fav.read(cx).favorites.is_empty();

        div()
            .on_action(cx.listener(Self::select_next))
            .on_action(cx.listener(Self::select_prev))
            .on_action(cx.listener(Self::confirm))
            .on_action(cx.listener(Self::cancel))
            .on_action(cx.listener(Self::toggle_favorite))
            .on_action(cx.listener(Self::focus_search))
            .on_key_down(cx.listener(|this, e: &KeyDownEvent, _window, cx| {
                if e.keystroke.modifiers.control {
                    if let Some(digit) = e.keystroke.key.chars().next().and_then(|c| c.to_digit(10))
                    {
                        if digit >= 1 && digit <= 6 {
                            let ix = digit.saturating_sub(1) as usize;
                            let item = this.fav.read(cx).favorites.get(ix).cloned();
                            if let Some(item) = item {
                                if this.launch_item(&item) {
                                    unsafe {
                                        let _ = ShowWindow(this.hwnd, SW_HIDE);
                                    }
                                    this.is_visible = false;
                                }
                            }
                        }
                    }
                }
            }))
            .bg(cx.theme().background)
            .rounded_xl()
            .p_2()
            .shadow_lg()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .h_full()
            .w_full()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .w_full()
                    .h_full()
                    .gap_3()
                    .child(
                        Input::new(&self.input)
                            .prefix(Icon::new(IconName::Search))
                            .suffix(kbd::Kbd::new(Keystroke::parse("ctrl-k").unwrap()))
                            .cleanable(true),
                    )
                    .when(has_favorites, |this| this.child(self.fav.clone()))
                    .child(self.list.clone()),
            )
    }
}
