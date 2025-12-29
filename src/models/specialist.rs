//! 质量专责相关数据模型

use serde::{Deserialize, Serialize};

/// 专业类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpecialtyType {
    /// 承压类专责
    Pressure,
    /// 机电类专责
    Mechanical,
}

impl SpecialtyType {
    /// 获取显示名称
    pub fn display_name(&self) -> &'static str {
        match self {
            SpecialtyType::Pressure => "承压类",
            SpecialtyType::Mechanical => "机电类",
        }
    }
}

/// 质量专责
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualitySpecialist {
    /// 专责ID
    pub id: String,
    /// 姓名
    pub name: String,
    /// 所属部门ID
    pub department_id: String,
    /// 专业类型
    pub specialty: SpecialtyType,
}

impl QualitySpecialist {
    /// 创建新的质量专责
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        department_id: impl Into<String>,
        specialty: SpecialtyType,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            department_id: department_id.into(),
            specialty,
        }
    }
}
