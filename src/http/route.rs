use crate::{
    AppState,
    http::{
        method::Method,
        request::{Request, RequestLine},
        response::Response,
        status::StatusCode,
    },
};

use crate::oidc::{authorize, discovery::WellKnownConfig, token};

fn build_not_found_response() -> Response {
    let mut response = Response::new();
    response.set_status(StatusCode::NotFound);
    response
}

pub fn handle_route(
    request_line: &RequestLine,
    request: &mut Request,
    state: &AppState,
) -> Response {
    match (request_line.get_method(), request_line.get_path().as_str()) {
        (Method::Get, "/.well-known/openid-configuration") => {
            WellKnownConfig::handle_discovery(state)
        }
        //(Method::Get, "/authorize") => handle_authorize(request_line, request),
        //(Method::Post, "/token") => handle_token(request_line, request),
        _ => build_not_found_response(),
    }
}
