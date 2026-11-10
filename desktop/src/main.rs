#[cfg(feature = "native-ui")]
mod native {
    pub(crate) use photon_app::App;
    pub(crate) use photon_app::Intent;
    pub(crate) use photon_blocker::AdblockFilter;
    pub(crate) use photon_engine::HeadlessEngine;
    pub(crate) use photon_store::SqliteStore;
    pub(crate) use tao::{
        event::{Event, WindowEvent},
        event_loop::{ControlFlow, EventLoop},
        window::WindowBuilder,
    };
    pub(crate) use wry::{
        dpi::{LogicalPosition, LogicalSize},
        Rect, WebView, WebViewBuilder,
    };

    pub(crate) fn run() -> Result<(), Box<dyn std::error::Error>> {
        let mut app = App::new(
            HeadlessEngine::default(),
            SqliteStore::open_in_memory().unwrap(),
            AdblockFilter::with_seed(),
        );
        app.handle(Intent::OpenTab {
            url: "https://example.com".into(),
        })?;

        let event_loop = EventLoop::new();
        let window = WindowBuilder::new()
            .with_title("Photon")
            .with_inner_size(tao::dpi::LogicalSize::new(1200, 800))
            .build(&event_loop)?;

        #[cfg(not(any(
            target_os = "windows",
            target_os = "macos",
            target_os = "ios",
            target_os = "android"
        )))]
        let webview: WebView = {
            use gtk::prelude::*;
            use tao::platform::unix::WindowExtUnix;
            use wry::WebViewBuilderExtUnix;
            let fixed = gtk::Fixed::new();
            let vbox = window.default_vbox().unwrap();
            vbox.pack_start(&fixed, true, true, 0);
            fixed.show_all();
            let size = window.inner_size().to_logical::<u32>(window.scale_factor());
            WebViewBuilder::new()
                .with_bounds(Rect {
                    position: LogicalPosition::new(0, 0).into(),
                    size: LogicalSize::new(size.width, size.height).into(),
                })
                .with_url("https://example.com")
                .build_gtk(&fixed)?
        };

        #[cfg(any(
            target_os = "windows",
            target_os = "macos",
            target_os = "ios",
            target_os = "android"
        ))]
        let webview: WebView = {
            let size = window.inner_size().to_logical::<u32>(window.scale_factor());
            WebViewBuilder::new()
                .with_bounds(Rect {
                    position: LogicalPosition::new(0, 0).into(),
                    size: LogicalSize::new(size.width, size.height).into(),
                })
                .with_url("https://example.com")
                .build(&window)?
        };

        event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Wait;
            match event {
                Event::WindowEvent {
                    event: WindowEvent::Resized(new_size),
                    ..
                } => {
                    let s = new_size.to_logical::<u32>(window.scale_factor());
                    let _ = webview.set_bounds(Rect {
                        position: LogicalPosition::new(0, 0).into(),
                        size: LogicalSize::new(s.width, s.height).into(),
                    });
                }
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => *control_flow = ControlFlow::Exit,
                _ => {}
            }
        });
    }
}

#[cfg(not(feature = "native-ui"))]
mod headless {
    pub(crate) use photon_app::App;
    pub(crate) use photon_app::Intent;
    pub(crate) use photon_blocker::AdblockFilter;
    pub(crate) use photon_engine::HeadlessEngine;
    pub(crate) use photon_store::SqliteStore;

    pub(crate) fn run() -> Result<(), Box<dyn std::error::Error>> {
        let mut app = App::new(
            HeadlessEngine::default(),
            SqliteStore::open_in_memory().unwrap(),
            AdblockFilter::with_seed(),
        );
        app.handle(Intent::OpenTab {
            url: "https://example.com".into(),
        })?;

        let state = app.state();
        println!("tabs: {}", state.tabs.len());
        for tab in &state.tabs {
            println!("  {} {} {:?}", tab.id, tab.url, tab.state);
        }
        println!("active tab: {:?}", state.active_tab);
        println!(
            "blocker: doubleclick.net blocked={}",
            app.should_block("https://doubleclick.net/x", "https://example.com")
        );
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "native-ui")]
    return native::run();
    #[cfg(not(feature = "native-ui"))]
    return headless::run();
}
