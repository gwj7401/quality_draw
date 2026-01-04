//! 抽签记录相关数据模型

use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use super::SpecialtyType;

/// 抽签记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrawRecord {
    /// 记录ID
    pub id: String,
    /// 抽签时间
    pub timestamp: DateTime<Local>,
    /// 被检查部门ID
    pub target_department_id: String,
    /// 被检查部门名称
    pub target_department_name: String,
    /// 抽取的专责类型
    pub specialty_type: SpecialtyType,
    /// 抽中的人员ID
    pub selected_specialist_id: String,
    /// 抽中的人员姓名
    pub selected_specialist_name: String,
    /// 抽中人员所属部门ID
    pub selected_from_department_id: String,
    /// 抽中人员所属部门名称
    pub selected_from_department_name: String,
}

impl DrawRecord {
    /// 创建新的抽签记录
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        target_department_id: impl Into<String>,
        target_department_name: impl Into<String>,
        specialty_type: SpecialtyType,
        selected_specialist_id: impl Into<String>,
        selected_specialist_name: impl Into<String>,
        selected_from_department_id: impl Into<String>,
        selected_from_department_name: impl Into<String>,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: Local::now(),
            target_department_id: target_department_id.into(),
            target_department_name: target_department_name.into(),
            specialty_type,
            selected_specialist_id: selected_specialist_id.into(),
            selected_specialist_name: selected_specialist_name.into(),
            selected_from_department_id: selected_from_department_id.into(),
            selected_from_department_name: selected_from_department_name.into(),
        }
    }
}
