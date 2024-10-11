#[derive(Debug)]
pub enum ResponseCode {
    NOERROR,
    FORMERR,
    NXDOMAIN,
    SERVFAIL,
}

impl From<ResponseCode> for u8 {
    fn from(rc: ResponseCode) -> Self {
        match rc {
            ResponseCode::NOERROR => 0,
            ResponseCode::FORMERR => 1,
            ResponseCode::SERVFAIL => 2,
            ResponseCode::NXDOMAIN => 3,
        }
    }
}
