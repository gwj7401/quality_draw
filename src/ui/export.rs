//! 导出管理器

use rust_xlsxwriter::{Workbook, Format, FormatAlign, Color};
use crate::models::DrawRecord;
use std::path::PathBuf;

/// 导出管理器
pub struct ExportManager;

impl ExportManager {
    /// 导出抽签记录到Excel
    pub fn export_to_excel(records: &[DrawRecord], path: &PathBuf) -> Result<(), String> {
        let mut workbook = Workbook::new();
        let worksheet = workbook.add_worksheet();
        
        // 设置列宽
        worksheet.set_column_width(0, 20).map_err(|e| e.to_string())?;
        worksheet.set_column_width(1, 15).map_err(|e| e.to_string())?;
        worksheet.set_column_width(2, 12).map_err(|e| e.to_string())?;
        worksheet.set_column_width(3, 12).map_err(|e| e.to_string())?;
        worksheet.set_column_width(4, 15).map_err(|e| e.to_string())?;
        
        // 标题格式
        let header_format = Format::new()
            .set_bold()
            .set_font_size(12)
            .set_align(FormatAlign::Center)
            .set_background_color(Color::RGB(0x4472C4))
            .set_font_color(Color::White);
        
        // 数据格式
        let data_format = Format::new()
            .set_align(FormatAlign::Center);
        
        // 写入标题行
        worksheet.write_string_with_format(0, 0, "抽签时间", &header_format).map_err(|e| e.to_string())?;
        worksheet.write_string_with_format(0, 1, "被检查部门", &header_format).map_err(|e| e.to_string())?;
        worksheet.write_string_with_format(0, 2, "专责类型", &header_format).map_err(|e| e.to_string())?;
        worksheet.write_string_with_format(0, 3, "抽中人员", &header_format).map_err(|e| e.to_string())?;
        worksheet.write_string_with_format(0, 4, "所属部门", &header_format).map_err(|e| e.to_string())?;
        
        // 写入数据
        for (idx, record) in records.iter().enumerate() {
            let row = (idx + 1) as u32;
            
            worksheet.write_string_with_format(
                row, 0,
                &record.timestamp.format("%Y-%m-%d %H:%M:%S").to_string(),
                &data_format
            ).map_err(|e| e.to_string())?;
            
            worksheet.write_string_with_format(
                row, 1,
                &record.target_department_name,
                &data_format
            ).map_err(|e| e.to_string())?;
            
            worksheet.write_string_with_format(
                row, 2,
                record.specialty_type.display_name(),
                &data_format
            ).map_err(|e| e.to_string())?;
            
            worksheet.write_string_with_format(
                row, 3,
                &record.selected_specialist_name,
                &data_format
            ).map_err(|e| e.to_string())?;
            
            worksheet.write_string_with_format(
                row, 4,
                &record.selected_from_department_name,
                &data_format
            ).map_err(|e| e.to_string())?;
        }
        
        // 保存文件
        workbook.save(path).map_err(|e| e.to_string())?;
        
        Ok(())
    }
    
    /// 生成打印内容（HTML格式）
    pub fn generate_print_html(records: &[DrawRecord]) -> String {
        let mut html = String::from(r#"
<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>质量监督检查抽签结果</title>
    <style>
        body { font-family: "Microsoft YaHei", sans-serif; padding: 20px; }
        h1 { text-align: center; color: #333; }
        table { width: 100%; border-collapse: collapse; margin-top: 20px; }
        th, td { border: 1px solid #ddd; padding: 10px; text-align: center; }
        th { background-color: #4472C4; color: white; }
        tr:nth-child(even) { background-color: #f9f9f9; }
        .footer { margin-top: 30px; text-align: right; color: #666; }
    </style>
</head>
<body>
    <h1>宁夏特检院质量监督检查抽签结果</h1>
    <table>
        <tr>
            <th>序号</th>
            <th>抽签时间</th>
            <th>被检查部门</th>
            <th>专责类型</th>
            <th>抽中人员</th>
            <th>所属部门</th>
        </tr>
"#);
        
        for (idx, record) in records.iter().enumerate() {
            html.push_str(&format!(
                r#"        <tr>
            <td>{}</td>
            <td>{}</td>
            <td>{}</td>
            <td>{}</td>
            <td>{}</td>
            <td>{}</td>
        </tr>
"#,
                idx + 1,
                record.timestamp.format("%Y-%m-%d %H:%M:%S"),
                record.target_department_name,
                record.specialty_type.display_name(),
                record.selected_specialist_name,
                record.selected_from_department_name,
            ));
        }
        
        html.push_str(&format!(
            r#"    </table>
    <div class="footer">
        <p>打印时间: {}</p>
    </div>
</body>
</html>"#,
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
        ));
        
        html
    }
    
    /// 保存HTML并用浏览器打开打印
    pub fn print_records(records: &[DrawRecord]) -> Result<(), String> {
        let html = Self::generate_print_html(records);
        
        // 保存到临时文件
        let temp_path = std::env::temp_dir().join("quality_draw_print.html");
        std::fs::write(&temp_path, html).map_err(|e| e.to_string())?;
        
        // 使用默认浏览器打开
        #[cfg(target_os = "windows")]
        {
            std::process::Command::new("cmd")
                .args(["/C", "start", "", temp_path.to_str().unwrap()])
                .spawn()
                .map_err(|e| e.to_string())?;
        }
        
        Ok(())
    }
}
