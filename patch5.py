import re

with open('src/components/theme_switcher.rs', 'r', encoding='utf-8') as f:
    content = f.read()

content = "use gpui_component::scroll::ScrollableElement;\n" + content

with open('src/components/theme_switcher.rs', 'w', encoding='utf-8') as f:
    f.write(content)
