use inotify::WatchMask;
use regex::Regex;
use tokio::sync::mpsc::{self, Receiver};

use crate::app;
use crate::watcher::{self, Event};

pub async fn run() -> crate::Result<()> {
    let app::Anotify {
        mask,
        recursive,
        regex,
        targets,
    } = app::parse()?;

    let (handler_tx, mut handler_rx) = mpsc::channel(1024);
    let (fliter_tx, fliter_rx) = mpsc::channel(1024);
    let (err_tx, mut err_rx) = mpsc::channel(1);

    // 将初始路径添加到监视器中
    let tx = handler_tx.clone();
    tokio::spawn(async move {
        for target in targets {
            tx.send(target).await.unwrap();
        }
    });

    tokio::spawn(fliter(fliter_rx, mask.clone(), regex));

    loop {
        tokio::select! {
            // 递归模式，添加新的监控路径
            Some(new_entry) = handler_rx.recv() => {
                let handler = handler_tx.clone();
                let fliter = fliter_tx.clone();
                let err_tx = err_tx.clone();
                tokio::spawn(async move {
                    if let Err(e) = watcher::watch(new_entry, &mask, recursive, handler, fliter).await {
                        err_tx.send(Err(e)).await.unwrap();
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
                if let Some(re) = re.as_ref() {
                    if ! re.is_match(&event.path().as_path().to_str().unwrap()) {
                        break;
                    }
                }

                let mask = mask & *event.mask();
                println!("{:?}: {}", mask, event.path().to_str().unwrap());
            },
            _ = tokio::signal::ctrl_c() => {
                return Ok(())
            }
        }
    }
    Ok(())
}
