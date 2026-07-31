use crate::http::status::StatusCode;
use std::collections::HashMap;
use std::io::Write;
use std::net::TcpStream;

pub struct Response {
    pub status: StatusCode,
    pub headers: HashMap<String, String>,
    body: String,
}

impl Response {
    pub fn new() -> Self {
        Response {
            status: StatusCode::InternalServerError,
            headers: HashMap::new(),
            body: String::new(),
        }
    }

    pub fn set_status(&mut self, status: StatusCode) {
        self.status = status;
    }

    pub fn set_response_header(&mut self, key: String, value: String) {
        self.headers.insert(key, value);
    }

    pub fn set_response_body(&mut self, body: String) {
        self.body = body;
    }

    pub fn send(&self, writer: &mut TcpStream) {
        let mut send_data = String::new();
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
