# PopMax

A fast, keyboard-driven Windows application launcher built with Rust and [GPUI](https://github.com/zed-industries/zed).

Search your Start Menu apps, launch with one keystroke, use bang shortcuts for web searches, and pin favorites — all from a sleek popup window.

![screenshot](https://placehold.co/600x500/1a1b26/c0caf5?text=PopMax)

## Features

- **App launcher** — scans Start Menu `.lnk` files and presents them in a searchable, virtualized list
- **Instant search** — filter-as-you-type with fuzzy name matching
- **Bang shortcuts** — type `!g query` to search Google, `!y query` for YouTube, and more
- **Favorites** — pin up to 6 apps, launch with `Ctrl+1` through `Ctrl+6`
- **System tray** — background operation with right-click context menu
- **Global hotkey** — `Alt+Space` toggles the launcher from anywhere
- **Themes** — 21 built-in themes (Tokyo Night, Catppuccin, Gruvbox, etc.)
- **App icons** — extracts and caches Windows app icons for a polished look
- **Single instance** — prevents duplicate processes, brings existing window to front

### Bang Shortcuts

| Bang | Site |
|------|------|
| `!g` | Google |
| `!y` | YouTube |
| `!w` | Wikipedia |
| `!gh` | GitHub |
| `!r` | Reddit |
| `!a` | Amazon |
| `!s` | Stack Overflow |
| `!m` | MDN |
| `!x` | X / Twitter |

## Installation

### From the installer

1. Download `PopMax-Setup.exe` from the latest release.
2. Run the installer — it places PopMax in `%LOCALAPPDATA%\Programs\PopMax` (per-user, no UAC prompt) and adds a desktop shortcut.
3. Launch PopMax. The tray icon appears in the notification area.
4. Press `Alt+Space` to open the search window.

### From source

```powershell
git clone https://github.com/your-username/PopMax.git
cd PopMax
cargo run --release
```

The first build takes longer because GPUI and `gpui-component` are fetched from Git.

## Usage

| Key | Action |
|-----|--------|
| `Alt+Space` | Toggle launcher window |
| `Type` | Filter apps by name |
| `↑` / `↓` | Navigate list |
| `Enter` | Launch selected app |
| `Ctrl+D` | Toggle favorite on selected item |
| `Ctrl+1`..`6` | Launch favorite #1–6 |
| `Ctrl+K` | Focus search bar |
| `Escape` | Hide window |

### Bang search

Type a bang prefix, space, then your query:

```
!g rust async await        → opens Google search
!y lofi beats              → opens YouTube search
!w quantum mechanics       → opens Wikipedia
```

The list shows a virtual "Search Google for: rust async await" item — press `Enter` to open in your default browser.

## Project Structure

```
src/
├── main.rs             # App entry, single instance, key bindings
├── launcher.rs         # Main view, search input, confirm/cancel actions
├── bangs.rs            # Bang shortcut definitions and URL generation
├── types.rs            # Item, Kind, RunCommand types
├── tray.rs             # System tray icon and Win32 message loop
├── hotkey.rs           # Global hotkey (Alt+Space) registration
├── scanner.rs          # Start Menu scanning, icon extraction orchestration
├── windows_icons.rs    # Win32 icon extraction (ExtractIconExW, DrawIconEx)
├── load_themes.rs      # GPUI component theme loader
├── utils.rs            # Path helpers (exe dir, asset path, config dir)
├── components/
│   ├── mod.rs
│   ├── list.rs         # Virtualized results list with favorites
│   └── fav.rs          # Favorites bar with launch-on-click
└── themes/             # 21 JSON theme files
assets/
├── PopMaxIcon.ico      # App and tray icon
└── audio/
    └── popup_Sound.mp3
installer/
└── PopMax.iss          # Inno Setup installer script
```

## Building

```powershell
cargo check          # Fast type-check
cargo build          # Debug build
cargo build --release # Release build
cargo test           # Run tests
cargo clippy         # Lint checks
```

The `build.rs` script embeds `PopMaxIcon.ico` as the app window icon via `embed-resource`.

## Configuration

- **Favorites** are persisted to `%LOCALAPPDATA%\PopMax\favorites.json`
- **Themes** live in `src/themes/` — the default theme is Tokyo Night
- **Icon cache** is stored in `%LOCALAPPDATA%\PopMax\icons\`

## Technology

- **[GPUI](https://github.com/zed-industries/zed)** — Rust GUI framework (from Zed)
- **[gpui-component](https://github.com/longbridge/gpui-component)** — UI component library (Input, List, Button, etc.)
- **[windows](https://crates.io/crates/windows)** — Official Windows API bindings
- **[rayon](https://crates.io/crates/rayon)** — Parallel icon extraction
- **[Inno Setup](https://jrsoftware.org/isinfo.php)** — Windows installer
- **Win32 APIs** — `Shell_NotifyIconW`, `ExtractIconExW`, `DrawIconEx`, `RegisterHotKey`, `CreateMutexW`
 
## License

This project is licensed under the MIT License. See the LICENSE file for details.
