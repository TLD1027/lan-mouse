#[cfg(target_os = "linux")]
use adw::Application;
#[cfg(target_os = "linux")]
use adw::prelude::{ApplicationExt, GtkWindowExt};
#[cfg(target_os = "linux")]
use async_channel::{Receiver, Sender};
#[cfg(target_os = "linux")]
use gtk::{
    gdk_pixbuf::Pixbuf,
    glib::{self, prelude::ObjectExt},
};
#[cfg(target_os = "linux")]
use crate::window::Window;

#[cfg(target_os = "linux")]
#[derive(Clone, Copy, Debug)]
pub enum TrayCommand {
    Show,
    Quit,
}

#[cfg(target_os = "linux")]
#[derive(Clone)]
struct LanMouseTray {
    sender: Sender<TrayCommand>,
    icon_pixmap: Vec<ksni::Icon>,
}

#[cfg(target_os = "linux")]
impl ksni::Tray for LanMouseTray {
    fn title(&self) -> String {
        "Lan Mouse".to_string()
    }

    fn icon_name(&self) -> String {
        String::new()
    }

    fn icon_pixmap(&self) -> Vec<ksni::Icon> {
        self.icon_pixmap.clone()
    }

    fn activate(&mut self, _x: i32, _y: i32) {
        let _ = self.sender.try_send(TrayCommand::Show);
    }

    fn menu(&self) -> Vec<ksni::menu::MenuItem<Self>> {
        vec![
            ksni::menu::StandardItem {
                label: "Open".to_string(),
                activate: Box::new(|tray: &mut LanMouseTray| {
                    let _ = tray.sender.try_send(TrayCommand::Show);
                }),
                ..Default::default()
            }
            .into(),
            ksni::menu::StandardItem {
                label: "Quit".to_string(),
                activate: Box::new(|tray: &mut LanMouseTray| {
                    let _ = tray.sender.try_send(TrayCommand::Quit);
                }),
                ..Default::default()
            }
            .into(),
        ]
    }
}

#[cfg(target_os = "linux")]
pub fn setup_tray(app: &Application, window: &Window) {
    let (sender, receiver) = async_channel::bounded(8);
    let app_weak = app.downgrade();
    let window_weak = window.downgrade();

    glib::spawn_future_local(async move {
        handle_commands(receiver, app_weak, window_weak).await;
    });

    let icon_pixmap = load_tray_icon_pixmap();
    let tray = LanMouseTray {
        sender,
        icon_pixmap,
    };
    let service = ksni::TrayService::new(tray);
    std::thread::spawn(move || {
        if let Err(err) = service.run() {
            log::warn!("tray icon failed: {err}");
        }
    });
}

#[cfg(target_os = "linux")]
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

#[cfg(target_os = "linux")]
fn load_tray_icon_pixmap() -> Vec<ksni::Icon> {
    const ICON_RESOURCE: &str = "/de/feschber/LanMouse/icons/tray-icon.png";
    let pixbuf = match Pixbuf::from_resource_at_scale(ICON_RESOURCE, 22, 22, true) {
        Ok(pixbuf) => pixbuf,
        Err(err) => {
            log::warn!("tray icon load failed: {err}");
            return Vec::new();
        }
    };

    let width = pixbuf.width();
    let height = pixbuf.height();
    let rowstride = pixbuf.rowstride() as usize;
    let n_channels = pixbuf.n_channels() as usize;
    let has_alpha = pixbuf.has_alpha();
    let bytes = pixbuf.read_pixel_bytes();
    let data = bytes.as_ref();

    if width <= 0 || height <= 0 || n_channels < 3 {
        return Vec::new();
    }

    let mut argb = Vec::with_capacity((width * height * 4) as usize);
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
            argb.extend_from_slice(&[a, r, g, b]);
        }
    }

    vec![ksni::Icon {
        width,
        height,
        data: argb,
    }]
}
