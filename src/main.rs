// Hide the console window for release builds on Windows.
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

use std::{
    backtrace::Backtrace,
    fs::OpenOptions,
    io::Write as _,
    panic,
    time::{SystemTime, UNIX_EPOCH},
};

use gpui::*;
use hfs_rs::i18n::{self, Msg};
use hfs_rs::ui::HfsApp;

fn main() {
    install_panic_log();

    let cfg = hfs_rs::AppConfig::load_or_default();
    hfs_rs::i18n::set(cfg.locale);

    let app = gpui_platform::application();

    app.run(move |cx| {
        let window_options = WindowOptions {
            window_bounds: Some(WindowBounds::centered(size(px(1080.), px(720.)), cx)),
            titlebar: Some(TitlebarOptions {
                title: Some(Msg::AppTitle.get(i18n::current()).into()),
                ..Default::default()
            }),
            ..Default::default()
        };

        cx.spawn(async move |cx| {
            cx.open_window(window_options, |window, cx| {
                cx.new(|cx| HfsApp::new(window, cx))
            })
            .expect("Failed to open window");
        })
        .detach();
    });
}

fn install_panic_log() {
    let default_hook = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |duration| duration.as_secs());
        let path = std::env::temp_dir().join("hfs-rs-crash.log");
        if let Ok(mut log) = OpenOptions::new().create(true).append(true).open(path) {
            let backtrace = Backtrace::force_capture();
            let _ = writeln!(log, "[{timestamp}] {info}\n{backtrace}");
        }
        default_hook(info);
    }));
}
