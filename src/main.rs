mod logger;
mod platform;
mod tracker;

use logger::Logger;
use std::{thread, time::Duration};
use tracker::Tracker;

fn main() ->std::io::Result<()> {
    let mut tracker = Tracker::new();
    let mut logger = Logger::new("log.txt")?;
    
    println!("Winrak has started.");
    println!("Writing sessions to log.txt");
    println!("Press Ctrl+C to stop.\n");

    loop {
        let app = platform::get_active_app();

        if let Some(session) = tracker.update(app){
            println!(
                "{} | {} - {} | {} seconds",
                session.app_name,
                session.started_at.format("%H:%M:%S"),
                session.ended_at.format("%H:%M:%S"),
                session.duration.as_secs(),
            );

            if let Err(error) = logger.log_session(&session) {
                eprintln!("Failed to write log entry: {error}");
            }
        }

        thread::sleep(Duration::from_secs(1));
    }
}
