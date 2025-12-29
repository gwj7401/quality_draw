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

fn main() -> eframe::Result<()> {
    // 设置中文字体
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1024.0, 700.0])
            .with_min_inner_size([800.0, 600.0])
            .with_title("宁夏特检院质量监督检查抽签程序"),
        ..Default::default()
    };
    
    eframe::run_native(
        "质量监督检查抽签程序",
        options,
        Box::new(|cc| {
            // 配置中文字体
            setup_fonts(&cc.egui_ctx);
            Ok(Box::new(QualityDrawApp::new(cc)))
        }),
    )
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
