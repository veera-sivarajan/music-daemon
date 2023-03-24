use std::fs::File;

use inotify::{
    Inotify,
    WatchMask,
};

use daemonize::Daemonize;
use chrono::prelude::*;

fn main() {
    let stdout = File::create("/tmp/musicd-logs/musicd.out").unwrap();
    let stderr = File::create("/tmp/musicd-logs/musicd.err").unwrap();

    let daemonize = Daemonize::new()
        .pid_file("/tmp/musicd-logs/musicd.pid") // Every method except `new` and `start`
        .working_directory("/tmp/musicd-logs/") // for default behaviour.
        .umask(0o777)    // Set umask, `0o027` by default.
        .stdout(stdout)  // Redirect stdout to `/tmp/daemon.out`.
        .stderr(stderr);  // Redirect stderr to `/tmp/daemon.err`.

    
    let mut inotify = Inotify::init()
        .expect("Error while initializing inotify instance");

    // Watch for modify and close events.
    inotify
        .add_watch(
            "/home/veera/music",
            WatchMask::OPEN
        )
        .expect("Failed to add file watch");

    match daemonize.start() {
        Ok(_) =>  {
            loop {
                let mut buffer = [0; 1024];
                let events = inotify.read_events_blocking(&mut buffer)
                    .expect("Error while reading events");

                for event in events {
                    // Handle event
                    let now = Local::now();
                    let now_str = now.format("%b %d, %Y -- %H:%M");
                    if let Some(title) = event.name {
                        println!("{now_str} {title:?}");
                    }
                }
            }
        }
        Err(e) => eprintln!("Error, {e}"),
    }
}
