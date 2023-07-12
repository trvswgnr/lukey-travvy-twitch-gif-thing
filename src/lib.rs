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

#[derive(Debug, Clone, Copy)]
enum Route {
    Index,
    Hello,
    Send,
    NotFound,
}

impl fmt::Display for Route {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Route::Index => "index",
            Route::Hello => "hello",
            Route::Send => "send",
            Route::NotFound => "404",
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
            _ => Route::NotFound,
        }
    }
}

pub struct Request {
    method: Method,
    route: Route,
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

        Self { method, route }
    }
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
            _ => (Status::NotFound, Route::NotFound),
        };

        Self { status, route }
    }

    pub async fn to_string(&self) -> String {
        let path = format!("routes/{}.html", self.route);
        let body = fs::read_to_string(path)
            .await
            .unwrap_or_else(|_| String::from("Error reading file"));

        format!("{}{}", self.status, body)
    }
}
