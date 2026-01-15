#[cfg(target_os = "windows")]
use adw::Application;
#[cfg(target_os = "windows")]
use adw::prelude::{ApplicationExt, GtkWindowExt};
#[cfg(target_os = "windows")]
use async_channel::Receiver;
#[cfg(target_os = "windows")]
use gtk::glib;
#[cfg(target_os = "windows")]
use gtk::glib::prelude::ObjectExt;
#[cfg(target_os = "windows")]
use image::GenericImageView;
#[cfg(target_os = "windows")]
use tao::event::{Event, StartCause};
#[cfg(target_os = "windows")]
use tao::event_loop::{ControlFlow, EventLoop};
#[cfg(target_os = "windows")]
use tray_icon::{
    menu::{Menu, MenuEvent, MenuItem},
    TrayIconBuilder,
};

#[cfg(target_os = "windows")]
use crate::window::Window;

#[cfg(target_os = "windows")]
#[derive(Clone, Copy, Debug)]
pub enum TrayCommand {
    Show,
    Quit,
}

#[cfg(target_os = "windows")]
pub fn setup_tray(app: &Application, window: &Window) {
    let (sender, receiver) = async_channel::bounded(8);
    let app_weak = app.downgrade();
    let window_weak = window.downgrade();

    glib::spawn_future_local(async move {
        handle_commands(receiver, app_weak, window_weak).await;
    });

    std::thread::spawn(move || {
        let icon = match load_tray_icon() {
            Some(icon) => icon,
            None => {
                log::warn!("tray icon failed: icon not loaded");
                return;
            }
        };
        let menu = Menu::new();
        let open_item = MenuItem::new("Open", true, None);
        let quit_item = MenuItem::new("Quit", true, None);
        let _ = menu.append(&open_item);
        let _ = menu.append(&quit_item);

        let menu_receiver = MenuEvent::receiver();
        let open_id = open_item.id().clone();
        let quit_id = quit_item.id().clone();

        let event_loop = EventLoop::new();
        let mut tray = None;
        let mut menu = Some(menu);
        let mut icon = Some(icon);
        event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Wait;
            if matches!(event, Event::NewEvents(StartCause::Init)) {
                if tray.is_none() {
                    let menu = menu.take().unwrap();
                    let icon = icon.take().unwrap();
                    tray = TrayIconBuilder::new()
                        .with_tooltip("Lan Mouse")
                        .with_menu(Box::new(menu))
                        .with_icon(icon)
                        .build()
                        .ok();
                    if tray.is_none() {
                        log::warn!("tray icon failed: build error");
                    }
                }
                return;
            }
            while let Ok(event) = menu_receiver.try_recv() {
                if event.id == open_id {
                    let _ = sender.try_send(TrayCommand::Show);
                } else if event.id == quit_id {
                    let _ = sender.try_send(TrayCommand::Quit);
                }
            }
        });
    });
}

#[cfg(target_os = "windows")]
async fn handle_commands(
    receiver: Receiver<TrayCommand>,
    app_weak: glib::WeakRef<Application>,
    window_weak: glib::WeakRef<Window>,
) {
    while let Ok(cmd) = receiver.recv().await {
        match cmd {
            TrayCommand::Show => {
                if let Some(window) = window_weak.upgrade() {
                    window.present();
                }
            }
            TrayCommand::Quit => {
                if let Some(app) = app_weak.upgrade() {
                    app.quit();
                }
            }
        }
    }
}

#[cfg(target_os = "windows")]
fn load_tray_icon() -> Option<tray_icon::Icon> {
    if let Ok(icon) = tray_icon::Icon::from_resource_name("IDI_ICON1", None) {
        return Some(icon);
    }

    let bytes = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/resources/tray-icon.png"));
    let image = image::load_from_memory(bytes).ok()?;
    let rgba = image.into_rgba8();
    let (width, height) = rgba.dimensions();
    tray_icon::Icon::from_rgba(rgba.into_raw(), width, height).ok()
}
