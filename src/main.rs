#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod ai;
mod app;
#[cfg(target_os = "macos")]
mod menu_bar;
mod ui;

use std::path::PathBuf;

fn parse_screenshot_path() -> Option<PathBuf> {
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == "--screenshot" {
            return Some(args.next().map(PathBuf::from).unwrap_or_else(|| {
                PathBuf::from("clipflow-screenshot.png")
            }));
        }
        if let Some(rest) = arg.strip_prefix("--screenshot=") {
            return Some(PathBuf::from(rest));
        }
    }
    None
}

fn main() -> eframe::Result<()> {
    let shot_path = parse_screenshot_path();
    let screenshot = shot_path.is_some();

    let mut viewport = eframe::egui::ViewportBuilder::default()
        .with_title("ClipFlow")
        .with_inner_size([1040.0, 700.0])
        .with_min_inner_size([820.0, 520.0])
        .with_app_id("clipflow");

    if screenshot {
        // Never steal focus from what the person is doing.
        viewport = viewport.with_active(false);
        // Lets a preview run use a smaller window, to check small-screen layouts.
        if let Some(size) = std::env::var("CLIPFLOW_SIZE").ok().and_then(|text| {
            let (w, h) = text.split_once(['x', 'X'])?;
            Some([w.trim().parse::<f32>().ok()?, h.trim().parse::<f32>().ok()?])
        }) {
            viewport = viewport.with_inner_size(size);
        }
        #[cfg(target_os = "windows")]
        {
            viewport = viewport.with_window_level(eframe::egui::WindowLevel::AlwaysOnBottom);
        }
    }

    let mut native = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    #[cfg(target_os = "macos")]
    if screenshot {
        native.event_loop_builder = Some(Box::new(|builder| {
            use winit::platform::macos::EventLoopBuilderExtMacOS as _;
            builder.with_activate_ignoring_other_apps(false);
        }));
    }

    let app = eframe::run_native(
        "ClipFlow",
        native,
        Box::new(move |cc| Ok(Box::new(app::ClipFlow::new(cc, shot_path.clone())))),
    );
    app
}
