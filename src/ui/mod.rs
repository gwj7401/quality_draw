//! UI模块

mod main_panel;
mod animation;
mod settings;
mod history;
mod export;

pub use main_panel::MainPanel;
pub use animation::AnimationState;
pub use settings::SettingsPanel;
pub use history::HistoryPanel;
pub use export::ExportManager;
