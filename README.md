# Niri Keypad

Niri Keypad is a global keypad and shortcut helper designed specifically for the [Niri](https://github.com/YaLTeR/niri) window manager. It provides a visual, interactive grid of shortcuts that can be contextual (based on the focused app) or global.

## Features

*   **Visual Keypad**: Displays a grid of actionable cards mapped to keys (F-keys, QWERTY layout).
*   **Context-Aware**: Show different menus depending on which application is currently focused (e.g., specific shortcuts for Firefox).
*   **Interactive**: Supports both keyboard (Hotkeys, Arrow Keys + Enter) and Mouse clicks.
*   **Icon Theme Support**: Automatically finds and uses system icons (XDG compliant) or supports direct file paths.
*   **Daemon Mode**: Runs in the background for instant appearance.
*   **Rich Configuration**: Fully customizable layouts via `config.toml`.

## Installation

### Dependencies
You need GTK4 and Layer Shell support libraries.
On Debian/Ubuntu:
```bash
sudo apt install libgtk-4-dev libgtk4-layer-shell-dev
```

### Build
Clone the repository and build with Cargo:
```bash
cargo build --release
```
The binary will be at `target/release/niri-keypad`.

## Configuration

Configuration is located at `~/.config/niri-keypad/config.toml`.

### Basic Structure

```toml
[settings]
width = 1800
height = 800
theme = "dark"
icon_theme = "Papirus" # Optional: Force specific icon theme

# Global F-Key Actions (Always visible on top row)
[[global]]
key = "F1"
name = "Help"
icon = "help-browser" # Uses system icon
cmd = "xdg-open https://github.com/fortystory/niri-keypad"

# Switcher Menu Action
[[global]]
key = "F2"
name = "Switch App"
icon = "preferences-system-windows"
action = "menu:switcher"

# Define a Menu
[[menu]]
name = "switcher"
title = "Application Switcher"
items = [
    { key = "f", name = "Firefox", icon = "firefox", cmd = "niri msg action focus-window --class firefox" },
    { key = "k", name = "Kitty",   icon = "kitty",   cmd = "niri msg action focus-window --class kitty" }
]

# Context Mapping (Auto-switch menu based on focused app)
[[context]]
app_id = "firefox"
menu = "browser_shortcuts"
```

See `config.example.toml` for a complete reference.

### Helper Scripts
You can use helper scripts for complex actions. A `switch-app.sh` script is often useful for robust application switching:
```bash
# ~/.config/niri-keypad/switch-app.sh
APP_ID="$1"
WIN_ID=$(niri msg -j windows | jq -r ".[] | select(.app_id == \"$APP_ID\") | .id" | head -n1)

if [ -n "$WIN_ID" ]; then
    niri msg action focus-window --id "$WIN_ID"
else
    notify-send "Niri Keypad" "Application '$APP_ID' not found."
fi
```

## Usage

1.  **Start the Server**:
    Run the daemon in your Niri startup config (`spawn-at-startup`).
    ```bash
    niri-keypad server
    ```

2.  **Trigger the Keypad**:
    Bind a key in `niri.kdl` to open the keypad.
    ```kdl
    binds {
        Mod+Space { spawn "niri-keypad" "open"; }
    }
    ```

3.  **Command Mode**:
    You can open specific menus directly:
    ```bash
    niri-keypad open --menu switcher
    ```

## Development

*   **Language**: Rust
*   **GUI**: GTK4
*   **Protocol**: Wayland (Layer Shell)

## License

MIT
