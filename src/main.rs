#![deny(clippy::unwrap_used)]

use std::collections::HashMap;

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
};

use serde::{Deserialize, Serialize};

use lib::get_env_var;

const BUFFER_SIZE: usize = 16384;

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

                        let request_string = String::from_utf8_lossy(&buf[..n]);
                        let mut lines = request_string.lines();

                        let request_line = lines.next().unwrap_or("");

                        let (method, path, query, http_version) = {
                            let mut parts = request_line.split_whitespace();
                            let method = parts.next().unwrap_or("GET");
                            let path = parts.next().unwrap_or("/404");
                            let mut qparts = path.split('?');
                            let path = qparts.next().unwrap_or("");
                            let query = parse_query(qparts.next().unwrap_or(""));
                            let http_version = parts.next().unwrap_or("HTTP/1.1");
                            (method, path, query, http_version)
                        };

                        let response_string = match (method, path) {
                            ("GET", "/") => {
                                let content = include_str!("../routes/index.html");
                                html_response(200, content)
                            }
                            ("POST", "/search") => {
                                // skip headers and collect body all in one
                                let body: String = lines
                                    .skip_while(|line| !line.trim().is_empty())
                                    .skip(1) // skip the empty line between headers and body
                                    .collect();

                                let parsed_body = parse_query(&body);

                                let search_query = *parsed_body.get("search").unwrap_or(&"");

                                let api_key: String = get_env_var("GIPHY_KEY")
                                    .unwrap_or_else(|_| panic!("GIPHY_KEY not set"));

                                let api_url = format!("https://api.giphy.com/v1/gifs/search?api_key={}&q={}&limit=25&offset=0", api_key, search_query);

                                let giphy_response = reqwest::get(api_url)
                                    .await
                                    .expect("Failed to get response")
                                    .json::<GiphyResponse>()
                                    .await
                                    .expect("Failed to parse JSON");

                                let gif_objs = giphy_response.data;

                                let gif_cards = gif_objs
                                    .iter()
                                    .map(|gif| giphy_card(&gif.images.downsized.url))
                                    .collect::<String>();

                                html_response(200, &gif_cards)
                            }
                            _ => html_response(404, "<h1>WHOA WTF WHERE AM I????</h1>"),
                        };

                        println!("{}", request_string);

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

// search=asdfa&test=dddfasdfasd
fn parse_query(body: &str) -> HashMap<&str, &str> {
    let mut query = HashMap::new();

    for pair in body.split('&') {
        let mut key_value = pair.split('=');
        let key = key_value.next().unwrap_or("");
        let value = key_value.next().unwrap_or("");
        query.insert(key, value);
    }

    query
}

fn html_response(status: u16, content: &str) -> String {
    format!(
        "HTTP/1.1 {}\r\nContent-Type: text/html\r\n\r\n{}",
        status, content
    )
}
