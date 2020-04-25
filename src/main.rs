use crate::net::HttpServer;
use std::time::Duration;
use crate::http::{HttpResponse, Method, HttpRequest};
use crate::http::response::html;

mod http;
mod net;
mod parser;

fn main() {
    let mut server = HttpServer::new();
    server.register_default(handle_default);
    server.register_handler(Method::Get, "/test".to_string(), |_| html("custom!".to_string()));
    server.run("127.0.0.1:5000".parse().unwrap()).unwrap();
}

fn handle_default(req: &HttpRequest) -> HttpResponse {
    let mut buf = String::with_capacity(2048);
    buf.push_str(r"<!DOCTYPE html>
<html lang='en'>
    <head>
        <title>Amazing site!</title>
        <meta charset='utf-8'>
    </head>
    <body>
        <h1>");
    buf.push_str(req.path.as_str());
    buf.push_str(r"</h1>
        <table border='1'>");
    for header in &req.headers {
        buf.push_str("<tr><th>");
        buf.push_str(header.0.as_str());
        buf.push_str("</th><th>");
        buf.push_str(header.1.as_str());
        buf.push_str("</th></tr>");
    }
    buf.push_str(r"        </table>
    </body>
</html>");
    html(buf)
}
