#[cfg(target_os = "windows")]
use adw::Application;
#[cfg(target_os = "windows")]
use adw::prelude::{ApplicationExt, GtkWindowExt};
#[cfg(target_os = "windows")]
use async_channel::Receiver;
#[cfg(target_os = "windows")]
use gtk::{gdk_pixbuf::Pixbuf, glib};
#[cfg(target_os = "windows")]
use gtk::glib::prelude::ObjectExt;
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
            None => return,
        };
        let menu = Menu::new();
        let open_item = MenuItem::new("Open", true, None);
        let quit_item = MenuItem::new("Quit", true, None);
        let _ = menu.append(&open_item);
        let _ = menu.append(&quit_item);

        let _tray = TrayIconBuilder::new()
            .with_tooltip("Lan Mouse")
            .with_menu(Box::new(menu))
            .with_icon(icon)
            .build();

        let menu_receiver = MenuEvent::receiver();
        let open_id = open_item.id();
        let quit_id = quit_item.id();

        let event_loop = EventLoop::new();
        event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Wait;
            if matches!(event, Event::NewEvents(StartCause::Init)) {
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
    const ICON_RESOURCE: &str = "/de/feschber/LanMouse/icons/tray-icon.png";
    let pixbuf = Pixbuf::from_resource_at_scale(ICON_RESOURCE, 22, 22, true).ok()?;

    let width = pixbuf.width();
    let height = pixbuf.height();
    let rowstride = pixbuf.rowstride() as usize;
    let n_channels = pixbuf.n_channels() as usize;
    let has_alpha = pixbuf.has_alpha();
    let bytes = pixbuf.read_pixel_bytes();
    let data = bytes.as_ref();

    if width <= 0 || height <= 0 || n_channels < 3 {
        return None;
    }

    let mut rgba = Vec::with_capacity((width * height * 4) as usize);
    for y in 0..height as usize {
        let row = &data[y * rowstride..];
        for x in 0..width as usize {
            let idx = x * n_channels;
            let r = row[idx];
            let g = row[idx + 1];
            let b = row[idx + 2];
            let a = if has_alpha && n_channels >= 4 {
                row[idx + 3]
            } else {
                255
            };
            rgba.extend_from_slice(&[r, g, b, a]);
        }
    }

    tray_icon::Icon::from_rgba(rgba, width as u32, height as u32).ok()
}
