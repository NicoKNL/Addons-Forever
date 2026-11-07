//! System tray (menu + click-to-restore). Windows-first; on Linux the tray is
//! not linked (keeps the dependency tree slim, no gtk needed for dev builds).

#[allow(dead_code)] // variants used on Windows/macOS
pub enum TrayAction {
    Show,
    Check,
    Quit,
    None,
}

#[cfg(any(windows, target_os = "macos"))]
mod imp {
    use tray_icon::menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem};
    use tray_icon::{Icon, TrayIcon, TrayIconBuilder, TrayIconEvent};

    use super::TrayAction;

    pub struct TrayCtl {
        _tray: TrayIcon,
        id_show: tray_icon::menu::MenuId,
        id_check: tray_icon::menu::MenuId,
        id_quit: tray_icon::menu::MenuId,
    }

    impl TrayCtl {
        pub fn setup(app_version: &str) -> Result<Self, String> {
            let (w, h, rgba) = crate::icon::tray_icon_rgba();
            let icon = Icon::from_rgba(rgba, w, h).map_err(|e| e.to_string())?;

            let menu = Menu::new();
            let item_show = MenuItem::new(format!("Open Addons Forever ({app_version})"), true, None);
            let item_check = MenuItem::new("Check for updates", true, None);
            let sep = PredefinedMenuItem::separator();
            let item_quit = MenuItem::new("Quit", true, None);
            menu.append(&item_show).map_err(|e| e.to_string())?;
            menu.append(&item_check).map_err(|e| e.to_string())?;
            menu.append(&sep).map_err(|e| e.to_string())?;
            menu.append(&item_quit).map_err(|e| e.to_string())?;

            let builder = TrayIconBuilder::new()
                .with_menu(Box::new(menu))
                .with_tooltip("Addons Forever - WoW: Forever addon manager")
                .with_icon(icon);
            // left click restores the window on Windows; menu stays on right click
            #[cfg(windows)]
            let builder = builder.with_menu_on_left_click(false);
            let tray = builder.build().map_err(|e| e.to_string())?;

            Ok(TrayCtl {
                _tray: tray,
                id_show: item_show.id().clone(),
                id_check: item_check.id().clone(),
                id_quit: item_quit.id().clone(),
            })
        }

        pub fn pump(&self) -> TrayAction {
            let mut action = TrayAction::None;
            while let Ok(ev) = TrayIconEvent::receiver().try_recv() {
                if matches!(
                    ev,
                    TrayIconEvent::Click {
                        button: tray_icon::MouseButton::Left,
                        button_state: tray_icon::MouseButtonState::Up,
                        ..
                    }
                ) {
                    action = TrayAction::Show;
                }
            }
            while let Ok(ev) = MenuEvent::receiver().try_recv() {
                if ev.id() == &self.id_show {
                    action = TrayAction::Show;
                } else if ev.id() == &self.id_check {
                    action = TrayAction::Check;
                } else if ev.id() == &self.id_quit {
                    action = TrayAction::Quit;
                }
            }
            action
        }
    }
}

#[cfg(not(any(windows, target_os = "macos")))]
mod imp {
    pub struct TrayCtl;
    impl TrayCtl {
        pub fn setup(_app_version: &str) -> Result<Self, String> {
            Err("system tray is supported on Windows/macOS builds".into())
        }
        pub fn pump(&self) -> super::TrayAction {
            super::TrayAction::None
        }
    }
}

pub use imp::TrayCtl;
