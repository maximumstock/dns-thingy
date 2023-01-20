use std::{net::UdpSocket, sync::Arc};

use dns::resolver::resolve_pipe;

const DEFAULT_DNS: &str = "1.1.1.1";
const DEFAULT_PORT: &str = "53000";

fn main() {
    let dns = Arc::new(std::env::var("DNS").unwrap_or_else(|_| DEFAULT_DNS.into()));
    let port: u16 = std::env::var("PORT")
        .unwrap_or_else(|_| DEFAULT_PORT.into())
        .parse()
        .expect("Port must be a number");
    let internal_socket = UdpSocket::bind(("0.0.0.0", port)).unwrap();
    let external_socket = UdpSocket::bind(("0.0.0.0", 0)).unwrap();

    println!("Started DNS blocker on 127.0.0.1::{}", port);

    let mut handles = vec![];

    for _ in 0..4 {
        let socket = internal_socket.try_clone().unwrap();
        let external_socket = external_socket.try_clone().unwrap();
        let dns = Arc::clone(&dns);
        let handle = std::thread::spawn(move || loop {
            let mut incoming_query = [0; 512];
            let (_, sender) = socket.recv_from(&mut incoming_query).unwrap();
            match resolve_pipe(
                &incoming_query,
                &dns,
                Some(external_socket.try_clone().unwrap()),
            ) {
                Ok((_, reply)) => {
                    socket.send_to(&reply, sender).unwrap();
                }
                Err(e) => {
                    dbg!(e);
                }
            }
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }
}
