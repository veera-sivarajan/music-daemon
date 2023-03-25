use std::fs::OpenOptions;

use inotify::{Inotify, WatchMask};

use chrono::prelude::*;
use daemonize::Daemonize;
use rusqlite::Connection;

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
        .add_watch("/home/veera/music", WatchMask::OPEN)
        .expect("Failed to add file watch");

    let database = Connection::open("/tmp/musicd-logs/history.sqlite")
        .expect("Unable to open database.");

    database
        .execute(
            "create table if not exists music_history (
             id integer primary key,
             title text not null,
             date text not null,
             time text not null
        )",
            [],
        )
        .expect("Unable to create table.");

    match daemonize.start() {
        Ok(_) => {
            loop {
                let mut buffer = [0; 1024];
                let events = inotify
                    .read_events_blocking(&mut buffer)
                    .expect("Error while reading events");

                for event in events {
                    // Handle event
                    let now = Local::now();
                    let date = now.format("%b %d, %Y");
                    let time = now.format("%H:%M");
                    if let Some(title) = event.name {
                        let title = title.to_str().unwrap().to_string();
                        println!("{date} -- {time} {title:?}");
                        database.execute(
                            "INSERT INTO music_history (title, date, time) values (?1, ?2, ?3)",
                            [&title, &date.to_string(), &time.to_string()]
                        ).expect("Unable to insert data in music_history database.");
                    }
                }
            }
        }
        Err(e) => eprintln!("Error, {e}"),
    }
}
