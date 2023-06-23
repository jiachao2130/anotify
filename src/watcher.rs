use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};

use futures_util::StreamExt;
use inotify::{EventMask, EventStream, Inotify, WatchMask};
use path_absolutize::*;
use tokio::sync::mpsc::Sender;

#[derive(Debug)]
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
    fn set_watchmask(&mut self, mask: &WatchMask) {
        self.watchmask = mask.clone();
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
    fn into_event_stream<T>(self, buffer: T) -> crate::Result<EventStream<T>>
    where
        T: AsMut<[u8]> + AsRef<[u8]>,
    {
        Ok(self.inotify.into_event_stream(buffer)?)
    }
}

#[derive(Clone, Debug)]
pub struct Event {
    root: OsString,
    mask: WatchMask,
    name: Option<OsString>,
}

impl Event {
    pub fn path(&self) -> PathBuf {
        let mut path = PathBuf::from(&self.root);
        if let Some(name) = &self.name {
            path.push(name);
        }

        path
    }

    pub fn mask(&self) -> &WatchMask {
        &self.mask
    }
}

pub async fn watch<P>(
    path: P,
    mask: &WatchMask,
    recursive: bool,
    handler: Sender<OsString>,
    fliter: Sender<Event>,
) -> crate::Result<()>
where
    P: AsRef<Path> + std::convert::AsRef<OsStr>,
{
    // check path, convert to abs_path
    let root = Path::new(&path).absolutize()?;
    if !root.exists() {
        return Err(format!("Error: {:?} is not exists", root).into());
    }

    let mut watcher = Watcher::init()?;
    watcher.set_watchmask(mask);
    watcher.add_path(&root)?;

    let mut buffer = [0; 1024];
    let mut stream = watcher.into_event_stream(&mut buffer)?;
    if recursive && root.is_dir() {
        let sub_dirs = sub_dir(&path)?;

        let handler = handler.clone();
        tokio::spawn(async move {
            for dir in sub_dirs {
                handler.send(dir).await.unwrap();
            }
        });
    }

    while let Some(event) = stream.next().await {
        // 若为递归模式，create new dir 需将其发送至 handler
        if recursive
            && !(event.as_ref().unwrap().mask & EventMask::CREATE).is_empty()
            && !(event.as_ref().unwrap().mask & EventMask::ISDIR).is_empty()
        {
            let handler = handler.clone();
            handler
                .send(
                    root.join(event.as_ref().unwrap().name.clone().unwrap())
                        .into(),
                )
                .await?;
        }

        // 当监控文件被删除，则退出当前任务
        if !(event.as_ref().unwrap().mask & EventMask::IGNORED).is_empty() {
            return Ok(());
        }

        // 监控文件变动并发送至 fliter
        fliter
            .send(Event {
                root: root.as_os_str().to_os_string(),
                mask: WatchMask::from_bits(
                    event.as_ref().unwrap().mask.bits() & WatchMask::ALL_EVENTS.bits(),
                )
                .unwrap(),
                name: event.as_ref().unwrap().name.clone(),
            })
            .await?;
    }

    Ok(())
}

fn sub_dir<P>(path: P) -> crate::Result<Vec<OsString>>
where
    P: AsRef<Path> + std::convert::AsRef<OsStr>,
{
    let mut res = vec![];
    let path = Path::new(&path);

    for entry in path.read_dir().expect("failed to read_dir") {
        if let Ok(entry) = entry {
            if let Ok(file_type) = entry.file_type() {
                if file_type.is_dir() {
                    res.push(path.join(entry.path()).into_os_string());
                }
            }
        }
    }

    Ok(res)
}
