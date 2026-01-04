//! Tauri 命令模块 - 按部门抽签版本

use tauri::State;
use std::sync::Mutex;
use rand::seq::SliceRandom;
use crate::models::{Department, DrawRecord, SpecialtyType, DepartmentType, default_departments};
use crate::storage::DataStore;

/// 应用状态
pub struct AppState {
    pub store: Mutex<DataStore>,
    /// 本轮已抽中的承压部门列表 (被检部门ID, 抽中部门ID)
    pub current_round_pressure_depts: Mutex<Vec<(String, String)>>,
    /// 本轮已抽中的机电部门列表 (被检部门ID, 抽中部门ID)
    pub current_round_mechanical_depts: Mutex<Vec<(String, String)>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            store: Mutex::new(DataStore::new()),
            current_round_pressure_depts: Mutex::new(Vec::new()),
            current_round_mechanical_depts: Mutex::new(Vec::new()),
        }
    }
}

/// 获取所有部门
#[tauri::command]
pub fn get_departments(state: State<AppState>) -> Vec<Department> {
    let store = state.store.lock().unwrap();
    store.load_departments()
}

/// 获取抽签记录
#[tauri::command]
pub fn get_records(state: State<AppState>) -> Vec<DrawRecord> {
    let store = state.store.lock().unwrap();
    store.load_records()
}

/// 清空抽签记录
#[tauri::command]
pub fn clear_records(state: State<AppState>) {
    let store = state.store.lock().unwrap();
    store.clear_records();
}

/// 开始新一轮抽签（清空本轮已抽中列表）
#[tauri::command]
pub fn start_new_round(state: State<AppState>) {
    let mut pressure_depts = state.current_round_pressure_depts.lock().unwrap();
    let mut mechanical_depts = state.current_round_mechanical_depts.lock().unwrap();
    pressure_depts.clear();
    mechanical_depts.clear();
}

/// 获取本轮已抽中的记录
#[tauri::command]
pub fn get_current_round_status(state: State<AppState>) -> (Vec<(String, String)>, Vec<(String, String)>) {
    let pressure_depts = state.current_round_pressure_depts.lock().unwrap();
    let mechanical_depts = state.current_round_mechanical_depts.lock().unwrap();
    (pressure_depts.clone(), mechanical_depts.clone())
}

/// 获取可抽取的部门列表（用于动画滚动）
#[tauri::command]
pub fn get_candidate_departments(
    target_department_id: String,
    specialty_type: String,
    state: State<AppState>,
) -> Vec<String> {
    let store = state.store.lock().unwrap();
    let departments = store.load_departments();
    
    let current_round = if specialty_type == "Pressure" {
        state.current_round_pressure_depts.lock().unwrap()
    } else {
        state.current_round_mechanical_depts.lock().unwrap()
    };
    
    // 本轮已被抽中作为检查员的部门ID
    let already_selected: Vec<&String> = current_round.iter().map(|(_, selected)| selected).collect();
    
    // 交叉回避：如果当前被检部门曾被派去检查其他部门，那些部门不能来检查这个部门
    let cross_avoidance: Vec<&String> = current_round.iter()
        .filter(|(_, selected)| selected == &target_department_id)
        .map(|(target, _)| target)
        .collect();
    
    departments.iter()
        .filter(|d| {
            // 根据专业类型筛选
            let type_match = if specialty_type == "Pressure" {
                d.department_type == DepartmentType::Comprehensive || 
                d.department_type == DepartmentType::Pressure
            } else {
                d.department_type == DepartmentType::Comprehensive || 
                d.department_type == DepartmentType::Mechanical
            };
            
            type_match
            // 排除被检查的部门
            && d.id != target_department_id
            // 排除本轮已被抽中的部门
            && !already_selected.contains(&&d.id)
            // 排除交叉回避的部门
            && !cross_avoidance.contains(&&d.id)
        })
        .map(|d| d.name.clone())
        .collect()
}

/// 执行抽签的结果
#[derive(serde::Serialize)]
pub struct DrawResult {
    pub success: bool,
    pub department_name: Option<String>,
    pub department_id: Option<String>,
    pub specialty_type: Option<String>,
    pub message: Option<String>,
}

/// 执行抽签（抽取部门）
#[tauri::command]
pub fn execute_draw(
    target_department_id: String,
    specialty_type: String,
    state: State<AppState>,
) -> DrawResult {
    let store = state.store.lock().unwrap();
    let departments = store.load_departments();
    
    // 获取当前轮次已抽中列表
    let (mut current_round, dept_type_filter) = if specialty_type == "Pressure" {
        let round = state.current_round_pressure_depts.lock().unwrap().clone();
        (round, vec![DepartmentType::Comprehensive, DepartmentType::Pressure])
    } else if specialty_type == "Mechanical" {
        let round = state.current_round_mechanical_depts.lock().unwrap().clone();
        (round, vec![DepartmentType::Comprehensive, DepartmentType::Mechanical])
    } else {
        return DrawResult {
            success: false,
            department_name: None,
            department_id: None,
            specialty_type: None,
            message: Some("无效的专责类型".to_string()),
        };
    };
    
    // 找到目标部门
    let target_department = match departments.iter().find(|d| d.id == target_department_id) {
        Some(d) => d,
        None => return DrawResult {
            success: false,
            department_name: None,
            department_id: None,
            specialty_type: None,
            message: Some("未找到目标部门".to_string()),
        },
    };
    
    // 本轮已被抽中作为检查员的部门ID
    let already_selected: Vec<&String> = current_round.iter().map(|(_, selected)| selected).collect();
    
    // 交叉回避
    let cross_avoidance: Vec<&String> = current_round.iter()
        .filter(|(_, selected)| selected == &target_department_id)
        .map(|(target, _)| target)
        .collect();
    
    // 获取候选部门
    let candidates: Vec<&Department> = departments.iter()
        .filter(|d| {
            dept_type_filter.contains(&d.department_type)
            && d.id != target_department_id
            && !already_selected.contains(&&d.id)
            && !cross_avoidance.contains(&&d.id)
        })
        .collect();
    
    if candidates.is_empty() {
        return DrawResult {
            success: false,
            department_name: None,
            department_id: None,
            specialty_type: None,
            message: Some("没有符合条件的候选部门".to_string()),
        };
    }
    
    // 随机抽取
    let mut rng = rand::thread_rng();
    let selected = candidates.choose(&mut rng).unwrap();
    
    // 保存到本轮列表
    if specialty_type == "Pressure" {
        let mut pressure_depts = state.current_round_pressure_depts.lock().unwrap();
        pressure_depts.push((target_department_id.clone(), selected.id.clone()));
    } else {
        let mut mechanical_depts = state.current_round_mechanical_depts.lock().unwrap();
        mechanical_depts.push((target_department_id.clone(), selected.id.clone()));
    }
    
    // 保存历史记录
    let specialty = if specialty_type == "Pressure" {
        SpecialtyType::Pressure
    } else {
        SpecialtyType::Mechanical
    };
    
    let record = DrawRecord::new(
        &target_department.id,
        &target_department.name,
        specialty,
        &selected.id,        // 用部门ID代替人员ID
        &selected.name,      // 用部门名称代替人员名称
        &selected.id,
        &selected.name,
    );
    store.add_record(record);
    
    DrawResult {
        success: true,
        department_name: Some(selected.name.clone()),
        department_id: Some(selected.id.clone()),
        specialty_type: Some(specialty_type),
        message: None,
    }
}

/// 导出记录到 Excel
#[tauri::command]
pub fn export_to_excel(state: State<AppState>) -> Result<String, String> {
    use rust_xlsxwriter::*;
    
    let store = state.store.lock().unwrap();
    let records = store.load_records();
    
    if records.is_empty() {
        return Err("没有可导出的记录".to_string());
    }
    
    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();
    
    let header_format = Format::new()
        .set_bold()
        .set_align(FormatAlign::Center);
    
    worksheet.write_with_format(0, 0, "序号", &header_format).map_err(|e| e.to_string())?;
    worksheet.write_with_format(0, 1, "抽签时间", &header_format).map_err(|e| e.to_string())?;
    worksheet.write_with_format(0, 2, "被检部门", &header_format).map_err(|e| e.to_string())?;
    worksheet.write_with_format(0, 3, "专责类型", &header_format).map_err(|e| e.to_string())?;
    worksheet.write_with_format(0, 4, "抽中部门", &header_format).map_err(|e| e.to_string())?;
    
    for (i, record) in records.iter().enumerate() {
        let row = (i + 1) as u32;
        worksheet.write(row, 0, (i + 1) as i32).map_err(|e| e.to_string())?;
        worksheet.write(row, 1, record.timestamp.format("%Y-%m-%d %H:%M:%S").to_string()).map_err(|e| e.to_string())?;
        worksheet.write(row, 2, &record.target_department_name).map_err(|e| e.to_string())?;
        worksheet.write(row, 3, record.specialty_type.display_name()).map_err(|e| e.to_string())?;
        worksheet.write(row, 4, &record.selected_specialist_name).map_err(|e| e.to_string())?; // 实际是部门名称
    }
    
    worksheet.set_column_width(0, 8).map_err(|e| e.to_string())?;
    worksheet.set_column_width(1, 20).map_err(|e| e.to_string())?;
    worksheet.set_column_width(2, 20).map_err(|e| e.to_string())?;
    worksheet.set_column_width(3, 12).map_err(|e| e.to_string())?;
    worksheet.set_column_width(4, 20).map_err(|e| e.to_string())?;
    
    let filename = format!("抽签记录_{}.xlsx", chrono::Local::now().format("%Y%m%d_%H%M%S"));
    let desktop_path = dirs::desktop_dir()
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());
    let file_path = desktop_path.join(&filename);
    
    workbook.save(&file_path).map_err(|e| e.to_string())?;
    
    Ok(file_path.to_string_lossy().to_string())
}

/// 导出记录到 PDF
#[tauri::command]
pub fn export_to_pdf(state: State<AppState>) -> Result<String, String> {
    use genpdf::{Document, SimplePageDecorator};
    use genpdf::fonts;
    use genpdf::elements::{Paragraph, TableLayout, Text, Break};
    use genpdf::style::{Style, StyledString};
    
    let store = state.store.lock().unwrap();
    let records = store.load_records();
    
    if records.is_empty() {
        return Err("没有可导出的记录".to_string());
    }
    
    // 尝试加载中文字体，Windows下通常是 simhei.ttf
    let font_dir = "C:/Windows/Fonts";
    // 尝试列表：黑体 -> 仿宋 -> 楷体 -> 宋体黑 -> Arial
    // 注意：msyh.ttc 是 TrueType Collection 格式，genpdf 可能不支持
    let font_candidates = vec!["simhei.ttf", "simfang.ttf", "simkai.ttf", "simsunb.ttf", "arial.ttf"];
    
    let mut font_data = None;
    for name in font_candidates {
        let path = std::path::Path::new(font_dir).join(name);
        if path.exists() {
            if let Ok(data) = std::fs::read(&path) {
                font_data = Some(data);
                break;
            }
        }
    }
    
    let font_data = font_data.ok_or("无法找到合适的中文字体(simhei.ttf, simfang.ttf, simkai.ttf, simsunb.ttf, arial.ttf)".to_string())?;
    
    // 解析字体数据
    let font = fonts::FontData::new(font_data, None)
        .map_err(|e| format!("解析字体数据失败: {}", e))?;
        
    // 创建字体族，强制所有样式都使用同一个字体文件
    // 因为中文字体通常只有一个文件，没有分开的 Bold/Italic 文件
    let font_family = fonts::FontFamily {
        regular: font.clone(),
        bold: font.clone(),
        italic: font.clone(),
        bold_italic: font,
    };
    
    let mut doc = Document::new(font_family);
    doc.set_title("抽签历史记录");
    
    // 页面装饰
    let mut decorator = SimplePageDecorator::new();
    decorator.set_margins(20);
    doc.set_page_decorator(decorator);
    
    // 标题
    let title_style = Style::new().bold().with_font_size(18);
    doc.push(Paragraph::new(StyledString::new("宁夏特检院质量监督检查抽签记录", title_style)));
    doc.push(Break::new(1));
    
    // 导出时间
    let time_str = format!("导出时间: {}", chrono::Local::now().format("%Y年%m月%d日 %H:%M:%S"));
    doc.push(Paragraph::new(time_str));
    doc.push(Break::new(1));
    
    // 表格
    let mut table = TableLayout::new(vec![1, 3, 3, 2, 3]);
    table.set_cell_decorator(genpdf::elements::FrameCellDecorator::new(true, true, false));
    
    // 表头
    let header_style = Style::new().bold();
    let row = table.row();
    row.element(Text::new(StyledString::new("序号", header_style.clone())))
       .element(Text::new(StyledString::new("抽签时间", header_style.clone())))
       .element(Text::new(StyledString::new("被检部门", header_style.clone())))
       .element(Text::new(StyledString::new("专责类型", header_style.clone())))
       .element(Text::new(StyledString::new("抽中部门", header_style)))
       .push()
       .map_err(|e| format!("添加表头失败: {}", e))?;
    
    // 数据行
    for (i, record) in records.iter().enumerate() {
        let row = table.row();
        row.element(Text::new(format!("{}", i + 1)))
           .element(Text::new(record.timestamp.format("%Y-%m-%d %H:%M").to_string()))
           .element(Text::new(&record.target_department_name))
           .element(Text::new(record.specialty_type.display_name()))
           .element(Text::new(&record.selected_specialist_name))
           .push()
           .map_err(|e| format!("添加数据行失败: {}", e))?;
    }
    
    doc.push(table);
    
    // 统计信息
    doc.push(Break::new(1));
    let summary = format!("共计 {} 条抽签记录", records.len());
    doc.push(Paragraph::new(summary));
    
    // 保存到桌面
    let filename = format!("抽签记录_{}.pdf", chrono::Local::now().format("%Y%m%d_%H%M%S"));
    let desktop_path = dirs::desktop_dir()
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());
    let file_path = desktop_path.join(&filename);
    
    doc.render_to_file(&file_path).map_err(|e| format!("保存PDF失败: {}", e))?;
    
    Ok(file_path.to_string_lossy().to_string())
}
