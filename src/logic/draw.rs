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
    /// - `current_round_selected_ids`: 本轮已抽中的人员ID列表
    /// - `cross_avoidance_dept_ids`: 需要交叉回避的部门ID列表（这些部门的专责不能参与）
    /// 
    /// # 返回
    /// 符合条件的候选人列表
    pub fn get_candidates<'a>(
        specialists: &'a [QualitySpecialist],
        _departments: &[Department],
        target_department_id: &str,
        specialty_type: SpecialtyType,
        last_selected_id: Option<&str>,
        current_round_selected_ids: &[String],
        cross_avoidance_dept_ids: &[String],
    ) -> Vec<&'a QualitySpecialist> {
        specialists
            .iter()
            .filter(|s| {
                // 1. 专业类型必须匹配
                s.specialty == specialty_type
                // 2. 不能是被检查部门的人
                && s.department_id != target_department_id
                // 3. 不能是上次抽中的人（连续回避）
                && last_selected_id.map_or(true, |id| s.id != id)
                // 4. 不能是本轮已抽中的人
                && !current_round_selected_ids.contains(&s.id)
                // 5. 不能是需要交叉回避的部门的人
                && !cross_avoidance_dept_ids.contains(&s.department_id)
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
    /// - `current_round_selected_ids`: 本轮已抽中的人员ID列表
    /// 
    /// # 返回
    /// 抽中的人员及其所属部门名称
    pub fn execute_draw(
        specialists: &[QualitySpecialist],
        departments: &[Department],
        target_department: &Department,
        specialty_type: SpecialtyType,
        records: &[DrawRecord],
        current_round_selected_ids: &[String],
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
        
        // 计算交叉回避的部门列表
        // 逻辑：如果目标部门(A)被其他部门(B)的某专责检查过，那么A的同类型专责也不能去检查B
        // 即：找到所有 "抽中的人所属部门是target_department" 的记录，这些记录的被检部门需要回避
        let cross_avoidance_dept_ids: Vec<String> = records
            .iter()
            .filter(|r| {
                // 找到本类型中，抽中人员来自目标部门的记录
                r.specialty_type == specialty_type
                && r.selected_from_department_id == target_department.id
            })
            .map(|r| r.target_department_id.clone())
            .collect();
        
        // 获取候选人
        let candidates = Self::get_candidates(
            specialists,
            departments,
            &target_department.id,
            specialty_type,
            last_selected_id,
            current_round_selected_ids,
            &cross_avoidance_dept_ids,
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
        current_round_selected_ids: &[String],
        cross_avoidance_dept_ids: &[String],
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
                // 4. 不能是本轮已抽中的人
                && !current_round_selected_ids.contains(&s.id)
                // 5. 不能是需要交叉回避的部门的人
                && !cross_avoidance_dept_ids.contains(&s.department_id)
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
            &[],
            &[],
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
            &[],
            &[],
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
            &[],
            &[],
        );
        
        // 只应该有机电类的人，且不包含石嘴山分院的赵六
        assert!(!candidates.iter().any(|s| s.name == "赵六"));
        assert!(candidates.iter().any(|s| s.name == "钱七"));
    }
}
