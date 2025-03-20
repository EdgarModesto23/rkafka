use bytes::BytesMut;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

use crate::protocol::schema::requests::apiversions::ApiVersionRequest;
use crate::protocol::schema::requests::describetopic::DescribeTopicPartitions;
use crate::protocol::schema::Respond;
use crate::protocol::RequestBase;

pub enum Request {
    ApiVersions,
    DescribeTopicsPartitions,
    Unknown,
}

fn get_request(key: i16) -> Request {
    match key {
        18 => Request::ApiVersions,
        75 => Request::DescribeTopicsPartitions,
        _ => Request::Unknown,
    }
}

async fn respond(socket: &mut TcpStream, buf: &[u8]) {
    if let Err(e) = socket.write_all(buf).await {
        eprintln!("failed to write to socket; err = {e:?}");
        return;
    }
    let _ = socket.flush().await;
}

pub async fn dispatch_request(req: RequestBase, buf: &mut BytesMut, socket: &mut TcpStream) {
    let api_key = get_request(req.api_key);

    let past_base = req.base_size as usize;

    match api_key {
        Request::ApiVersions => {
            let api_versions = match ApiVersionRequest::new(req, &buf[past_base..]) {
                Ok(api_version) => api_version,
                Err(e) => {
                    eprintln!("Error while parsing api request: {e:?}");
                    return;
                }
            };
            let response = match api_versions.get_response() {
                Ok(val) => val,
                Err(e) => {
                    eprintln!("Error while parsing api request: {e:?}");
                    return;
                }
            };
            respond(socket, &response[..]).await;
        }
        Request::DescribeTopicsPartitions => {
            let describe_t_p = match DescribeTopicPartitions::new(req, &buf[past_base + 1..]) {
                Ok(request) => request,
                Err(e) => {
                    eprintln!("Error while parsing describe topics partitions: {e:?}");
                    return;
                }
            };
            let response = match describe_t_p.get_response() {
                Ok(val) => val,
                Err(e) => {
                    eprintln!("Error while parsing api request: {e:?}");
                    return;
                }
            };
            respond(socket, &response[..]).await;
        }
        Request::Unknown => {}
    }
}
