import re

with open('src/launcher.rs', 'r', encoding='utf-8') as f:
    content = f.read()

replacement = '''                        h_flex()
                            .w_full()
                            .gap_2()
                            .child(
                                Input::new(&self.input)
                                    .prefix(Icon::new(IconName::Search))
                                    .suffix(kbd::Kbd::new(Keystroke::parse("ctrl-k").unwrap()))
                                    .cleanable(true),
                            )
                            .child(
                                gpui_component::button::Button::new("theme_btn")
                                    .icon(IconName::Settings)
                                    .on_click(|_event, _window, cx| {
                                        crate::components::theme_switcher::open(cx);
                                    })
                            ),'''

content = content.replace('''                        Input::new(&self.input)
                            .prefix(Icon::new(IconName::Search))
                            .suffix(kbd::Kbd::new(Keystroke::parse("ctrl-k").unwrap()))
                            .cleanable(true),''', replacement)

with open('src/launcher.rs', 'w', encoding='utf-8') as f:
    f.write(content)
