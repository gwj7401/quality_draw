//! 数据模型模块

mod department;
mod specialist;
mod record;

pub use department::{Department, DepartmentType, default_departments};
pub use specialist::{QualitySpecialist, SpecialtyType};
pub use record::DrawRecord;
