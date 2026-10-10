import re

with open('src/components/theme_switcher.rs', 'r', encoding='utf-8') as f:
    content = f.read()

replacement = '''            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_3()
                    .child(
                        Button::new("back_btn")
                            .label("Back")
                            .on_click(|_event, window, _cx| {
                                window.remove_window();
                            })
                    )
                    .child(
                        div()
                            .text_xl()
                            .font_weight(FontWeight::BOLD)
                            .child("Theme Switcher"),
                    )
            )'''

content = re.sub(
    r'\.child\(\s*div\(\)\s*\.text_xl\(\)\s*\.font_weight\(FontWeight::BOLD\)\s*\.child\("Theme Switcher"\),\s*\)',
    replacement,
    content,
    flags=re.MULTILINE
)

with open('src/components/theme_switcher.rs', 'w', encoding='utf-8') as f:
    f.write(content)
