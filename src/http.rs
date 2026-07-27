use std::{collections::HashMap, io::{BufRead, BufReader, Read, Take}, net::{
    SocketAddr, TcpListener, TcpStream
}, num::ParseIntError, println, thread, time::{Duration, Instant}};

enum StatusCode {
    Ok,
    Created,
    NoContent,
    BadRequest,
    Unauthorized,
    Forbidden,
    NotFound,
    MethodNotAllowed,
    PayloadTooLarge,
    UnsupportedMediaType,
    InternalServerError,
    NotImplemented,
    ServiceUnavailable,
}

impl StatusCode {
    fn code(&self) -> u16 {
        match self {
            StatusCode::Ok => 200,
            StatusCode::Created => 201,
            StatusCode::NoContent => 204,
            StatusCode::BadRequest => 400,
            StatusCode::Unauthorized => 401,
            StatusCode::Forbidden => 403,
            StatusCode::NotFound => 404,
            StatusCode::MethodNotAllowed => 405,
            StatusCode::PayloadTooLarge => 413,
            StatusCode::UnsupportedMediaType => 415,
            StatusCode::InternalServerError => 500,
            StatusCode::NotImplemented => 501,
            StatusCode::ServiceUnavailable => 503,
        }
    }

    fn reason_msg(&self) -> &'static str {
        match self {
            StatusCode::Ok => "OK",
            StatusCode::Created => "Created",
            StatusCode::NoContent => "No Content",
            StatusCode::BadRequest => "Bad Request",
            StatusCode::Unauthorized => "Unauthorized",
            StatusCode::Forbidden => "Forbidden",
            StatusCode::NotFound => "Not Found",
            StatusCode::MethodNotAllowed => "Method Not Allowed",
            StatusCode::PayloadTooLarge => "Payload Too Large",
            StatusCode::UnsupportedMediaType => "Unsupported Media Type",
            StatusCode::InternalServerError => "Internal Server Error",
            StatusCode::NotImplemented => "Not Implemented",
            StatusCode::ServiceUnavailable => "Service Unavailable",
        }
    }
}

struct Response{
    status: StatusCode,
    headers: HashMap<String, String>,
    body: String
}

impl Response{
    fn new() -> Self{
        Response{
            status: StatusCode::InternalServerError,
            headers: HashMap::new(),
            body: String::new()
        }
    }

    fn set_status(&mut self, status: StatusCode){
        self.status = status;
    }

    fn set_response_header(&mut self, key: String, value: String){
        self.headers.insert(key, value);
    }

    fn set_response_body(&mut self, body: String){
        self.body = body;
    }

    fn send_error(writer: &mut TcpStream, status: StatusCode, message: &str) {

    }
}

fn is_valid_header_key(key: &str) -> bool {
    !key.is_empty() && key.chars().all(|c: char| {
        c.is_ascii_alphanumeric() || "!#$%&'*+-.^_`|~".contains(c)
    })
}

fn is_valid_header_value(value: &str) -> bool {
    !value.chars().any(|c: char| c == '\r' || c == '\n')
}

struct Request{
    reader: BufReader<TcpStream>,
    headers: HashMap<String, String>,
    body_length: u64,
    total_header_bytes: usize
}

impl Request{
    fn new(socket: TcpStream) -> Self{
        Request{
            reader: BufReader::new(socket),
            headers: HashMap::new(),
            body_length: 0,
            total_header_bytes: 102400

        }
    }

    fn insert_header(&mut self, key: String, value: String){
        self.headers.insert(key, value);
    }

    fn print_headers(&self){
        println!("{:?}", self.headers);
    }

    fn parse_content_length(&self, value: &str) -> Result<u64, ParseIntError> {
        return value.parse::<u64>();
    }

    fn is_content_length_allowed(&self, body_length: u64) -> bool{
        //  1MB - kanske fixar så den kan modifieras på egen hand sen
        if body_length >= 1048576{       
            println!("Content-Length is too large.");
            return false;
        }

        return true;   
    }

    fn set_body_length(&mut self, body_length: u64){
        self.body_length = body_length;
    }
}

//  handle_connection behöver en funktion som räknar timear ut när hela headern tar för lång tid, inte bara per rad

fn handle_connection(socket: TcpStream, addr: SocketAddr){
    let run_timer: Instant = Instant::now();

    let Ok(write_socket) = socket.try_clone() else {
        println!("Failed to clone socket.");
        return;
    };
    
    let mut request: Request = Request::new(socket);
    let response: Response = Response::new();
    
    let mut request_line: String = String::new();
    if let Err(e) = request.reader.read_line(&mut request_line){
        println!("Failed to read request line: {e:?}");
        return;
    };

    let mut header_line: String = String::new();
    let mut read_byte_count: usize = 0;

    loop{
        let Ok(buffer_len) = request.reader.read_line(&mut header_line) else{
            println!("Failed to read line."); 
            return;
        };
        
        if run_timer.elapsed().as_secs() > 10{
            return;
        }

        if buffer_len == 0 {
            break;
        }

        let new_total_byte_count: usize = read_byte_count + buffer_len;
        if new_total_byte_count > request.total_header_bytes{
            println!("Header too large.");
            return;
        }
        read_byte_count = new_total_byte_count;        

        if header_line == "\r\n" || header_line == "\n" {
            break;
        }

        let key_value_pair: Vec<&str> = header_line.splitn(2,":").collect();
        if key_value_pair.len() < 2{
            header_line.clear();
            continue;
        };
        
        let key: String = key_value_pair[0].trim().to_owned();
        let value: String = key_value_pair[1].trim().to_owned();

        if !is_valid_header_key(&key) || !is_valid_header_value(&value){
            println!("Header contained illegal characters.");
            return;
        }

        if key.eq_ignore_ascii_case("content-length"){
            let Ok(parsed_req_value) = request.parse_content_length(&value) else{
                println!("Failed to parse header value.");
                return;
            };

            if !request.is_content_length_allowed(parsed_req_value){
                return;
            }

            request.set_body_length(parsed_req_value);
        }

        request.insert_header(key, value);
        header_line.clear();
    };

    let mut body_buffer = vec![0; request.body_length as usize];
    let mut body: Take<BufReader<TcpStream>>  = request.reader.take(request.body_length);

    if let Err(e) = body.read_exact(&mut body_buffer){
        println!("Failed to read body: {e:?}");
        return;
    };
}

pub fn listen(ip: &str, port: u16){
    let host = format!("{}:{}", ip, port);
    let Ok(listener) = TcpListener::bind(host) else{
        println!("Failed to bind to port.");
        return;
    };
    
    loop{
        match listener.accept() {
            Ok((socket, addr)) => {
                if let Err(e) = socket.set_read_timeout(Some(Duration::from_secs(5))){
                    println!("Failed to set read timeout: {e:?}");
                    return;
                };

                thread::spawn(move ||{
                    handle_connection(socket, addr);
                });
            },
            Err(e) => println!("Couldn't get client: {e:?}"),   //    Detta skulle kunna visas i loggar
        }
    };
}