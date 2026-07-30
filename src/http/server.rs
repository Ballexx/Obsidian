use crate::http::{
    method::Method,
    request::{Request, RequestLine},
    response::Response,
    status::StatusCode,
};
use crate::log::{LogEntry, LogLevel};
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
        LogEntry::new()
            .set_level(LogLevel::Error)
            .set_message("Error cloning socket.");
        return;
    };

    let mut request: Request = Request::new(socket);
    let mut response: Response = Response::new();

    let mut request_line_str: String = String::new();
    if let Err(e) = request.get_reader().read_line(&mut request_line_str) {
        LogEntry::new()
            .set_level(LogLevel::Error)
            .set_message(format!("Failed to read request line: {e:?}"));

        response.set_status(StatusCode::InternalServerError);
        response.send(&mut write_socket);
        return;
    };

    let request_line: RequestLine = match RequestLine::parse(request_line_str) {
        Ok(req) => req,
        Err(status_code_err) => {
            LogEntry::new()
                .set_level(LogLevel::Error)
                .set_message(format!("Failed to parse request line."));

            response.set_status(status_code_err);
            response.send(&mut write_socket);
            return;
        }
    };

    let mut header_line: String = String::new();
    let mut read_byte_count: usize = 0;

    loop {
        let Ok(buffer_len) = request.get_reader().read_line(&mut header_line) else {
            LogEntry::new()
                .set_level(LogLevel::Error)
                .set_message(format!("Failed to read request line."));

            response.set_status(StatusCode::InternalServerError);
            response.send(&mut write_socket);
            return;
        };

        if run_timer.elapsed().as_secs() > 10 {
            LogEntry::new()
                .set_level(LogLevel::Error)
                .set_message("Failed to read header line.");

            response.set_status(StatusCode::RequestTimeout);
            response.send(&mut write_socket);
            return;
        }

        if buffer_len == 0 {
            break;
        }

        let new_total_byte_count: usize = read_byte_count + buffer_len;
        if new_total_byte_count > *request.get_total_header_bytes() {
            LogEntry::new()
                .set_level(LogLevel::Error)
                .set_message(format!(
                    "Content of header exceeded the max of {} bytes",
                    request.get_total_header_bytes()
                ));

            response.set_status(StatusCode::PayloadTooLarge);
            response.send(&mut write_socket);
            return;
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
            LogEntry::new()
                .set_level(LogLevel::Error)
                .set_message("Header value or key contained illegal characters.");

            response.set_status(StatusCode::BadRequest);
            response.send(&mut write_socket);
            return;
        }

        if key == "content-length" {
            let Ok(parsed_req_value) = request.parse_content_length(&value) else {
                LogEntry::new()
                    .set_level(LogLevel::Error)
                    .set_message("Content-Length was unabled to be parsed to u64.");

                response.set_status(StatusCode::BadRequest);
                response.send(&mut write_socket);
                return;
            };

            if !request.is_content_length_allowed(parsed_req_value) {
                LogEntry::new()
                    .set_level(LogLevel::Error)
                    .set_message(format!(
                        "The max allowed body length of {} bytes was exceeded.",
                        request.get_max_body_len_bytes()
                    ));

                response.set_status(StatusCode::PayloadTooLarge);
                response.send(&mut write_socket);
                return;
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
            LogEntry::new()
                .set_level(LogLevel::Error)
                .set_message(format!("Could not read body."));

            response.set_status(e);
            response.send(&mut write_socket);
            return;
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
