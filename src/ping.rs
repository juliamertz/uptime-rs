use std::net::TcpStream;

fn ping(address: &str) -> PingResponse {
    let stream = TcpStream::connect(address).unwrap();
}
