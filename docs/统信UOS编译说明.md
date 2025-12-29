# 统信 UOS / Linux 平台编译说明

本程序支持跨平台编译，可在 Windows 和统信 UOS（Deepin/Linux）系统上运行。

## 编译环境准备

### 1. 安装 Rust 工具链

在统信 UOS 终端中执行：

```bash
# 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 重新加载环境变量
source ~/.cargo/env

# 验证安装
rustc --version
cargo --version
```

### 2. 安装系统依赖

统信 UOS 基于 Debian，使用 apt 包管理器：

```bash
# 更新软件源
sudo apt update

# 安装编译依赖
sudo apt install -y \
    build-essential \
    cmake \
    pkg-config \
    libgtk-3-dev \
    libxcb-render0-dev \
    libxcb-shape0-dev \
    libxcb-xfixes0-dev \
    libxkbcommon-dev \
    libssl-dev \
    libfontconfig1-dev \
    libfreetype6-dev
```

### 3. 安装中文字体（如果缺失）

程序会自动尝试加载以下字体之一：
- Noto Sans CJK（推荐）
- 文泉驿正黑/微米黑
- 思源黑体

```bash
# 安装 Noto CJK 字体（推荐）
sudo apt install -y fonts-noto-cjk

# 或安装文泉驿字体
sudo apt install -y fonts-wqy-zenhei fonts-wqy-microhei
```

## 编译步骤

### 1. 获取源代码

```bash
# 从 Windows 拷贝或通过 Git 获取
git clone <repository-url>
cd 质量监督检查抽签程序
```

### 2. 编译 Release 版本

```bash
# 编译优化版本
cargo build --release

# 编译完成后，可执行文件位于：
# ./target/release/quality_draw
```

### 3. 运行程序

```bash
./target/release/quality_draw
```

## 创建桌面快捷方式

创建 `.desktop` 文件以便从桌面启动：

```bash
# 创建 desktop 文件
cat > ~/.local/share/applications/quality_draw.desktop << EOF
[Desktop Entry]
Name=质量监督检查抽签程序
Name[zh_CN]=质量监督检查抽签程序
Comment=宁夏特检院质量监督检查随机抽签工具
Exec=/path/to/quality_draw
Icon=utilities-terminal
Terminal=false
Type=Application
Categories=Office;Utility;
EOF
```

> **注意**：将 `/path/to/quality_draw` 替换为实际的可执行文件路径。

## 常见问题

### Q1: 程序启动后中文显示为方块

**原因**：系统缺少中文字体

**解决方案**：
```bash
sudo apt install -y fonts-noto-cjk fonts-wqy-zenhei
```

### Q2: 编译报错缺少 GTK 开发库

**解决方案**：
```bash
sudo apt install -y libgtk-3-dev
```

### Q3: 运行时报 "无法打开显示器" 错误

**原因**：在无图形界面环境下运行

**解决方案**：确保在桌面环境下运行程序，或设置 DISPLAY 环境变量

### Q4: Wayland 下程序无法启动

**解决方案**：尝试使用 X11 后端
```bash
export WAYLAND_DISPLAY=""
./quality_draw
```

## 跨平台编译（在 Windows 上为 Linux 编译）

如果需要在 Windows 上交叉编译 Linux 版本：

```bash
# 添加 Linux 目标
rustup target add x86_64-unknown-linux-gnu

# 交叉编译（需要配置交叉编译工具链）
cargo build --release --target x86_64-unknown-linux-gnu
```

> **注意**：交叉编译需要额外配置链接器和系统库，建议直接在目标系统上编译。

## 技术说明

### 支持的平台

| 平台 | 架构 | 状态 |
|------|------|------|
| Windows 10/11 | x86_64 | ✅ 完全支持 |
| 统信 UOS | x86_64 | ✅ 完全支持 |
| Deepin | x86_64 | ✅ 完全支持 |
| Ubuntu/Debian | x86_64 | ✅ 完全支持 |
| 麒麟 Kylin | x86_64/arm64 | ⚠️ 未测试 |

### 字体加载顺序

程序会按以下顺序尝试加载中文字体：

**Linux/统信UOS**:
1. Noto Sans CJK
2. 文泉驿正黑 (WenQuanYi Zen Hei)
3. 文泉驿微米黑 (WenQuanYi Micro Hei)
4. Droid Sans Fallback
5. 思源黑体 (Source Han Sans)

**Windows**:
1. 微软雅黑 (Microsoft YaHei)
2. 黑体 (SimHei)
3. 宋体 (SimSun)
