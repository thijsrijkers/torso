use std::net::TcpListener;

pub fn setup_server() {
    let listener = TcpListener::bind("127.0.0.1:4978").unwrap();

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        println!("Connection established!");
    }
}
