//! 主应用程序

use eframe::egui;
use crate::models::{Department, QualitySpecialist, DrawRecord};
use crate::storage::DataStore;
use crate::ui::{MainPanel, SettingsPanel, HistoryPanel, ExportManager};

/// 应用程序状态
pub struct QualityDrawApp {
    /// 数据存储
    store: DataStore,
    /// 部门列表
    departments: Vec<Department>,
    /// 质量专责列表
    specialists: Vec<QualitySpecialist>,
    /// 抽签记录
    records: Vec<DrawRecord>,
    /// 主面板
    main_panel: MainPanel,
    /// 设置面板
    settings_panel: SettingsPanel,
    /// 历史记录面板
    history_panel: HistoryPanel,
    /// 状态消息
    status_message: Option<String>,
}

impl QualityDrawApp {
    /// 创建新应用
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let store = DataStore::new();
        let departments = store.load_departments();
        let specialists = store.load_specialists();
        let records = store.load_records();
        
        Self {
            store,
            departments,
            specialists,
            records,
            main_panel: MainPanel::default(),
            settings_panel: SettingsPanel::default(),
            history_panel: HistoryPanel::default(),
            status_message: None,
        }
    }
    
    /// 导出到Excel
    fn export_to_excel(&mut self) {
        if self.records.is_empty() {
            self.status_message = Some("没有可导出的记录".to_string());
            return;
        }
        
        // 生成文件名
        let filename = format!(
            "抽签结果_{}.xlsx",
            chrono::Local::now().format("%Y%m%d_%H%M%S")
        );
        
        // 保存到桌面
        let desktop = dirs::desktop_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
        let path = desktop.join(&filename);
        
        match ExportManager::export_to_excel(&self.records, &path) {
            Ok(_) => {
                self.status_message = Some(format!("已导出到: {}", path.display()));
                // 打开文件位置
                #[cfg(target_os = "windows")]
                {
                    let _ = std::process::Command::new("explorer")
                        .args(["/select,", path.to_str().unwrap_or("")])
                        .spawn();
                }
            }
            Err(e) => {
                self.status_message = Some(format!("导出失败: {}", e));
            }
        }
    }
    
    /// 打印记录
    fn print_records(&mut self) {
        if self.records.is_empty() {
            self.status_message = Some("没有可打印的记录".to_string());
            return;
        }
        
        match ExportManager::print_records(&self.records) {
            Ok(_) => {
                self.status_message = Some("已在浏览器中打开打印预览".to_string());
            }
            Err(e) => {
                self.status_message = Some(format!("打印失败: {}", e));
            }
        }
    }
}

impl eframe::App for QualityDrawApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 请求持续重绘（用于动画）
        if self.main_panel.pressure_animation.is_running() || 
           self.main_panel.mechanical_animation.is_running() {
            ctx.request_repaint();
        }
        
        // 更新动画状态
        let new_records = self.main_panel.update(
            &self.specialists,
            &self.departments,
            &self.records,
            &self.store,
        );
        self.records.extend(new_records);
        
        // 顶部标题栏
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.add_space(5.0);
            ui.horizontal(|ui| {
                ui.heading(egui::RichText::new("宁夏特检院质量监督检查抽签程序")
                    .size(24.0)
                    .color(egui::Color32::from_rgb(50, 100, 180)));
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(format!("专责: {} 人", self.specialists.len()));
                });
            });
            ui.add_space(5.0);
        });
        
        // 底部工具栏
        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.add_space(5.0);
            ui.horizontal(|ui| {
                if ui.button("📜 查看历史").clicked() {
                    self.history_panel.visible = true;
                }
                
                if ui.button("⚙ 数据管理").clicked() {
                    self.settings_panel.visible = true;
                }
                
                ui.separator();
                
                if ui.button("📊 导出Excel").clicked() {
                    self.export_to_excel();
                }
                
                if ui.button("🖨 打印").clicked() {
                    self.print_records();
                }
                
                ui.separator();
                
                // 显示本轮已抽中数量
                let round_count = self.main_panel.current_round_pressure_depts.len() 
                    + self.main_panel.current_round_mechanical_depts.len();
                if round_count > 0 {
                    ui.label(format!("本轮已抽: {}", round_count));
                }
                
                if ui.button("🔄 开始新一轮").clicked() {
                    self.main_panel.current_round_pressure_depts.clear();
                    self.main_panel.current_round_mechanical_depts.clear();
                    self.main_panel.pressure_result = None;
                    self.main_panel.mechanical_result = None;
                    self.status_message = Some("已开始新一轮抽签".to_string());
                }
                
                // 状态消息
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if let Some(msg) = &self.status_message {
                        ui.label(egui::RichText::new(msg).color(egui::Color32::from_rgb(100, 150, 200)));
                    } else {
                        ui.label(&self.main_panel.status_message);
                    }
                });
            });
            ui.add_space(5.0);
        });
        
        // 左侧部门选择面板
        egui::SidePanel::left("department_panel")
            .resizable(true)
            .default_width(180.0)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    self.main_panel.show_department_selector(ui, &self.departments);
                });
            });
        
        // 中央区域
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(30.0);
                
                // 抽签动画区域
                self.main_panel.show_draw_area(ui, &self.departments);
                
                ui.add_space(30.0);
                
                // 控制按钮
                self.main_panel.show_controls(ui, &self.specialists, &self.departments, &self.records);
                
                ui.add_space(30.0);
                
                // 结果显示
                self.main_panel.show_results(ui, &self.departments);
            });
        });
        
        // 弹窗
        self.settings_panel.show(ctx, &mut self.specialists, &mut self.departments, &self.store);
        self.history_panel.show(ctx, &mut self.records, &self.store);
        
        // 清除状态消息（5秒后）
        // 注意：简化实现，实际可以使用计时器
    }
}
