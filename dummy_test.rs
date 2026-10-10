use gpui::*;
use gpui_component::*;

pub struct Dummy;

impl Render for Dummy {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let _ = cx.open_window(WindowOptions::default(), |window, cx| cx.new(|cx| Dummy));
        div()
    }
}
