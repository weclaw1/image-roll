
# Image Roll
![Image Roll](https://raw.githubusercontent.com/weclaw1/image-roll/main/src/resources/com.github.weclaw1.ImageRoll.svg)

**Image Roll** is a simple and fast GTK image viewer with basic image manipulation tools.

## Features
- Written in Rust
- uses modern GTK 3
- adaptive - can be used on desktop and mobile devices
- crop image
- rotate image
- resize image
- undo and redo image edits

![Screenshot](https://raw.githubusercontent.com/weclaw1/image-roll/main/src/resources/screenshot.png)

## Installation

### Requirements
If you use Flatpak you may skip this section.

For this application you are required to have at least GTK 3.22.

#### Ubuntu/Debian
```
sudo apt install libgtk-3-dev
```
#### Fedora/CentOS
```
sudo yum install gtk3-devel glib2-devel
```

### Flatpak
Flatpak is the recommended install method.
In order to install Image Roll using Flatpak run:
```
flatpak install flathub com.github.weclaw1.ImageRoll
```

### Precompiled binaries
Ready-to-go executables can be found on the releases page.

### Cargo
To install Image Roll using cargo run the following command:
```
cargo install image-roll
```