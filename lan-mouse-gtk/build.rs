fn main() {
    println!("cargo:rerun-if-changed=resources/resources.gresource.xml");
    println!("cargo:rerun-if-changed=resources/client_row.ui");
    println!("cargo:rerun-if-changed=resources/window.ui");
    println!("cargo:rerun-if-changed=resources/authorization_window.ui");
    println!("cargo:rerun-if-changed=resources/fingerprint_window.ui");
    println!("cargo:rerun-if-changed=resources/key_row.ui");
    println!("cargo:rerun-if-changed=resources/de.feschber.LanMouse.svg");
    println!("cargo:rerun-if-changed=resources/tray-icon.png");
    // composite_templates
    glib_build_tools::compile_resources(
        &["resources"],
        "resources/resources.gresource.xml",
        "lan-mouse.gresource",
    );
}
