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
    /// 错误提示消息
    pub error_message: Option<String>,
    
    // --- 搜索和筛选状态 ---
    /// 搜索文本（姓名）
    pub search_text: String,
    /// 部门筛选
    pub filter_dept: Option<String>,
    /// 专业筛选
    pub filter_specialty: Option<SpecialtyType>,
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
            error_message: None,
            search_text: String::new(),
            filter_dept: None,
            filter_specialty: None,
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
                            &mut self.error_message,
                            &mut self.search_text,
                            &mut self.filter_dept,
                            &mut self.filter_specialty,
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
        error_message: &mut Option<String>,
        search_text: &mut String,
        filter_dept: &mut Option<String>,
        filter_specialty: &mut Option<SpecialtyType>,
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
                        // 检查是否存在重复：同一部门、同一姓名、同一专业
                        let name_trimmed = new_name.trim();
                        let is_duplicate = specialists.iter().any(|s| {
                            s.name == name_trimmed 
                                && s.department_id == *new_dept 
                                && s.specialty == *new_type
                        });
                        
                        if is_duplicate {
                            // 设置错误消息
                            *error_message = Some(format!(
                                "⚠ 重复添加：{} 在该部门的{}专业已存在！",
                                name_trimmed,
                                new_type.display_name()
                            ));
                        } else {
                            let new_id = uuid::Uuid::new_v4().to_string();
                            specialists.push(QualitySpecialist::new(
                                new_id,
                                name_trimmed,
                                new_dept.as_str(),
                                *new_type,
                            ));
                            store.save_specialists(specialists);
                            new_name.clear();
                            // 清除错误消息
                            *error_message = None;
                        }
                    }
                }
            });
            
            // 显示错误消息
            let mut should_clear_error = false;
            if let Some(msg) = error_message.as_ref() {
                ui.add_space(5.0);
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(msg.as_str()).color(egui::Color32::RED).strong());
                    if ui.small_button("✖").clicked() {
                        should_clear_error = true;
                    }
                });
            }
            if should_clear_error {
                *error_message = None;
            }
        });
        
        ui.separator();
        
        // 筛选工具栏
        ui.horizontal(|ui| {
            ui.label("🔍 搜索:");
            ui.text_edit_singleline(search_text);
            
            ui.label("筛选:");
            
            // 部门筛选
            egui::ComboBox::from_id_salt("filter_dept")
                .selected_text(
                    filter_dept.as_ref()
                        .and_then(|id| departments.iter().find(|d| &d.id == id))
                        .map(|d| d.name.as_str())
                        .unwrap_or("所有部门")
                )
                .show_ui(ui, |ui| {
                    ui.selectable_value(filter_dept, None, "所有部门");
                    for dept in departments {
                        ui.selectable_value(
                            filter_dept,
                            Some(dept.id.clone()),
                            &dept.name,
                        );
                    }
                });
                
            // 专业筛选
            egui::ComboBox::from_id_salt("filter_specialty")
                .selected_text(
                    filter_specialty.as_ref()
                        .map(|s| s.display_name())
                        .unwrap_or("所有专业")
                )
                .show_ui(ui, |ui| {
                    ui.selectable_value(filter_specialty, None, "所有专业");
                    ui.selectable_value(filter_specialty, Some(SpecialtyType::Pressure), "承压类");
                    ui.selectable_value(filter_specialty, Some(SpecialtyType::Mechanical), "机电类");
                });
                
            if ui.button("❌ 重置").clicked() {
                search_text.clear();
                *filter_dept = None;
                *filter_specialty = None;
            }
        });
        
        ui.separator();
        
        // 专责列表
        
        // 创建按部门排序并在筛选后的索引列表
        let mut sorted_indices: Vec<usize> = specialists.iter()
            .enumerate()
            .filter(|(_, s)| {
                // 姓名筛选
                if !search_text.is_empty() && !s.name.contains(search_text.as_str()) {
                    return false;
                }
                // 部门筛选
                if let Some(dept_id) = filter_dept {
                    if &s.department_id != dept_id {
                        return false;
                    }
                }
                // 专业筛选
                if let Some(specialty) = filter_specialty {
                    if &s.specialty != specialty {
                        return false;
                    }
                }
                true
            })
            .map(|(i, _)| i)
            .collect();
            
        ui.heading(format!("专责列表 (显示 {} / 共 {} 人)", sorted_indices.len(), specialists.len()));

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
