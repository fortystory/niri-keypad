# Niri Keypad - Niri 快捷键助手

Niri Keypad 是专为 [Niri](https://github.com/YaLTeR/niri) 窗口管理器设计的全局键盘与快捷键助手。它提供了一个可视化的、可交互的网格界面，用于显示和执行快捷键，支持根据当前聚焦的应用显示上下文菜单。

## 功能特性

*   **可视化键盘**: 显示映射到 F 键和 QWERTY 布局的交互式卡片。
*   **上下文感知**: 根据当前聚焦的应用程序自动切换不同的菜单（例如：Firefox 专用快捷键）。
*   **交互支持**: 支持键盘操作（热键、方向键+回车）和鼠标点击。
*   **图标主题支持**: 遵循 XDG 规范自动查找并使用系统图标，也支持直接指定图标文件路径。
*   **守护进程模式**: 在后台运行，确保界面瞬间呼出。
*   **强大的配置**: 通过 `config.toml` 完全自定义布局和行为。

## 安装指南

### 依赖项
你需要安装 GTK4 和 Layer Shell 支持库。
在 Debian/Ubuntu 上：
```bash
sudo apt install libgtk-4-dev libgtk4-layer-shell-dev
```

### 编译
克隆仓库并使用 Cargo 编译：
```bash
cargo build --release
```
编译后的二进制文件位于 `target/release/niri-keypad`。

## 配置说明

配置文件位于 `~/.config/niri-keypad/config.toml`。

### 基础结构示例

```toml
[settings]
width = 1800
height = 800
theme = "dark"
icon_theme = "Papirus" # 可选: 强制使用特定的图标主题

# 全局 F 键动作 (始终显示在顶部行)
[[global]]
key = "F1"
name = "帮助"
icon = "help-browser" # 使用系统图标
cmd = "xdg-open https://github.com/fortystory/niri-keypad"

# 切换器菜单动作
[[global]]
key = "F2"
name = "切换应用"
icon = "preferences-system-windows"
action = "menu:switcher"

# 定义一个菜单
[[menu]]
name = "switcher"
title = "应用切换器"
items = [
    { key = "f", name = "Firefox", icon = "firefox", cmd = "niri msg action focus-window --class firefox" },
    { key = "k", name = "Kitty",   icon = "kitty",   cmd = "niri msg action focus-window --class kitty" }
]

# 上下文映射 (根据聚焦的应用自动切换菜单)
[[context]]
app_id = "firefox"
menu = "browser_shortcuts"
```

完整参考请查看 `config.example.toml`。

### 辅助脚本
对于复杂的交互（如确保应用运行），推荐使用辅助脚本。例如 `switch-app.sh`：
```bash
# ~/.config/niri-keypad/switch-app.sh
APP_ID="$1"
# 查询窗口 ID
WIN_ID=$(niri msg -j windows | jq -r ".[] | select(.app_id == \"$APP_ID\") | .id" | head -n1)

if [ -n "$WIN_ID" ]; then
    # 找到窗口，直接聚焦
    niri msg action focus-window --id "$WIN_ID"
else
    # 未找到，发送通知
    notify-send "Niri Keypad" "应用 '$APP_ID' 未找到或未运行。"
fi
```

## 使用方法

1.  **启动服务端**:
    在你的 Niri 启动配置 (`spawn-at-startup`) 中运行守护进程。
    ```bash
    niri-keypad server
    ```

2.  **呼出界面**:
    在 `niri.kdl` 中绑定快捷键来打开界面。
    ```kdl
    binds {
        Mod+Space { spawn "niri-keypad" "open"; }
    }
    ```

3.  **命令行模式**:
    你可以直接通过命令打开特定菜单：
    ```bash
    niri-keypad open --menu switcher
    ```

## 开发信息

*   **语言**: Rust
*   **GUI 框架**: GTK4
*   **协议**: Wayland (Layer Shell)

## 许可证

MIT
