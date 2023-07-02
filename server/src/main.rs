use std::{io, sync::Arc, net::SocketAddr, env};
use tokio::{sync::mpsc, net};

use packetz::{server::*, packet::PacketWrite};
use crossbeam_queue::SegQueue;

pub struct Client {
    ip: SocketAddr,
    connection: PacketWrite<net::TcpStream>
}

pub struct Message {
    server: bool,
    ip: Option<SocketAddr>,
    contents: Vec<u8>
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let server: Server<String> = Server::bind(env::args().nth(1).expect("Please provide a dns name and port. eg: localhost:5515")); // The extra &str is the type of the argumet in `server::bind()`, so that server::bind is able to be a const fn.
    let listener: ServerListener = server.listen().await?;

    let q: Arc<SegQueue<Message>> = Arc::new(SegQueue::new());
    let (tx_clients, mut rx_clients) = mpsc::unbounded_channel();

    let q2 = Arc::clone(&q);
    let _: tokio::task::JoinHandle<Result<(), io::Error>> = tokio::spawn(async move {  
        let q = q2;  
        loop {
            let x = listener.accept().await;
            match x {
                Ok((connection, ip)) => {
                    let q = Arc::clone(&q);
                    let tx_client = tx_clients.clone();
                    let (mut connection_read, connection_write)= connection.split();
                    
                    if tx_client.send(Client {
                        ip,
                        connection: connection_write
                    }).is_ok() {
                        tokio::spawn(async move {
                            'l: loop {
                                let msg = connection_read.recv().await;
                                if msg.is_ok() {
                                    let msg = msg.unwrap();
                                    println!("[{}]: {}", ip, String::from_utf8_lossy(msg.body.as_slice()));
                                    q.push(Message {
                                        contents: msg.body,
                                        ip: Some(ip),
                                        server: false
                                    });
                                } else {
                                    break 'l
                                }
                            }
                            Ok::<(), io::Error>(())
                        });
                    }
                }
                Err(e) => {
                    eprintln!("{:?}", e)
                }
            }
        }
    });

    let clients = Arc::new(SegQueue::new());


    let clients2 = Arc::clone(&clients);
    let q2 = Arc::clone(&q);
    tokio::task::spawn(async move {
        'l: loop {
            match rx_clients.recv().await {
                Some(v) => {
                    println!("[SERVER]: {} just connected!", v.ip);
                    q2.push(Message { server: true, ip: None, contents: format!("{} just connected!", v.ip).into_bytes() });
                    clients2.push(v);
                }
                None => {
                    break 'l;
                }
            }
        }
    });
    
    loop {
        if let Some(v) = q.pop() {
            
            for _ in 0..clients.len() {
                (async {
                    let mut c = clients.pop().unwrap();
                    if v.server {
                        if let Err(_) = c.connection.send(format!("[SERVER]: {}", String::from_utf8_lossy(v.contents.as_slice()))).await {
                            return;
                        };
                    } else {
                        if let Err(_) = c.connection.send(format!("[{}]: {}", v.ip.unwrap(), String::from_utf8_lossy(v.contents.as_slice()))).await {
                            return;
                        };
                    }
                    clients.push(c);
                }).await
            }
        }
    }

    // Ok(())
}