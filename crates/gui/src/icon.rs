//! App + tray icons. The real artwork (docs/ART.md, files in `assets/`) is
//! embedded into the binary at compile time; the procedural tile below remains
//! as a fallback so the build never depends on the asset files.

/// Decode a PNG into 8-bit RGBA. Returns None for unsupported color types.
pub fn decode_png(bytes: &[u8]) -> Option<(u32, u32, Vec<u8>)> {
    use std::io::Cursor;
    // png 0.18 expands palette images to RGB(A) by default.
    let mut reader = png::Decoder::new(Cursor::new(bytes)).read_info().ok()?;
    let buf_len = reader.output_buffer_size()?;
    let mut buf = vec![0u8; buf_len];
    let info = reader.next_frame(&mut buf).ok()?;
    let src = &buf[..info.buffer_size()];
    let rgba = match info.color_type {
        png::ColorType::Rgba => src.to_vec(),
        png::ColorType::Rgb => {
            let mut out = Vec::with_capacity(src.len() / 3 * 4);
            for c in src.chunks_exact(3) {
                out.extend_from_slice(c);
                out.push(255);
            }
            out
        }
        png::ColorType::GrayscaleAlpha => {
            let mut out = Vec::with_capacity(src.len() * 2);
            for c in src.chunks_exact(2) {
                out.extend_from_slice(&[c[0], c[0], c[0], c[1]]);
            }
            out
        }
        png::ColorType::Grayscale => {
            let mut out = Vec::with_capacity(src.len() * 4);
            for &g in src {
                out.extend_from_slice(&[g, g, g, 255]);
            }
            out
        }
        _ => return None,
    };
    Some((info.width, info.height, rgba))
}

pub const WINDOW_ICON_PNG: &[u8] = include_bytes!("../../../assets/icon-32.png");
// tray is linked on Windows/macOS only (see tray.rs)
#[cfg(any(windows, target_os = "macos"))]
pub const TRAY_ICON_PNG: &[u8] = include_bytes!("../../../assets/tray-32.png");

/// Window/taskbar icon for eframe (real art, procedural fallback).
pub fn window_icon() -> egui::IconData {
    decode_png(WINDOW_ICON_PNG)
        .map(|(width, height, rgba)| egui::IconData { rgba, width, height })
        .unwrap_or_else(|| {
            let (width, height, rgba) = icon_rgba32();
            egui::IconData { rgba, width, height }
        })
}

/// Tray icon (dedicated high-contrast art, procedural fallback).
#[cfg(any(windows, target_os = "macos"))]
pub fn tray_icon_rgba() -> (u32, u32, Vec<u8>) {
    decode_png(TRAY_ICON_PNG).unwrap_or_else(icon_rgba32)
}

/// Procedural 32x32 RGBA fallback: a blue gradient tile with a white border
/// and a "Forever" starfield.
pub fn icon_rgba32() -> (u32, u32, Vec<u8>) {
    const S: u32 = 32;
    let mut px = vec![0u8; (S * S * 4) as usize];
    for y in 0..S {
        for x in 0..S {
            let t = y as f32 / S as f32;
            let r = (0x18u32 as f32 + (0x3a as f32 - 0x18 as f32) * t) as u8;
            let g = (0x3bu32 as f32 + (0x6d as f32 - 0x3b as f32) * t) as u8;
            let b = (0x7fu32 as f32 + (0xf4 as f32 - 0x7f as f32) * t) as u8;
            let mut c = [r, g, b, 255u8];
            // border
            if x < 1 || y < 1 || x >= S - 1 || y >= S - 1 {
                c = [0xe8, 0xec, 0xf5, 255];
            }
            // sparse starfield
            if (x * 7 + y * 13) % 29 == 0 {
                c = [0xff, 0xff, 0xff, 220];
            }
            let i = ((y * S + x) * 4) as usize;
            px[i..i + 4].copy_from_slice(&c);
        }
    }
    (S, S, px)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(unused_mut)] // the tray asset is only registered on Windows/macOS
    fn embedded_assets_decode_to_rgba() {
        let mut set: Vec<(&str, &[u8])> = vec![
            ("icon-32", WINDOW_ICON_PNG as &[u8]),
            ("empty-addons", crate::tabs::ART_EMPTY_ADDONS),
            ("empty-discover", crate::tabs::ART_EMPTY_DISCOVER),
            ("empty-log", crate::tabs::ART_EMPTY_LOG),
            ("badge-update", crate::tabs::ART_BADGE_UPDATE),
        ];
        #[cfg(any(windows, target_os = "macos"))]
        set.push(("tray-32", TRAY_ICON_PNG));
        for (name, bytes) in set {
            let (w, h, rgba) = decode_png(bytes).unwrap_or_else(|| panic!("{name} failed to decode"));
            assert!(w >= 16 && h >= 16, "{name} too small: {w}x{h}");
            assert_eq!(rgba.len(), (w * h * 4) as usize, "{name} not RGBA");
        }
    }
}
