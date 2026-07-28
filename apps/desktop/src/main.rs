use std::path::PathBuf;

use gpui::{App, AppContext, Application, Bounds, WindowBounds, WindowOptions, px, size};

mod app;
mod projects_view;
mod theme;

use app::OpenCutApp;

pub fn projects_root() -> PathBuf {
    std::env::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".local/share/opencut/projects")
}

fn main() {
    Application::new().run(|cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(1280.), px(720.)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| cx.new(|cx| OpenCutApp::new(cx)),
        )
        .unwrap();
        cx.activate(true);
    });
}
