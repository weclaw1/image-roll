use std::process::Command;

fn main() {
    Command::new("glib-compile-resources")
        .args(&["src/resources/resources.xml", "--sourcedir=src/resources"])
        .status()
        .unwrap();

    println!("cargo:rerun-if-changed=src/resources/resources.xml");
    println!("cargo:rerun-if-changed=src/resources/image-roll_ui.glade");
    println!("cargo:rerun-if-changed=src/resources/com.github.weclaw1.ImageRoll.svg");
    println!("cargo:rerun-if-changed=src/resources/crop_icon.svg");
    println!("cargo:rerun-if-changed=src/resources/resize_icon.svg");
}
