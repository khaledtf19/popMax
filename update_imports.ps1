$files = Get-ChildItem -Path "D:\dev\learing\PopMax\src" -Recurse -Filter *.rs
foreach ($file in $files) {
    $content = Get-Content $file.FullName -Raw
    $new_content = $content -replace 'gpui_component::', 'gpui_kit::component::'
    $new_content = $new_content -replace 'gpui_component_assets::', 'gpui_kit::assets::'
    $new_content = $new_content -replace 'gpui_platform::', 'gpui_kit::platform::'
    $new_content = $new_content -replace 'use gpui::\*;', 'use gpui_kit::*;'
    $new_content = $new_content -replace 'use gpui::prelude::', 'use gpui_kit::prelude::'
    $new_content = $new_content -replace 'gpui::', 'gpui_kit::'
    if ($content -ne $new_content) {
        Set-Content $file.FullName $new_content
    }
}
