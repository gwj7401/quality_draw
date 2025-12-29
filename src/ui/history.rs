//! 历史记录面板

use eframe::egui;
use crate::models::DrawRecord;
use crate::storage::DataStore;

/// 历史记录面板
pub struct HistoryPanel {
    /// 是否显示
    pub visible: bool,
}

impl Default for HistoryPanel {
    fn default() -> Self {
        Self { visible: false }
    }
}

impl HistoryPanel {
    /// 显示历史记录面板
    pub fn show(
        &mut self,
        ctx: &egui::Context,
        records: &mut Vec<DrawRecord>,
        store: &DataStore,
    ) {
        if !self.visible {
            return;
        }
        
        egui::Window::new("📜 抽签历史记录")
            .open(&mut self.visible)
            .default_width(700.0)
            .default_height(500.0)
            .resizable(true)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.heading(format!("共 {} 条记录", records.len()));
                    
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("🗑 清空记录").clicked() {
                            records.clear();
                            store.clear_records();
                        }
                    });
                });
                
                ui.separator();
                
                if records.is_empty() {
                    ui.centered_and_justified(|ui| {
                        ui.label("暂无抽签记录");
                    });
                    return;
                }
                
                egui::ScrollArea::vertical()
                    .max_height(420.0)
                    .show(ui, |ui| {
                        egui::Grid::new("history_grid")
                            .num_columns(5)
                            .spacing([20.0, 8.0])
                            .striped(true)
                            .min_col_width(60.0)
                            .show(ui, |ui| {
                                // 表头
                                ui.label(egui::RichText::new("时间").strong().size(14.0));
                                ui.label(egui::RichText::new("被检部门").strong().size(14.0));
                                ui.label(egui::RichText::new("专责类型").strong().size(14.0));
                                ui.label(egui::RichText::new("抽中人员").strong().size(14.0));
                                ui.label(egui::RichText::new("所属部门").strong().size(14.0));
                                ui.end_row();
                                
                                // 数据行 - 按时间倒序显示
                                for record in records.iter().rev() {
                                    ui.label(record.timestamp.format("%m-%d %H:%M").to_string());
                                    ui.label(&record.target_department_name);
                                    ui.label(record.specialty_type.display_name());
                                    ui.label(egui::RichText::new(&record.selected_specialist_name)
                                        .color(egui::Color32::from_rgb(0, 150, 255))
                                        .strong());
                                    ui.label(&record.selected_from_department_name);
                                    ui.end_row();
                                }
                            });
                    });
            });
    }
}
