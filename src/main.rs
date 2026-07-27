pub mod http;
pub mod log;

fn main() {
    http::listen("127.0.0.1", 3000);
}
