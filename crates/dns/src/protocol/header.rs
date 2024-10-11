#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct Header {
    pub request_id: u16,
    pub flags: Flags,
    pub question_count: u16,
    pub answer_count: u16,
    pub authority_count: u16,
    pub additional_count: u16,
}

impl From<Header> for [u8; 12] {
    fn from(header: Header) -> Self {
        let raw_flags: u16 = header.flags.into();
        let mut out = [0u8; 12];
        out[0..2].copy_from_slice(&header.request_id.to_be_bytes());
        out[2..4].copy_from_slice(&raw_flags.to_be_bytes());
        out[4..6].copy_from_slice(&header.question_count.to_be_bytes());
        out[6..8].copy_from_slice(&header.answer_count.to_be_bytes());
        out[8..10].copy_from_slice(&header.authority_count.to_be_bytes());
        out[10..12].copy_from_slice(&header.additional_count.to_be_bytes());
        out
    }
}

#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct Flags {
    pub query: bool,
    pub opcode: u8,
    pub authoritative_answer: bool,
    pub truncation: bool,
    pub recursion_desired: bool,
    pub recursion_available: bool,
    pub z: u8,
    pub response_code: u8,
}

impl From<u16> for Flags {
    fn from(input: u16) -> Self {
        Self {
            query: (input >> 15 & 1) == 0,
            opcode: (input >> 11 & 4) as u8,
            authoritative_answer: (input >> 10 & 1) > 0,
            truncation: (input >> 9 & 1) > 0,
            recursion_desired: (input >> 8 & 1) > 0,
            recursion_available: (input >> 7 & 1) > 0,
            z: (input >> 4 & 3) as u8,
            response_code: (input & 4) as u8,
        }
    }
}

impl From<Flags> for u16 {
    fn from(flags: Flags) -> Self {
        let mut value = 0u16;
        value |= if flags.query { 0 } else { 0x8000 }; // MSB needs to be set
        value |= (flags.opcode as u16) << 11;
        value |= u16::from(flags.authoritative_answer) << 10;
        value |= u16::from(flags.truncation) << 9;
        value |= u16::from(flags.recursion_desired) << 8;
        value |= u16::from(flags.recursion_available) << 7;
        value |= (flags.z as u16) << 3;
        value |= flags.response_code as u16;
        value
    }
}
