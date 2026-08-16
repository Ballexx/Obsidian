use crate::http::{
    method::Method,
    request::{Request, RequestLine},
    response::Response,
    status::StatusCode,
};

use crate::oidc::{authorize, discovery, token};

pub fn route(request_line: &RequestLine, request: &mut Request) -> Response {
    match (request_line.get_method(), request_line.get_path().as_str()) {
        (Method::Get, "/.well-known/openid-configuration") => handle_discovery(),
        (Method::Get, "/authorize") => handle_authorize(request_line, request),
        (Method::Post, "/token") => handle_token(request_line, request),
        _ => build_not_found_response(),
    }
}
