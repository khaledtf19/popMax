use crate::{
    list::LauncherDelegate,
    scanner::run_scan,
    types::{Item, Kind, RunCommand},
};
use gpui::*;
use gpui_component::{
    input::{Input, InputEvent, InputState},
    list::{List, ListState},
};

pub struct LauncherState {
    input: Entity<InputState>,
    list: Entity<ListState<LauncherDelegate>>,
}

actions!(launcher, [SelectNext, SelectPrev, Confirm, Cancel]);

impl LauncherState {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let input = cx.new(|cx| InputState::new(window, cx));
        let items = run_scan();

        let list = cx.new(|cx| ListState::new(LauncherDelegate::new(items), window, cx));

        cx.subscribe_in::<_, InputEvent>(&input, window, |view, state, _event, _window, cx| {
            let input = state.read(cx).value();
            if input.is_empty() {
                view.list.update(cx, |list, cx| {
                    let delegate = list.delegate_mut();
                    delegate.filtered = delegate.items.clone();
                    delegate.selected_index = None;
                    cx.notify();
                });
                return;
            }
            view.list.update(cx, |list, cx| {
                let filtered = list
                    .delegate()
                    .items
                    .iter()
                    .filter(|item| {
                        item.name
                            .to_lowercase()
                            .contains(input.to_lowercase().as_str())
                    })
                    .cloned()
                    .collect::<Vec<_>>();

                let delegate = list.delegate_mut();
                delegate.filtered = filtered;
                delegate.selected_index = None;
                cx.notify();
            });
        })
        .detach();

        Self { input, list }
    }

    fn select_next(&mut self, _: &SelectNext, window: &mut Window, cx: &mut Context<Self>) {
        self.list.update(cx, |list, cx| {
            let ix = list.delegate().select_next();
            list.set_selected_index(ix, window, cx);
            list.scroll_to_selected_item(window, cx);
        });
    }

    fn select_prev(&mut self, _: &SelectPrev, window: &mut Window, cx: &mut Context<Self>) {
        self.list.update(cx, |list, cx| {
            let ix = list.delegate().select_prev();
            list.set_selected_index(ix, window, cx);
            list.scroll_to_selected_item(window, cx);
        });
    }

    fn confirm(&mut self, _: &Confirm, window: &mut Window, cx: &mut Context<Self>) {
        let list = self.list.read(&cx).delegate();
        let Some(ix) = list.selected_index else {
            return;
        };
        let Some(item) = list.filtered.get(ix.row) else {
            return;
        };
        let Some(command) = &item.running_command else {
            return;
        };
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

    fn cancel(&mut self, _: &Cancel, window: &mut Window, _cx: &mut Context<Self>) {
        window.remove_window();
    }
}

impl Render for LauncherState {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .on_action(cx.listener(Self::select_next))
            .on_action(cx.listener(Self::select_prev))
            .on_action(cx.listener(Self::confirm))
            .on_action(cx.listener(Self::cancel))
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .h_full()
            .w_full()
            .child(
                div()
                    .p_2()
                    .flex()
                    .flex_col()
                    .w_full()
                    .h_full()
                    .gap_5()
                    .child(Input::new(&self.input).cleanable(true))
                    .child(List::new(&self.list).bg(gpui::rgb(0x0a0a0a)).rounded_md()),
            )
    }
}
