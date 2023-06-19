use std::path::Path;

use futures_util::StreamExt;
use inotify::{EventStream, Inotify, WatchMask};

use crate::app;

struct Watcher {
    inotify: Inotify,
    watchmask: WatchMask,
}

impl Watcher {
    /// 初始化一个 Watcher, 包括一个 Inotify 实例和一个空 WatchMask
    fn init() -> crate::Result<Self> {
        let inotify = Inotify::init().expect("Failed to initialize Inotify");
        Ok(Watcher {
            inotify,
            watchmask: WatchMask::empty(),
        })
    }

    /// 设置新的 WatchMask
    fn set_watchmask(&mut self, mask: WatchMask) {
        self.watchmask = mask;
    }

    /// 向 Inotify 里添加要监视的路径，使用当前 WatchMask
    fn add_path<P>(&mut self, path: P) -> crate::Result<()>
    where
        P: AsRef<Path>,
    {
        self.inotify.watches().add(path, self.watchmask)?;
        Ok(())
    }

    /// 将自身转换为一个事件流，此操作将消耗自身
    fn into_event_stream<T>(self, buffer: T) -> crate::Result<(EventStream<T>, WatchMask)>
    where
        T: AsMut<[u8]> + AsRef<[u8]>,
    {
        Ok((self.inotify.into_event_stream(buffer)?, self.watchmask))
    }
}

pub async fn run() -> crate::Result<()> {
    //let anotify = app::parse()?;
    let app::Anotify {
        mask,
        recursive,
        regex,
        targets,
    } = app::parse()?;

    let mut watcher = Watcher::init()?;
    watcher.set_watchmask(mask);
    for target in targets {
        watcher.add_path(target)?;
    }

    let mut buffer = [0; 1024];
    let (mut stream, mask) = watcher.into_event_stream(&mut buffer)?;

    while let Some(event) = stream.next().await {
        println!("{:?}", event);
        //let mask = event.as_ref().unwrap().mask;
        //let name = event
        //    .as_ref()
        //    .unwrap()
        //    .name
        //    .as_ref()
        //    .unwrap()
        //    .clone()
        //    .into_string()
        //    .unwrap();

        //println!("{:?}: {}", mask, name);
    }

    Ok(())
}
