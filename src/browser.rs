use std::{thread, time::Duration};

pub async fn open_browser(url: String) {
    thread::sleep(Duration::from_secs(1));
    if webbrowser::open(&url).is_err() {
        eprintln!("Failed to open web browser");
    }
}