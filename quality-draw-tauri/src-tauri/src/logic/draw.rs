//! 抽签算法实现

use rand::seq::SliceRandom;
use crate::models::{Department, QualitySpecialist, SpecialtyType, DrawRecord};

/// 抽签引擎
pub struct DrawEngine;

impl DrawEngine {
    /// 获取可抽取的候选人列表
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
                s.specialty == specialty_type
                && s.department_id != target_department_id
                && last_selected_id.map_or(true, |id| s.id != id)
                && !current_round_selected_ids.contains(&s.id)
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
        let cross_avoidance_dept_ids: Vec<String> = records
            .iter()
            .filter(|r| {
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
                s.specialty == specialty_type
                && s.department_id != target_department_id
                && last_selected_id.map_or(true, |id| s.id != id)
                && !current_round_selected_ids.contains(&s.id)
                && !cross_avoidance_dept_ids.contains(&s.department_id)
            })
            .map(|s| s.name.clone())
            .collect()
    }
}
