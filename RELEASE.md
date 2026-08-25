# Harbour OS Release Notes

This document defines what "finished enough to ship" means for Harbour OS 0.1.

## 0.1 scope

Harbour OS 0.1 is a small retro virtual desktop, not a general-purpose operating system. The release is considered complete when it can be downloaded, launched, used with the keyboard, and used to create/read/export notes without touching the source tree.

## Ship criteria

- `cargo test --all-targets` passes.
- `cargo build --release --bin harbour_shell` passes.
- The app launches with the default assets and theme.
- The desktop can launch every built-in app.
- TextEditor can create text, save it to the VFS, and reopen it through Files.
- Files can create, rename, delete, import, and export text files/folders.
- Clock, Dictionary, Calculator, and Snake are reachable and return to the desktop.
- Runtime files are kept out of Git: `harbour_vfs.yaml`, `harbour_import/`, and `harbour_export/`.
- A release bundle can be produced with `./tools/package_release.sh`.

## Manual smoke test

Run this from a clean checkout:

```bash
rm -f harbour_vfs.yaml
rm -rf harbour_import harbour_export
mkdir -p harbour_import
printf 'Imported from host.\n' > harbour_import/host-note.txt
cargo run
```

Inside Harbour OS:

1. Launch TextEditor, type a short note, press `Ctrl+S`, then close it.
2. Launch Files and confirm `/Untitled.txt` exists.
3. Rename `/Untitled.txt` to `note.txt`.
4. Press `I` in Files and confirm `host-note.txt` appears.
5. Select `note.txt`, press `E`, and confirm `harbour_export/note.txt` exists on the host after quitting.
6. Launch Calculator and verify `1+2*3` returns `7`.
7. Launch Snake, move once, then return to the desktop.

## Packaging

```bash
./tools/package_release.sh
```

The generated bundle is written to:

```text
dist/harbour-os-linux-x86_64.tar.gz
```

The bundle includes:

- `harbour_shell`
- `launch.sh`
- `run.sh`
- `README.md`
- `assets/`
- `fonts/`
