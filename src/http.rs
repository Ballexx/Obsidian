pub mod method;
pub mod request;
pub mod response;
pub mod server;
pub mod status;

const URI_MAX_LEN: usize = 8192;
const TOTAL_HEADER_BYTES: usize = 102400;
const MAX_BODY_LEN_BYTES: u64 = 1048576;
