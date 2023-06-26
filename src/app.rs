use std::ffi::OsString;

use clap::{Arg, ArgAction, Command};
use inotify::WatchMask;

/// Anotify
/// todo
pub struct Anotify {
    /// inotify::WatchMask, the event mask to watched
    pub mask: WatchMask,

    /// the regex to fliter file name
    pub regex: Option<String>,

    /// does it watch sub dir
    pub recursive: bool,

    /// all pathes to be watched
    pub targets: Vec<OsString>,
}

fn app() -> Command {
    let cmd = Command::new("anotify")
        .author("CC")
        .arg(
            Arg::new("access")
                .long("access")
                .short('a')
                .action(ArgAction::SetTrue)
                .help("File was accessed"),
        )
        .arg(
            Arg::new("attrib")
                .long("attrib")
                .short('A')
                .action(ArgAction::SetTrue)
                .help("Metadata (permissions, timestamps, …) changed"),
        )
        .arg(
            Arg::new("close_write")
                .long("close_write")
                .short('x')
                .action(ArgAction::SetTrue)
                .help("File opened for writing was closed"),
        )
        .arg(
            Arg::new("close_nowrite")
                .long("close_nowrite")
                .short('q')
                .action(ArgAction::SetTrue)
                .help("File or directory not opened for writing was closed"),
        )
        .arg(
            Arg::new("create")
                .long("create")
                .short('c')
                .action(ArgAction::SetTrue)
                .help("File/directory created in watched directory"),
        )
        .arg(
            Arg::new("delete")
                .long("delete")
                .short('d')
                .action(ArgAction::SetTrue)
                .help("File/directory deleted from watched directory"),
        )
        .arg(
            Arg::new("delete_self")
                .long("delete_self")
                .short('D')
                .action(ArgAction::SetTrue)
                .help("Watched file/directory was deleted"),
        )
        .arg(
            Arg::new("modify")
                .long("modify")
                .short('m')
                .action(ArgAction::SetTrue)
                .help("File was modified"),
        )
        .arg(
            Arg::new("move_self")
                .long("move_self")
                .short('S')
                .action(ArgAction::SetTrue)
                .help("Watched file/directory was moved"),
        )
        .arg(
            Arg::new("moved_from")
                .long("moved_from")
                .short('F')
                .action(ArgAction::SetTrue)
                .help("File was renamed/moved; watched directory contained old name"),
        )
        .arg(
            Arg::new("moved_to")
                .long("moved_to")
                .short('T')
                .action(ArgAction::SetTrue)
                .help("File was renamed/moved; watched directory contains new name"),
        )
        .arg(
            Arg::new("open")
                .long("open")
                .short('o')
                .action(ArgAction::SetTrue)
                .help("File or directory was opened"),
        )
        .arg(
            Arg::new("move")
                .long("move")
                .short('M')
                .action(ArgAction::SetTrue)
                .help("Watch for all move events"),
        )
        .arg(
            Arg::new("close")
                .long("close")
                .short('Q')
                .action(ArgAction::SetTrue)
                .help("Watch for all close events"),
        )
        .arg(
            Arg::new("all")
                .long("all")
                .action(ArgAction::SetTrue)
                .help("Watch for all events"),
        )
        .arg(
            Arg::new("dont_follow")
                .long("dont_follow")
                .action(ArgAction::SetTrue)
                .help("Don’t dereference the path if it is a symbolic link"),
        )
        .arg(
            Arg::new("excl_unlink")
                .long("excl_unlink")
                .action(ArgAction::SetTrue)
                .help("Filter events for directory entries that have been unlinked"),
        )
        .arg(
            Arg::new("mask_add")
                .long("mask_add")
                .action(ArgAction::SetTrue)
                .help("If a watch for the inode exists, amend it instead of replacing it"),
        )
        .arg(
            Arg::new("oneshot")
                .long("oneshot")
                .action(ArgAction::SetTrue)
                .help("Only receive one event, then remove the watch"),
        )
        .arg(
            Arg::new("dir")
                .long("dir")
                .action(ArgAction::SetTrue)
                .help("Only watch path, if it is a directory"),
        )
        .arg(
            Arg::new("recursive")
                .short('R')
                .long("recursive")
                .action(ArgAction::SetTrue)
                .help("Recursive monitor a path"),
        )
        .arg(
            Arg::new("regex")
                .short('E')
                .long("regex")
                .action(ArgAction::Set)
                .help("Use regex to match file name, only matched will report"),
        )
        .arg(
            Arg::new("target")
                .action(ArgAction::Append)
                .default_value("./"),
        );

    cmd
}

pub fn parse() -> crate::Result<Anotify> {
    let _args = app().get_matches();
    let mut mask = WatchMask::empty();

    // access
    if *_args.get_one::<bool>("access").unwrap() {
        mask |= WatchMask::ACCESS;
    }

    // attrib
    if *_args.get_one::<bool>("attrib").unwrap() {
        mask |= WatchMask::ATTRIB;
    }

    // close write
    if *_args.get_one::<bool>("close_write").unwrap() {
        mask |= WatchMask::CLOSE_WRITE;
    }

    // close nowrite
    if *_args.get_one::<bool>("close_nowrite").unwrap() {
        mask |= WatchMask::CLOSE_NOWRITE;
    }

    // create
    if *_args.get_one::<bool>("create").unwrap() {
        mask |= WatchMask::CREATE;
    }

    // delete
    if *_args.get_one::<bool>("delete").unwrap() {
        mask |= WatchMask::DELETE;
    }

    // delete self
    if *_args.get_one::<bool>("delete_self").unwrap() {
        mask |= WatchMask::DELETE_SELF;
    }

    // modify
    if *_args.get_one::<bool>("modify").unwrap() {
        mask |= WatchMask::MODIFY;
    }

    // move self
    if *_args.get_one::<bool>("move_self").unwrap() {
        mask |= WatchMask::MOVE_SELF;
    }

    // move from
    if *_args.get_one::<bool>("moved_from").unwrap() {
        mask |= WatchMask::MOVED_FROM;
    }

    // move to
    if *_args.get_one::<bool>("moved_to").unwrap() {
        mask |= WatchMask::MOVED_TO;
    }

    // open
    if *_args.get_one::<bool>("open").unwrap() {
        mask |= WatchMask::OPEN;
    }

    // all events
    if *_args.get_one::<bool>("all").unwrap() {
        mask |= WatchMask::ALL_EVENTS;
    }

    // move events
    if *_args.get_one::<bool>("move").unwrap() {
        mask |= WatchMask::MOVE;
    }

    // move events
    if *_args.get_one::<bool>("close").unwrap() {
        mask |= WatchMask::CLOSE;
    }

    // don't follow events
    if *_args.get_one::<bool>("dont_follow").unwrap() {
        mask |= WatchMask::DONT_FOLLOW;
    }

    // excl unlink
    if *_args.get_one::<bool>("excl_unlink").unwrap() {
        mask |= WatchMask::EXCL_UNLINK;
    }

    // mask add
    if *_args.get_one::<bool>("mask_add").unwrap() {
        mask |= WatchMask::MASK_ADD;
    }

    // oneshot
    if *_args.get_one::<bool>("oneshot").unwrap() {
        mask |= WatchMask::ONESHOT;
    }

    // only dir
    if *_args.get_one::<bool>("dir").unwrap() {
        mask |= WatchMask::ONLYDIR;
    }

    if mask.is_empty() {
        return Err("Error: You must point at least one EVENT".into());
    }

    let mut recursive = false;
    if *_args.get_one::<bool>("recursive").unwrap() {
        recursive = true;
    }

    let mut regex = None;
    if let Some(_regex) = _args.get_one::<String>("regex") {
        regex = Some(String::from(_regex));
    }

    let mut targets: Vec<OsString> = vec![];
    let mut target = _args.get_many::<String>("target").unwrap();
    while let Some(_target) = target.next() {
        targets.push(_target.into());
    }

    Ok(Anotify {
        mask,
        recursive,
        regex,
        targets,
    })
}
