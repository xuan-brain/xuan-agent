//! 分类系统
//!
//! 提供自动标签生成、文献分类等功能

pub mod tagging;

// 重新导出常用类型
pub use tagging::{Tag, TagCategory, TaggingService};
