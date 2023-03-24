use std::fs::OpenOptions;

use inotify::{
    Inotify,
    WatchMask,
};

use daemonize::Daemonize;
use chrono::prelude::*;

fn main() {
    let stdout = OpenOptions::new()
        .append(true)
        .open("/tmp/musicd-logs/musicd.out")
        .expect("Cannot open stdout file.");

    let stderr = OpenOptions::new()
        .append(true)
        .open("/tmp/musicd-logs/musicd.err")
        .expect("Cannot open stderr file.");

    let daemonize = Daemonize::new()
        .working_directory("/tmp/musicd-logs/") 
        .stdout(stdout)  
        .stderr(stderr); 

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
