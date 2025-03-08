use bytes::BytesMut;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

static SERVER_ADDRESS: &str = "127.0.0.1:9092";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind(SERVER_ADDRESS).await?;
    println!("Starting server at {SERVER_ADDRESS}");

    loop {
        let (mut socket, _) = listener.accept().await?;

        tokio::spawn(async move {
            let mut buf = BytesMut::with_capacity(1024);

            loop {
                buf.resize(buf.capacity(), 0);
                let _n = match socket.read(&mut buf).await {
                    Ok(0) => {
                        println!("Connection closed by client.");
                        return;
                    }
                    Ok(n) => {
                        println!("Read {n} bytes from the socket");
                        n
                    }
                    Err(e) => {
                        eprintln!("failed to read from socket; err = {e:?}");
                        return;
                    }
                };

                let mut res = BytesMut::with_capacity(8);

                res.extend_from_slice(&[0, 0, 0, 0, 0, 0, 0, 7]);

                if let Err(e) = socket.write_all(&res[..]).await {
                    eprintln!("failed to write to socket; err = {e:?}");
                    break;
                }
                let _ = socket.flush().await;
            }
        });
    }
}
