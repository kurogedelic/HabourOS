# Harbour OS

Harbour OS is a small retro virtual operating system built with Rust and SDL2.
It presents a keyboard-first desktop, a persistent virtual filesystem, and a
set of practical built-in applications in a crisp CRT/workstation style.

![OS](https://img.shields.io/badge/OS-Harbour_0.1-amber)
![Rust](https://img.shields.io/badge/Rust-2021-orange)
![SDL2](https://img.shields.io/badge/SDL2-0.38-blue)

## Current Status

Harbour OS is currently a usable desktop shell prototype. The main executable is
`harbour_shell`, and the application can be launched with `./run.sh` or
`cargo run`.

Implemented pieces include:

- Desktop launcher with a responsive icon grid.
- Opening and closing scale animations from the selected desktop icon.
- Shared top bar, window frame, command bar, and pixel pointer cursor.
- Keyboard operation throughout the shell, plus pointer support on the desktop
  and file browser.
- YAML-backed virtual filesystem snapshots.
- Host import/export from the virtual filesystem.
- Runtime-tinted PNG assets and a YAML theme palette.
- Latin and Japanese BDF font rendering, including Japanese text entered via the
  host OS IME.

## Built-in Apps

| App | Status | Notes |
| --- | --- | --- |
| TextEditor | Implemented | Multi-line editing, scrolling, cursor movement, save/open through VFS, undo, Japanese text display/input. |
| Files | Implemented | VFS browser with folder navigation, file open, new file/folder, delete, rename, host export, and host import. |
| Clock | Implemented | Current time, date, and day display. |
| Dictionary | Implemented | Small built-in searchable sample dictionary. |
| Calculator | Implemented | Minimal arithmetic calculator with decimal support. |
| Snake | Implemented | Monochrome keyboard-controlled mini game. |

## Design Direction

Harbour OS favors a first useful computer over a decorative UI demo. The design
is intentionally restrained:

- Black background with a small amber palette.
- Pixel-oriented rendering with no anti-aliased UI chrome.
- White/transparent PNG assets tinted at runtime by SDL2.
- CRT terminal and workstation mood rather than early Mac imitation alone.
- Common app command bars showing the active keys for the current screen.
- Keyboard-first operation with pointer input where it improves navigation.

The default theme lives at `assets/themes/default.yaml`.

## Virtual Filesystem

The VFS is an in-memory Unix-style tree persisted as a YAML snapshot. By default,
Harbour OS reads and writes `harbour_vfs.yaml` in the current directory. If the
snapshot does not exist or cannot be loaded, the app starts from sample content:

```text
/
|-- README.txt
|-- notes/
|   |-- welcome.txt
|   `-- todo.txt
`-- docs/
    `-- manual.txt
```

Set a custom snapshot path with `HARBOUR_VFS_PATH`:

```bash
HARBOUR_VFS_PATH=/home/pi/harbour/harbour_vfs.yaml ./run.sh
```

The Files app can also exchange data with host directories:

```bash
HARBOUR_IMPORT_DIR=/home/pi/harbour/import \
HARBOUR_EXPORT_DIR=/home/pi/harbour/export \
./run.sh
```

## Controls

### Global

| Key | Action |
| --- | --- |
| `F11` | Toggle fullscreen desktop mode. |
| `Ctrl+Q` | Quit Harbour OS. |

### Desktop

| Key / Input | Action |
| --- | --- |
| `Left` / `Right` / `Up` / `Down` | Move icon selection. |
| `Home` / `End` | Select the first or last icon. |
| `Enter` | Launch the selected app. |
| Double-click | Launch the clicked app. |

### TextEditor

| Key | Action |
| --- | --- |
| Printable text | Insert text, including host IME text. |
| `Enter` | New line. |
| `Backspace` / `Delete` | Delete text. |
| `Left` / `Right` / `Up` / `Down` | Move the cursor. |
| `Home` / `End` | Move to the start or end of the line. |
| `PageUp` / `PageDown` | Page through the document. |
| `Tab` | Insert two spaces. |
| `Ctrl+S` | Save to the VFS. |
| `Ctrl+Z` | Undo. |
| `Ctrl+B` | Switch to Files. |
| `Esc` | Show the quit confirmation dialog. |

### Files

| Key / Input | Action |
| --- | --- |
| `Left` / `Right` / `Up` / `Down` | Move selection. |
| `Home` / `End` | Select the first or last entry. |
| `Enter` | Open a file or enter a directory. |
| `Backspace` | Go to the parent directory. |
| `N` | Create a new file. |
| `Shift+N` | Create a new directory. |
| `R` | Rename the selected entry. |
| `D` | Delete the selected entry after confirmation. |
| `E` | Export the selected entry to `HARBOUR_EXPORT_DIR`. |
| `I` | Import files from `HARBOUR_IMPORT_DIR` into the current VFS directory. |
| `Esc` | Return to the desktop. |
| Double-click | Open the clicked entry. |
| Mouse wheel | Scroll the icon grid. |

### Clock

| Key | Action |
| --- | --- |
| `Esc` | Return to the desktop. |

### Dictionary

| Key | Action |
| --- | --- |
| Printable text | Enter a search term. |
| `Enter` | Search. |
| `Backspace` | Delete one character. |
| `Esc` | Return to the desktop. |

### Calculator

| Key | Action |
| --- | --- |
| Numbers / `+` / `-` / `*` / `/` / `.` | Enter an expression. |
| `Enter` | Calculate. |
| `Backspace` | Delete one character. |
| `C` | Clear. |
| `Esc` | Return to the desktop. |

### Snake

| Key | Action |
| --- | --- |
| `Left` / `Right` / `Up` / `Down` | Turn. |
| `R` / `Space` | Restart after game over. |
| `Esc` | Return to the desktop. |

## Requirements

- Rust 2021 toolchain.
- SDL2 with SDL2_image support.
- Python 3 and Pillow only if regenerating PNG assets.

On macOS with Homebrew:

```bash
brew install sdl2 sdl2_image
```

## Running

From this repository:

```bash
./run.sh
```

Or run Cargo directly:

```bash
cargo run
```

Build without running:

```bash
cargo build
cargo build --release
```

The run script sets Homebrew-style SDL2 library paths before building and
launching `harbour_shell`.

## Runtime Configuration

These environment variables are read at startup:

| Variable | Purpose |
| --- | --- |
| `HARBOUR_WIDTH` | Initial window width. Values below 160 are ignored. |
| `HARBOUR_HEIGHT` | Initial window height. Values below 160 are ignored. |
| `HARBOUR_FULLSCREEN` | Start in desktop fullscreen mode when set to `1`, `true`, `yes`, or `on` variants. |
| `HARBOUR_VFS_PATH` | YAML snapshot path for the virtual filesystem. |
| `HARBOUR_IMPORT_DIR` | Host directory used by Files import. Defaults to `harbour_import`. |
| `HARBOUR_EXPORT_DIR` | Host directory used by Files export. Defaults to `harbour_export`. |

Examples:

```bash
HARBOUR_WIDTH=800 HARBOUR_HEIGHT=480 ./run.sh
HARBOUR_FULLSCREEN=1 ./run.sh
HARBOUR_VFS_PATH=/home/pi/harbour/harbour_vfs.yaml ./run.sh
```

## Asset Generation

Most UI assets are white/transparent PNGs and are colorized at runtime. Regenerate
them with:

```bash
python3 tools/gen_widgets.py
python3 tools/gen_widgets.py --status
python3 tools/gen_widgets.py --list
```

The Unicode to JIS X 0208 conversion table is generated separately:

```bash
python3 tools/gen_jis_table.py > src/jis_table.rs
```

## Project Layout

```text
src/
|-- main.rs         # SDL2 entry point, shell state machine, app transitions
|-- lib.rs          # Shared library root
|-- app.rs          # Built-in app trait and app result API
|-- input.rs        # OS-level keyboard and pointer input records
|-- desktop.rs      # Desktop launcher and icon grid
|-- text_editor.rs  # TextEditor implementation
|-- file_browser.rs # Files app and VFS import/export actions
|-- vfs.rs          # YAML-backed virtual filesystem
|-- dialog.rs       # Confirmation dialog
|-- scale_anim.rs   # Opening and closing scale animations
|-- harbour.rs      # Window model
|-- seashore.rs     # Top bar and shell drawing helpers
|-- calculator.rs   # Calculator app
|-- clock.rs        # Clock app
|-- dictionary.rs   # Dictionary app
|-- snake.rs        # Snake app
|-- font.rs         # Font system and fallback handling
|-- bdf.rs          # Runtime BDF loader
|-- jis.rs          # Unicode to JIS X 0208 conversion
|-- jis_table.rs    # Generated conversion table
|-- theme.rs        # Theme loading and named colors
|-- ui.rs           # Pixel drawing helpers
`-- assets.rs       # PNG asset loading and runtime tinting

assets/
|-- images/         # Icons, status images, and widget PNGs
`-- themes/         # YAML theme files

fonts/              # M+ BDF fonts
tools/              # Asset and table generation scripts
```

## Test and Utility Binaries

The repository includes small development binaries under `src/bin/`:

```bash
cargo run --bin test_text
cargo run --bin test_widget
cargo run --bin test_png
cargo run --bin test_editor
```

Use `cargo test` for the Rust unit tests.
