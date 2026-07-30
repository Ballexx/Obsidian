use crate::http::{method::Method, status::StatusCode};
use std::collections::HashMap;
use std::io::{BufReader, Read, Take};
use std::net::TcpStream;
use std::num::ParseIntError;
use std::str::{Bytes, Chars, FromStr, Split, from_utf8};

fn is_valid_query_value(value: &str) -> bool {
    return !value.chars().any(|c| c.is_control());
}

fn decode_query(query: &str) -> Result<String, StatusCode> {
    let mut byte_list: Bytes = query.bytes();

    let mut decoded_query: String = String::new();

    while let Some(c) = byte_list.next() {
        if c != b'%' {
            decoded_query.push_str(&(c as char).to_string());
            continue;
        }

        let a: Option<u8> = byte_list.next();
        let b: Option<u8> = byte_list.next();

        if let (Some(a), Some(b)) = (a, b) {}
    }
    Ok(decoded_query)
}

pub struct RequestLine {
    method: Method,
    path: String,
    query: HashMap<String, String>,
    version: String,
    uri_max_len: usize,
}

impl RequestLine {
    pub fn new() -> Self {
        RequestLine {
            method: Method::Get,
            path: "/".to_owned(),
            query: HashMap::new(),
            version: String::new(),
            uri_max_len: 8192,
        }
    }

    pub fn get_method(&self) -> &Method {
        return &self.method;
    }

    pub fn get_path(&self) -> &String {
        return &self.path;
    }

    pub fn get_query(&self) -> &HashMap<String, String> {
        return &self.query;
    }

    pub fn get_version(&self) -> &String {
        return &self.version;
    }

    pub fn parse(request_line: String) -> Result<Self, StatusCode> {
        let mut request: RequestLine = RequestLine::new();

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

        if request.path.len() >= request.uri_max_len {
            return Err(StatusCode::UriTooLong);
        }

        if let Some((path_part, query_part)) = request.path.split_once('?') {
            if !is_valid_query_value(query_part) {
                return Err(StatusCode::BadRequest);
            }

            // VIKTIGT : MÅSTE DECODA QUERY - PALLAR DOCK INTE NU LMAO

            let query_pairs: Vec<&str> = query_part.split("&").collect();

            for pair in query_pairs {
                let key_value: Vec<&str> = pair.splitn(2, "=").collect();

                if key_value.len() != 2 {
                    return Err(StatusCode::BadRequest);
                }

                if !is_valid_query_value(key_value[0]) || !is_valid_query_value(key_value[1]) {
                    return Err(StatusCode::BadRequest);
                }

                request
                    .query
                    .insert(key_value[0].to_owned(), key_value[1].to_owned());
            }

            request.path = path_part.to_owned();
        }

        if !request.path.starts_with('/')
            || request.path.chars().any(|c| c.is_control() || c == ' ')
        {
            return Err(StatusCode::BadRequest);
        }

        if request.version != "HTTP/1.0" && request.version != "HTTP/1.1" {
            return Err(StatusCode::HttpVersionNotSupported);
        }

        Ok(request)
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

    pub fn read_body(&mut self) -> Result<String, StatusCode> {
        let mut body_buffer: Vec<u8> = vec![0; self.body_length as usize];
        let mut body: Take<&mut BufReader<TcpStream>> = (&mut self.reader).take(self.body_length);

        if let Err(_) = body.read_exact(&mut body_buffer) {
            return Err(StatusCode::BadRequest);
        };

        let Ok(return_body) = from_utf8(&body_buffer) else {
            return Err(StatusCode::BadRequest);
        };

        Ok(return_body.to_owned())
    }
}
