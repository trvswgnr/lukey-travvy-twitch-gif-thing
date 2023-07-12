use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
};

use lib::{Request, Response};

const BUFFER_SIZE: usize = 1024;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:6969").await?;
    loop {
        let (mut stream, mut _address) = listener.accept().await?;
        tokio::spawn(async move {
            let mut buf = [0; BUFFER_SIZE];
            loop {
                match stream.read(&mut buf).await {
                    Ok(n) => {
                        if n == 0 {
                            return;
                        }

                        let request_str = String::from_utf8_lossy(&buf[..n]);
                        let request = Request::from(request_str.as_ref());
                        let response = Response::from(request).to_string().await;

                        if let Err(e) = stream.write_all(response.as_bytes()).await {
                            eprintln!("failed to write to socket; err = {:?}", e);
                            return;
                        }

                        // * close the connection
                        if let Err(e) = stream.shutdown().await {
                            eprintln!("failed to close socket; err = {:?}", e);
                            return;
                        }
                    }

                    Err(e) => {
                        eprintln!("failed to read from socket; err = {:?}", e);
                        return;
                    }
                }
            }
        });
    }
}
