use reqwest::{get, Body, Client, Url};
use serde::{Deserialize, Serialize};
use serde_json::from_str;
use serde_json::Value;
use std::fmt;
use tokio::fs;

enum Status {
    Ok,
    NotFound,
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let status = match self {
            Status::Ok => "200 OK",
            Status::NotFound => "404 NOT FOUND",
        };

        write!(f, "HTTP/1.1 {status}\r\nContent-Type: text/html\r\n\r\n")
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Route {
    Index,
    Hello,
    Send,
    NotFound,
    Search,
}

impl fmt::Display for Route {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Route::Index => "index",
            Route::Hello => "hello",
            Route::Send => "send",
            Route::NotFound => "404",
            Route::Search => "search",
        };

        write!(f, "{}", name)
    }
}

enum Method {
    Get,
    Post,
}

impl From<&str> for Method {
    fn from(method: &str) -> Self {
        match method {
            "GET" => Method::Get,
            "POST" => Method::Post,
            _ => panic!("Method not supported"),
        }
    }
}

impl From<&str> for Route {
    fn from(route: &str) -> Self {
        match route {
            "/" => Route::Index,
            "/hello" => Route::Hello,
            "/send" => Route::Send,
            "/search" => Route::Search,
            _ => Route::NotFound,
        }
    }
}

pub struct Request {
    method: Method,
    route: Route,
    // body: Option<Body>,
}

impl From<&str> for Request {
    fn from(request: &str) -> Self {
        println!("Request: {}", request);
        let mut lines = request.lines();

        // first line is the request line
        let line = lines.next().unwrap();
        let mut parts = line.split_whitespace();
        let method = Method::from(parts.next().unwrap());
        let route = Route::from(parts.next().unwrap());

        Self {
            method,
            route,
            // body: Some(Body::from(body)),
        }
    }
}

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

pub struct Response {
    status: Status,
    route: Route,
}

impl Response {
    pub fn from(request: Request) -> Self {
        let (status, route) = match (request.method, request.route) {
            (Method::Get, Route::Index) => (Status::Ok, request.route),
            (Method::Get, Route::Hello) => (Status::Ok, request.route),
            (Method::Post, Route::Send) => {
                // we could rate limit or something here
                (Status::Ok, request.route)
            }
            (Method::Post, Route::Search) => (Status::Ok, request.route),
            (Method::Get, Route::Search) => (Status::Ok, request.route),
            _ => (Status::NotFound, Route::NotFound),
        };

        Self { status, route }
    }

    pub async fn to_string(&self) -> Result<String, Box<dyn std::error::Error>> {
        if self.route == Route::Search {
            let client = Client::new();
            let api_key: String = get_env_var("GIPHY_KEY").unwrap();
            let endpoint = format!(
                "https://api.giphy.com/v1/gifs/search?api_key={api_key}&q=pog&limit=25&offset=0"
            );

            let giphy_response = reqwest::get(&endpoint)
                .await?
                .json::<GiphyResponse>()
                .await?;

            let gif_objs = giphy_response.data;

            for gif in gif_objs {
                let gif_url = gif.images.downsized.url;

                println!("URL: {}", gif_url);
            }

            return Ok(format!("{}{}", self.status, "body"));
        }
        let path = format!("routes/{}.html", self.route);
        let body = fs::read_to_string(path)
            .await
            .unwrap_or_else(|_| String::from("Error reading file"));

        Ok(format!("{}{}", self.status, body))
    }
}

use dotenv::dotenv;
use std::{env, error::Error, str::FromStr, sync::Once};

static INIT: Once = Once::new();

pub fn get_env_var<T: FromStr>(key: &str) -> Result<T, Box<dyn Error>> {
    INIT.call_once(|| match dotenv().ok() {
        Some(_) => println!(".env file detected, loading..."),
        None => println!("No .env file found."),
    });

    match env::var(key) {
        Ok(val) => {
            let parsed: Result<T, Box<dyn Error>> = match val.parse::<T>() {
                Ok(parsed) => Ok(parsed),
                Err(_) => Err(format!("Failed to parse {key}").into()),
            };

            parsed
        }
        Err(_) => Err(format!("{key} not set").into()),
    }
}
