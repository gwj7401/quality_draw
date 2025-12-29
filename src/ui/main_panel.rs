//! 主抽签面板

use eframe::egui;
use crate::models::{Department, DepartmentType, QualitySpecialist, DrawRecord, SpecialtyType};
use crate::logic::DrawEngine;
use crate::storage::DataStore;
use super::animation::{AnimationState, AnimationPhase};

/// 抽签类型（综合类部门需要两种）
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DrawType {
    /// 只抽承压
    PressureOnly,
    /// 只抽机电
    MechanicalOnly,
    /// 同时抽取（综合类）
    Both,
}

/// 当前正在抽取的专责类型
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CurrentDrawing {
    /// 抽取承压类
    Pressure,
    /// 抽取机电类
    Mechanical,
}

/// 主面板
pub struct MainPanel {
    /// 选中的部门ID
    pub selected_department_id: Option<String>,
    /// 承压类动画状态
    pub pressure_animation: AnimationState,
    /// 机电类动画状态
    pub mechanical_animation: AnimationState,
    /// 当前抽签结果 - 承压
    pub pressure_result: Option<(String, String)>, // (姓名, 部门)
    /// 当前抽签结果 - 机电
    pub mechanical_result: Option<(String, String)>,
    /// 状态消息
    pub status_message: String,
    /// 是否正在抽签
    pub is_drawing: bool,
    /// 当前正在抽取的类型
    pub current_drawing: Option<CurrentDrawing>,
}

impl Default for MainPanel {
    fn default() -> Self {
        Self {
            selected_department_id: None,
            pressure_animation: AnimationState::default(),
            mechanical_animation: AnimationState::default(),
            pressure_result: None,
            mechanical_result: None,
            status_message: "请选择被检查部门，然后点击开始抽签".to_string(),
            is_drawing: false,
            current_drawing: None,
        }
    }
}

impl MainPanel {
    /// 获取当前选中部门应该抽取的类型
    pub fn get_draw_type(&self, departments: &[Department]) -> Option<DrawType> {
        let dept_id = self.selected_department_id.as_ref()?;
        let dept = departments.iter().find(|d| &d.id == dept_id)?;
        
        Some(match dept.department_type {
            DepartmentType::Pressure => DrawType::PressureOnly,
            DepartmentType::Mechanical => DrawType::MechanicalOnly,
            DepartmentType::Comprehensive => DrawType::Both,
        })
    }
    
    
    /// 开始抽签 - 已移动到下方 (show_controls附近)
    
    /// 停止抽签
    
    /// 停止抽签
    pub fn stop_draw(&mut self) {
        self.pressure_animation.request_stop();
        self.mechanical_animation.request_stop();
        self.status_message = "减速中...".to_string();
    }
    
    /// 更新动画并检查完成状态
    pub fn update(
        &mut self,
        specialists: &[QualitySpecialist],
        departments: &[Department],
        records: &[DrawRecord],
        store: &DataStore,
    ) -> Vec<DrawRecord> {
        let mut new_records = Vec::new();
        
        // 更新动画
        self.pressure_animation.update();
        self.mechanical_animation.update();
        
        // 检查承压动画是否完成
        if self.pressure_animation.phase == AnimationPhase::Stopped && self.pressure_result.is_none() {
            if let Some(name) = &self.pressure_animation.final_result {
                // 找到对应的专责
                if let Some(specialist) = specialists.iter().find(|s| &s.name == name) {
                    let dept_name = departments.iter()
                        .find(|d| d.id == specialist.department_id)
                        .map(|d| d.name.clone())
                        .unwrap_or_else(|| "未知".to_string());
                    
                    self.pressure_result = Some((name.clone(), dept_name.clone()));
                    
                    // 创建记录
                    if let Some(target_dept_id) = &self.selected_department_id {
                        if let Some(target_dept) = departments.iter().find(|d| &d.id == target_dept_id) {
                            let record = DrawRecord::new(
                                target_dept_id,
                                &target_dept.name,
                                SpecialtyType::Pressure,
                                &specialist.id,
                                &specialist.name,
                                &specialist.department_id,
                                dept_name,
                            );
                            store.add_record(record.clone());
                            new_records.push(record);
                        }
                    }
                }
            }
        }
        
        // 检查机电动画是否完成
        if self.mechanical_animation.phase == AnimationPhase::Stopped && self.mechanical_result.is_none() {
            if let Some(name) = &self.mechanical_animation.final_result {
                if let Some(specialist) = specialists.iter().find(|s| &s.name == name) {
                    let dept_name = departments.iter()
                        .find(|d| d.id == specialist.department_id)
                        .map(|d| d.name.clone())
                        .unwrap_or_else(|| "未知".to_string());
                    
                    self.mechanical_result = Some((name.clone(), dept_name.clone()));
                    
                    if let Some(target_dept_id) = &self.selected_department_id {
                        if let Some(target_dept) = departments.iter().find(|d| &d.id == target_dept_id) {
                            let record = DrawRecord::new(
                                target_dept_id,
                                &target_dept.name,
                                SpecialtyType::Mechanical,
                                &specialist.id,
                                &specialist.name,
                                &specialist.department_id,
                                dept_name,
                            );
                            store.add_record(record.clone());
                            new_records.push(record);
                        }
                    }
                }
            }
        }
        
        // 检查是否全部完成
        let pressure_done = !self.pressure_animation.is_running() || self.pressure_animation.phase == AnimationPhase::Idle;
        let mechanical_done = !self.mechanical_animation.is_running() || self.mechanical_animation.phase == AnimationPhase::Idle;
        
        if self.is_drawing && pressure_done && mechanical_done {
            self.is_drawing = false;
            self.status_message = "抽签完成！".to_string();
        }
        
        new_records
    }
    
    /// 显示部门选择器
    pub fn show_department_selector(&mut self, ui: &mut egui::Ui, departments: &[Department]) {
        ui.heading("选择被检查部门");
        ui.add_space(10.0);
        
        // 综合类部门
        ui.label(egui::RichText::new("━━ 综合类（承压+机电）━━").color(egui::Color32::from_rgb(100, 180, 100)));
        for dept in departments.iter().filter(|d| d.department_type == DepartmentType::Comprehensive) {
            let is_selected = self.selected_department_id.as_ref() == Some(&dept.id);
            if ui.selectable_label(is_selected, &dept.name).clicked() {
                self.selected_department_id = Some(dept.id.clone());
                self.pressure_result = None;
                self.mechanical_result = None;
            }
        }
        
        ui.add_space(10.0);
        
        // 承压类部门
        ui.label(egui::RichText::new("━━ 承压类 ━━").color(egui::Color32::from_rgb(200, 100, 100)));
        for dept in departments.iter().filter(|d| d.department_type == DepartmentType::Pressure) {
            let is_selected = self.selected_department_id.as_ref() == Some(&dept.id);
            if ui.selectable_label(is_selected, &dept.name).clicked() {
                self.selected_department_id = Some(dept.id.clone());
                self.pressure_result = None;
                self.mechanical_result = None;
            }
        }
        
        ui.add_space(10.0);
        
        // 机电类部门
        ui.label(egui::RichText::new("━━ 机电类 ━━").color(egui::Color32::from_rgb(100, 150, 200)));
        for dept in departments.iter().filter(|d| d.department_type == DepartmentType::Mechanical) {
            let is_selected = self.selected_department_id.as_ref() == Some(&dept.id);
            if ui.selectable_label(is_selected, &dept.name).clicked() {
                self.selected_department_id = Some(dept.id.clone());
                self.pressure_result = None;
                self.mechanical_result = None;
            }
        }
    }
    
    /// 显示抽签动画区域
    pub fn show_draw_area(&mut self, ui: &mut egui::Ui, departments: &[Department]) {
        let draw_type = self.get_draw_type(departments);
        
        ui.vertical_centered(|ui| {
            // 根据部门类型显示一个或两个滚动区域
            match draw_type {
                Some(DrawType::PressureOnly) => {
                    self.show_single_animation(ui, "承压类抽选", &self.pressure_animation.clone(), self.pressure_result.clone());
                }
                Some(DrawType::MechanicalOnly) => {
                    self.show_single_animation(ui, "机电类抽选", &self.mechanical_animation.clone(), self.mechanical_result.clone());
                }
                Some(DrawType::Both) => {
                    ui.push_id("dual_wheels", |ui| {
                        let _available_width = ui.available_width();
                        // 强制使用双列布局，确保两个都显示
                        ui.columns(2, |columns| {
                            columns[0].vertical_centered(|ui| {
                                self.show_single_animation(ui, "承压类抽选", &self.pressure_animation.clone(), self.pressure_result.clone());
                            });
                            columns[1].vertical_centered(|ui| {
                                self.show_single_animation(ui, "机电类抽选", &self.mechanical_animation.clone(), self.mechanical_result.clone());
                            });
                        });
                    });
                }
                None => {
                    ui.label("请选择被检查部门");
                }
            }
        });
    }
    
    /// 显示单个动画区域 - 大转盘效果
    fn show_single_animation(
        &self,
        ui: &mut egui::Ui,
        title: &str,
        animation: &AnimationState,
        result: Option<(String, String)>,
    ) {
        use std::f32::consts::PI;
        
        let is_running = animation.is_running();
        
        // 转盘参数 - 根据可用空间动态调整
        let available_width = ui.available_width();
        // 计算最大可用半径（留出边距）
        let max_radius = (available_width - 60.0) / 2.0;
        // 使用较小的值：最大140或可用空间允许的最大值
        let wheel_radius = max_radius.min(140.0).max(60.0); // 最小60，最大140
        let center_radius = wheel_radius * 0.25; // 按比例计算中心大小
        
        ui.vertical_centered(|ui| {
            // 标题
            let title_color = if is_running {
                egui::Color32::from_rgb(255, 215, 0)
            } else if result.is_some() {
                egui::Color32::from_rgb(50, 255, 100)
            } else {
                egui::Color32::from_rgb(150, 180, 220)
            };
            
            ui.label(egui::RichText::new(format!("❖ {} ❖", title))
                .size(20.0)
                .color(title_color)
                .strong());
            
            ui.add_space(8.0);
            
            let (response, painter) = ui.allocate_painter(
                egui::vec2(wheel_radius * 2.0 + 40.0, wheel_radius * 2.0 + 40.0),
                egui::Sense::hover(),
            );
            let center = response.rect.center();
            
            // 1. 绘制阴影（底座）
            painter.circle_filled(center + egui::vec2(8.0, 8.0), wheel_radius + 5.0, egui::Color32::from_black_alpha(60));
            
            // 2. 绘制金属外壳（多层同心圆模拟渐变）
            let outer_rim_width = 12.0;
            let full_radius = wheel_radius + outer_rim_width;
            
            // 模拟金属拉丝效果 - 深色底
            painter.circle_filled(center, full_radius, egui::Color32::from_rgb(40, 43, 48));
            // 金属光泽环
            painter.circle_stroke(center, full_radius - 2.0, egui::Stroke::new(2.0, egui::Color32::from_rgb(80, 85, 95)));
            painter.circle_stroke(center, full_radius - 5.0, egui::Stroke::new(4.0, egui::Color32::from_rgb(30, 32, 36)));
            painter.circle_stroke(center, full_radius - 8.0, egui::Stroke::new(1.0, egui::Color32::from_rgb(100, 105, 115)));
            
            // 3. 转盘背景
            painter.circle_filled(center, wheel_radius, egui::Color32::from_rgb(25, 25, 30));

            // 中奖状态显示
            if let Some((name, dept)) = &result {
                // 绘制静态的中奖结果盘面
                
                // 绘制选中扇形的高亮背景（占满全圆，但稍微暗一点）
                painter.circle_filled(center, wheel_radius, egui::Color32::from_rgb(30, 40, 30));
                
                // 绘制独特的发光环，表示锁定
                for i in 0..5 {
                    let alpha = (100 - i * 20) as u8;
                    painter.circle_stroke(center, wheel_radius - i as f32 * 2.0, 
                        egui::Stroke::new(2.0, egui::Color32::from_rgba_unmultiplied(50, 255, 100, alpha)));
                }

                // 中心发光区
                painter.circle_filled(center, wheel_radius * 0.7, egui::Color32::from_black_alpha(100));
                
                // 名字
                painter.text(
                    center,
                    egui::Align2::CENTER_CENTER,
                    name,
                    egui::FontId::proportional(40.0),
                    egui::Color32::from_rgb(255, 230, 100),
                );
                
                // 部门和小字
                painter.text(
                    center + egui::vec2(0.0, 45.0),
                    egui::Align2::CENTER_CENTER,
                    dept,
                    egui::FontId::proportional(14.0),
                    egui::Color32::from_rgb(180, 200, 180),
                );
                
                painter.text(
                    center + egui::vec2(0.0, -50.0),
                    egui::Align2::CENTER_CENTER,
                    "🎉 中签 🎉",
                    egui::FontId::proportional(16.0),
                    egui::Color32::from_rgb(100, 255, 100),
                );
                
                // 绘制简化的中心装饰
                painter.circle_stroke(center, center_radius + 50.0, egui::Stroke::new(1.0, egui::Color32::from_white_alpha(50)));

                return;
            }

            // 正常转盘显示
            let candidates = &animation.candidates;
            // 如果没候选人
            if candidates.is_empty() && !is_running {
                 painter.text(center, egui::Align2::CENTER_CENTER, "准备就绪", egui::FontId::proportional(20.0), egui::Color32::GRAY);
                 return;
            }

            // 为了让转盘视觉效果更好，当候选人少于6人时，复制填充
            let min_segments = 6;
            let display_candidates: Vec<&String> = if candidates.len() < min_segments && !candidates.is_empty() {
                // 复制候选人填充到至少 min_segments 个
                let mut expanded = Vec::new();
                while expanded.len() < min_segments {
                    for c in candidates {
                        expanded.push(c);
                        if expanded.len() >= min_segments {
                            break;
                        }
                    }
                }
                expanded
            } else {
                candidates.iter().collect()
            };

            let num_segments = display_candidates.len().max(1);
            let angle_per_segment = 2.0 * PI / num_segments as f32;
            // 确保 scroll_position 取模后在有效范围内，防止旋转角度计算溢出
            // 注意：scroll_position 是基于原始 candidates 长度的，需要按比例转换
            let original_len = candidates.len().max(1) as f32;
            let display_len = num_segments as f32;
            let scale_factor = display_len / original_len;
            let normalized_position = if num_segments > 0 {
                (animation.scroll_position * scale_factor) % num_segments as f32
            } else {
                0.0
            };
            let rotation_angle = normalized_position * angle_per_segment;
            
            // 高级配色方案 (Material Design 500/600 series)
            let colors = [
                egui::Color32::from_rgb(244, 67, 54),   // Red
                egui::Color32::from_rgb(255, 193, 7),   // Amber
                egui::Color32::from_rgb(76, 175, 80),   // Green
                egui::Color32::from_rgb(33, 150, 243),  // Blue
                egui::Color32::from_rgb(156, 39, 176),  // Purple
                egui::Color32::from_rgb(255, 87, 34),   // Deep Orange
                egui::Color32::from_rgb(0, 188, 212),   // Cyan
                egui::Color32::from_rgb(63, 81, 181),   // Indigo
            ];
            
            for i in 0..num_segments {
                let start_angle = i as f32 * angle_per_segment - rotation_angle - PI / 2.0;
                let end_angle = start_angle + angle_per_segment;
                let color = colors[i % colors.len()];
                
                // 4. 绘制扇形 (细分以平滑曲线)
                let segments = 12;
                let mut points = Vec::with_capacity(segments + 2);
                points.push(center);
                
                for j in 0..=segments {
                    let a = start_angle + (j as f32 / segments as f32) * angle_per_segment;
                    points.push(center + egui::vec2(a.cos() * wheel_radius, a.sin() * wheel_radius));
                }
                
                painter.add(egui::Shape::convex_polygon(points, color, egui::Stroke::NONE));
                
                // 5. 扇形高光/阴影效果 (让扇形看起来有立体折痕)
                // 在扇形的一侧叠加黑色透明，另一侧叠加白色透明
                let shadow_angle_end = start_angle + angle_per_segment * 0.2;
                let p_shadow_1 = center + egui::vec2(start_angle.cos() * wheel_radius, start_angle.sin() * wheel_radius);
                let p_shadow_2 = center + egui::vec2(shadow_angle_end.cos() * wheel_radius, shadow_angle_end.sin() * wheel_radius);
                painter.add(egui::Shape::convex_polygon(vec![center, p_shadow_1, p_shadow_2], egui::Color32::from_black_alpha(40), egui::Stroke::NONE));

                let highlight_angle_start = start_angle + angle_per_segment * 0.8;
                let p_highlight_1 = center + egui::vec2(highlight_angle_start.cos() * wheel_radius, highlight_angle_start.sin() * wheel_radius);
                let p_highlight_2 = center + egui::vec2(end_angle.cos() * wheel_radius, end_angle.sin() * wheel_radius);
                painter.add(egui::Shape::convex_polygon(vec![center, p_highlight_1, p_highlight_2], egui::Color32::from_white_alpha(40), egui::Stroke::NONE));
                
                // 6. 分隔线 (金色)
                let line_end = center + egui::vec2(end_angle.cos() * wheel_radius, end_angle.sin() * wheel_radius);
                painter.line_segment([center, line_end], egui::Stroke::new(1.5, egui::Color32::from_rgb(255, 223, 128))); // Gold line

                // 7. 文字
                if let Some(name) = display_candidates.get(i) {
                    let text_angle = start_angle + angle_per_segment / 2.0;
                    let text_dist = wheel_radius * 0.68;
                    let text_pos = center + egui::vec2(text_angle.cos() * text_dist, text_angle.sin() * text_dist);
                    
                    // 文字阴影
                    painter.text(
                        text_pos + egui::vec2(1.0, 1.0),
                        egui::Align2::CENTER_CENTER,
                        &name.chars().take(3).collect::<String>(),
                        egui::FontId::proportional(14.0),
                        egui::Color32::from_black_alpha(150),
                    );
                    
                    painter.text(
                        text_pos,
                        egui::Align2::CENTER_CENTER,
                        &name.chars().take(3).collect::<String>(),
                        egui::FontId::proportional(14.0),
                        egui::Color32::WHITE,
                    );
                }
            }

            // 8. 外围灯泡 (闪烁效果)
            let num_lights = 24;
            let time_ms = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis();
            let phase_shift = if is_running { (time_ms / 100) as usize } else { 0 };
            
            for i in 0..num_lights {
                let angle = (i as f32 / num_lights as f32) * 2.0 * PI - PI / 2.0;
                let bulb_dist = wheel_radius + outer_rim_width / 2.0;
                let pos = center + egui::vec2(angle.cos() * bulb_dist, angle.sin() * bulb_dist);
                
                let lit = if is_running { (i + phase_shift) % 2 == 0 } else { true };
                let color = if lit { egui::Color32::from_rgb(255, 235, 59) } else { egui::Color32::from_rgb(66, 66, 66) };
                
                painter.circle_filled(pos, 3.5, color);
                if lit {
                    painter.circle_stroke(pos, 4.0, egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(255, 235, 59, 100)));
                }
            }
            
            // 9. 中心装饰 (精密部件风格)
            // 外环
            painter.circle_filled(center, center_radius, egui::Color32::from_rgb(20, 20, 25));
            painter.circle_stroke(center, center_radius, egui::Stroke::new(3.0, egui::Color32::from_rgb(200, 180, 100))); // Gold ring
            
            // 内环（旋转）
            let inner_angle = if is_running { -(time_ms as f32 * 0.005) } else { 0.0 };
            let sub_radius = center_radius * 0.6;
            painter.circle_stroke(center, sub_radius, egui::Stroke::new(1.5, egui::Color32::from_rgb(100, 200, 255)));
            
            // 准星十字
            let cross_len = sub_radius - 2.0;
            let p1 = center + egui::vec2(inner_angle.cos() * cross_len, inner_angle.sin() * cross_len);
            let p2 = center - egui::vec2(inner_angle.cos() * cross_len, inner_angle.sin() * cross_len);
            let p3 = center + egui::vec2((inner_angle + PI/2.0).cos() * cross_len, (inner_angle + PI/2.0).sin() * cross_len);
            let p4 = center - egui::vec2((inner_angle + PI/2.0).cos() * cross_len, (inner_angle + PI/2.0).sin() * cross_len);
            
            painter.line_segment([p1, p2], egui::Stroke::new(1.0, egui::Color32::from_rgb(100, 200, 255)));
            painter.line_segment([p3, p4], egui::Stroke::new(1.0, egui::Color32::from_rgb(100, 200, 255)));
            
            // 中心点
            painter.circle_filled(center, 4.0, egui::Color32::RED);

            // 10. 指针 (顶部倒三角)
            let pointer_tip = center + egui::vec2(0.0, -full_radius + 2.0);
            let pointer_w = 16.0;
            let pointer_h = 24.0;
            painter.add(egui::Shape::convex_polygon(
                vec![
                    pointer_tip, 
                    pointer_tip + egui::vec2(-pointer_w/2.0, -pointer_h), 
                    pointer_tip + egui::vec2(pointer_w/2.0, -pointer_h)
                ],
                egui::Color32::from_rgb(255, 60, 60),
                egui::Stroke::new(2.0, egui::Color32::WHITE)
            ));

            ui.add_space(10.0);
            if is_running {
                 ui.label(egui::RichText::new("⚡ 正在选定...").size(14.0).color(egui::Color32::LIGHT_YELLOW));
            }
        });
    }
    
    /// 显示控制按钮
    pub fn show_controls(
        &mut self, 
        ui: &mut egui::Ui, 
        specialists: &[QualitySpecialist], 
        departments: &[Department],
        records: &[DrawRecord],
    ) {
        ui.horizontal(|ui| {
            let is_running = self.pressure_animation.is_running() || self.mechanical_animation.is_running();
            
            ui.add_enabled_ui(!is_running && self.selected_department_id.is_some(), |ui| {
                if ui.add_sized([120.0, 40.0], egui::Button::new(
                    egui::RichText::new("🎲 开始抽签").size(16.0)
                )).clicked() {
                    self.start_draw(specialists, departments, records);
                }
            });
            
            ui.add_space(20.0);
            
            ui.add_enabled_ui(is_running, |ui| {
                if ui.add_sized([120.0, 40.0], egui::Button::new(
                    egui::RichText::new("⏹ 停止").size(16.0)
                )).clicked() {
                    self.stop_draw();
                }
            });
        });
    }

    /// 开始抽签
    pub fn start_draw(
        &mut self,
        specialists: &[QualitySpecialist],
        departments: &[Department],
        records: &[DrawRecord],
    ) {
        let dept_id = match &self.selected_department_id {
            Some(id) => id.clone(),
            None => {
                self.status_message = "请先选择被检查部门".to_string();
                return;
            }
        };
        
        let draw_type = match self.get_draw_type(departments) {
            Some(t) => t,
            None => return,
        };
        
        // 重置结果和动画状态
        self.pressure_result = None;
        self.mechanical_result = None;
        self.pressure_animation = AnimationState::default();
        self.mechanical_animation = AnimationState::default();
        self.is_drawing = true;

        // 辅助函数：根据部门ID和专业类型查找上次中签者ID
        let get_last_selected_id = |target_dept_id: &str, specialty: SpecialtyType| -> Option<&str> {
            records.iter()
                .rev()
                .find(|r| r.target_department_id == target_dept_id && r.specialty_type == specialty)
                .map(|r| r.selected_specialist_id.as_str())
        };

        // 核心逻辑：获取候选人名单（处理单人强制排班的情况）
        let get_candidates_logic = |specialty: SpecialtyType, type_name: &str| -> (Vec<String>, String) {
            // 1. 获取该专业所有符合基本条件（部门回避）的人
            // 传入 None 作为 last_id 来获取全量合法名单
            let all_candidates = DrawEngine::get_rolling_names(specialists, &dept_id, specialty, None);
            
            if all_candidates.is_empty() {
                return (Vec::new(), format!("没有可抽取的{}人员！", type_name));
            }

            // 2. 如果只有1个人，强制选中，忽略连续回避
            if all_candidates.len() == 1 {
                return (all_candidates, format!("正在抽取{}人员 (唯一候选)...", type_name));
            }

            // 3. 如果有多人，执行连续回避
            let last_id = get_last_selected_id(&dept_id, specialty);
            let filtered_candidates = DrawEngine::get_rolling_names(specialists, &dept_id, specialty, last_id);
            
            if filtered_candidates.is_empty() {
                // 照理说不该发生（总量>1，排除1个后应该还有），除非数据异常，这里做兜底
                return (all_candidates, format!("正在抽取{}人员 (候选重置)...", type_name));
            }

            (filtered_candidates, format!("正在抽取{}人员...", type_name))
        };
        
        match draw_type {
            DrawType::PressureOnly => {
                let (names, msg) = get_candidates_logic(SpecialtyType::Pressure, "承压类");
                if names.is_empty() {
                    self.status_message = msg;
                    self.is_drawing = false;
                    return;
                }
                self.pressure_animation.start(names);
                self.current_drawing = Some(CurrentDrawing::Pressure);
                self.status_message = msg;
            }
            DrawType::MechanicalOnly => {
                let (names, msg) = get_candidates_logic(SpecialtyType::Mechanical, "机电类");
                if names.is_empty() {
                    self.status_message = msg;
                    self.is_drawing = false;
                    return;
                }
                self.mechanical_animation.start(names);
                self.current_drawing = Some(CurrentDrawing::Mechanical);
                self.status_message = msg;
            }
            DrawType::Both => {
                // 综合类：同时启动两个动画
                let (p_names, _p_msg) = get_candidates_logic(SpecialtyType::Pressure, "承压类");
                let (m_names, _m_msg) = get_candidates_logic(SpecialtyType::Mechanical, "机电类");
                
                if p_names.is_empty() && m_names.is_empty() {
                    self.status_message = "没有可抽取的人员！".to_string();
                    self.is_drawing = false;
                    return;
                }
                
                if !p_names.is_empty() {
                    self.pressure_animation.start(p_names.clone());
                }
                if !m_names.is_empty() {
                    self.mechanical_animation.start(m_names.clone());
                }
                self.current_drawing = None; // 表示同时抽取
                
                // 组合消息
                let status = if (p_names.len() == 1 && !p_names.is_empty()) || (m_names.len() == 1 && !m_names.is_empty()) {
                    "正在抽取... (包含唯一候选岗位)".to_string()
                } else {
                    "正在抽取...".to_string()
                };
                self.status_message = status;
            }
        }
    }
    
    /// 显示抽签结果
    pub fn show_results(&self, ui: &mut egui::Ui, departments: &[Department]) {
        if self.pressure_result.is_none() && self.mechanical_result.is_none() {
            return;
        }
        
        let dept_name = self.selected_department_id.as_ref()
            .and_then(|id| departments.iter().find(|d| &d.id == id))
            .map(|d| d.name.as_str())
            .unwrap_or("未知");
        
        let draw_type = self.get_draw_type(departments);
        
        ui.group(|ui| {
            ui.heading(format!("📋 {} 抽签结果", dept_name));
            ui.separator();
            
            // 根据 DrawType 过滤显示
            let show_pressure = matches!(draw_type, Some(DrawType::PressureOnly) | Some(DrawType::Both));
            let show_mechanical = matches!(draw_type, Some(DrawType::MechanicalOnly) | Some(DrawType::Both));
            
            if show_pressure {
                if let Some((name, from_dept)) = &self.pressure_result {
                    ui.horizontal(|ui| {
                        ui.label("承压类专责：");
                        ui.label(egui::RichText::new(name)
                            .color(egui::Color32::from_rgb(50, 150, 250))
                            .strong()
                            .size(16.0));
                        ui.label(format!("（{}）", from_dept));
                    });
                }
            }
            
            if show_mechanical {
                if let Some((name, from_dept)) = &self.mechanical_result {
                    ui.horizontal(|ui| {
                        ui.label("机电类专责：");
                        ui.label(egui::RichText::new(name)
                            .color(egui::Color32::from_rgb(50, 200, 100))
                            .strong()
                            .size(16.0));
                        ui.label(format!("（{}）", from_dept));
                    });
                }
            }
        });
    }
}
