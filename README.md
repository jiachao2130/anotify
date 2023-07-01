# anotify
一个异步基于 Linux inotify 的目录监视工具

## Example

### 工具

可作为小程序使用

```bash
$ anotify -h
Usage: anotify [OPTIONS] [target]...

Arguments:
  [target]...  [default: ./]

Options:
  -a, --access         File was accessed
  -A, --attrib         Metadata (permissions, timestamps, …) changed
  -x, --close_write    File opened for writing was closed
  -q, --close_nowrite  File or directory not opened for writing was closed
  -c, --create         File/directory created in watched directory
  -d, --delete         File/directory deleted from watched directory
  -D, --delete_self    Watched file/directory was deleted
  -m, --modify         File was modified
  -S, --move_self      Watched file/directory was moved
  -F, --moved_from     File was renamed/moved; watched directory contained old name
  -T, --moved_to       File was renamed/moved; watched directory contains new name
  -o, --open           File or directory was opened
  -M, --move           Watch for all move events
  -Q, --close          Watch for all close events
      --all            Watch for all events
      --dont_follow    Don’t dereference the path if it is a symbolic link
      --excl_unlink    Filter events for directory entries that have been unlinked
      --mask_add       If a watch for the inode exists, amend it instead of replacing it
      --oneshot        Only receive one event, then remove the watch
      --dir            Only watch path, if it is a directory
  -R, --recursive      Recursive monitor a path
  -E, --regex <regex>  Use regex to match file name, only matched will report
  -h, --help           Print help
```

### anotify 库

使用 `tokio` ，调用 anotfiy 的异步库函数 `handler::run()`。

```rust
use std::ffi::OsString;
use async_inotify::{
    Anotify,
    Event,
    WatchMask,
}

#[tokio::main]
async fn main() {
    let anotify = Anotify {
        mask: WatchMask::CREATE,
        regex: None,
        recursive: true,
        targets: vec![OsString::from("/tmp/cc")],
    };

    let (tx, mut rx) = tokio::sync::broadcast::channel::<Event>(128);
    tokio::spawn(async move {
        loop {
            if let Ok(event) = rx.recv().await {
                println!("{:?}: {:?}", event.mask(), event.path());
            }
        }
    });

    match async_inotify::handler::run(anotify, Some(tx), tokio::signal::ctrl_c()).await {
        // press ctrl_c
        Ok(()) => {},
        // catch error
        Err(e) => panic!("{}", e),
    };
}
```

或直接使用 Watcher 并自定义 handler：

```rust
use async_inotify::{WatchMask, Watcher};

#[tokio::main]
async fn main() {
    let mut watcher = Watcher::init();
    let mask = WatchMask::CREATE;

    let wd = watcher.add("/tmp/cc", &mask).unwrap();

    // watch once
    if let Some(event) = watcher.next().await {
        println!("{:?}: {:?}", event.mask(), event.path());
    }

    watcher.remove(wd).unwrap();
}
```
