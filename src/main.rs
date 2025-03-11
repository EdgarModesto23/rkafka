use bytes::BytesMut;
use codecrafters_kafka::handler::dispatch_request;
use codecrafters_kafka::protocol::RequestBase;
use tokio::io::AsyncReadExt;
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
                    Ok(n) => n,
                    Err(e) => {
                        eprintln!("failed to read from socket; err = {e:?}");
                        return;
                    }
                };

                let base_request = if let Ok(val) = RequestBase::new(&buf) {
                    val
                } else {
                    eprintln!("Failed to parse request");
                    return;
                };

                dispatch_request(base_request, &mut buf, &mut socket).await;
            }
        });
    }
}
