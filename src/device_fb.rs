//! Optional direct Linux framebuffer and evdev backend for Cardputer Zero.
//! Activated only by HARBOUR_FBDEV. Desktop, Wayland and Web paths are unchanged.
//! This is an experimental device adapter, not a replacement OS or a display driver.

use std::ffi::c_void;
use std::fs::{File, OpenOptions};
use std::io::{self, Read};
use std::os::fd::AsRawFd;
use std::os::unix::fs::{FileExt, OpenOptionsExt};

use sdl2::event::Event;
use sdl2::keyboard::{Keycode, Mod};
use sdl2::pixels::PixelFormatEnum;
use sdl2::render::Canvas;
use sdl2::video::Window;

const FBIOGET_VSCREENINFO: usize = 0x4600;
const FBIOGET_FSCREENINFO: usize = 0x4602;
const FB_VISUAL_TRUECOLOR: u32 = 2;
const O_NONBLOCK: i32 = 0o4000;
const EV_KEY: u16 = 1;

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct Bitfield {
    offset: u32,
    length: u32,
    msb_right: u32,
}

#[repr(C)]
#[derive(Default)]
struct VarScreeninfo {
    xres: u32,
    yres: u32,
    xres_virtual: u32,
    yres_virtual: u32,
    xoffset: u32,
    yoffset: u32,
    bits_per_pixel: u32,
    grayscale: u32,
    red: Bitfield,
    green: Bitfield,
    blue: Bitfield,
    transp: Bitfield,
    nonstd: u32,
    activate: u32,
    height: u32,
    width: u32,
    accel_flags: u32,
    pixclock: u32,
    left_margin: u32,
    right_margin: u32,
    upper_margin: u32,
    lower_margin: u32,
    hsync_len: u32,
    vsync_len: u32,
    sync: u32,
    vmode: u32,
    rotate: u32,
    colorspace: u32,
    reserved: [u32; 4],
}

#[repr(C)]
#[derive(Default)]
struct FixScreeninfo {
    id: [u8; 16],
    smem_start: usize,
    smem_len: u32,
    fb_type: u32,
    type_aux: u32,
    visual: u32,
    xpanstep: u16,
    ypanstep: u16,
    ywrapstep: u16,
    line_length: u32,
    mmio_start: usize,
    mmio_len: u32,
    accel: u32,
    capabilities: u16,
    reserved: [u16; 2],
}

unsafe extern "C" {
    fn ioctl(fd: i32, request: usize, arg: *mut c_void) -> i32;
}

pub struct Framebuffer {
    file: File,
    var: VarScreeninfo,
    fix: FixScreeninfo,
    bytes_per_pixel: usize,
    row: Vec<u8>,
}

impl Framebuffer {
    pub fn open(path: &str, width: u32, height: u32) -> Result<Self, String> {
        let file = OpenOptions::new()
            .write(true)
            .open(path)
            .map_err(|e| format!("open framebuffer {path}: {e}"))?;
        let mut var = VarScreeninfo::default();
        let mut fix = FixScreeninfo::default();
        // SAFETY: Linux fb.h ABI structs with their exact repr(C) fields.
        // ioctl only writes into the live structs; the file descriptor is valid.
        unsafe {
            if ioctl(
                file.as_raw_fd(),
                FBIOGET_VSCREENINFO,
                &mut var as *mut _ as *mut c_void,
            ) < 0
            {
                return Err(format!(
                    "FBIOGET_VSCREENINFO {path}: {}",
                    io::Error::last_os_error()
                ));
            }
            if ioctl(
                file.as_raw_fd(),
                FBIOGET_FSCREENINFO,
                &mut fix as *mut _ as *mut c_void,
            ) < 0
            {
                return Err(format!(
                    "FBIOGET_FSCREENINFO {path}: {}",
                    io::Error::last_os_error()
                ));
            }
        }

        if var.xres != width || var.yres != height {
            return Err(format!(
                "{path} is {}x{}, but HarbourOS requested {width}x{height}",
                var.xres, var.yres
            ));
        }
        if !valid_rgb_format(&var, &fix) {
            return Err(format!(
                "unsupported framebuffer format: {} bpp, visual {}, RGB offsets ({},{},{})",
                var.bits_per_pixel, fix.visual, var.red.offset, var.green.offset, var.blue.offset
            ));
        }
        let bytes_per_pixel = (var.bits_per_pixel / 8) as usize;
        let stride = fix.line_length as usize;
        let row_end = (var.xoffset as usize + width as usize)
            .checked_mul(bytes_per_pixel)
            .ok_or("framebuffer horizontal offset overflow")?;
        let last_byte = (var.yoffset as usize + height as usize - 1)
            .checked_mul(stride)
            .and_then(|v| v.checked_add(row_end))
            .ok_or("framebuffer vertical offset overflow")?;
        if var.xres_virtual < var.xoffset.saturating_add(width)
            || var.yres_virtual < var.yoffset.saturating_add(height)
            || row_end > stride
            || last_byte > fix.smem_len as usize
        {
            return Err(format!(
                "{path}: framebuffer stride or memory size is too small"
            ));
        }

        eprintln!(
            "[cardputer] {path}: {}x{} {}bpp, stride={stride}, offsets=({}, {})",
            var.xres, var.yres, var.bits_per_pixel, var.xoffset, var.yoffset
        );
        Ok(Self {
            file,
            var,
            fix,
            bytes_per_pixel,
            row: vec![0; width as usize * bytes_per_pixel],
        })
    }

    pub fn present(&mut self, canvas: &Canvas<Window>) -> Result<(), String> {
        let pixels = canvas
            .read_pixels(None, PixelFormatEnum::RGB24)
            .map_err(|e| format!("SDL read_pixels: {e}"))?;
        let width = self.var.xres as usize;
        let height = self.var.yres as usize;
        if pixels.len() != width * height * 3 {
            return Err(format!(
                "SDL returned {} RGB24 bytes, expected {}",
                pixels.len(),
                width * height * 3
            ));
        }
        for y in 0..height {
            let start = y * width * 3;
            encode_rgb24_row(
                &pixels[start..start + width * 3],
                &mut self.row,
                self.bytes_per_pixel,
                &self.var,
            )?;
            let offset = ((y + self.var.yoffset as usize) * self.fix.line_length as usize
                + self.var.xoffset as usize * self.bytes_per_pixel) as u64;
            let mut written = 0;
            while written < self.row.len() {
                match self
                    .file
                    .write_at(&self.row[written..], offset + written as u64)
                {
                    Ok(0) => return Err("short framebuffer write".to_string()),
                    Ok(n) => written += n,
                    Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
                    Err(e) => return Err(format!("framebuffer write: {e}")),
                }
            }
        }
        Ok(())
    }
}

// Linux fbdev publishes bit positions rather than a fixed RGB565/BGR888
// encoding. Reject unusual or overlapping formats instead of writing garbage.
fn valid_rgb_format(var: &VarScreeninfo, fix: &FixScreeninfo) -> bool {
    if !matches!(var.bits_per_pixel, 16 | 24 | 32)
        || fix.visual != FB_VISUAL_TRUECOLOR
        || var.grayscale != 0
        || var.nonstd != 0
        || [var.red, var.green, var.blue].iter().any(|f| f.length == 0)
    {
        return false;
    }
    let mut occupied = 0_u32;
    for field in [var.red, var.green, var.blue, var.transp] {
        if field.msb_right != 0
            || field.length > 8
            || field.offset + field.length > var.bits_per_pixel
        {
            return false;
        }
        let mask = if field.length == 0 {
            0
        } else {
            ((1_u32 << field.length) - 1) << field.offset
        };
        if occupied & mask != 0 {
            return false;
        }
        occupied |= mask;
    }
    true
}

// Same RGB24 source layout is used by the desktop SDL simulator and the
// hardware's fbdev sink. This pure conversion is exercised in CI.
fn encode_rgb24_row(
    src: &[u8],
    dest: &mut [u8],
    bytes_per_pixel: usize,
    var: &VarScreeninfo,
) -> Result<(), String> {
    if src.len() % 3 != 0 || dest.len() != src.len() / 3 * bytes_per_pixel {
        return Err("invalid RGB24 source or framebuffer row length".to_string());
    }
    for (rgb, native) in src
        .chunks_exact(3)
        .zip(dest.chunks_exact_mut(bytes_per_pixel))
    {
        let packed = pack_channel(rgb[0], var.red)
            | pack_channel(rgb[1], var.green)
            | pack_channel(rgb[2], var.blue)
            | opaque_bits(var.transp);
        native.copy_from_slice(&packed.to_ne_bytes()[..bytes_per_pixel]);
    }
    Ok(())
}

fn pack_channel(value: u8, field: Bitfield) -> u32 {
    if field.length == 0 {
        0
    } else {
        let max = (1_u32 << field.length) - 1;
        (((value as u32 * max + 127) / 255) & max) << field.offset
    }
}

fn opaque_bits(field: Bitfield) -> u32 {
    if field.length == 0 {
        0
    } else {
        ((1_u32 << field.length) - 1) << field.offset
    }
}

/// Reads kernel TCA8418 evdev events. The driver normally converts FN+F/Z/X/C
/// to arrow keys before they reach this layer. No raw-I2C access is performed.
pub struct Keyboard {
    file: File,
    shift: bool,
    ctrl: bool,
    alt: bool,
}

impl Keyboard {
    pub fn open(path: &str) -> Result<Self, String> {
        let file = OpenOptions::new()
            .read(true)
            .custom_flags(O_NONBLOCK)
            .open(path)
            .map_err(|e| format!("open keyboard {path}: {e}"))?;
        eprintln!("[cardputer] keyboard: {path}");
        Ok(Self {
            file,
            shift: false,
            ctrl: false,
            alt: false,
        })
    }

    pub fn poll(&mut self) -> Result<Vec<Event>, String> {
        // 64-bit Linux input_event: timeval (2x i64), type u16, code u16, value i32.
        // Cardputer Zero (aarch64) uses this ABI.
        let mut events = Vec::new();
        for _ in 0..128 {
            let mut record = [0_u8; 24];
            match self.file.read(&mut record) {
                Ok(24) => {}
                Ok(0) => break,
                Ok(n) => return Err(format!("unexpected evdev record length: {n}")),
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => break,
                Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
                Err(e) => return Err(format!("keyboard read: {e}")),
            }
            let kind = u16::from_ne_bytes([record[16], record[17]]);
            if kind != EV_KEY {
                continue;
            }
            let code = u16::from_ne_bytes([record[18], record[19]]);
            let value = i32::from_ne_bytes([record[20], record[21], record[22], record[23]]);
            if !matches!(value, 0 | 1 | 2) {
                continue;
            }
            match code {
                42 | 54 => self.shift = value != 0,
                29 | 97 => self.ctrl = value != 0,
                56 | 100 => self.alt = value != 0,
                _ => {}
            }
            let Some(key) = map_key(code) else { continue };
            let keymod = self.modifiers();
            let down = value != 0;
            if down {
                events.push(Event::KeyDown {
                    timestamp: 0,
                    window_id: 0,
                    keycode: Some(key),
                    scancode: None,
                    keymod,
                    repeat: value == 2,
                });
                if value == 1 && !self.ctrl && !self.alt {
                    if let Some(c) = printable(code, self.shift) {
                        events.push(Event::TextInput {
                            timestamp: 0,
                            window_id: 0,
                            text: c.to_string(),
                        });
                    }
                }
            } else {
                events.push(Event::KeyUp {
                    timestamp: 0,
                    window_id: 0,
                    keycode: Some(key),
                    scancode: None,
                    keymod,
                    repeat: false,
                });
            }
        }
        Ok(events)
    }

    fn modifiers(&self) -> Mod {
        let mut mods = Mod::NOMOD;
        if self.shift {
            mods |= Mod::LSHIFTMOD;
        }
        if self.ctrl {
            mods |= Mod::LCTRLMOD;
        }
        if self.alt {
            mods |= Mod::LALTMOD;
        }
        mods
    }
}

fn map_key(code: u16) -> Option<Keycode> {
    Some(match code {
        1 => Keycode::Escape,
        2 => Keycode::Num1,
        3 => Keycode::Num2,
        4 => Keycode::Num3,
        5 => Keycode::Num4,
        6 => Keycode::Num5,
        7 => Keycode::Num6,
        8 => Keycode::Num7,
        9 => Keycode::Num8,
        10 => Keycode::Num9,
        11 => Keycode::Num0,
        12 => Keycode::Minus,
        13 => Keycode::Equals,
        14 => Keycode::Backspace,
        15 => Keycode::Tab,
        16 => Keycode::Q,
        17 => Keycode::W,
        18 => Keycode::E,
        19 => Keycode::R,
        20 => Keycode::T,
        21 => Keycode::Y,
        22 => Keycode::U,
        23 => Keycode::I,
        24 => Keycode::O,
        25 => Keycode::P,
        26 => Keycode::LeftBracket,
        27 => Keycode::RightBracket,
        28 | 96 => Keycode::Return,
        29 => Keycode::LCtrl,
        30 => Keycode::A,
        31 => Keycode::S,
        32 => Keycode::D,
        33 => Keycode::F,
        34 => Keycode::G,
        35 => Keycode::H,
        36 => Keycode::J,
        37 => Keycode::K,
        38 => Keycode::L,
        39 => Keycode::Semicolon,
        40 => Keycode::Quote,
        41 => Keycode::Backquote,
        42 => Keycode::LShift,
        43 => Keycode::Backslash,
        44 => Keycode::Z,
        45 => Keycode::X,
        46 => Keycode::C,
        47 => Keycode::V,
        48 => Keycode::B,
        49 => Keycode::N,
        50 => Keycode::M,
        51 => Keycode::Comma,
        52 => Keycode::Period,
        53 => Keycode::Slash,
        54 => Keycode::RShift,
        56 => Keycode::LAlt,
        57 => Keycode::Space,
        59 => Keycode::F1,
        60 => Keycode::F2,
        61 => Keycode::F3,
        62 => Keycode::F4,
        63 => Keycode::F5,
        64 => Keycode::F6,
        65 => Keycode::F7,
        66 => Keycode::F8,
        67 => Keycode::F9,
        68 => Keycode::F10,
        87 => Keycode::F11,
        88 => Keycode::F12,
        97 => Keycode::RCtrl,
        100 => Keycode::RAlt,
        102 => Keycode::Home,
        103 => Keycode::Up,
        104 => Keycode::PageUp,
        105 => Keycode::Left,
        106 => Keycode::Right,
        107 => Keycode::End,
        108 => Keycode::Down,
        109 => Keycode::PageDown,
        110 => Keycode::Insert,
        111 => Keycode::Delete,
        _ => return None,
    })
}

fn printable(code: u16, shift: bool) -> Option<char> {
    let letter = match code {
        16..=25 => Some("qwertyuiop".as_bytes()[(code - 16) as usize] as char),
        30..=38 => Some("asdfghjkl".as_bytes()[(code - 30) as usize] as char),
        44..=50 => Some("zxcvbnm".as_bytes()[(code - 44) as usize] as char),
        _ => None,
    };
    if let Some(c) = letter {
        return Some(if shift { c.to_ascii_uppercase() } else { c });
    }
    match (code, shift) {
        (2..=11, false) => Some("1234567890".as_bytes()[(code - 2) as usize] as char),
        (2..=11, true) => Some("!@#$%^&*()".as_bytes()[(code - 2) as usize] as char),
        (12, false) => Some('-'),
        (12, true) => Some('_'),
        (13, false) => Some('='),
        (13, true) => Some('+'),
        (26, false) => Some('['),
        (26, true) => Some('{'),
        (27, false) => Some(']'),
        (27, true) => Some('}'),
        (39, false) => Some(';'),
        (39, true) => Some(':'),
        (40, false) => Some('\''),
        (40, true) => Some('"'),
        (41, false) => Some('`'),
        (41, true) => Some('~'),
        (43, false) => Some('\\'),
        (43, true) => Some('|'),
        (51, false) => Some(','),
        (51, true) => Some('<'),
        (52, false) => Some('.'),
        (52, true) => Some('>'),
        (53, false) => Some('/'),
        (53, true) => Some('?'),
        (57, _) => Some(' '),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rgb565_pack() {
        assert_eq!(
            pack_channel(
                255,
                Bitfield {
                    offset: 11,
                    length: 5,
                    msb_right: 0
                }
            ),
            0xf800
        );
        assert_eq!(
            pack_channel(
                255,
                Bitfield {
                    offset: 5,
                    length: 6,
                    msb_right: 0
                }
            ),
            0x07e0
        );
    }

    #[test]
    fn framebuffer_rgb565_conversion() {
        let mut v = VarScreeninfo {
            xres: 2,
            yres: 1,
            xres_virtual: 2,
            yres_virtual: 1,
            bits_per_pixel: 16,
            red: Bitfield {
                offset: 11,
                length: 5,
                ..Default::default()
            },
            green: Bitfield {
                offset: 5,
                length: 6,
                ..Default::default()
            },
            blue: Bitfield {
                offset: 0,
                length: 5,
                ..Default::default()
            },
            ..Default::default()
        };
        let fix = FixScreeninfo {
            visual: FB_VISUAL_TRUECOLOR,
            ..Default::default()
        };
        assert!(valid_rgb_format(&v, &fix));
        let mut pixels = [0_u8; 4];
        encode_rgb24_row(&[255, 0, 0, 0, 0, 255], &mut pixels, 2, &v).unwrap();
        assert_eq!(pixels, [0x00, 0xf8, 0x1f, 0x00]);
        // Reject overlapping channels, which would otherwise corrupt colors.
        v.green.offset = 11;
        assert!(!valid_rgb_format(&v, &fix));
    }

    #[test]
    fn framebuffer_bgra8888_conversion() {
        let v = VarScreeninfo {
            bits_per_pixel: 32,
            red: Bitfield {
                offset: 16,
                length: 8,
                ..Default::default()
            },
            green: Bitfield {
                offset: 8,
                length: 8,
                ..Default::default()
            },
            blue: Bitfield {
                offset: 0,
                length: 8,
                ..Default::default()
            },
            transp: Bitfield {
                offset: 24,
                length: 8,
                ..Default::default()
            },
            ..Default::default()
        };
        let fix = FixScreeninfo {
            visual: FB_VISUAL_TRUECOLOR,
            ..Default::default()
        };
        assert!(valid_rgb_format(&v, &fix));
        let mut pixels = [0_u8; 4];
        encode_rgb24_row(&[0x12, 0x34, 0x56], &mut pixels, 4, &v).unwrap();
        assert_eq!(pixels, [0x56, 0x34, 0x12, 0xff]);
        assert!(encode_rgb24_row(&[1, 2], &mut pixels, 4, &v).is_err());
    }

    #[test]
    fn evdev_ascii() {
        assert_eq!(printable(30, false), Some('a'));
        assert_eq!(printable(30, true), Some('A'));
        assert_eq!(printable(2, true), Some('!'));
        assert_eq!(printable(57, false), Some(' '));
    }
}
