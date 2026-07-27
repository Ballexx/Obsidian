use std::{
    collections::HashMap,
    io::{BufRead, BufReader, Read, Take, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    num::ParseIntError,
    println,
    str::FromStr,
    thread,
    time::{Duration, Instant},
};

use crate::log::{LogEntry, LogLevel};
use crate::method::Method;
use crate::status::StatusCode;

struct Response {
    status: StatusCode,
    headers: HashMap<String, String>,
    body: String,
}

impl Response {
    fn new() -> Self {
        Response {
            status: StatusCode::InternalServerError,
            headers: HashMap::new(),
            body: String::new(),
        }
    }

    fn set_status(&mut self, status: StatusCode) {
        self.status = status;
    }

    fn set_response_header(&mut self, key: String, value: String) {
        self.headers.insert(key, value);
    }

    fn set_response_body(&mut self, body: String) {
        self.body = body;
    }

    fn send(&self, writer: &mut TcpStream) {
        let mut send_data: String = String::new();
        send_data.push_str(&format!(
            "HTTP/1.1 {} {}\r\n",
            self.status.code(),
            self.status.reason_msg(),
        ));

        let content_len = self.body.len();
        send_data.push_str(&format!("Content-Length: {}\r\n", content_len));

        for (key, value) in &self.headers {
            if key.eq_ignore_ascii_case("content-length") {
                continue;
            }

            send_data.push_str(&format!("{}: {}\r\n", key, value));
        }

        send_data.push_str(&format!("\r\n{}", self.body));
        let data_as_bytes = send_data.into_bytes();

        if let Err(e) = writer.write_all(&data_as_bytes) {
            println!("Error sending response: {e:?}");
        }
    }
}

fn is_valid_header_key(key: &str) -> bool {
    !key.is_empty()
        && key
            .chars()
            .all(|c: char| c.is_ascii_alphanumeric() || "!#$%&'*+-.^_`|~".contains(c))
}

fn is_valid_header_value(value: &str) -> bool {
    !value.chars().any(|c: char| c == '\r' || c == '\n')
}

struct Request {
    reader: BufReader<TcpStream>,
    headers: HashMap<String, String>,
    body_length: u64,
    max_body_len_bytes: u64,
    total_header_bytes: usize,
}

impl Request {
    fn new(socket: TcpStream) -> Self {
        Request {
            reader: BufReader::new(socket),
            headers: HashMap::new(),
            body_length: 0,
            total_header_bytes: 102400,
            max_body_len_bytes: 1048576,
        }
    }

    fn insert_header(&mut self, key: String, value: String) {
        self.headers.insert(key, value);
    }

    fn print_headers(&self) {
        println!("{:?}", self.headers);
    }

    fn parse_content_length(&self, value: &str) -> Result<u64, ParseIntError> {
        return value.parse::<u64>();
    }

    fn is_content_length_allowed(&self, body_length: u64) -> bool {
        if body_length >= self.max_body_len_bytes {
            println!("Content-Length is too large.");
            return false;
        }

        return true;
    }

    fn set_body_length(&mut self, body_length: u64) {
        self.body_length = body_length;
    }

    fn handle_request_line(request_line: &str) -> Result<Vec<&str>, StatusCode> {
        let split_request_line: Vec<&str> = request_line.split_whitespace().collect();

        if split_request_line.len() != 3 {
            return Err(StatusCode::BadRequest);
        }

        if let Err(e) = Method::from_str(split_request_line[0]) {
            return Err(e);
        };

        if !split_request_line[1].starts_with('/')
            || split_request_line[1]
                .chars()
                .any(|c| c.is_control() || c == ' ')
        {
            return Err(StatusCode::BadRequest);
        }

        if split_request_line[2] != "HTTP/1.0" && split_request_line[2] != "HTTP/1.1" {
            return Err(StatusCode::HttpVersionNotSupported);
        }

        return Ok(split_request_line);
    }
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

    let mut request_line: String = String::new();
    if let Err(e) = request.reader.read_line(&mut request_line) {
        LogEntry::new()
            .set_level(LogLevel::Error)
            .set_message(format!("Failed to read request line: {e:?}"));

        response.set_status(StatusCode::InternalServerError);
        response.send(&mut write_socket);
        return;
    };

    request.handle_request_line();

    if split_request_line.len() != 3 {
        LogEntry::new()
            .set_level(LogLevel::Error)
            .set_message(format!(
                "Malformed request line: expected 3 parts got {}",
                split_request_line.len()
            ));

        response.set_status(StatusCode::BadRequest);
        response.send(&mut write_socket);
        return;
    }

    let mut header_line: String = String::new();
    let mut read_byte_count: usize = 0;

    loop {
        let Ok(buffer_len) = request.reader.read_line(&mut header_line) else {
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
        if new_total_byte_count > request.total_header_bytes {
            LogEntry::new()
                .set_level(LogLevel::Error)
                .set_message(format!(
                    "Content of header exceeded the max of {} bytes",
                    request.total_header_bytes
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

        let key: String = key_value_pair[0].trim().to_owned();
        let value: String = key_value_pair[1].trim().to_owned();

        if !is_valid_header_key(&key) || !is_valid_header_value(&value) {
            LogEntry::new()
                .set_level(LogLevel::Error)
                .set_message("Header value or key contained illegal characters.");

            response.set_status(StatusCode::BadRequest);
            response.send(&mut write_socket);
            return;
        }

        if key.eq_ignore_ascii_case("content-length") {
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
                        request.max_body_len_bytes
                    ));

                response.set_status(StatusCode::PayloadTooLarge);
                response.send(&mut write_socket);
                return;
            }

            request.set_body_length(parsed_req_value);
        }

        request.insert_header(key, value);
        header_line.clear();
    }

    let mut body_buffer = vec![0; request.body_length as usize];
    let mut body: Take<BufReader<TcpStream>> = request.reader.take(request.body_length);

    if let Err(e) = body.read_exact(&mut body_buffer) {
        LogEntry::new()
            .set_level(LogLevel::Error)
            .set_message(format!("Could not read body: {e:?}"));

        response.set_status(StatusCode::BadRequest);
        response.send(&mut write_socket);
        return;
    };

    println!("{request_line}");

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
