use std::future::Future;

use inotify::EventMask;
use tokio::sync::{broadcast, mpsc};

use crate::app::Anotify;
use crate::new_watcher::{Watcher, Event};

pub async fn run(
    anotify: Anotify,
    output: Option<broadcast::Sender<Event>>,
    shutdown: impl Future,
) -> crate::Result<()> {
    tokio::select! {
        Err(e) = handler(anotify) => Err(e),
        _ = shutdown => Ok(()),
    }
}

async fn handler(anotify: Anotify) -> crate::Result<()> {
    let Anotify {
        mask,
        recursive,
        regex,
        targets,
    } = anotify;

    let mut watcher = Watcher::init();
    let (tx, mut rx) = mpsc::channel(1);

    loop {
        tokio::select!{
            // here handle the event
            Some(event) = watcher.next() => {
                println!("{:?}", event);
                // if recursive mode && found new dir
                if recursive
                    && !(*event.mask() & EventMask::CREATE).is_empty()
                    && !(*event.mask() & EventMask::ISDIR).is_empty()
                {
                    tx.send(event.path().as_os_str().to_os_string());
                }
            },
            // add new watch task
            Some(target) = rx.recv() => {
                watcher.add(target, &mask)?;
            },
        }
    }
    Ok(())
}
