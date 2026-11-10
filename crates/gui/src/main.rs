#![cfg_attr(all(not(debug_assertions), target_os = "windows"), windows_subsystem = "windows")]
//! addons-forever — portable WoW: Forever addon manager (egui GUI).

mod app;
mod cli;
mod icon;
mod selfupdate;
mod single;
mod tabs;
mod tray;
mod worker;

use std::sync::Arc;

use app::AppGui;

fn main() -> eframe::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let is_cli = args.get(1).map(|a| a.as_str()) == Some("cli");
    let data_dir = addons_forever_core::util::data_dir().expect("cannot determine app data directory");

    // config first: the instance lock is keyed to the AddOns installation
    let config_path = addons_forever_core::config::AppConfig::config_path(&data_dir);
    let mut cfg = match addons_forever_core::config::AppConfig::load(&config_path) {
        Ok(c) => c,
        Err(e) => {
            let msg = format!("Could not read {}:\n{e}", config_path.display());
            if is_cli {
                eprintln!("{msg}");
                std::process::exit(1);
            }
            fatal(&msg);
            return Ok(());
        }
    };
    addons_forever_core::store::StateStore::migrate_into_cfg(&data_dir, &mut cfg);
    let cfg = Arc::new(cfg);

    // AF-22: the lock covers CLI and GUI alike, keyed to the installation
    // being mutated, so no two writers race the same AddOns folder
    let _single = match single::SingleInstanceGuard::acquire(&data_dir, cfg.app.addons_dir.as_deref()) {
        Some(g) => g,
        None => {
            if is_cli {
                eprintln!("addons-forever is already running for this installation; close it (or the GUI) first");
                std::process::exit(1);
            }
            #[cfg(windows)]
            unsafe {
                use std::ffi::c_void;
                use windows_sys::Win32::UI::WindowsAndMessaging::{
                    MB_ICONINFORMATION, MB_OK, MessageBoxW,
                };
                let msg: Vec<u16> =
                    "addons-forever is already running (check the system tray).\0".encode_utf16().collect();
                let title: Vec<u16> = "addons-forever\0".encode_utf16().collect();
                MessageBoxW(std::ptr::null_mut() as *mut c_void, msg.as_ptr(), title.as_ptr(), MB_OK | MB_ICONINFORMATION);
            }
            #[cfg(not(windows))]
            eprintln!("addons-forever is already running");
            return Ok(());
        }
    };

    if is_cli {
        std::process::exit(cli::run(&args[2..], data_dir, cfg));
    }

    init_logging();
    tracing::info!("addons-forever {} starting; data dir {}", env!("CARGO_PKG_VERSION"), data_dir.display());

    let version: &'static str = env!("CARGO_PKG_VERSION");
    let handle = worker::spawn(cfg.clone(), data_dir.clone(), version);

    let opts = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Addons Forever")
            .with_inner_size([cfg.ui.window_w, cfg.ui.window_h])
            .with_min_inner_size([820.0, 540.0])
            .with_icon(icon::window_icon()),
        // no persistence file: window size lives in the ini, zoom too
        persistence_path: None,
        renderer: eframe::Renderer::Glow,
        ..Default::default()
    };

    eframe::run_native(
        "addons-forever",
        opts,
        Box::new(move |cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            setup_style(&cc.egui_ctx);
            Ok(Box::new(AppGui::new(cc, cfg, data_dir, handle, version)))
        }),
    )
}

/// Load the OS UI font (Segoe UI / SF / system sans) so text looks native and
/// glyph coverage is complete; falls back to egui's bundled fonts.
fn ui_font_data() -> Vec<egui::FontData> {
    let paths: &[&str] = if cfg!(windows) {
        &[
            "C:\\Windows\\Fonts\\segoeui.ttf",
            "C:\\Windows\\Fonts\\tahoma.ttf",
            "C:\\Windows\\Fonts\\arial.ttf",
        ]
    } else if cfg!(target_os = "macos") {
        &[
            "/System/Library/Fonts/SFNS.ttf",
            "/System/Library/Fonts/SanFrancisco.ttc",
            "/System/Library/Fonts/Helvetica.ttc",
            "/Library/Fonts/Arial.ttf",
        ]
    } else {
        &[
            "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
            "/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf",
            "/usr/share/fonts/truetype/noto/NotoSans-Regular.ttf",
            "/usr/share/fonts/truetype/cantarell/Cantarell-VF.otf",
        ]
    };
    paths
        .iter()
        .filter_map(|p| std::fs::read(p).ok())
        .map(egui::FontData::from_owned)
        .collect()
}

fn setup_style(ctx: &egui::Context) {
    let os_fonts = ui_font_data();
    if !os_fonts.is_empty() {
        let mut defs = egui::FontDefinitions::default();
        for (i, f) in os_fonts.into_iter().enumerate() {
            defs.font_data.insert(format!("os-ui-{i}"), std::sync::Arc::new(f));
            if let Some(fam) = defs.families.get_mut(&egui::FontFamily::Proportional) {
                fam.insert(0, format!("os-ui-{i}"));
            }
            if let Some(fam) = defs.families.get_mut(&egui::FontFamily::Monospace) {
                fam.push(format!("os-ui-{i}"));
            }
        }
        ctx.set_fonts(defs);
    }
    // Sizes and spacing for both themes: egui follows the OS appearance, and
    // light mode otherwise keeps egui's defaults (9 pt Small next to 13 pt Body).
    ctx.all_styles_mut(|style| {
        style.spacing.item_spacing = egui::vec2(7.0, 5.0);
        style.text_styles.insert(egui::TextStyle::Body, egui::FontId::proportional(15.5));
        style.text_styles.insert(egui::TextStyle::Button, egui::FontId::proportional(15.0));
        style.text_styles.insert(egui::TextStyle::Heading, egui::FontId::proportional(19.0));
        style.text_styles.insert(egui::TextStyle::Small, egui::FontId::proportional(13.0));
    });
    ctx.style_mut_of(egui::Theme::Dark, |style| {
        style.visuals = egui::Visuals::dark();
        style.visuals.panel_fill = egui::Color32::from_rgb(24, 26, 31);
        style.visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(30, 33, 39);
        // contrast: readable body/strong/weak text on the dark background
        style.visuals.widgets.noninteractive.fg_stroke.color = egui::Color32::from_gray(236);
        style.visuals.widgets.inactive.fg_stroke.color = egui::Color32::from_gray(226);
        style.visuals.widgets.hovered.fg_stroke.color = egui::Color32::WHITE;
        style.visuals.widgets.active.fg_stroke.color = egui::Color32::WHITE;
        style.visuals.weak_text_color = Some(egui::Color32::from_gray(188));
    });
}

/// Stderr-only logging: the app keeps no log files (the Log tab can export on
/// demand). RUST_LOG overrides the default filter.
fn init_logging() {
    use tracing_subscriber::prelude::*;
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "info,addons_forever=debug,addons_forever_core=debug,reqwest=warn,hyper=warn,notify=warn".into());
    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer().with_writer(std::io::stderr))
        .init();
}

fn fatal(msg: &str) {
    tracing::error!("{msg}");
    #[cfg(windows)]
    unsafe {
        use std::ffi::c_void;
        use windows_sys::Win32::UI::WindowsAndMessaging::{MB_ICONERROR, MB_OK, MessageBoxW};
        // AF-23: MessageBoxW needs a null-terminated wide string
        let w: Vec<u16> = msg.encode_utf16().chain(std::iter::once(0)).collect();
        let t: Vec<u16> = "Addons Forever\0".encode_utf16().collect();
        MessageBoxW(std::ptr::null_mut() as *mut c_void, w.as_ptr(), t.as_ptr(), MB_OK | MB_ICONERROR);
    }
    #[cfg(not(windows))]
    eprintln!("{msg}");
}
