use std::path::{Path, PathBuf};
use std::collections::HashMap;

use futures_util::StreamExt;
use inotify::{EventMask, EventStream, Inotify, WatchDescriptor, WatchMask};
use path_absolutize::*;

/// A Watcher contains EventStream，maintained a `HashMap` about `WatchDescriptor` to `PathBuf`
#[derive(Debug)]
pub struct Watcher {
    stream: EventStream<[u8; 1024]>,
    wds: HashMap<WatchDescriptor, PathBuf>,
}

impl Watcher {
    pub fn init() -> Self {
        let inotify = Inotify::init().expect("Failed to initialize Inotify");
        let buffer = [0; 1024];
        let stream = inotify.into_event_stream(buffer).unwrap();
        Watcher {
            stream,
            wds: HashMap::new(),
        }
    }

    pub fn add<P>(&mut self, path: P, mask: &WatchMask) -> crate::Result<()>
    where
        P: AsRef<Path> + std::convert::AsRef<std::ffi::OsStr>,
    {
        // check path exists
        let root = Path::new(&path).absolutize()?;
        if !root.exists() {
            return Err(format!("Error: {:?} is not exists", root).into());
        }

        let wd = self.stream.watches().add(&root, mask.clone())?;
        self.wds.insert(wd, root.to_path_buf());
        Ok(())
    }

    pub fn remove(&mut self, wd: WatchDescriptor) -> crate::Result<()> {
        self.stream.watches().remove(wd.clone())?;
        self.wds.remove(&wd);
        Ok(())
    }

    pub async fn next(&mut self) -> Option<Event> {
        match self.stream.next().await {
            // 获取事件，转换为 `Watcher::Event`
            Some(event) => {
                let inotify::Event {
                    wd,
                    mask,
                    cookie: _,
                    name,
                } = event.unwrap();

                let mut root = self.wds.get(&wd).unwrap().clone();
                if let Some(name) = name {
                    root.push(name);
                }

                return Some(Event {
                    root,
                    mask,
                })
            },
            _ => return None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Event {
    root: PathBuf,
    mask: EventMask,
}

impl Event {
    pub fn path(&self) -> &Path {
        &self.root
    }

    pub fn mask(&self) -> &EventMask {
        &self.mask
    }
}
