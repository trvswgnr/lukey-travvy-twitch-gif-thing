use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
};

use serde::{Deserialize, Serialize};

use lib::{get_env_var, Request, Response};

const BUFFER_SIZE: usize = 1024;

#[derive(Debug, Deserialize, Serialize)]
pub struct GiphyResponse {
    data: Vec<Gif>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Gif {
    images: Images,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Images {
    downsized: Downsized,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Downsized {
    url: String,
}

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

                        let mut response_string = "".to_string();

                        let request_string = String::from_utf8_lossy(&buf[..n]);
                        let mut lines = request_string.lines();

                        let request_line = lines.next().unwrap();

                        if request_line == "GET / HTTP/1.1" {
                            let content = include_str!("../routes/index.html");
                            response_string = format!(
                                "HTTP/1.1 {}\r\nContent-Type: text/html\r\n\r\n{}",
                                200, content
                            );
                        }

                        if request_line == "POST /search HTTP/1.1" {
                            for line in lines.by_ref() {
                                if line.trim().is_empty() {
                                    break;
                                }
                            }

                            let body: String = lines.collect();

                            let search_query = body.split('=').last().unwrap();

                            let api_key: String = get_env_var("GIPHY_KEY").unwrap();

                            let endpoint = format!("https://api.giphy.com/v1/gifs/search?api_key={api_key}&q={search_query}&limit=25&offset=0");

                            let giphy_response = reqwest::get(endpoint)
                                .await
                                .unwrap()
                                .json::<GiphyResponse>()
                                .await
                                .unwrap();
                            let gif_objs = giphy_response.data;

                            let mut gif_cards = String::new();

                            for gif in gif_objs {
                                let gif_url = gif.images.downsized.url;

                                gif_cards.push_str(&giphy_card(&gif_url));

                                println!("URL: {}", gif_url);
                            }

                            response_string = format!(
                                "HTTP/1.1 {}\r\nContent-Type: text/html\r\n\r\n{}",
                                200, gif_cards
                            );
                            println!("search query = {search_query}");
                        }

                        // println!("{request_string}");

                        // let lines = request_string.line();
                        // let req_header = lines.next().unwrap();

                        if let Err(e) = stream.write_all(response_string.as_bytes()).await {
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

const GIPHY_CARD_PARTIAL: &str = include_str!("../partials/giphy-card.html");
fn giphy_card(src: &str) -> String {
    GIPHY_CARD_PARTIAL.replace("{src}", src)
}
