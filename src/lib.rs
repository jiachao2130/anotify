pub mod app;

pub mod watcher;

/// 定义 crate::Error
/// 大部分函数返回的错误
pub type Error = Box<dyn std::error::Error + Send + Sync>;

/// 定义 crate::Result
pub type Result<T> = std::result::Result<T, Error>;
