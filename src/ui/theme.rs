//! Semantic Theme tokens for HFS-RS (Dark & Light modes with System Auto-detection).

use gpui::Rgba;
use serde::{Deserialize, Serialize};

pub const fn rgba_const(hex: u32) -> Rgba {
    let [r, g, b, a] = hex.to_be_bytes();
    Rgba::new(
        r as f32 / 255.0,
        g as f32 / 255.0,
        b as f32 / 255.0,
        a as f32 / 255.0,
    )
}

pub const TRANSPARENT: Rgba = rgba_const(0x00000000);
pub const WHITE: Rgba = rgba_const(0xffffffff);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ThemeMode {
    #[default]
    System,
    Dark,
    Light,
}

#[cfg(windows)]
pub fn detect_system_is_dark() -> bool {
    #[link(name = "advapi32")]
    unsafe extern "system" {
        fn RegOpenKeyExW(
            hKey: usize,
            lpSubKey: *const u16,
            ulOptions: u32,
            samDesired: u32,
            phkResult: *mut usize,
        ) -> i32;
        fn RegQueryValueExW(
            hKey: usize,
            lpValueName: *const u16,
            lpReserved: *mut u32,
            lpType: *mut u32,
            lpData: *mut u8,
            lpcbData: *mut u32,
        ) -> i32;
        fn RegCloseKey(hKey: usize) -> i32;
    }

    const HKEY_CURRENT_USER: usize = 0x80000001;
    const KEY_READ: u32 = 0x20019;

    let subkey: Vec<u16> = "Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize\0"
        .encode_utf16()
        .collect();
    let value_name: Vec<u16> = "AppsUseLightTheme\0".encode_utf16().collect();

    let mut hkey: usize = 0;
    unsafe {
        if RegOpenKeyExW(HKEY_CURRENT_USER, subkey.as_ptr(), 0, KEY_READ, &mut hkey) == 0 {
            let mut data: u32 = 0;
            let mut data_len = std::mem::size_of::<u32>() as u32;
            let mut val_type: u32 = 0;
            let res = RegQueryValueExW(
                hkey,
                value_name.as_ptr(),
                std::ptr::null_mut(),
                &mut val_type,
                &mut data as *mut u32 as *mut u8,
                &mut data_len,
            );
            let _ = RegCloseKey(hkey);
            if res == 0 {
                return data == 0; // 0 = Dark mode, 1 = Light mode
            }
        }
    }
    true // default to Dark
}

#[cfg(not(windows))]
pub fn detect_system_is_dark() -> bool {
    true
}

#[derive(Clone, Debug)]
pub struct Theme {
    pub is_dark: bool,
    pub bg: Rgba,
    pub header_bg: Rgba,
    pub panel_bg: Rgba,
    pub card_bg: Rgba,
    pub card_hover: Rgba,
    pub card_border: Rgba,
    pub card_border_hover: Rgba,
    pub text_primary: Rgba,
    pub text_secondary: Rgba,
    pub text_muted: Rgba,
    pub accent: Rgba,
    pub accent_hover: Rgba,
    pub accent_subtle: Rgba,
    pub success: Rgba,
    pub success_subtle: Rgba,
    pub warning: Rgba,
    pub warning_subtle: Rgba,
    pub danger: Rgba,
    pub danger_subtle: Rgba,
    pub input_bg: Rgba,
    pub input_border: Rgba,
    pub hover_overlay: Rgba,
    pub active_overlay: Rgba,
    pub selection: Rgba,
    pub tree_guide: Rgba,
    pub graph_out: Rgba,
    pub graph_in: Rgba,
    pub track_on: Rgba,
    pub track_off: Rgba,
    pub thumb: Rgba,
}

impl Theme {
    pub fn dark() -> Self {
        Self {
            is_dark: true,
            bg: rgba_const(0x13151aff),
            header_bg: rgba_const(0x181a20ff),
            panel_bg: rgba_const(0x16181fff),
            card_bg: rgba_const(0x1e2029ff),
            card_hover: rgba_const(0x252834ff),
            card_border: rgba_const(0x2d313eff),
            card_border_hover: rgba_const(0x3e4456ff),
            text_primary: rgba_const(0xf3f4f6ff),
            text_secondary: rgba_const(0xa1a1aaff),
            text_muted: rgba_const(0x71717aff),
            accent: rgba_const(0x3b82f6ff),
            accent_hover: rgba_const(0x60a5faff),
            accent_subtle: rgba_const(0x3b82f626),
            success: rgba_const(0x10b981ff),
            success_subtle: rgba_const(0x10b98126),
            warning: rgba_const(0xf59e0bff),
            warning_subtle: rgba_const(0xf59e0b26),
            danger: rgba_const(0xef4444ff),
            danger_subtle: rgba_const(0xef444426),
            input_bg: rgba_const(0x181a22ff),
            input_border: rgba_const(0x323644ff),
            hover_overlay: rgba_const(0xffffff0f),
            active_overlay: rgba_const(0xffffff1a),
            selection: rgba_const(0x3b82f638),
            tree_guide: rgba_const(0x2e3240ff),
            graph_out: rgba_const(0xf43f5eff),
            graph_in: rgba_const(0xeab308ff),
            track_on: rgba_const(0x3b82f6ff),
            track_off: rgba_const(0x2d313eff),
            thumb: rgba_const(0xffffffff),
        }
    }

    pub fn light() -> Self {
        Self {
            is_dark: false,
            bg: rgba_const(0xf4f5f7ff),
            header_bg: rgba_const(0xebedf1ff),
            panel_bg: rgba_const(0xffffffff),
            card_bg: rgba_const(0xffffffff),
            card_hover: rgba_const(0xf8fafcff),
            card_border: rgba_const(0xdcdfe4ff),
            card_border_hover: rgba_const(0xc0c4ccff),
            text_primary: rgba_const(0x181a1fff),
            text_secondary: rgba_const(0x4b5563ff),
            text_muted: rgba_const(0x9ca3afff),
            accent: rgba_const(0x2563ebff),
            accent_hover: rgba_const(0x1d4ed8ff),
            accent_subtle: rgba_const(0x2563eb1f),
            success: rgba_const(0x059669ff),
            success_subtle: rgba_const(0x0596691f),
            warning: rgba_const(0xd97706ff),
            warning_subtle: rgba_const(0xd977061f),
            danger: rgba_const(0xdc2626ff),
            danger_subtle: rgba_const(0xdc26261f),
            input_bg: rgba_const(0xffffffff),
            input_border: rgba_const(0xd1d5dbff),
            hover_overlay: rgba_const(0x0000000a),
            active_overlay: rgba_const(0x00000014),
            selection: rgba_const(0x2563eb28),
            tree_guide: rgba_const(0xe2e8f0ff),
            graph_out: rgba_const(0xe11d48ff),
            graph_in: rgba_const(0xca8a04ff),
            track_on: rgba_const(0x2563ebff),
            track_off: rgba_const(0xd1d5dbff),
            thumb: rgba_const(0xffffffff),
        }
    }

    pub fn for_mode(mode: ThemeMode) -> Self {
        match mode {
            ThemeMode::Dark => Self::dark(),
            ThemeMode::Light => Self::light(),
            ThemeMode::System => {
                if detect_system_is_dark() {
                    Self::dark()
                } else {
                    Self::light()
                }
            }
        }
    }
}
