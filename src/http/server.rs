use crate::log::{LogEntry, LogLevel};
use crate::respond_and_return;
use crate::{
    http::{
        method::Method,
        request::{Request, RequestLine},
        response::Response,
        status::StatusCode,
    },
    log_err,
};
use std::io::{BufRead, BufReader, Read, Take};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::thread;
use std::time::{Duration, Instant};

fn is_valid_header_key(key: &str) -> bool {
    !key.is_empty()
        && key
            .chars()
            .all(|c: char| c.is_ascii_alphanumeric() || "!#$%&'*+-.^_`|~".contains(c))
}

fn is_valid_header_value(value: &str) -> bool {
    !value.chars().any(|c: char| c == '\r' || c == '\n')
}

fn handle_connection(socket: TcpStream, addr: SocketAddr) {
    let run_timer: Instant = Instant::now();

    let Ok(mut write_socket) = socket.try_clone() else {
        log_err!(LogLevel::Error, "Error cloning socket.");
        return;
    };

    let mut request: Request = Request::new(socket);
    let mut response: Response = Response::new();

    let mut request_line_str: String = String::new();
    if let Err(_) = request.get_reader().read_line(&mut request_line_str) {
        log_err!(LogLevel::Error, "Failed to read request line.");
        respond_and_return!(StatusCode::InternalServerError, response, &mut write_socket);
    };

    let request_line: RequestLine = match RequestLine::parse(request_line_str) {
        Ok(req) => req,
        Err(status_code_err) => {
            log_err!(LogLevel::Error, "Failed to parse request line.");
            respond_and_return!(status_code_err, response, &mut write_socket);
        }
    };

    let mut header_line: String = String::new();
    let mut read_byte_count: usize = 0;

    loop {
        let Ok(buffer_len) = request.get_reader().read_line(&mut header_line) else {
            log_err!(LogLevel::Error, "Failed to read request line");
            respond_and_return!(StatusCode::InternalServerError, response, &mut write_socket);
        };

        if run_timer.elapsed().as_secs() > 10 {
            log_err!(LogLevel::Error, "Failed to read header line.");
            respond_and_return!(StatusCode::RequestTimeout, response, &mut write_socket);
        }

        if buffer_len == 0 {
            break;
        }

        let new_total_byte_count: usize = read_byte_count + buffer_len;
        if new_total_byte_count > *request.get_total_header_bytes() {
            log_err!(
                LogLevel::Error,
                format!(
                    "Content of header exceeded the max of {} bytes",
                    request.get_total_header_bytes()
                )
            );
            respond_and_return!(StatusCode::PayloadTooLarge, response, &mut write_socket);
        }
        read_byte_count = new_total_byte_count;

        if header_line == "\r\n" || header_line == "\n" {
            break;
        }

        let key_value_pair: Vec<&str> = header_line.splitn(2, ":").collect();
        if key_value_pair.len() < 2 {
            header_line.clear();
            continue;
        };

        let key: String = key_value_pair[0].trim().to_owned().to_lowercase();
        let value: String = key_value_pair[1].trim().to_owned().to_lowercase();

        if !is_valid_header_key(&key) || !is_valid_header_value(&value) {
            log_err!(
                LogLevel::Error,
                "Header value or key contained illegal characters."
            );
            respond_and_return!(StatusCode::BadRequest, response, &mut write_socket);
        }

        if key == "content-length" {
            let Ok(parsed_req_value) = request.parse_content_length(&value) else {
                log_err!(
                    LogLevel::Error,
                    "Content-Length was unable to be parsed to u64."
                );
                respond_and_return!(StatusCode::BadRequest, response, &mut write_socket);
            };

            if !request.is_content_length_allowed(parsed_req_value) {
                log_err!(
                    LogLevel::Error,
                    format!(
                        "The max allowed body length of {} bytes was exceeded.",
                        request.get_max_body_len_bytes()
                    )
                );
                respond_and_return!(StatusCode::PayloadTooLarge, response, &mut write_socket);
            }

            request.set_body_length(parsed_req_value);
        }

        request.insert_header(key, value);
        request_line.verify_query_by_headers(request.get_headers(), request_line.get_query());

        header_line.clear();
    }

    let body: String = match request.read_body() {
        Ok(v) => v,
        Err(e) => {
            log_err!(LogLevel::Error, "Could not read body.");
            respond_and_return!(StatusCode::BadRequest, response, &mut write_socket);
        }
    };

    response.set_status(StatusCode::Ok);
    response.send(&mut write_socket);
}

pub fn listen(ip: &str, port: u16) {
    let host = format!("{}:{}", ip, port);
    let Ok(listener) = TcpListener::bind(host) else {
        println!("Failed to bind to port.");
        return;
    };

    loop {
        match listener.accept() {
            Ok((socket, addr)) => {
                if let Err(e) = socket.set_read_timeout(Some(Duration::from_secs(5))) {
                    println!("Failed to set read timeout: {e:?}");
                    return;
                };

                thread::spawn(move || {
                    handle_connection(socket, addr);
                });
            }
            Err(e) => {
                LogEntry::new()
                    .set_level(LogLevel::Warn)
                    .set_message(format!("Failed to accept client: {e:?}"));
            }
        }
    }
}
