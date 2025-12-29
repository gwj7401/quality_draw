//! 抽签算法实现

use rand::seq::SliceRandom;
use crate::models::{Department, QualitySpecialist, SpecialtyType, DrawRecord};

/// 抽签引擎
pub struct DrawEngine;

impl DrawEngine {
    /// 获取可抽取的候选人列表
    /// 
    /// # 参数
    /// - `specialists`: 所有质量专责列表
    /// - `departments`: 所有部门列表
    /// - `target_department_id`: 被检查部门ID（需要规避的部门）
    /// - `specialty_type`: 需要抽取的专责类型
    /// - `last_selected_id`: 上一次抽中的人员ID（需要规避连续抽取）
    /// 
    /// # 返回
    /// 符合条件的候选人列表
    pub fn get_candidates<'a>(
        specialists: &'a [QualitySpecialist],
        _departments: &[Department],
        target_department_id: &str,
        specialty_type: SpecialtyType,
        last_selected_id: Option<&str>,
    ) -> Vec<&'a QualitySpecialist> {
        specialists
            .iter()
            .filter(|s| {
                // 1. 专业类型必须匹配
                s.specialty == specialty_type
                // 2. 不能是被检查部门的人
                && s.department_id != target_department_id
                // 3. 不能是上次抽中的人
                && last_selected_id.map_or(true, |id| s.id != id)
            })
            .collect()
    }
    
    /// 从候选人中随机抽取一人
    pub fn draw_one<'a>(candidates: &[&'a QualitySpecialist]) -> Option<&'a QualitySpecialist> {
        if candidates.is_empty() {
            return None;
        }
        let mut rng = rand::thread_rng();
        candidates.choose(&mut rng).copied()
    }
    
    /// 执行一次完整的抽签
    /// 
    /// # 参数
    /// - `specialists`: 所有质量专责列表
    /// - `departments`: 所有部门列表
    /// - `target_department`: 被检查部门
    /// - `specialty_type`: 需要抽取的专责类型
    /// - `records`: 历史抽签记录（用于查找上次抽中的人）
    /// 
    /// # 返回
    /// 抽中的人员及其所属部门名称
    pub fn execute_draw(
        specialists: &[QualitySpecialist],
        departments: &[Department],
        target_department: &Department,
        specialty_type: SpecialtyType,
        records: &[DrawRecord],
    ) -> Option<(QualitySpecialist, String)> {
        // 查找上次抽取该部门该类型时抽中的人
        let last_selected_id = records
            .iter()
            .rev()
            .find(|r| {
                r.target_department_id == target_department.id
                && r.specialty_type == specialty_type
            })
            .map(|r| r.selected_specialist_id.as_str());
        
        // 获取候选人
        let candidates = Self::get_candidates(
            specialists,
            departments,
            &target_department.id,
            specialty_type,
            last_selected_id,
        );
        
        // 随机抽取
        if let Some(selected) = Self::draw_one(&candidates) {
            // 获取部门名称
            let dept_name = departments
                .iter()
                .find(|d| d.id == selected.department_id)
                .map(|d| d.name.clone())
                .unwrap_or_else(|| "未知部门".to_string());
            
            Some((selected.clone(), dept_name))
        } else {
            None
        }
    }
    
    /// 获取随机滚动显示的名单（用于动画）
    pub fn get_rolling_names(
        specialists: &[QualitySpecialist],
        target_department_id: &str,
        specialty_type: SpecialtyType,
        last_selected_id: Option<&str>,
    ) -> Vec<String> {
        specialists
            .iter()
            .filter(|s| {
                // 1. 专业类型必须匹配
                s.specialty == specialty_type
                // 2. 不能是被检查部门的人
                && s.department_id != target_department_id
                // 3. 不能是上次抽中的人
                && last_selected_id.map_or(true, |id| s.id != id)
            })
            .map(|s| s.name.clone())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::default_departments;
    
    fn create_test_specialists() -> Vec<QualitySpecialist> {
        vec![
            QualitySpecialist::new("1", "张三", "szs", SpecialtyType::Pressure),
            QualitySpecialist::new("2", "李四", "nd", SpecialtyType::Pressure),
            QualitySpecialist::new("3", "王五", "cy1", SpecialtyType::Pressure),
            QualitySpecialist::new("4", "赵六", "szs", SpecialtyType::Mechanical),
            QualitySpecialist::new("5", "钱七", "jd1", SpecialtyType::Mechanical),
        ]
    }
    
    #[test]
    fn test_department_avoidance() {
        let specialists = create_test_specialists();
        let departments = default_departments();
        
        // 抽取石嘴山分院的承压类专责，应该排除张三（石嘴山分院的人）
        let candidates = DrawEngine::get_candidates(
            &specialists,
            &departments,
            "szs",
            SpecialtyType::Pressure,
            None,
        );
        
        assert!(!candidates.iter().any(|s| s.name == "张三"));
        assert!(candidates.iter().any(|s| s.name == "李四"));
        assert!(candidates.iter().any(|s| s.name == "王五"));
    }
    
    #[test]
    fn test_consecutive_avoidance() {
        let specialists = create_test_specialists();
        let departments = default_departments();
        
        // 上次抽中李四，这次应该排除李四
        let candidates = DrawEngine::get_candidates(
            &specialists,
            &departments,
            "szs",
            SpecialtyType::Pressure,
            Some("2"), // 李四的ID
        );
        
        assert!(!candidates.iter().any(|s| s.name == "李四"));
        assert!(candidates.iter().any(|s| s.name == "王五"));
    }
    
    #[test]
    fn test_specialty_filter() {
        let specialists = create_test_specialists();
        let departments = default_departments();
        
        // 抽取机电类专责
        let candidates = DrawEngine::get_candidates(
            &specialists,
            &departments,
            "szs",
            SpecialtyType::Mechanical,
            None,
        );
        
        // 只应该有机电类的人，且不包含石嘴山分院的赵六
        assert!(!candidates.iter().any(|s| s.name == "赵六"));
        assert!(candidates.iter().any(|s| s.name == "钱七"));
    }
}
