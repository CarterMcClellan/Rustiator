use std::{thread, time::Duration};

pub async fn open_browser(url: String) {
    thread::sleep(Duration::from_secs(1));
    if let Err(e) = webbrowser::open(&url) {
        log::error!("Failed to open web browser: {e}");
    }
}
