use crate::{list::LauncherList, scanner::run_scan};
use gpui::*;
use gpui_component::{
    ActiveTheme, Icon, IconName, Sizable,
    input::{Input, InputEvent, InputState},
};

pub struct LauncherState {
    input: Entity<InputState>,
    list: Entity<LauncherList>,
}

actions!(launcher, [SelectNext, SelectPrev, Confirm, Cancel]);

impl LauncherState {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let input = cx.new(|cx| InputState::new(window, cx).placeholder("Search..."));
        let items = run_scan();

        let list = cx.new(|_cx| LauncherList::new(items));

        cx.subscribe_in::<_, InputEvent>(&input, window, |view, state, _event, _window, cx| {
            let input = state.read(cx).value();
            view.list.update(cx, |list, cx| {
                list.update_filtered(&input, cx);
            });
        })
        .detach();

        Self { input, list }
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

    fn confirm(&mut self, _: &Confirm, window: &mut Window, cx: &mut Context<Self>) {
        let list = self.list.read(&cx);
        let Some(ix) = list.selected_index else {
            return;
        };
        let Some(item) = list.filtered.get(ix) else {
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
                    .gap_1()
                    .child(
                        Input::new(&self.input)
                            .prefix(Icon::new(IconName::Search).small())
                            .cleanable(true),
                    )
                    .child(self.list.clone()),
            )
    }
}
