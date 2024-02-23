pub mod bmc;
pub mod request;
pub mod response;

// request
// +--------------------+
// | rsAddr             | 6 bytes
// | netFn/rsLUN        |
// | 1st checksum       |
// | rqAddr             |
// | rqSeq              |
// | cmd                |
// +--------------------+
// | request data bytes |
// +--------------------+
// | 2nd checksum       | 1 bytes
// +--------------------+

// response
// +---------------------+
// | rqAddr              | 7 bytes
// | netFn/rsLUN         |
// | 1st checksum        |
// | rsAddr              |
// | rqSeq               |
// | cmd                 |
// | completion code     |
// +---------------------+
// | response data bytes |
// +---------------------+
// | 2nd checksum        | 1 bytes
// +---------------------+
