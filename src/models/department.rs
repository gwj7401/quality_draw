//! 部门相关数据模型

use serde::{Deserialize, Serialize};

/// 部门类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DepartmentType {
    /// 承压类部门
    Pressure,
    /// 机电类部门
    Mechanical,
    /// 综合类部门（既有承压又有机电）
    Comprehensive,
}

impl DepartmentType {
    /// 获取显示名称
    pub fn display_name(&self) -> &'static str {
        match self {
            DepartmentType::Pressure => "承压类",
            DepartmentType::Mechanical => "机电类",
            DepartmentType::Comprehensive => "综合类",
        }
    }
    
    /// 判断是否需要抽取承压类专责
    pub fn needs_pressure(&self) -> bool {
        matches!(self, DepartmentType::Pressure | DepartmentType::Comprehensive)
    }
    
    /// 判断是否需要抽取机电类专责
    pub fn needs_mechanical(&self) -> bool {
        matches!(self, DepartmentType::Mechanical | DepartmentType::Comprehensive)
    }
}

/// 部门
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Department {
    /// 部门ID
    pub id: String,
    /// 部门名称
    pub name: String,
    /// 部门类型
    pub department_type: DepartmentType,
}

impl Department {
    /// 创建新部门
    pub fn new(id: impl Into<String>, name: impl Into<String>, department_type: DepartmentType) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            department_type,
        }
    }
}

/// 预置部门列表
pub fn default_departments() -> Vec<Department> {
    vec![
        // 综合类（需要同时抽取承压和机电）
        Department::new("nd", "宁东分院", DepartmentType::Comprehensive),
        Department::new("szs", "石嘴山分院", DepartmentType::Comprehensive),
        Department::new("wz", "吴忠分院", DepartmentType::Comprehensive),
        Department::new("zw", "中卫分院", DepartmentType::Comprehensive),
        Department::new("gy", "固原分院", DepartmentType::Comprehensive),
        // 承压类
        Department::new("cy1", "承压特种设备一部", DepartmentType::Pressure),
        Department::new("cy2", "承压特种设备二部", DepartmentType::Pressure),
        Department::new("zh", "综合检验检测站", DepartmentType::Pressure),
        // 机电类
        Department::new("jd1", "机电特种设备一部", DepartmentType::Mechanical),
        Department::new("jd2", "机电特种设备二部", DepartmentType::Mechanical),
    ]
}
