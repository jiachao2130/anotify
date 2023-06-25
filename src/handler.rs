use std::ffi::OsString;
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
struct Handler {
    handler_tx: Sender<OsString>,
    handler_rx: Receiver<OsString>,
    fliter_tx: Sender<Event>,
    err_rx: Receiver<crate::Error>
}

pub async fn run(anotify: Anotify, handler: Option<broadcast::Sender<Event>>) -> crate::Result<()> {
    let Anotify {
        mask,
        recursive,
        regex,
        targets,
    } = anotify;

    let (handler_tx, mut handler_rx) = mpsc::channel(1024);
    let (fliter_tx, fliter_rx) = mpsc::channel(1024);
    let (err_tx, mut err_rx) = mpsc::channel(1);
    let counter = Arc::new(Mutex::new(0));

    // 将初始路径添加到监视器中
    let tx = handler_tx.clone();
    tokio::spawn(async move {
        for target in targets {
            tx.send(target).await.unwrap();
        }
    });

    tokio::spawn(fliter(fliter_rx, mask.clone(), regex, handler));

    loop {
        tokio::select! {
            // 递归模式，添加新的监控路径
            Some(new_entry) = handler_rx.recv() => {
                let mut _counter = counter.lock().await;
                *_counter += 1;
                let handler = handler_tx.clone();
                let fliter = fliter_tx.clone();
                let err_tx = err_tx.clone();
                let counter = Arc::clone(&counter);
                tokio::spawn(async move {
                    match watcher::watch(new_entry, &mask, recursive, handler, fliter).await {
                        Ok(_) => {},
                        Err(e) => {
                            err_tx.send(Err(e)).await.unwrap();
                        }
                    }
                    let mut _counter = counter.lock().await;
                    *_counter -= 1;
                    if *_counter == 0 {
                        err_tx.send(Err("Error: All watches FD were removed.".into())).await.unwrap();
                    }
                });
            },
            Some(e) = err_rx.recv() => {
                return e
            },
            // ctrl + c 正常退出
            _ = tokio::signal::ctrl_c() => {
                return Ok(())
            }
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
        tokio::select! {
            // 过滤并处理 inotfiy 事件
            Some(event) = rx.recv() => {
                // regex 匹配过滤
                if re.is_some() && ! re.as_ref().unwrap().is_match(&event.path().as_path().to_str().unwrap()) {
                    continue
                }

                if handler.is_some() {
                    let _ = handler.as_ref().unwrap().send(event)?;
                    continue
                }

                let mask = mask & *event.mask();
                println!("{:?}: {}", mask, event.path().to_str().unwrap());
            },
            _ = tokio::signal::ctrl_c() => {
                return Ok(())
            }
        }
    }
}
