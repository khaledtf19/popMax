import re

with open('src/launcher.rs', 'r', encoding='utf-8') as f:
    content = f.read()

content = content.replace("h_flex()", "div().flex().w_full().gap_2().items_center()")

with open('src/launcher.rs', 'w', encoding='utf-8') as f:
    f.write(content)
