use crate::http::{method::Method, status::StatusCode};
use std::collections::HashMap;
use std::io::{BufReader, Read, Take};
use std::net::TcpStream;
use std::num::ParseIntError;
use std::str::FromStr;

pub struct RequestLine {
    method: Method,
    path: String,
    version: String,
}

impl RequestLine {
    pub fn new() -> Self {
        RequestLine {
            method: Method::Get,
            path: "/".to_owned(),
            version: String::new(),
        }
    }

    pub fn get_method(&self) -> &Method {
        return &self.method;
    }

    pub fn get_path(&self) -> &String {
        return &self.path;
    }

    pub fn get_version(&self) -> &String {
        return &self.version;
    }

    pub fn parse(request_line: String) -> Result<Self, StatusCode> {
        let mut request = RequestLine::new();
        let split_request_line: Vec<String> = request_line
            .split_whitespace()
            .map(|s| s.to_owned())
            .collect();

        if split_request_line.len() != 3 {
            return Err(StatusCode::BadRequest);
        }

        request.method = Method::from_str(&split_request_line[0])?;
        request.path = split_request_line[1].clone();
        request.version = split_request_line[2].clone();

        if !request.path.starts_with('/')
            || request.path.chars().any(|c| c.is_control() || c == ' ')
        {
            return Err(StatusCode::BadRequest);
        }

        if request.version != "HTTP/1.0" && request.version != "HTTP/1.1" {
            return Err(StatusCode::HttpVersionNotSupported);
        }

        return Ok(request);
    }
}

pub struct Request {
    reader: BufReader<TcpStream>,
    headers: HashMap<String, String>,
    body_length: u64,
    max_body_len_bytes: u64,
    total_header_bytes: usize,
}

impl Request {
    pub fn new(socket: TcpStream) -> Self {
        Request {
            reader: BufReader::new(socket),
            headers: HashMap::new(),
            body_length: 0,
            total_header_bytes: 102400,
            max_body_len_bytes: 1048576,
        }
    }

    pub fn get_reader(&mut self) -> &mut BufReader<TcpStream> {
        return &mut self.reader;
    }

    pub fn get_total_header_bytes(&self) -> &usize {
        return &self.total_header_bytes;
    }

    pub fn get_max_body_len_bytes(&self) -> &u64 {
        return &self.max_body_len_bytes;
    }

    pub fn insert_header(&mut self, key: String, value: String) {
        self.headers.insert(key, value);
    }

    pub fn parse_content_length(&self, value: &str) -> Result<u64, ParseIntError> {
        return value.parse::<u64>();
    }

    pub fn is_content_length_allowed(&self, body_length: u64) -> bool {
        if body_length >= self.max_body_len_bytes {
            println!("Content-Length is too large.");
            return false;
        }

        return true;
    }

    pub fn set_body_length(&mut self, body_length: u64) {
        self.body_length = body_length;
    }

    pub fn read_body(&mut self) -> std::io::Result<Vec<u8>> {
        let mut body_buffer: Vec<u8> = vec![0; self.body_length as usize];
        let mut body: Take<&mut BufReader<TcpStream>> = (&mut self.reader).take(self.body_length);
        body.read_exact(&mut body_buffer)?;
        Ok(body_buffer)
    }
}
