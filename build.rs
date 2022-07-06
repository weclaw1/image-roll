use std::{env, process::Command};

fn main() {
    Command::new("glib-compile-resources")
        .args(&["src/resources/resources.xml", "--sourcedir=src/resources"])
        .status()
        .unwrap();

    let python_installed = Command::new("sh")
        .args(&["-c", "command -v python3"])
        .status()
        .unwrap()
        .success();
    let pip_installed = Command::new("sh")
        .args(&["-c", "command -v pip3"])
        .status()
        .unwrap()
        .success();
    let wget_installed = Command::new("sh")
        .args(&["-c", "command -v wget"])
        .status()
        .unwrap()
        .success();

    if python_installed && pip_installed && wget_installed {
        Command::new("pip3")
            .args(&["install", "aiohttp", "toml"])
            .status()
            .unwrap();
        Command::new("wget").arg("https://raw.githubusercontent.com/flatpak/flatpak-builder-tools/master/cargo/flatpak-cargo-generator.py").status().unwrap();
        Command::new("python3")
            .args(&[
                "flatpak-cargo-generator.py",
                "Cargo.lock",
                "-o",
                "src/resources/cargo-sources.json",
            ])
            .status()
            .unwrap();
        Command::new("rm")
            .arg("flatpak-cargo-generator.py")
            .status()
            .unwrap();
    }

    if Ok("debug".to_owned()) == env::var("PROFILE") {
        Command::new("sh")
            .args(&["-c", "mkdir -p $HOME/.local/share/glib-2.0/schemas"])
            .status()
            .unwrap();
        Command::new("sh").args(&["-c", "install -D src/resources/com.github.weclaw1.ImageRoll.gschema.xml $HOME/.local/share/glib-2.0/schemas/"]).status().unwrap();
        Command::new("sh")
            .args(&[
                "-c",
                "glib-compile-schemas $HOME/.local/share/glib-2.0/schemas/",
            ])
            .status()
            .unwrap();
    }

    println!("cargo:rerun-if-changed=src/resources/resources.xml");
    println!("cargo:rerun-if-changed=src/resources/image-roll.ui");
    println!("cargo:rerun-if-changed=src/resources/icons/crop-symbolic.svg");
    println!("cargo:rerun-if-changed=src/resources/com.github.weclaw1.ImageRoll.svg");
    println!("cargo:rerun-if-changed=src/resources/com.github.weclaw1.ImageRoll.gschema.xml");
    println!("cargo:rerun-if-changed=Cargo.lock");
}
