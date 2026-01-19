# Niri-Keypad 设计文档

**日期:** 2026-01-16
**状态:** 已批准
**主题:** Niri 窗口管理器的全局 "Which-Key" HUD 工具

## 1. 概述

**Niri-Keypad** 是一个服务于 Niri 窗口管理器的全局、模态快捷键 HUD (Heads-Up Display) 工具。它的功能类似于 `vim-which-key` 或 `doom-emacs` 的 leader key，但工作在系统窗口管理器层面。

**核心价值:**
- **可发现性:** 可视化可用的快捷键。
- **上下文感知:** 根据当前聚焦的应用程序显示不同的快捷键。
- **速度:** 基于守护进程 (Daemon) 的架构，实现瞬间显示。
- **组织性:** 将快捷键分为 "全局" (系统) 和 "本地" (程序专属) 两个域。

## 2. 架构

### 2.1 技术栈
- **语言:** Rust
- **GUI 工具包:** GTK4 + `gtk4-layer-shell`
- **配置:** TOML
- **IPC:** 通过 Unix Socket 与 Niri 通信 (监听焦点变化) 并通过 CLI 发送动作。

### 2.2 组件模型
1.  **守护进程 (Daemon Process):**
    - 在后台运行 (`niri-keypad server`)。
    - 将配置加载到内存中。
    - 保持 GTK 窗口处于隐藏状态（或经过优化后的即时创建/销毁）。
    - 监听 Niri 事件流以追踪 `active_window_id` 和 `app_id`。
2.  **客户端/触发器 (Client/Trigger):**
    - `niri-keypad open` (触发守护进程显示窗口)。
    - 支持 `--menu <name>` 等参数以强制打开特定菜单。

## 3. 配置设计

使用 TOML 进行声明式配置。

**文件路径:** `~/.config/niri-keypad/config.toml`

### 3.1 结构
配置引入了 **菜单 (Menus)** (动作集合) 和 **上下文 (Contexts)** (将应用映射到菜单) 的概念。

```toml
[settings]
width = 800
height = 600
theme = "dark" # 或 path/to/style.css

# --- 全局动作 (顶部面板) ---
# 通常映射到 F1-F12 键
[[global]]
key = "F1"
name = "帮助"
cmd = "xdg-open https://github.com/..."

[[global]]
key = "F2"
name = "切换应用"
action = "menu:switcher" # 导航到下方的 'switcher' 菜单

# --- 具名菜单 (底部面板) ---
# 可复用的快捷键集合

[[menu]]
name = "switcher"
title = "应用程序切换器"
items = [
    { key = "f", name = "Firefox", cmd = "niri msg action focus-window --app-id firefox" },
    { key = "k", name = "Kitty",   cmd = "niri msg action focus-window --app-id kitty" }
]

[[menu]]
name = "browser_defaults"
title = "浏览器动作"
items = [
    { key = "n", name = "新窗口", cmd = "xdotool key Ctrl+n" },
    { key = "t", name = "新标签页", cmd = "xdotool key Ctrl+t" },
    { key = "w", name = "关闭标签页", cmd = "xdotool key Ctrl+w" }
]

# --- 上下文绑定 ---
# 将应用程序 ID 映射到菜单

# 当 Firefox 聚焦时，在底部面板显示 'browser_defaults'
[[context]]
app_id = "firefox"
menu = "browser_defaults"

# 如果没有匹配的 app_id，则使用回退/默认上下文
[[context]]
app_id = "default"
menu = "system_general"
```

## 4. UI/UX 设计

界面采用 **虚拟键盘** 布局，以最大化肌肉记忆。

### 4.1 布局
窗口居中显示，垂直分为两个不同的面板：

**A. 顶部面板 (全局功能行)**
- **网格:** 1 行 x 12 列。
- **按键:** `F1` - `F12`。
- **用途:** 系统级开关 (音量, 亮度, 截图) 或 模式切换 (切换到媒体模式, 窗口模式)。
- **样式:** 扁平矩形卡片。

**B. 底部面板 (主键盘区)**
- **网格:** 3 行 x 10 列。
- **按键:**
    - 第 1 行: `Q` `W` `E` `R` `T` `Y` `U` `I` `O` `P`
    - 第 2 行: `A` `S` `D` `F` `G` `H` `J` `K` `L` `;`
    - 第 3 行: `Z` `X` `C` `V` `B` `N` `M` `,` `.` `/`
- **用途:** 上下文相关动作。
- **样式:** 类似物理键帽的正方形卡片。
- **内容:**
    - 中心: 图标 (可选) 或 大字字符。
    - 角落: 绑定的按键字符。
    - 底部: 动作名称。

### 4.2 应用程序流程
1.  **空闲:** Daemon 在后台等待。
2.  **触发:** 用户按下 `Mod+W` (绑定到 `niri-keypad`)。
3.  **显示:** 窗口瞬间出现。
    - **逻辑:** `current_menu = resolve_context(active_window.app_id)`
    - **输入:** 独占键盘输入 (Keyboard Grab)。
4.  **交互:**
    - **按 F 键:** 执行全局动作 或 切换底部面板内容 (如果 `action = "menu:name"`)。
    - **按 字母键:** 执行底部面板的动作。
    - **按 Esc:** 关闭窗口。
5.  **执行:**
    - 如果 `cmd` 是 shell 命令: 启动进程，关闭窗口。
    - 如果 `action` 是菜单跳转: 更新 UI，保持窗口打开。

## 5. 开发计划

1.  **项目设置:** Rust, GTK4 bindings, Layer Shell protocol。
2.  **配置解析器:** 实现 TOML 反序列化。
3.  **Niri IPC:** 实现 Socket 监听器以获取窗口焦点。
4.  **UI 实现:**
    - 构建网格布局 (Grid Layouts)。
    - 创建 CSS 主题支持。
5.  **逻辑核心:** 处理输入和执行命令的状态机。
