use std::{
    env,
    io::{BufReader, Write},
    net::{Shutdown, TcpListener, TcpStream},
};

use resp::Decoder;
mod commands;
use commands::process_client_request;

fn main() {
    //01.  get tcp address from std::env
    let address = env::args()
        .skip(1)
        .next()
        .unwrap_or("127.0.0.1:6379".to_owned());
    // 02. create listner on that address
    let listener = TcpListener::bind(&address).unwrap();
    println!("listening in {}", address);
    // 03. listen to incoming request
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        println!("Request recived: {:?}", stream);
        handle_client(stream);
    }
}

fn handle_client(stream: TcpStream) {
    // 01. parse stream_buffer as Decoder
    let mut stream_buffer = BufReader::new(stream);
    let decoder = Decoder::new(&mut stream_buffer).decode(); // Ok(RESP)
                                                             // 02. return Decoded reply value on success
    match decoder {
        Ok(value) => {
            // 03. If RESP value found, we process and send a reply stream back to client
            let reply = process_client_request(value);
            stream_buffer.get_mut().write_all(&reply).unwrap()
        }
        Err(value) => {
            println!("invalid command {:?}", value);
            let _ = stream_buffer.get_mut().shutdown(Shutdown::Both); //shutdown() closes the socket
        }
    }
}
