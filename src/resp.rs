use bytes::{BufMut, Bytes, BytesMut};
use std::io::Error;

const BULK_STRING: u8 = b'$'; // 0x24
const ARRAY: u8 = b'*'; // 0x2a

pub fn parse_lenght(input: &[u8], len: &mut usize) -> usize {
    let mut pos: usize = 0;
    *len = 0;
    while input[pos] != b'\r' {
        *len = *len * 10 + (input[pos] - b'0') as usize;
        pos += 1;
    }
    pos + 2
}

pub fn parse_bulk_string(input: &[u8], result: &mut String) -> Result<usize, Error> {
    if input[0] != BULK_STRING {
        return Err(Error::new(std::io::ErrorKind::InvalidData, "invalid data"));
    }

    let mut pos: usize = 1;
    let mut string_lemgth = 0;
    pos += parse_lenght(&input[pos..], &mut string_lemgth);

    *result = String::from_utf8_lossy(&input[pos..pos + string_lemgth]).to_string();
    Ok(pos + string_lemgth + 2)
}

pub fn parse_array(input: &[u8]) -> Result<Vec<String>, Error> {
    if input[0] != ARRAY {
        return Err(Error::new(std::io::ErrorKind::InvalidData, "invalid data"));
    }

    let mut pos: usize = 1;
    let mut array_len = 0;
    pos += parse_lenght(&input[pos..], &mut array_len);

    let mut array: Vec<String> = Vec::with_capacity(array_len);
    for _ in 0..array_len {
        let mut arg = String::new();
        pos += parse_bulk_string(&input[pos..], &mut arg)?;
        array.push(arg);
    }

    Ok(array)
}

pub fn bulk_string(s: &str) -> String {
    format!("${}\r\n{}\r\n", s.len(), s)
}

pub fn rdb_file(data: &[u8]) -> Bytes {
    let mut bytes = BytesMut::new();
    bytes.put_u8(b'$');
    bytes.extend_from_slice(data.len().to_string().as_bytes());
    bytes.extend_from_slice(b"\r\n");
    bytes.extend_from_slice(data);

    bytes.freeze()
}
