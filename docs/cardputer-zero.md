# Cardputer Zero: experimental Linux port

> **Not verified on a physical Cardputer Zero.** This is an initial ARM64
> build and SDL2/Wayland launch path, not a complete framebuffer-native port
> and not a replacement operating-system image.

## Target and existing code

- M5Stack **Cardputer Zero / Zero Lite**, Raspberry Pi CM0, AArch64 Linux
  (not the ESP32-S3 Cardputer / Cardputer ADV).
- LCD: **320 × 170**, no touch; keyboard: **46 keys**.
- HarbourOS remains an **SDL2 application**. All existing built-in apps, VFS,
  drawing and rendering stay in Rust, without a separate GUI implementation.
- Compact UI: a **Classic Mac System 6/7-inspired desktop** at the native
  320×170 resolution. Its four warm amber tones are dark ink `#382209`,
  secondary `#B8945C`, pale amber desktop `#F0D6A0`, and light window
  paper `#FFE8B8`. A 16px menu strip sits at the top, and six 20px icon
  assets are arranged on the right in two columns by three rows.
  The selected label inverts to dark ink with light lettering; the left side
  remains a usable, uncluttered workspace. App windows retain a thin Mac-style
  frame. On this small screen, app frames fill the width and every pixel
  below the 16px menu strip (no outer margin). The 1px window outlines
  and title stripes are aligned around the measured system-font title, and the
  12px anchor is optically centered in the menu bar. Icons and fonts are drawn
  pixel-sharp without bitmap scaling. Press Enter to open the selected icon,
  or use 1–6 for direct launches. The system menu begins with an original
  12×12 pixel Harbour anchor mark.
- Compact pointer: an external USB mouse works through SDL2. The on-screen
  12×16 pointer is drawn in black with a white interior (no displaced
  shadow). The Web simulator hides the host browser/macOS cursor when hovering
  over the LCD, leaving only the HarbourOS cursor. Without a mouse,
  press **Ctrl+M** to toggle pointer mode, use **W/A/S/D** (or arrows)
  to move 5 pixels per keypress, **Enter / Space** to click, and **Esc**
  to return to ordinary keyboard operation. The pointer is separate from
  the normal desktop icon-selection navigation; text input is unchanged
  unless pointer mode is explicitly enabled. Physical USB Host input is
  unverified on an actual Cardputer Zero.
- System typography: compact menus and window titles use a chunky,
  proportionally spaced **Chicago-inspired** treatment of HarbourOS's
  existing M+ bitmap glyphs (1px heavier strokes). Icon captions, application
  bodies and Japanese text keep their existing M+/JIS rendering.
  This does **not** distribute Apple's original Chicago font.
- The handheld launcher selects `assets/themes/cardputer-zero.yaml` via
  `HARBOUR_THEME`, keeping the desktop default theme unchanged.
- Initial goal: launch the desktop and navigate to one app using the built-in
  keyboard, then verify text input and file persistence.

Official M5 hardware docs: <https://docs.m5stack.com/en/CardputerZero>
Keyboard instructions: <https://docs.m5stack.com/ja/compute/cardputerzero/usage>
Community launcher and framebuffer reference:
<https://github.com/CardputerZero/launcher>

## Browser simulator (320 × 170)

The provided flat Cardputer Zero HTML mockup is now a **working browser
front-end** at `web/cardputer-zero.html`. It uses the same Rust HarbourOS
SDL2/Emscripten bundle as the existing 3D simulator, but shows it directly on
the screen inside a flat Cardputer Zero enclosure. No screenshot substitution
or second simulated operating system is used.

- Native 320 × 170 drawing (pixelated nearest-neighbor at displayed size).
- Four-row, 46-key flat keyboard with clickable keycaps.
- FN is sticky for one key: `FN + F` Up, `FN + Z` Left,
  `FN + X` Down, `FN + C` Right.
- The up-arrow and Enter keys are clickable; Ctrl is sticky for one command.
- The two round auxiliary buttons emulate Left and Right navigation.
- The orange power-shaped control restarts the browser simulator.
- Browser physical keyboard input goes directly into the existing SDL2 runtime.
- VFS is persisted through IndexedDB, matching the 3D simulator.
- This tests layout and browser keyboard behavior, **not** the device's
  framebuffer, Linux keyboard driver, or actual render speed.

Build and run both simulators locally:

```sh
./web/build.sh
python3 -m http.server 8080 --directory web
# Open http://localhost:8080/cardputer-zero.html
```

This PR also builds a runnable `harbour-os-cardputer-zero-web` artifact
under **Cardputer Zero Web simulator preview** in GitHub Actions. Extract the
artifact and serve the directory with a local HTTP server. The site's direct
link `https://kurogedelic.github.io/HabourOS/cardputer-zero.html` becomes
available **only after the PR is merged to main and Pages redeploys**.

The existing `web/index.html` 3D simulator is unchanged.

## Get an ARM64 binary

On this repo's GitHub Actions page, run **Cardputer Zero ARM64
(experimental)** via workflow dispatch, or download the artifact from the PR.
It builds on an ARM64 runner in Debian 12 (Bookworm) and packages a release
binary with `assets/`, `fonts/`, and `run-cardputer-zero.sh`.

Extract the `harbour-os-cardputer-zero-aarch64.tar.gz` archive into a directory
on the device. Keep the executable and asset directories together.

Alternatively, build on the device (512 MiB RAM; a native release build might
run out of memory):

```sh
sudo apt-get update
sudo apt-get install pkg-config libsdl2-dev libsdl2-image-dev build-essential
# rustup / Rust toolchain must already be installed
CARGO_BUILD_JOBS=1 cargo build --locked --release --bin harbour_shell
```

## First launch

The stock Cardputer Zero uses a Linux framebuffer (often `/dev/fb0`).
**SDL2 has no SDL1-style `fbcon` backend**. This experimental version instead
tries a Wayland session, if one is running; it does **not** open `/dev/fb0`
itself. Some community SDL2 demos have failed to start on the device, so a
successful build is not proof of hardware support.

From a shell belonging to the running Wayland session:

```sh
chmod +x ./run-cardputer-zero.sh
./run-cardputer-zero.sh
```

When connecting via SSH, first determine the graphical user's UID and active
Wayland socket. For example, **only if the graphical session uses these values**:

```sh
XDG_RUNTIME_DIR=/run/user/1000 WAYLAND_DISPLAY=wayland-0 ./run-cardputer-zero.sh
```

The script defaults to `SDL_VIDEODRIVER=wayland`, `HARBOUR_WIDTH=320`,
`HARBOUR_HEIGHT=170`, `HARBOUR_FULLSCREEN=1`, and the Cardputer Zero theme;
all are overrideable.
It saves VFS data under `$XDG_DATA_HOME/harbour-os/` (or
`~/.local/share/harbour-os/`), instead of inside the executable directory.

When the system launcher owns the display, starting HarbourOS may require
using the launcher's normal app-management mechanism. Do not replace
`APPLaunch.service` or force a framebuffer takeover during this first test.

## Keyboard and UI checks

The Cardputer Zero quick-start documents arrow navigation as
**Fn + F / X / Z / C** and an **OK/ENTER** key. The Linux input mapping must
expose these as SDL2 arrow and Enter events for HarbourOS to use them.
The OS already supports keyboard navigation; the physical Ctrl shortcuts
(`Ctrl+S`, `Ctrl+B`, etc.) may need a dedicated Cardputer mapping.

Smoke-test checklist:

1. Application starts on the internal LCD without a separate display.
2. All six desktop icons are reachable on the 320 × 170 screen.
3. Arrow and Enter navigation open and close at least one app.
4. Text entry, Backspace, Escape, and persistent VFS saving work.
5. No launcher corruption or display ownership conflicts after exit.

## Next stage if SDL2/Wayland cannot display

Keep the SDL2 UI and state machine. Introduce a **device-only output/input
adapter** that renders through an SDL2 software canvas, copies pixels into the
Linux framebuffer (`/dev/fb0`), and reads the TCA8418 keyboard from evdev
(e.g. `/dev/input/by-path/platform-3f804000.i2c-event`). Query framebuffer
dimensions, stride and pixel format; do **not** assume an unpadded
320 × 170 RGB565 framebuffer. This backend should be gated by a build
feature so the desktop and Web builds remain unchanged.

Reference: <https://github.com/eggfly/CardputerZero-Examples>

Native device testing is still required before describing this as
supported hardware.
