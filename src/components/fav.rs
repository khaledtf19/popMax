use std::path::PathBuf;

use crate::{
    types::Item,
    utils::{asset_path, get_load_path},
};
use gpui::*;
use gpui_component::{
    ActiveTheme, IconName, Sizable,
    button::{Button, ButtonVariants},
    h_flex,
    kbd::Kbd,
    label::Label,
};

pub struct Fav {
    pub favorites: Vec<Item>,
}

impl Fav {
    pub fn new() -> Self {
        let favorites = Self::load_favorites();
        Self { favorites }
    }

    fn load_favorites() -> Vec<Item> {
        let Some(path) = get_load_path() else {
            return Vec::new();
        };
        let path = path.join("favorites.json");
        if !path.exists() {
            return Vec::new();
        }
        let Ok(content) = std::fs::read_to_string(&path) else {
            return Vec::new();
        };
        serde_json::from_str(&content).unwrap_or_default()
    }

    pub fn save_favorites_to_disk(&self) {
        let Some(path) = get_load_path() else {
            return;
        };
        let path = path.join("favorites.json");
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(content) = serde_json::to_string(&self.favorites) {
            let _ = std::fs::write(&path, content);
        }
    }

    pub fn add_favorite(&mut self, item: Item, cx: &mut Context<Self>) {
        if self.favorites.len() >= 6 {
            return;
        }
        if self.is_favorite(&item.id) {
            return;
        }
        self.favorites.push(item);
        self.save_favorites_to_disk();
        cx.notify();
    }

    pub fn is_favorite(&self, id: &str) -> bool {
        self.favorites.iter().any(|fav| fav.id == id)
    }

    pub fn remove_favorite(&mut self, id: &str, cx: &mut Context<Self>) {
        self.favorites.retain(|fav| fav.id != id);
        self.save_favorites_to_disk();
        cx.notify();
    }
}

impl Render for Fav {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div().grid().grid_cols(6).gap_2().children(
            self.favorites
                .iter()
                .enumerate()
                .map(|(ix, item)| {
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

                    let item_clone = item.clone();

                    div()
                        .id(format!("fav-item-{}", ix))
                        .flex()
                        .flex_col()
                        .items_center()
                        .gap_1()
                        .px_2()
                        .py_2()
                        .rounded_lg()
                        .bg(cx.theme().secondary)
                        .border_1()
                        .border_color(cx.theme().border)
                        .hover(|this| this.bg(cx.theme().secondary_hover))
                        .cursor_pointer()
                        .on_click(cx.listener(move |_this, _, _window, _cx| {
                            // this.(&item_clone, window);
                        }))
                        .child(icon.size_8())
                        .child(
                            h_flex().absolute().top_0().right_0().child(
                                Button::new(format!("remove-fav-{}", ix))
                                    .ghost()
                                    .icon(IconName::Close)
                                    .compact()
                                    .with_size(gpui_component::Size::XSmall)
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        cx.stop_propagation();
                                        this.remove_favorite(&item_clone.id, cx);
                                    })),
                            ),
                        )
                        .child(
                            Label::new(item.name.clone())
                                .text_sm()
                                .text_ellipsis()
                                .whitespace_nowrap(),
                        )
                        .child(Kbd::new(
                            Keystroke::parse(format!("ctrl-{}", ix + 1).as_str()).unwrap(),
                        ))
                })
                .collect::<Vec<_>>(),
        )
    }
}
