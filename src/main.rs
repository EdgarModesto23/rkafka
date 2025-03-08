use bytes::BytesMut;
use codecrafters_kafka::protocol::{self, RequestBase, ResponseBase};
use codecrafters_kafka::rpc::encode::Encode;
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

                let mut res_buf = BytesMut::new();

                let res = ResponseBase::new(0, base_request.correlation_id);

                res.encode(&mut res_buf);

                println!("{res_buf:?}");

                if let Err(e) = socket.write_all(&res_buf).await {
                    eprintln!("failed to write to socket; err = {e:?}");
                    break;
                }
                let _ = socket.flush().await;
            }
        });
    }
}
