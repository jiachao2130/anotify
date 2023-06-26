use std::ffi::OsString;
use std::future::Future;
use std::sync::Arc;

use regex::Regex;
use tokio::sync::Mutex;
use tokio::sync::{
    broadcast,
    mpsc::{self, Receiver, Sender},
};

use crate::app::Anotify;
use crate::watcher::{self, Event};
use crate::WatchMask;

/// channels collection, for handle func
#[derive(Debug)]
struct Handle {
    // send new path to watched
    pub handler_tx: Sender<OsString>,

    // handle new watch path
    pub handler_rx: Receiver<OsString>,

    // send watched event to fliter
    pub fliter_tx: Sender<Event>,

    // fileter: handle watched event
    pub fliter_rx: Receiver<Event>,

    // handler: handle err sended by watcher.
    pub err_tx: Sender<crate::Error>,

    // outside to handle events
    pub output: Option<broadcast::Sender<Event>>,
}

/// Start a inotify server to monitor files change.
/// When catch `shutdown` result, or some error were caused, quit.
///
/// You'd better define a `output` to recv all events, if not, the result would be print to stdout
/// like:
///
/// ```no_run
/// println!("{:?}: {:?}", event.mask(), event.path());
/// ```
pub async fn run(
    anotify: Anotify,
    output: Option<broadcast::Sender<Event>>,
    shutdown: impl Future,
) -> crate::Result<()> {
    // init all handle channels, all task communicate by them.
    let (handler_tx, handler_rx) = mpsc::channel(1024);
    let (fliter_tx, fliter_rx) = mpsc::channel(1024);
    let (err_tx, mut err_rx) = mpsc::channel(1);

    let handle = Handle {
        handler_tx,
        handler_rx,
        fliter_tx,
        fliter_rx,
        err_tx,
        output,
    };

    tokio::select! {
        Err(e) = handler(anotify, handle) => Err(e),
        Some(e) = err_rx.recv() => Err(e),
        _ = shutdown => Ok(()),
    }
}

async fn handler(anotify: Anotify, handle: Handle) -> crate::Result<()> {
    let Anotify {
        mask,
        recursive,
        regex,
        targets,
    } = anotify;
    let Handle {
        handler_tx,
        mut handler_rx,
        fliter_tx,
        fliter_rx,
        err_tx,
        output,
    } = handle;
    let counter = Arc::new(Mutex::new(0));

    let tx = handler_tx.clone();
    tokio::spawn(async move {
        for target in targets {
            tx.send(target).await.unwrap();
        }
    });

    tokio::spawn(fliter(fliter_rx, mask.clone(), regex, output));

    loop {
        if let Some(target) = handler_rx.recv().await {
            let handler = handler_tx.clone();
            let fliter = fliter_tx.clone();
            let err = err_tx.clone();
            let mut _counter = counter.lock().await;
            *_counter += 1;
            let counter = Arc::clone(&counter);
            tokio::spawn(async move {
                match watcher::watch(target, &mask, recursive, handler, fliter).await {
                    Ok(_) => {}
                    Err(e) => err.send(e).await.unwrap(),
                }

                let mut _counter = counter.lock().await;
                *_counter -= 1;
                if *_counter == 0 {
                    err.send("Error: All watches FD were removed.".into())
                        .await
                        .unwrap();
                }
            });
        }
    }
}

async fn fliter(
    mut rx: Receiver<Event>,
    mask: WatchMask,
    regex: Option<String>,
    handler: Option<broadcast::Sender<Event>>,
) -> crate::Result<()> {
    let mut re = None;
    if let Some(regex) = regex {
        re = Some(Regex::new(&regex).unwrap());
    }

    loop {
        // 过滤并处理 inotfiy 事件
        if let Some(event) = rx.recv().await {
            // regex 匹配过滤
            if re.is_some()
                && !re
                    .as_ref()
                    .unwrap()
                    .is_match(&event.path().as_path().to_str().unwrap())
            {
                continue;
            }

            if handler.is_some() {
                let _ = handler.as_ref().unwrap().send(event)?;
                continue;
            }

            let mask = mask & *event.mask();
            println!("{:?}: {:?}", mask, event.path());
        }
    }
}
