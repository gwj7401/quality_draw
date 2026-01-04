//! 滚动动画状态管理 - 转盘效果

use std::time::{Duration, Instant};
use rand::seq::SliceRandom;

/// 动画状态
#[derive(Debug, Clone, PartialEq)]
pub enum AnimationPhase {
    /// 空闲
    Idle,
    /// 滚动中
    Rolling,
    /// 减速中
    SlowingDown,
    /// 已停止，显示结果
    Stopped,
}

/// 转盘滚动动画状态
#[derive(Clone)]
pub struct AnimationState {
    /// 当前阶段
    pub phase: AnimationPhase,
    /// 候选人名单
    pub candidates: Vec<String>,
    /// 当前中心索引（浮点数用于平滑滚动）
    pub scroll_position: f32,
    /// 滚动速度（每秒滚动的项目数）
    pub scroll_speed: f32,
    /// 上次更新时间
    last_update: Instant,
    /// 最终结果
    pub final_result: Option<String>,
    /// 开始减速的时间
    pub slowdown_start: Option<Instant>,
    /// 减速持续时间
    pub slowdown_duration: Duration,
    /// 初始滚动速度
    initial_speed: f32,
}

impl Default for AnimationState {
    fn default() -> Self {
        Self {
            phase: AnimationPhase::Idle,
            candidates: Vec::new(),
            scroll_position: 0.0,
            scroll_speed: 20.0,  // 每秒滚动20个名字
            last_update: Instant::now(),
            final_result: None,
            slowdown_start: None,
            slowdown_duration: Duration::from_millis(3000), // 减速持续3秒
            initial_speed: 20.0,
        }
    }
}

impl AnimationState {
    /// 开始滚动动画
    pub fn start(&mut self, candidates: Vec<String>) {
        if candidates.is_empty() {
            return;
        }
        
        // 打乱顺序
        let mut shuffled = candidates;
        let mut rng = rand::thread_rng();
        shuffled.shuffle(&mut rng);
        
        // 根据候选人数量动态调整速度
        // 目标：无论人数多少，转盘都流畅
        // 速度 = 候选人数量 * 每秒圈数
        let candidate_count = shuffled.len() as f32;
        let rotations_per_second = 3.0; // 每秒转3圈
        let calculated_speed = candidate_count * rotations_per_second;
        
        // 限制最低速度为30，最高速度为80（确保流畅）
        let speed = calculated_speed.clamp(30.0, 80.0);
        
        self.candidates = shuffled;
        self.phase = AnimationPhase::Rolling;
        self.scroll_position = 0.0;
        self.scroll_speed = speed;
        self.initial_speed = speed;
        self.last_update = Instant::now();
        self.final_result = None;
        self.slowdown_start = None;
    }
    
    /// 请求停止（进入减速阶段）
    pub fn request_stop(&mut self) {
        if self.phase == AnimationPhase::Rolling {
            self.phase = AnimationPhase::SlowingDown;
            self.slowdown_start = Some(Instant::now());
            self.initial_speed = self.scroll_speed;
        }
    }
    
    /// 更新动画状态
    pub fn update(&mut self) -> bool {
        if self.candidates.is_empty() {
            return false;
        }
        
        let delta_time = self.last_update.elapsed().as_secs_f32();
        self.last_update = Instant::now();
        
        match self.phase {
            AnimationPhase::Idle | AnimationPhase::Stopped => false,
            
            AnimationPhase::Rolling => {
                // 持续滚动
                self.scroll_position += self.scroll_speed * delta_time;
                // 循环
                while self.scroll_position >= self.candidates.len() as f32 {
                    self.scroll_position -= self.candidates.len() as f32;
                }
                true
            }
            
            AnimationPhase::SlowingDown => {
                if let Some(start) = self.slowdown_start {
                    let elapsed = start.elapsed();
                    
                    if elapsed >= self.slowdown_duration {
                        // 动画结束，对齐到最近的整数索引
                        let final_index = self.scroll_position.round() as usize % self.candidates.len();
                        self.scroll_position = final_index as f32;
                        self.phase = AnimationPhase::Stopped;
                        self.final_result = self.candidates.get(final_index).cloned();
                        self.scroll_speed = 0.0;
                        return true;
                    }
                    
                    // 计算减速进度 (0.0 - 1.0)
                    let progress = elapsed.as_secs_f32() / self.slowdown_duration.as_secs_f32();
                    
                    // 使用缓出函数，使减速更加平滑自然
                    // easeOutCubic: 1 - (1 - progress)^3
                    let ease_progress = 1.0 - (1.0 - progress).powi(3);
                    
                    // 速度从初始速度渐变到接近0
                    self.scroll_speed = self.initial_speed * (1.0 - ease_progress);
                    
                    // 更新位置
                    self.scroll_position += self.scroll_speed * delta_time;
                    
                    // 循环
                    while self.scroll_position >= self.candidates.len() as f32 {
                        self.scroll_position -= self.candidates.len() as f32;
                    }
                    
                    true
                } else {
                    false
                }
            }
        }
    }
    
    /// 获取当前中心索引（整数）
    pub fn current_index(&self) -> usize {
        if self.candidates.is_empty() {
            return 0;
        }
        (self.scroll_position.round() as usize) % self.candidates.len()
    }
    
    /// 获取当前显示的名字
    pub fn current_name(&self) -> Option<&str> {
        self.candidates.get(self.current_index()).map(|s| s.as_str())
    }
    
    /// 获取指定偏移位置的名字（用于转盘显示）
    /// offset: 0 = 中心, -1 = 上一个, 1 = 下一个, etc.
    pub fn get_name_at_offset(&self, offset: i32) -> Option<&str> {
        if self.candidates.is_empty() {
            return None;
        }
        let len = self.candidates.len() as i32;
        let base_index = self.scroll_position.round() as i32;
        let mut index = (base_index + offset) % len;
        if index < 0 {
            index += len;
        }
        self.candidates.get(index as usize).map(|s| s.as_str())
    }
    
    /// 获取滚动的小数部分（用于平滑动画）
    pub fn get_scroll_fraction(&self) -> f32 {
        self.scroll_position - self.scroll_position.floor()
    }
    
    /// 是否正在运行
    pub fn is_running(&self) -> bool {
        matches!(self.phase, AnimationPhase::Rolling | AnimationPhase::SlowingDown)
    }
    
    /// 重置状态
    #[allow(dead_code)]
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}
