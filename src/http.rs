use std::collections::HashMap;
use std::hash::Hash;
use std::ops::Deref;

pub const MAX_CONTENT_SIZE: usize = 65535;

pub struct HttpRequest {
    pub method: Method,
    pub path: String,
    pub protocol: String,
    pub version: String,
    pub headers: HashMap<String, String>,
}

pub struct HttpResponse {
    pub buffer: [u8; 65535],
    pub len: usize,
    pub header: Vec<(String, String)>,
    pub code: ResponseCode,
}

#[derive(Hash, Eq, PartialEq, Clone)]
pub enum Method {
    Get,
    Post,
    Head,
    Put,
    Delete,
    Connect,
    Options,
    Trace,
    Patch,
    Custom(String),
    None,
}

pub enum ResponseCode {
    Continue,
    SwitchingProtocols,
    OK,
    Created,
    Accepted,
    NonAuthoritativeInformation,
    NoContent,
    ResetContent,
    PartialContent,
    MultipleChoices,
    MovedPermanently,
    Found,
    SeeOther,
    NotModified,
    UseProxy,
    TemporaryRedirect,
    BadRequest,
    Unauthorized,
    PaymentRequired,
    Forbidden,
    NotFound,
    MethodNotAllowed,
    NotAcceptable,
    ProxyAuthenticationRequired,
    RequestTimeout,
    Conflict,
    Gone,
    LengthRequired,
    PreconditionFailed,
    RequestEntityTooLarge,
    RequestURITooLarge,
    UnsupportedMediaType,
    RequestedRangeNotSatisfiable,
    ExpectationFailed,
    InternalServerError,
    NotImplemented,
    BadGateway,
    ServiceUnavailable,
    GatewayTimeout,
    HTTPVersionNotSupported,
    Custom(u16, String),
}

impl Method {
    pub fn stringify(&self) -> &str {
        match self {
            Method::Get => "GET",
            Method::Post => "POST",
            Method::Head => "HEAD",
            Method::Put => "PUT",
            Method::Delete => "DELETE",
            Method::Connect => "CONNECT",
            Method::Options => "OPTIONS",
            Method::Trace => "TRACE",
            Method::Patch => "PATCH",
            Method::Custom(s) => s.as_str(),
            Method::None => "",
        }
    }

    pub fn parse(method: &str) -> Self {
        match method.as_bytes() {
            b"GET" => Self::Get,
            b"POST" => Self::Post,
            b"HEAD" => Self::Head,
            b"PUT" => Self::Put,
            b"DELETE" => Self::Delete,
            b"CONNECT" => Self::Connect,
            b"OPTIONS" => Self::Options,
            b"TRACE" => Self::Trace,
            b"PATCH" => Self::Patch,
            _ => Self::Custom(method.to_string())
        }
    }
}

impl ResponseCode {
    pub fn get(&self) -> (u16, &str) {
        match self {
            Self::Continue => (100, "Continue"),
            Self::SwitchingProtocols => (101, "Switching Protocols"),
            Self::OK => (200, "OK"),
            Self::Created => (201, "Created"),
            Self::Accepted => (202, "Accepted"),
            Self::NonAuthoritativeInformation => (203, "Non-Authoritative Information"),
            Self::NoContent => (204, "No Content"),
            Self::ResetContent => (205, "Reset Content"),
            Self::PartialContent => (206, "Partial Content"),
            Self::MultipleChoices => (300, "Multiple Choices"),
            Self::MovedPermanently => (301, "Moved Permanently"),
            Self::Found => (302, "Found"),
            Self::SeeOther => (303, "See Other"),
            Self::NotModified => (304, "Not Modified"),
            Self::UseProxy => (305, "Use Proxy"),
            Self::TemporaryRedirect => (307, "Temporary Redirect"),
            Self::BadRequest => (400, "Bad Request"),
            Self::Unauthorized => (401, "Unauthorized"),
            Self::PaymentRequired => (402, "Payment Required"),
            Self::Forbidden => (403, "Forbidden"),
            Self::NotFound => (404, "Not Found"),
            Self::MethodNotAllowed => (405, "Method Not Allowed"),
            Self::NotAcceptable => (406, "Not Acceptable"),
            Self::ProxyAuthenticationRequired => (407, "Proxy Authentication Required"),
            Self::RequestTimeout => (408, "Request Time-out"),
            Self::Conflict => (409, "Conflict"),
            Self::Gone => (410, "Gone"),
            Self::LengthRequired => (411, "Length Required"),
            Self::PreconditionFailed => (412, "Precondition Failed"),
            Self::RequestEntityTooLarge => (413, "Request Entity Too Large"),
            Self::RequestURITooLarge => (414, "Request-URI Too Large"),
            Self::UnsupportedMediaType => (415, "Unsupported Media Type"),
            Self::RequestedRangeNotSatisfiable => (416, "Requested range not satisfiable"),
            Self::ExpectationFailed => (417, "Expectation Failed"),
            Self::InternalServerError => (500, "Internal Server Error"),
            Self::NotImplemented => (501, "Not Implemented"),
            Self::BadGateway => (502, "Bad Gateway"),
            Self::ServiceUnavailable => (503, "Service Unavailable"),
            Self::GatewayTimeout => (504, "Gateway Time-out"),
            Self::HTTPVersionNotSupported => (505, "HTTP Version not supported"),
            Self::Custom(code, info) => (*code, info.as_str())
        }
    }
}

pub mod response {
    use crate::http::{HttpResponse, ResponseCode};

    pub fn html(text: String) -> HttpResponse {
        let mut buf = [0u8; 65535];
        let len = if text.len() > 0 {
            let bytes = text.as_bytes();
            let len = bytes.len();
            buf[..len].copy_from_slice(&bytes);
            len
        } else {
            0
        };
        HttpResponse {
            buffer: buf,
            len: len,
            header: vec![("Content-Type".to_string(), "text/html".to_string()), ("Content-Length".to_string(), format!("{}", len))],
            code: ResponseCode::OK,
        }
    }
}
