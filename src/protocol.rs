use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

pub async fn read_varint<R: AsyncRead + Unpin>(stream: &mut R) -> std::io::Result<i32> {
    let mut num_read = 0;
    let mut result: u32 = 0;
    let mut buf = [0u8; 1];

    loop {
        stream.read_exact(&mut buf).await?;
        let value = (buf[0] & 0b01111111) as u32;
        if let Some(shifted) = value.checked_shl(7 * num_read) {
            result |= shifted;
        }

        num_read += 1;
        if num_read > 5 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "VarInt is too big",
            ));
        }

        if (buf[0] & 0b10000000) == 0 {
            break;
        }
    }

    Ok(result as i32)
}

pub async fn read_i32<R: AsyncRead + Unpin>(stream: &mut R) -> std::io::Result<i32> {
    let mut buf = [0u8; 4];
    stream.read_exact(&mut buf).await?;
    Ok(i32::from_be_bytes(buf))
}

pub async fn read_string<R: AsyncRead + Unpin>(stream: &mut R) -> std::io::Result<String> {
    let len = read_varint(stream).await?;
    let mut buf = vec![0u8; len as usize];
    stream.read_exact(&mut buf).await?;
    String::from_utf8(buf).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
}

pub async fn write_varint<W: AsyncWrite + Unpin>(stream: &mut W, mut value: i32) -> std::io::Result<()> {
    loop {
        let mut temp = (value & 0b01111111) as u8;
        value = (value >> 7) & (i32::MAX >> 6);
        if value != 0 {
            temp |= 0b10000000;
        }
        stream.write_all(&[temp]).await?;
        if value == 0 {
            break;
        }
    }
    Ok(())
}

pub fn write_varint_sync(buffer: &mut Vec<u8>, mut value: i32) {
    loop {
        let mut temp = (value & 0b01111111) as u8;
        value = (value >> 7) & (i32::MAX >> 6);
        if value != 0 {
            temp |= 0b10000000;
        }
        buffer.push(temp);
        if value == 0 {
            break;
        }
    }
}

pub fn write_string_sync(buffer: &mut Vec<u8>, val: &str) {
    write_varint_sync(buffer, val.len() as i32);
    buffer.extend_from_slice(val.as_bytes());
}

pub fn write_ushort_sync(buffer: &mut Vec<u8>, val: u16) {
    buffer.extend_from_slice(&val.to_be_bytes());
}

pub fn write_i16_sync(buffer: &mut Vec<u8>, val: i16) {
    buffer.extend_from_slice(&val.to_be_bytes());
}

pub fn write_i32_sync(buffer: &mut Vec<u8>, val: i32) {
    buffer.extend_from_slice(&val.to_be_bytes());
}

pub fn read_varint_slice(slice: &[u8], offset: &mut usize) -> Result<i32, std::io::Error> {
    let mut num_read = 0;
    let mut result: u32 = 0;

    loop {
        if *offset >= slice.len() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "VarInt slice EOF",
            ));
        }
        let byte = slice[*offset];
        *offset += 1;
        let value = (byte & 0b01111111) as u32;
        if let Some(shifted) = value.checked_shl(7 * num_read) {
            result |= shifted;
        }

        num_read += 1;
        if num_read > 5 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "VarInt is too big",
            ));
        }

        if (byte & 0b10000000) == 0 {
            break;
        }
    }

    Ok(result as i32)
}

pub fn read_string_slice(slice: &[u8], offset: &mut usize) -> Result<String, std::io::Error> {
    let len = read_varint_slice(slice, offset)? as usize;
    if *offset + len > slice.len() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::UnexpectedEof,
            "String slice EOF",
        ));
    }
    let s = String::from_utf8(slice[*offset..*offset + len].to_vec())
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    *offset += len;
    Ok(s)
}

pub fn read_i32_slice(slice: &[u8], offset: &mut usize) -> Result<i32, std::io::Error> {
    if *offset + 4 > slice.len() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::UnexpectedEof,
            "i32 slice EOF",
        ));
    }
    let mut buf = [0u8; 4];
    buf.copy_from_slice(&slice[*offset..*offset + 4]);
    *offset += 4;
    Ok(i32::from_be_bytes(buf))
}

#[allow(dead_code)]
pub fn read_i16_slice(slice: &[u8], offset: &mut usize) -> Result<i16, std::io::Error> {
    if *offset + 2 > slice.len() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::UnexpectedEof,
            "i16 slice EOF",
        ));
    }
    let mut buf = [0u8; 2];
    buf.copy_from_slice(&slice[*offset..*offset + 2]);
    *offset += 2;
    Ok(i16::from_be_bytes(buf))
}

pub fn read_u8_slice(slice: &[u8], offset: &mut usize) -> Result<u8, std::io::Error> {
    if *offset >= slice.len() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::UnexpectedEof,
            "u8 slice EOF",
        ));
    }
    let byte = slice[*offset];
    *offset += 1;
    Ok(byte)
}

pub fn read_i8_slice(slice: &[u8], offset: &mut usize) -> Result<i8, std::io::Error> {
    read_u8_slice(slice, offset).map(|b| b as i8)
}
