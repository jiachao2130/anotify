use std::ffi::{OsStr, OsString};
use std::future::Future;
use std::path::Path;

use inotify::EventMask;
use regex::Regex;
use tokio::sync::{broadcast, mpsc};

use crate::app::Anotify;
use crate::new_watcher::{Watcher, Event};

pub async fn run(
    anotify: Anotify,
    output: Option<broadcast::Sender<Event>>,
    shutdown: impl Future,
) -> crate::Result<()> {
    tokio::select! {
        Err(e) = handler(anotify, output) => Err(e),
        _ = shutdown => Ok(()),
    }
}

async fn handler(anotify: Anotify, output: Option<broadcast::Sender<Event>>) -> crate::Result<()> {
    let Anotify {
        mask,
        recursive,
        regex,
        targets,
    } = anotify;

    // init re mode
    let mut re = None;
    if let Some(regex) = regex {
        re = Some(Regex::new(&regex).unwrap());
    }

    let mut watcher = Watcher::init();
    let (handler_tx, mut handler_rx) = mpsc::channel(10);

    dispatch(targets, &handler_tx);

    loop {
        tokio::select!{
            // here handle the event
            Some(event) = watcher.next() => {
                // if recursive mode && found new dir
                if recursive
                    && !(*event.mask() & EventMask::CREATE).is_empty()
                    && !(*event.mask() & EventMask::ISDIR).is_empty()
                {
                    handler_tx.send(event.path().as_os_str().to_os_string()).await.unwrap();
                }

                // fliter
                if re.is_some()
                    && !re
                        .as_ref()
                        .unwrap()
                        .is_match(&event.path().to_str().unwrap()) 
                {
                    continue
                }

                // send event to output or 
                if output.is_some() {
                    let _ = output.as_ref().unwrap().send(event)?;
                } else {
                    println!("{:?}: {:?}", event.mask(), event.path());
                }
            },
            // add new watch task
            Some(target) = handler_rx.recv() => {
                watcher.add(&target, &mask)?;
                if recursive {
                    let targets = sub_dir(&target)?;

                    dispatch(targets, &handler_tx);
                }
            },
        }
    }
}

fn dispatch(targets: Vec<OsString>, tx: &mpsc::Sender<OsString>) {
    let tx = tx.clone();
    tokio::spawn(async move {
        for target in targets {
            tx.send(target).await.unwrap();
        }
    });
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
