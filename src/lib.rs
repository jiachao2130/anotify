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
/// use async_inotify::{
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
///     match async_inotify::handler::run(anotify, Some(tx), tokio::signal::ctrl_c()).await {
///         // press ctrl_c
///         Ok(()) => {},
///         // catch error
///         Err(e) => panic!("{}", e),
///     };
/// }
/// ```
///
/// Or operate Watcher as you like.
///
/// ```rust
/// use async_inotify::{WatchMask, Watcher};
/// 
/// #[tokio::main]
/// async fn main() {
///     let mut watcher = Watcher::init();
///     let mask = WatchMask::CREATE;
/// 
///     let wd = watcher.add("/tmp/cc", &mask).unwrap();
/// 
///     // watch once
///     if let Some(event) = watcher.next().await {
///         println!("{:?}: {:?}", event.mask(), event.path());
///     }
/// 
///     watcher.remove(wd).unwrap();
/// }
/// ```
pub mod app;

pub mod handler;
mod watcher;

pub use app::Anotify;
pub use inotify::WatchMask;
pub use watcher::Event;
pub use watcher::Watcher;

/// 定义 crate::Error
/// 大部分函数返回的错误
pub type Error = Box<dyn std::error::Error + Send + Sync>;

/// 定义 crate::Result
pub type Result<T> = std::result::Result<T, Error>;
