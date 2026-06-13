# Repository Guidelines

## Project Structure & Module Organization

PopMax is a Rust 2024 GPUI desktop launcher for Windows.

- `src/main.rs` initializes GPUI, themes, key bindings, and the main window.
- `src/launcher.rs` owns the launcher view, search input, actions, and command execution.
- `src/list.rs` renders and manages the virtualized results list.
- `src/scanner.rs` scans Start Menu `.lnk` files and builds launcher items.
- `src/windows_icons.rs` extracts Windows app icons and caches renderable images.
- `src/types.rs` defines shared item, command, and icon types.
- `src/load_themes.rs` loads GPUI component themes from `src/themes/*.json`.

There is no dedicated `tests/` directory yet. Add module tests with `#[cfg(test)]` or integration tests under `tests/` when behavior crosses module boundaries.

## Build, Test, and Development Commands

- `cargo run` builds and launches the app locally.
- `cargo build` compiles the app without running it.
- `cargo check` performs a fast type check during development.
- `cargo test` runs unit and integration tests.
- `cargo fmt` formats Rust code with `rustfmt`.
- `cargo clippy --all-targets --all-features` runs stricter lint checks.

First builds may take longer because GPUI and `gpui-component` are Git dependencies.

## Coding Style & Naming Conventions

Use standard Rust formatting with four-space indentation. Use `snake_case` for functions, variables, and modules; `PascalCase` for structs, enums, and GPUI actions.

Keep ownership boundaries clear: `launcher.rs` coordinates UI actions and windows, `list.rs` owns list selection and rendering, `scanner.rs` discovers apps, and `windows_icons.rs` handles Windows API details. Call `cx.notify()` after GPUI entity mutations that affect rendering.

## Testing Guidelines

Use Rust’s built-in test framework. Name tests after observable behavior, for example `select_next_wraps_to_first_item` or `empty_filter_clears_selection`.

Prioritize tests for scanner behavior, filtering, selection wrapping, icon cache paths, and command metadata parsing. Run `cargo test` before submitting changes.

## Commit & Pull Request Guidelines

Existing commits use short, imperative messages such as `add themes, update list` and `working demo`. Keep commits scoped to one logical change.

Pull requests should include a brief description, manual test steps such as `cargo run` or `cargo test`, and screenshots for visible UI changes. Mention changes to keyboard navigation, filtering, themes, icon extraction, or command spawning.

## Security & Configuration Tips

Avoid shell interpolation when launching apps. Prefer `std::process::Command::new(...).args(...)` with explicit arguments. Keep native Windows handles local to `windows_icons.rs`, clean them up promptly, and store cached image paths in `Item` rather than raw handles.
