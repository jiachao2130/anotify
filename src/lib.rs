/// Async Linux inotify wrapper for Rust.
///
/// # About
///
/// This is a wrapper for Linux inotify API, based `inotify-rs` to provide an async func to watch
/// path(es) changes.
///
/// # Example
///
/// ```
/// use std::ffi::OsString;
/// use anotify_rs::{
///     Anotify,
///     Event,
///     WatchMask,
/// }
///
/// #[tokio::main]
/// async fn main() {
///     let anotify = Anotify {
///         mask: WatchMask::CREATE,
///         regex: None,
///         recursive: true,
///         targets: vec![OsString::from("/tmp/cc")],
///     };
///
///     let (tx, mut rx) = tokio::sync::broadcast::channel::<Event>(128);
///     tokio::spawn(async move {
///         loop {
///             if let Ok(event) = rx.recv().await {
///                 println!("{:?}: {:?}", event.mask(), event.path());
///             }
///         }
///     });
///
///     match anotify_rs::handler::run(anotify, Some(tx), tokio::signal::ctrl_c()).await {
///         // press ctrl_c
///         Ok(()) => {},
///         // catch error
///         Err(e) => panic!("{}", e),
///     };
/// }
/// ```
pub mod app;

pub mod handler;
mod watcher;
mod new_watcher;
pub mod new_handler;

pub use app::Anotify;
pub use inotify::WatchMask;
pub use watcher::Event;

/// 定义 crate::Error
/// 大部分函数返回的错误
pub type Error = Box<dyn std::error::Error + Send + Sync>;

/// 定义 crate::Result
pub type Result<T> = std::result::Result<T, Error>;
