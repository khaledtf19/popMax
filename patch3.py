import re

with open('src/components/theme_switcher.rs', 'r', encoding='utf-8') as f:
    content = f.read()

content = content.replace(".overflow_y_scroll()", ".overflow_scroll()")

with open('src/components/theme_switcher.rs', 'w', encoding='utf-8') as f:
    f.write(content)
