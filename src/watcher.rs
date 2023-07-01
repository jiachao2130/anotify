use std::collections::HashMap;
use std::path::{Path, PathBuf};

use futures_util::StreamExt;
use inotify::{EventMask, EventStream, Inotify, WatchDescriptor, WatchMask};
use path_absolutize::*;

/// A Watcher contains EventStream，maintained a `HashMap` about `WatchDescriptor` to `PathBuf`
#[derive(Debug)]
pub struct Watcher {
    // An Inotfiy::EventStream
    stream: EventStream<[u8; 1024]>,

    // Use this Map to get real Path by wd.
    wds: HashMap<WatchDescriptor, PathBuf>,
}

impl Watcher {
    // init a new Watcher
    pub fn init() -> Self {
        let inotify = Inotify::init().expect("Failed to initialize Inotify");
        let buffer = [0; 1024];
        let stream = inotify.into_event_stream(buffer).unwrap();
        Watcher {
            stream,
            wds: HashMap::new(),
        }
    }

    // Add a new path with watchmask to Watcher.
    pub fn add<P>(&mut self, path: P, mask: &WatchMask) -> crate::Result<WatchDescriptor>
    where
        P: AsRef<Path> + std::convert::AsRef<std::ffi::OsStr>,
    {
        // check path exists
        let root = Path::new(&path).absolutize()?;
        if !root.exists() {
            return Err(format!("Error: {:?} is not exists", root).into());
        }

        let wd = self.stream.watches().add(&root, mask.clone())?;
        self.wds.insert(wd.clone(), root.to_path_buf());
        Ok(wd)
    }

    // remove a watch by WatchDescriptor
    pub fn remove(&mut self, wd: WatchDescriptor) -> crate::Result<()> {
        self.wds.remove(&wd);
        self.stream.watches().remove(wd)?;
        Ok(())
    }

    // Async call for Watched events.
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

                return Some(Event { wd, root, mask });
            }
            _ => return None,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Event {
    wd: WatchDescriptor,
    root: PathBuf,
    mask: EventMask,
}

impl Event {
    // Show the watched fd's WatchDescriptor
    pub fn wd(&self) -> &WatchDescriptor {
        &self.wd
    }

    // The full file path about the event
    pub fn path(&self) -> &Path {
        &self.root
    }

    // Show raw EventMask
    pub fn mask(&self) -> &EventMask {
        &self.mask
    }

    // Show the mask as WatchMask
    pub fn watchmask(&self) -> WatchMask {
        WatchMask::from_bits(&self.mask.bits() & WatchMask::ALL_EVENTS.bits()).unwrap()
    }

    // Is directory event
    pub fn is_dir(&self) -> bool {
        !(self.mask & EventMask::ISDIR).is_empty()
    }

    // Is new (create/moved_from) file
    pub fn is_new(&self) -> bool {
        !(self.mask & (EventMask::CREATE | EventMask::MOVED_FROM)).is_empty()
    }

    // Is watched fd be removed
    pub fn removed(&self) -> bool {
        !(self.mask & EventMask::IGNORED).is_empty()
    }
}
