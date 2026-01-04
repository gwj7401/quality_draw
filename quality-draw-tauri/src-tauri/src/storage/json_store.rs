//! JSON数据存储

use std::fs;
use std::path::PathBuf;
use crate::models::{Department, QualitySpecialist, DrawRecord, default_departments};

/// 数据存储管理器
pub struct DataStore {
    data_dir: PathBuf,
}

impl DataStore {
    /// 创建数据存储管理器
    pub fn new() -> Self {
        let data_dir = Self::get_data_dir();
        
        if !data_dir.exists() {
            fs::create_dir_all(&data_dir).ok();
        }
        
        Self { data_dir }
    }
    
    /// 获取数据存储目录（exe同目录下的data文件夹）
    fn get_data_dir() -> PathBuf {
        // 方法1：使用current_exe获取可执行文件路径
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(parent) = exe_path.parent() {
                let data_dir = parent.join("data");
                if fs::create_dir_all(&data_dir).is_ok() {
                    return data_dir;
                }
            }
        }
        
        // 方法2：使用当前工作目录
        if let Ok(cwd) = std::env::current_dir() {
            let data_dir = cwd.join("data");
            if fs::create_dir_all(&data_dir).is_ok() {
                return data_dir;
            }
        }
        
        // 方法3：使用相对路径作为最后回退
        PathBuf::from("data")
    }
    
    fn departments_path(&self) -> PathBuf {
        self.data_dir.join("departments.json")
    }
    
    fn specialists_path(&self) -> PathBuf {
        self.data_dir.join("specialists.json")
    }
    
    fn records_path(&self) -> PathBuf {
        self.data_dir.join("records.json")
    }
    
    pub fn load_departments(&self) -> Vec<Department> {
        let path = self.departments_path();
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(departments) = serde_json::from_str(&content) {
                    return departments;
                }
            }
        }
        let departments = default_departments();
        self.save_departments(&departments);
        departments
    }
    
    pub fn save_departments(&self, departments: &[Department]) {
        if let Ok(content) = serde_json::to_string_pretty(departments) {
            fs::write(self.departments_path(), content).ok();
        }
    }
    
    pub fn load_specialists(&self) -> Vec<QualitySpecialist> {
        let path = self.specialists_path();
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(specialists) = serde_json::from_str(&content) {
                    return specialists;
                }
            }
        }
        Vec::new()
    }
    
    pub fn save_specialists(&self, specialists: &[QualitySpecialist]) {
        if let Ok(content) = serde_json::to_string_pretty(specialists) {
            fs::write(self.specialists_path(), content).ok();
        }
    }
    
    pub fn load_records(&self) -> Vec<DrawRecord> {
        let path = self.records_path();
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(records) = serde_json::from_str(&content) {
                    return records;
                }
            }
        }
        Vec::new()
    }
    
    pub fn save_records(&self, records: &[DrawRecord]) {
        if let Ok(content) = serde_json::to_string_pretty(records) {
            fs::write(self.records_path(), content).ok();
        }
    }
    
    pub fn add_record(&self, record: DrawRecord) {
        let mut records = self.load_records();
        records.push(record);
        self.save_records(&records);
    }
    
    pub fn clear_records(&self) {
        self.save_records(&[]);
    }
}

impl Default for DataStore {
    fn default() -> Self {
        Self::new()
    }
}
