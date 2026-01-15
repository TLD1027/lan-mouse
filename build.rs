use shadow_rs::ShadowBuilder;

fn main() {
    ShadowBuilder::builder()
        .deny_const(Default::default())
        .build()
        .expect("shadow build");

    #[cfg(windows)]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon("lan-mouse-gtk/resources/de.feschber.LanMouse.ico");
        res.compile().expect("winres");
    }
}
