use serde::{Deserialize, Serialize};
use std::net::{TcpListener, TcpStream};
use std::thread;
use tungstenite::accept;

// The default server address.
const SERVER_ADDRESS: &str = "localhost";
// The default port on which the web socket server will be listening on.
const SERVER_PORT: &str = "27745";

fn main() {
    let address: String = format!("{}:{}", SERVER_ADDRESS, SERVER_PORT);

    let server = match TcpListener::bind(&address) {
        Ok(server) => server,
        Err(msg) => {
            println!("Unable to bind to the '{}' address and port '{}': {}", SERVER_ADDRESS, SERVER_PORT, msg);
            std::process::exit(1);
        }
    };

    println!("Server started at: {}", address);

    for stream in server.incoming() {
        let stream = match stream {
            Ok(stream) => stream,
            Err(msg) => {
                println!("Unable to handle the incoming TCP stream: {}", msg);
                continue
            }
        };

        thread::spawn(move || handle_client(stream));
    }
}

// Handles the incoming connection. It tries to grab the peer address and listens to incoming
// messages. On any error the function returns and closes the connection.
fn handle_client(stream: TcpStream) {
    let peer_addr = match stream.peer_addr() {
        Ok(pa) => pa,
        Err(msg) => {
            println!("Unable to retrieve the peer address: {}", msg);
            return;
        }
    };

    let mut websocket = match accept(stream) {
        Ok(ws) => ws,
        Err(msg) => {
            println!("Unable to accept connection: {}", msg);
            return;
        }
    };

    println!("Connected peer: {}", peer_addr);

    loop {
        let msg = match websocket.read_message() {
            Ok(msg) => msg,
            Err(err) => match err {
                tungstenite::error::Error::ConnectionClosed => break,
                other => {
                    println!("Error when receiving the message: {:?}", other);
                    break;
                }
            },
        };

        // Parse the incoming message into a more manageable struct.
        let msg_string = msg.to_string();

        let response_data: ResponseData = match serde_json::from_str(&msg_string) {
            Ok(parsed_msg) => parsed_msg,
            Err(err) => {
                println!("Unable to parse incoming message: {}", err);
                continue;
            }
        };

        println!("Current time: {}", response_data.current_time);
        println!("Playlist contents: {}", response_data.playlist_contents);
    }
}

#[derive(Serialize, Deserialize)]
struct ResponseData {
    #[serde(alias = "currentTime")]
    current_time: String,
    #[serde(alias = "playlistContents")]
    playlist_contents: String,
}
