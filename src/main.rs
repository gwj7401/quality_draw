//! 宁夏特检院质量监督检查抽签程序
//! 
//! 功能：
//! - 支持部门选择和随机抽签
//! - 同部门规避机制
//! - 连续抽取规避
//! - 滚动动画效果
//! - 结果导出到Excel
//! - 打印功能

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod models;
mod storage;
mod logic;
mod ui;
mod app;

use app::QualityDrawApp;
use eframe::egui;
use std::io::Write;

fn main() -> eframe::Result<()> {
    // 设置panic hook，将崩溃信息写入日志文件
    std::panic::set_hook(Box::new(|panic_info| {
        let log_path = get_log_path();
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path) 
        {
            let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
            let _ = writeln!(file, "[{}] PANIC: {}", timestamp, panic_info);
        }
    }));
    
    // 记录启动日志
    log_message("程序启动...");
    
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1024.0, 700.0])
            .with_min_inner_size([800.0, 600.0])
            .with_title("宁夏特检院质量监督检查抽签程序"),
        ..Default::default()
    };
    
    log_message("创建窗口...");
    
    let result = eframe::run_native(
        "质量监督检查抽签程序",
        options,
        Box::new(|cc| {
            log_message("初始化应用...");
            // 配置中文字体
            setup_fonts(&cc.egui_ctx);
            log_message("字体配置完成");
            Ok(Box::new(QualityDrawApp::new(cc)))
        }),
    );
    
    // 记录结果
    match &result {
        Ok(_) => log_message("程序正常退出"),
        Err(e) => log_message(&format!("程序错误: {:?}", e)),
    }
    
    result
}

/// 获取日志文件路径
fn get_log_path() -> std::path::PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("quality_draw.log")
}

/// 记录日志
fn log_message(msg: &str) {
    let log_path = get_log_path();
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path) 
    {
        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
        let _ = writeln!(file, "[{}] {}", timestamp, msg);
    }
}

/// 配置中文字体
fn setup_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    
    // 加载系统中文字体
    if let Ok(font_data) = std::fs::read("C:\\Windows\\Fonts\\msyh.ttc") {
        fonts.font_data.insert(
            "microsoft_yahei".to_owned(),
            std::sync::Arc::new(egui::FontData::from_owned(font_data)),
        );
        
        // 将中文字体设为首选
        fonts.families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(0, "microsoft_yahei".to_owned());
        
        fonts.families
            .entry(egui::FontFamily::Monospace)
            .or_default()
            .insert(0, "microsoft_yahei".to_owned());
    }
    
    ctx.set_fonts(fonts);
    
    // 设置默认样式
    let mut style = (*ctx.style()).clone();
    style.text_styles.insert(
        egui::TextStyle::Body,
        egui::FontId::new(14.0, egui::FontFamily::Proportional),
    );
    style.text_styles.insert(
        egui::TextStyle::Button,
        egui::FontId::new(14.0, egui::FontFamily::Proportional),
    );
    style.text_styles.insert(
        egui::TextStyle::Heading,
        egui::FontId::new(18.0, egui::FontFamily::Proportional),
    );
    ctx.set_style(style);
}
