//! 设置管理界面（专责管理、部门管理）

use eframe::egui;
use crate::models::{Department, DepartmentType, QualitySpecialist, SpecialtyType};
use crate::storage::DataStore;

/// 设置面板
pub struct SettingsPanel {
    /// 是否显示
    pub visible: bool,
    /// 当前标签页
    pub current_tab: SettingsTab,
    /// 新增专责表单
    pub new_specialist_name: String,
    pub new_specialist_dept: String,
    pub new_specialist_type: SpecialtyType,
    /// 新增部门表单
    pub new_dept_name: String,
    pub new_dept_type: DepartmentType,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SettingsTab {
    Specialists,
    Departments,
}

impl Default for SettingsPanel {
    fn default() -> Self {
        Self {
            visible: false,
            current_tab: SettingsTab::Specialists,
            new_specialist_name: String::new(),
            new_specialist_dept: String::new(),
            new_specialist_type: SpecialtyType::Pressure,
            new_dept_name: String::new(),
            new_dept_type: DepartmentType::Comprehensive,
        }
    }
}

impl SettingsPanel {
    /// 显示设置面板
    pub fn show(
        &mut self,
        ctx: &egui::Context,
        specialists: &mut Vec<QualitySpecialist>,
        departments: &mut Vec<Department>,
        store: &DataStore,
    ) {
        if !self.visible {
            return;
        }
        
        let mut open = self.visible;
        
        egui::Window::new("⚙ 数据管理")
            .open(&mut open)
            .default_width(600.0)
            .default_height(500.0)
            .resizable(true)
            .show(ctx, |ui| {
                // 标签页选择
                ui.horizontal(|ui| {
                    if ui.selectable_label(self.current_tab == SettingsTab::Specialists, "👤 质量专责管理").clicked() {
                        self.current_tab = SettingsTab::Specialists;
                    }
                    if ui.selectable_label(self.current_tab == SettingsTab::Departments, "🏢 部门管理").clicked() {
                        self.current_tab = SettingsTab::Departments;
                    }
                });
                
                ui.separator();
                
                match self.current_tab {
                    SettingsTab::Specialists => {
                        Self::show_specialists_ui(
                            ui,
                            &mut self.new_specialist_name,
                            &mut self.new_specialist_dept,
                            &mut self.new_specialist_type,
                            specialists,
                            departments,
                            store,
                        );
                    }
                    SettingsTab::Departments => {
                        Self::show_departments_ui(
                            ui,
                            &mut self.new_dept_name,
                            &mut self.new_dept_type,
                            departments,
                            store,
                        );
                    }
                }
            });
        
        self.visible = open;
    }
    
    /// 显示专责管理UI（静态方法避免借用冲突）
    fn show_specialists_ui(
        ui: &mut egui::Ui,
        new_name: &mut String,
        new_dept: &mut String,
        new_type: &mut SpecialtyType,
        specialists: &mut Vec<QualitySpecialist>,
        departments: &[Department],
        store: &DataStore,
    ) {
        // 新增表单
        ui.group(|ui| {
            ui.heading("添加新专责");
            ui.horizontal(|ui| {
                ui.label("姓名:");
                ui.text_edit_singleline(new_name);
                
                ui.label("部门:");
                egui::ComboBox::from_id_salt("new_specialist_dept")
                    .selected_text(
                        departments.iter()
                            .find(|d| &d.id == new_dept)
                            .map(|d| d.name.as_str())
                            .unwrap_or("请选择")
                    )
                    .show_ui(ui, |ui| {
                        for dept in departments {
                            ui.selectable_value(
                                new_dept,
                                dept.id.clone(),
                                &dept.name,
                            );
                        }
                    });
                
                ui.label("专业:");
                egui::ComboBox::from_id_salt("new_specialist_type")
                    .selected_text(new_type.display_name())
                    .show_ui(ui, |ui| {
                        ui.selectable_value(new_type, SpecialtyType::Pressure, "承压类");
                        ui.selectable_value(new_type, SpecialtyType::Mechanical, "机电类");
                    });
                
                if ui.button("➕ 添加").clicked() {
                    if !new_name.trim().is_empty() && !new_dept.is_empty() {
                        let new_id = uuid::Uuid::new_v4().to_string();
                        specialists.push(QualitySpecialist::new(
                            new_id,
                            new_name.trim(),
                            new_dept.as_str(),
                            *new_type,
                        ));
                        store.save_specialists(specialists);
                        new_name.clear();
                    }
                }
            });
        });
        
        ui.separator();
        
        // 专责列表
        ui.heading(format!("专责列表 (共{}人)", specialists.len()));
        
        // 创建按部门排序的索引列表
        let mut sorted_indices: Vec<usize> = (0..specialists.len()).collect();
        sorted_indices.sort_by(|&a, &b| {
            let dept_a = departments.iter()
                .find(|d| d.id == specialists[a].department_id)
                .map(|d| d.name.as_str())
                .unwrap_or("未知");
            let dept_b = departments.iter()
                .find(|d| d.id == specialists[b].department_id)
                .map(|d| d.name.as_str())
                .unwrap_or("未知");
            dept_a.cmp(dept_b)
        });
        
        egui::ScrollArea::vertical()
            .max_height(350.0)
            .show(ui, |ui| {
                let mut to_delete = None;
                
                for &idx in &sorted_indices {
                    let specialist = &specialists[idx];
                    let dept_name = departments.iter()
                        .find(|d| d.id == specialist.department_id)
                        .map(|d| d.name.as_str())
                        .unwrap_or("未知");
                    
                    ui.horizontal(|ui| {
                        ui.label(format!(
                            "{}  |  {}  |  {}",
                            specialist.name,
                            dept_name,
                            specialist.specialty.display_name()
                        ));
                        
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("🗑 删除").clicked() {
                                to_delete = Some(idx);
                            }
                        });
                    });
                    ui.separator();
                }
                
                if let Some(idx) = to_delete {
                    specialists.remove(idx);
                    store.save_specialists(specialists);
                }
            });
    }
    
    /// 显示部门管理UI（静态方法避免借用冲突）
    fn show_departments_ui(
        ui: &mut egui::Ui,
        new_name: &mut String,
        new_type: &mut DepartmentType,
        departments: &mut Vec<Department>,
        store: &DataStore,
    ) {
        // 新增表单
        ui.group(|ui| {
            ui.heading("添加新部门");
            ui.horizontal(|ui| {
                ui.label("名称:");
                ui.text_edit_singleline(new_name);
                
                ui.label("类型:");
                egui::ComboBox::from_id_salt("new_dept_type")
                    .selected_text(new_type.display_name())
                    .show_ui(ui, |ui| {
                        ui.selectable_value(new_type, DepartmentType::Comprehensive, "综合类");
                        ui.selectable_value(new_type, DepartmentType::Pressure, "承压类");
                        ui.selectable_value(new_type, DepartmentType::Mechanical, "机电类");
                    });
                
                if ui.button("➕ 添加").clicked() {
                    if !new_name.trim().is_empty() {
                        let new_id = uuid::Uuid::new_v4().to_string();
                        departments.push(Department::new(
                            new_id,
                            new_name.trim(),
                            *new_type,
                        ));
                        store.save_departments(departments);
                        new_name.clear();
                    }
                }
            });
        });
        
        ui.separator();
        
        // 部门列表
        ui.heading(format!("部门列表 (共{}个)", departments.len()));
        
        // 创建按类型和名称排序的索引列表
        let mut sorted_indices: Vec<usize> = (0..departments.len()).collect();
        sorted_indices.sort_by(|&a, &b| {
            let type_order = |t: &crate::models::DepartmentType| match t {
                crate::models::DepartmentType::Comprehensive => 0,
                crate::models::DepartmentType::Pressure => 1,
                crate::models::DepartmentType::Mechanical => 2,
            };
            let type_cmp = type_order(&departments[a].department_type)
                .cmp(&type_order(&departments[b].department_type));
            if type_cmp == std::cmp::Ordering::Equal {
                departments[a].name.cmp(&departments[b].name)
            } else {
                type_cmp
            }
        });
        
        egui::ScrollArea::vertical()
            .max_height(350.0)
            .show(ui, |ui| {
                let mut to_delete = None;
                
                for &idx in &sorted_indices {
                    let dept = &departments[idx];
                    ui.horizontal(|ui| {
                        ui.label(format!(
                            "{}  |  {}",
                            dept.name,
                            dept.department_type.display_name()
                        ));
                        
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("🗑 删除").clicked() {
                                to_delete = Some(idx);
                            }
                        });
                    });
                    ui.separator();
                }
                
                if let Some(idx) = to_delete {
                    departments.remove(idx);
                    store.save_departments(departments);
                }
            });
    }
}
