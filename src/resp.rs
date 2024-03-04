use anyhow::{bail, Result};
use bytes::{BufMut, Bytes, BytesMut};
use tokio::{io::AsyncReadExt, net::TcpStream};

const BULK_STRING: u8 = b'$'; // 0x24
const ARRAY: u8 = b'*'; // 0x2a

pub async fn parse_lenght(stream: &mut TcpStream) -> Result<(usize, usize)> {
    let mut size = 0;

    let mut buf = stream.read_u8().await?;
    let mut offset = 1;
    while buf != b'\r' {
        size = size * 10 + (buf - b'0') as usize;
        buf = stream.read_u8().await?;
        offset += 1;
    }
    stream.read_u8().await?; // consume \n
    offset += 1;

    Ok((size, offset))
}

pub async fn parse_simple_string(stream: &mut TcpStream) -> Result<(String, usize)> {
    if stream.read_u8().await? != b'+' {
        bail!("invalid data");
    }

    let mut buf = Vec::new();
    let mut read_buf = stream.read_u8().await?;
    let mut offset = 2;
    while read_buf != b'\r' {
        buf.push(read_buf);
        read_buf = stream.read_u8().await?;
        offset += 1;
    }
    stream.read_u8().await?; // consume \n
    offset += 1;

    Ok((String::from_utf8(buf)?, offset))
}

pub async fn parse_bulk_string(stream: &mut TcpStream) -> Result<(String, usize)> {
    if stream.read_u8().await? != BULK_STRING {
        bail!("invalid data");
    }
    let mut offset = 1;
    let (size, _offset) = parse_lenght(stream).await?;
    offset += _offset;
    let mut buf = vec![0; size];
    stream.read_exact(&mut buf).await?;
    stream.read_u16().await?; // consume \r\n
    offset += size + 2;

    Ok((String::from_utf8(buf)?, offset))
}

pub async fn parse_array(stream: &mut TcpStream) -> Result<(Vec<String>, usize)> {
    if stream.read_u8().await? != ARRAY {
        bail!("invalid data");
    }
    let mut offset = 1;

    let (size, _offset) = parse_lenght(stream).await?;
    offset += _offset;
    let mut array = Vec::with_capacity(size);

    for _ in 0..size {
        let (string, _offset) = parse_bulk_string(stream).await?;
        offset += _offset;
        array.push(string);
    }

    Ok((array, offset))
}

pub fn encoding_simple_string(s: &str) -> Bytes {
    let mut bytes = BytesMut::new();
    bytes.put_u8(b'+');
    bytes.extend_from_slice(s.as_bytes());
    bytes.extend_from_slice(b"\r\n");

    bytes.freeze()
}

pub fn encoding_bulk_string(s: &str) -> Bytes {
    let mut bytes = BytesMut::new();
    bytes.put_u8(b'$');
    bytes.extend_from_slice(s.len().to_string().as_bytes());
    bytes.extend_from_slice(b"\r\n");
    bytes.extend_from_slice(s.as_bytes());
    bytes.extend_from_slice(b"\r\n");

    bytes.freeze()
}

pub fn encoding_array(array: &[&str]) -> Bytes {
    let mut bytes = BytesMut::new();
    bytes.put_u8(b'*');
    bytes.extend_from_slice(array.len().to_string().as_bytes());
    bytes.extend_from_slice(b"\r\n");
    for s in array {
        bytes.extend_from_slice(encoding_bulk_string(s).as_ref());
    }

    bytes.freeze()
}

pub fn bulk_string(s: &str) -> Bytes {
    let mut bytes = BytesMut::new();
    bytes.put_u8(b'$');
    bytes.extend_from_slice(s.len().to_string().as_bytes());
    bytes.extend_from_slice(b"\r\n");
    bytes.extend_from_slice(s.as_bytes());
    bytes.extend_from_slice(b"\r\n");

    bytes.freeze()
}

pub fn rdb_file(data: &[u8]) -> Bytes {
    let mut bytes = BytesMut::new();
    bytes.put_u8(b'$');
    bytes.extend_from_slice(data.len().to_string().as_bytes());
    bytes.extend_from_slice(b"\r\n");
    bytes.extend_from_slice(data);

    bytes.freeze()
}

pub async fn receive_rdb_file(stream: &mut TcpStream) -> Result<Bytes> {
    if stream.read_u8().await? != BULK_STRING {
        bail!("invalid data");
    }
    let (size,_) = parse_lenght(stream).await?;
    println!("rdb file size: {}", size);
    let mut buf = vec![0; size];
    stream.read_exact(&mut buf).await?;

    Ok(Bytes::from(buf))
}
