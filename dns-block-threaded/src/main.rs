use std::{net::UdpSocket, sync::Arc};

use dns::resolver::{parse_query, resolve};

const DEFAULT_DNS: &str = "1.1.1.1";
const DEFAULT_PORT: &str = "53000";

fn main() {
    let dns = Arc::new(std::env::var("DNS").unwrap_or_else(|_| DEFAULT_DNS.into()));
    let port: u16 = std::env::var("PORT")
        .unwrap_or_else(|_| DEFAULT_PORT.into())
        .parse()
        .expect("Port must be a number");
    let socket = UdpSocket::bind(("127.0.0.1", port)).unwrap();

    println!("Started DNS blocker on 127.0.0.1::{}", port);

    let mut handles = vec![];

    for thread_id in 0..4 {
        let socket = socket.try_clone().unwrap();
        let dns = Arc::clone(&dns);
        let handle = std::thread::spawn(move || loop {
            let mut buf = [0; 512];
            let (_, sender) = socket.recv_from(&mut buf).unwrap();
            let (id, question) = parse_query(buf).unwrap();
            println!(
                "[Thread {}] Handling request for {:?}",
                thread_id, question.domain_name
            );
            match resolve(&question.domain_name, &dns, Some(id)) {
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
