mod http;
use http::server;

pub mod log;

fn main() {
    server::listen("127.0.0.1", 3000);
}
