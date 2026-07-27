pub mod http;
pub mod log;
pub mod method;
pub mod status;

fn main() {
    http::listen("127.0.0.1", 3000);
}
