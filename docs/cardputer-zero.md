# Cardputer Zero: experimental Linux port

> **Not verified on a physical Cardputer Zero.** This is an experimental ARM64
> SDL2/Linux framebuffer + evdev application port. It does not replace the
> device's Linux image or default app launcher.

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
  12×16 pointer is a solid black silhouette with its clickable hotspot at
  the upper-left pixel (no white interior or shadow). The Web simulator hides
  the host macOS/browser pointer over the LCD, preserves the exact 320:170
  framebuffer aspect ratio inside the bezel, and routes visible mouse
  coordinates to the native SDL canvas by pixel center. Without a mouse,
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

## Local SDL2 simulator (without real hardware)

For HarbourOS development on a macOS or Linux desktop, use:

```sh
bash run-cardputer-zero-sim.sh
```

This runs the **same Rust UI** at the native 320×170 resolution in an SDL2
window and enables M5-style desktop navigation: F=Up, X=Down, Z=Left, C=Right.
Inside the text editor these are regular letters; hold Fn with F/X/Z/C on
the physical keyboard to request arrow keys while typing. The browser
simulator also runs the same GUI compiled to WebAssembly.
Both are useful for layout, fonts and keyboard tests. The M5Stack official
SDL2 simulator itself is an **LVGL application**, not a general-purpose Linux
emulator: HarbourOS is not loaded into that LVGL process. The dedicated
HarbourOS simulator therefore avoids a second GUI or LVGL dependency.

The CI smoke test starts the SDL2 dummy-video software renderer for two
frames at 320×170 and exercises the real drawing loop. It is a rendering
startup test, **not** a framebuffer device or keyboard hardware test.

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

The default firmware exposes an ST7789V Linux framebuffer (commonly
`/dev/fb0`) and a TCA8418 kernel evdev keyboard. It **does not require
Wayland** to run HarbourOS. This port keeps the existing Rust/SDL2 drawing
and windows, renders them into an SDL software canvas, then copies RGB24
pixels into the framebuffer's reported RGB bitfields. It uses the framebuffer
driver's actual stride and visible offsets. No LVGL rewrite, root filesystem
replacement, or separate GUI is involved.

The default launch script **always chooses the Linux framebuffer**:
`SDL_VIDEODRIVER=dummy` + `HARBOUR_FBDEV=/dev/fb0`. This uses SDL2's
software renderer as an off-screen 320×170 drawing buffer and writes pixels
directly to fbdev, without a Wayland compositor. The kernel input device
supplies keyboard events through evdev.

Wayland remains an **explicit opt-in** for other Linux hosts:
`SDL_VIDEODRIVER=wayland`, with `HARBOUR_FBDEV` unset.

From a **safe application session** where APPLaunch has released the display:

```sh
chmod +x ./run-cardputer-zero.sh
./run-cardputer-zero.sh
```

The built-in APPLaunch service owns the display by default. **Do not launch
HarbourOS on top of an active launcher or overwrite `APPLaunch.service`.**
Use its external application handoff, or arrange a temporary manual test
session that releases display ownership. This script does not modify
systemd services, device configuration, or the boot image.

If auto detection chooses the wrong display path:

```sh
SDL_VIDEODRIVER=dummy HARBOUR_FBDEV=/dev/fb0 \
  HARBOUR_INPUT_DEVICE=/dev/input/by-path/platform-3f804000.i2c-event \
  ./run-cardputer-zero.sh
```

The default keyboard path is taken from the M5Stack launcher reference.
If it differs, check `ls -l /dev/input/by-path/` and `cat
/proc/bus/input/devices`; pass the matching `/dev/input/event*` path in
`HARBOUR_INPUT_DEVICE`. The user running HarbourOS needs write permission
on the framebuffer and read permission on the keyboard event device.
No `sudo` is needed if those permissions are correctly configured.

For an existing Wayland session, use:

```sh
SDL_VIDEODRIVER=wayland WAYLAND_DISPLAY=wayland-0 \
  XDG_RUNTIME_DIR=/run/user/1000 ./run-cardputer-zero.sh
```

Use the actual session's `WAYLAND_DISPLAY` and UID, not the example values.

Supported direct-fbdev pixel formats initially are 16/24/32-bit truecolor
with standard Linux bitfields and a visible 320×170 image. Unrecognized
formats, missing permissions, and size mismatches fail with explicit errors
rather than writing an unknown pixel layout. If the kernel display reports
a different format, collect the error and framebuffer metadata on the device
before extending the adapter.

VFS data is saved under `$XDG_DATA_HOME/harbour-os/` (or
`~/.local/share/harbour-os/`); this is independent of the executable bundle.

## Keyboard and UI checks

The Cardputer Zero quick-start documents arrow navigation as
**Fn + F / X / Z / C** and an **OK/ENTER** key. The evdev adapter reads the kernel's TCA8418 keycodes directly. The M5
APPLaunch firmware uses a non-PC mapping for many punctuation keys, so the
native adapter includes that default symbol table, including codes 69–94.
Fn+F/X/Z/C maps to arrows; plain F/X/Z/C navigate on the desktop but
remain literal letters in text-editing screens. The actual Fn and key-repeat
behavior still needs hardware testing. If the device uses a modified
`/usr/share/keymaps/tca8418_keypad_m5stack_keymap.map`, the built-in mapping
may need updating to match its contents.
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
