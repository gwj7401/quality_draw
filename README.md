# 宁夏特检院质量监督检查抽签程序

## 项目简介

这是一个用于宁夏特检院质量监督检查的抽签程序，旨在公平、公正地随机抽取检查对象和专家。项目使用 Rust 语言开发，基于 `eframe` GUI 框架。

## 功能特性

-   **随机抽签**: 支持按照部门、专业等条件进行随机抽取。
-   **动画效果**: 包含抽签转盘动画效果，提升用户体验。
-   **历史记录**: 自动保存抽签历史记录，方便追溯和查询。
-   **结果导出**: 支持将抽签结果导出为 Excel 表格。
-   **数据管理**: 支持人员库、部门库的维护和管理。
-   **系统设置**: 提供可配置的系统参数设置。

## 技术栈

-   **编程语言**: Rust
-   **GUI 框架**: [eframe](https://github.com/emilk/egui/tree/master/crates/eframe) (egui)
-   **数据存储**: JSON 文件存储
-   **Excel 处理**: rust_xlsxwriter

## 构建与运行

确保已安装 [Rust](https://www.rust-lang.org/tools/install) 开发环境。

### 运行开发版本

```bash
cargo run
```

### 构建发布版本

```bash
cargo build --release
```

构建产物将位于 `target/release/quality_draw.exe`。

## 目录结构

-   `src/ui`: 界面相关代码（主面板、历史记录、设置、导出等）
-   `src/logic`: 核心业务逻辑（抽签算法）
-   `src/models`: 数据模型定义（部门、专家、记录等）
-   `src/storage`: 数据持久化存储实现
