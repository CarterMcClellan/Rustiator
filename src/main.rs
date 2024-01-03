use anyhow::Result;
use clap::{Arg, Command}; // Note: It's `Command` in clap 3.x, not `App`
use tokio;
use server::http_server;
use server::browser::open_browser;

#[tokio::main]
async fn main() -> Result<()> {
    let matches = Command::new("Rustiator")
        .version("1.0")
        .author("Carter McClellan")
        .about("Starts a Rustiator server")
        .arg(Arg::new("hostname")
            .short('h')
            .long("hostname")
            .value_name("HOSTNAME")
            .help("Sets the hostname")
            .takes_value(true)
            .default_value("localhost"))
        .arg(Arg::new("port")
            .short('p')
            .long("port")
            .value_name("PORT")
            .help("Sets the port")
            .takes_value(true)
            .default_value("8080"))
        .get_matches();

    let hostname = matches.value_of("hostname").unwrap().to_string();
    let port = matches.value_of("port").unwrap().parse::<u16>().expect("Invalid port number");

    let server_future = http_server::start_server(hostname.clone(), port.clone());
    let open_browser_future = open_browser(format!("http://{}:{}", hostname, port));

    let (server_result, _browser_result) = tokio::join!(server_future, open_browser_future);

    // Handle the result of the HTTP server future
    if let Err(e) = server_result {
        eprintln!("Server encountered an error: {}", e);
        // Handle the error (e.g., retry, exit, etc.)
    }

    Ok(())
}
