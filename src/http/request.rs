use crate::http::{MAX_BODY_LEN_BYTES, URI_MAX_LEN};
use crate::http::{method::Method, status::StatusCode};
use std::collections::HashMap;
use std::io::{BufReader, Read, Take};
use std::net::TcpStream;
use std::num::ParseIntError;
use std::str::{Bytes, Chars, FromStr, Split, from_utf8};
use std::{path, println};

fn is_valid_query_value(value: &str) -> bool {
    return !value.chars().any(|c| c.is_control());
}

fn parse_query(request: &mut RequestLine) -> Result<(), StatusCode> {
    let Some((path_part, query_part)) = request.path.split_once('?') else {
        return Ok(());
    };

    if !is_valid_query_value(query_part) {
        return Err(StatusCode::BadRequest);
    }

    let Ok(decoded_query) = decode_query(query_part) else {
        return Err(StatusCode::BadRequest);
    };

    let query_pairs: Vec<&str> = decoded_query.split("&").collect();
    let new_path = path_part.to_owned();

    for pair in query_pairs {
        let key_value: Vec<&str> = pair.splitn(2, "=").collect();

        if key_value.len() != 2 {
            return Err(StatusCode::BadRequest);
        }

        if !is_valid_query_value(key_value[0]) || !is_valid_query_value(key_value[1]) {
            return Err(StatusCode::BadRequest);
        }

        request
            .get_query_mut()
            .insert(key_value[0].to_owned(), key_value[1].to_owned());
    }
    request.path = new_path;

    Ok(())
}

fn decode_query(query: &str) -> Result<String, StatusCode> {
    let mut query_as_bytes = query.bytes();
    let mut new_byte_list = Vec::new();

    while let Some(c) = query_as_bytes.next() {
        if c != b'%' {
            new_byte_list.push(c);
            continue;
        }

        let byte1 = query_as_bytes.next();
        let byte2 = query_as_bytes.next();

        if let (Some(byte1), Some(byte2)) = (byte1, byte2) {
            let bytes: [u8; 2] = [byte1, byte2];

            let Ok(hex_str) = std::str::from_utf8(&bytes) else {
                return Err(StatusCode::BadRequest);
            };
            let Ok(result) = u8::from_str_radix(hex_str, 16) else {
                return Err(StatusCode::BadRequest);
            };

            new_byte_list.push(result);
        } else {
            return Err(StatusCode::BadRequest);
        }
    }

    let Ok(decoded_query) = String::from_utf8(new_byte_list) else {
        return Err(StatusCode::BadRequest);
    };

    Ok(decoded_query)
}

pub struct RequestLine {
    method: Method,
    path: String,
    query: HashMap<String, String>,
    version: String,
}

impl RequestLine {
    pub fn new() -> Self {
        RequestLine {
            method: Method::Get,
            path: "/".to_owned(),
            query: HashMap::new(),
            version: String::new(),
        }
    }

    pub fn get_method(&self) -> &Method {
        &self.method
    }

    pub fn get_path(&self) -> &String {
        &self.path
    }

    pub fn get_query(&self) -> &HashMap<String, String> {
        &self.query
    }

    pub fn get_query_mut(&mut self) -> &mut HashMap<String, String> {
        &mut self.query
    }

    pub fn get_version(&self) -> &String {
        &self.version
    }

    pub fn handle_form_urlencoded(&self, query: String) -> String {
        String::from("Placeholder")
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

        if request.path.len() >= URI_MAX_LEN {
            return Err(StatusCode::UriTooLong);
        }

        if !request.path.starts_with('/')
            || request.path.chars().any(|c| c.is_control() || c == ' ')
        {
            return Err(StatusCode::BadRequest);
        }

        parse_query(&mut request);

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
}

impl Request {
    pub fn new(socket: TcpStream) -> Self {
        Request {
            reader: BufReader::new(socket),
            headers: HashMap::new(),
            body_length: 0,
        }
    }

    pub fn get_reader(&mut self) -> &mut BufReader<TcpStream> {
        &mut self.reader
    }

    pub fn insert_header(&mut self, key: String, value: String) {
        self.headers.insert(key, value);
    }

    pub fn get_headers(&self) -> &HashMap<String, String> {
        &self.headers
    }

    pub fn parse_content_length(&self, value: &str) -> Result<u64, ParseIntError> {
        value.parse::<u64>()
    }

    pub fn is_content_length_allowed(&self, body_length: u64) -> bool {
        if body_length >= MAX_BODY_LEN_BYTES {
            println!("Content-Length is too large.");
            return false;
        }

        true
    }

    pub fn set_body_length(&mut self, body_length: u64) {
        self.body_length = body_length;
    }

    pub fn read_body(&mut self) -> Result<String, StatusCode> {
        let mut body_buffer = vec![0; self.body_length as usize];
        let mut body = (&mut self.reader).take(self.body_length);

        if let Err(_) = body.read_exact(&mut body_buffer) {
            return Err(StatusCode::BadRequest);
        };

        let Ok(return_body) = from_utf8(&body_buffer) else {
            return Err(StatusCode::BadRequest);
        };

        Ok(return_body.to_owned())
    }
}
