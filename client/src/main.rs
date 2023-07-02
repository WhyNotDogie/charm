use std::{io, env};

use packetz::client;

#[tokio::main]
async fn main() -> Result<(), io::Error> {
    let client = client::connect(env::args().nth(1).expect("Please provide a dns name and port. eg: localhost:5515")).await?;
    let (mut receive, mut send) = client.split();
    let receive_task: tokio::task::JoinHandle<Result<(), io::Error>> = tokio::spawn(async move {
        loop {
            let msg = receive.recv().await?;
            println!("{}", String::from_utf8_lossy(msg.body.as_slice()));
        }
    });
    let send_task: tokio::task::JoinHandle<Result<(), io::Error>> = tokio::spawn(async move {
        loop {
            let msg: String = text_io::read!("{}\n");
            print!("\x1b[1A\x1b[K");
            send.send(msg).await?;
        }
    });
    let check_task1: tokio::task::JoinHandle<Result<(), io::Error>> = tokio::spawn(async move {
        receive_task.await??;
        Ok(())
    });
    let check_task2: tokio::task::JoinHandle<Result<(), io::Error>> = tokio::spawn(async move {
        send_task.await??;
        Ok(())
    });
    let j = tokio::join!(check_task1, check_task2);
    j.0??;
    j.1??;
    Ok(())
}