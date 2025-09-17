/// Ascender AgentHub (c) 2025
/// Proprietary software. All rights reserved.
/// See the project license for terms of use.

use crate::attp::shared::errors::DecodeError;

#[inline]
pub fn read_var_u32(buf: &[u8], off: &mut usize) -> Result<u32, DecodeError> {
    let mut x: u32 = 0;
    let mut s: u32 = 0;
    for i in 0..5 { // max 5 bytes for u32
        let b = *buf.get(*off + i).ok_or(DecodeError::Incomplete)?;
        x |= ((b & 0x7F) as u32) << s;
        if (b & 0x80) == 0 {
            *off += i + 1;
            return Ok(x);
        }
        s += 7;
    }
    Err(DecodeError::BadVarint)
}


#[inline]
pub fn take_exact<'a>(buf: &'a [u8], off: &mut usize, n: usize) -> Result<&'a [u8], DecodeError> {
    let end = *off + n;
    let slice = buf.get(*off..end).ok_or(DecodeError::Incomplete)?;
    *off = end;
    Ok(slice)
}


#[inline]
pub fn write_var_u32(out: &mut Vec<u8>, mut v: u32) {
    while v >= 0x80 {
        out.push(((v as u8) & 0x7F) | 0x80);
        v >>= 7;
    }
    out.push(v as u8);
}


#[inline]
pub fn parse_connection_string(con_str: String) -> Result<(String, u16), &'static str> {
    if !con_str.starts_with("attp://") {
        return Err("Failed to parse attp connection string!");
    }

    let clean_con_str = con_str.strip_prefix("attp://");

    if clean_con_str.is_none() {
        return Err("Attp connection string should contain `attp://` prefix");        
    }

    let split: Vec<&str> = clean_con_str.unwrap().split(":").collect();

    let mut host: String = String::new();
    let port: u16;

    if split.len() < 2 {
        return Err("ATTP connection string uses wrong format!");
    }

    host.push_str(split.get(0).unwrap());
    
    if let Ok(parsed) = split.get(1).unwrap().parse::<u16>() {
        port = parsed;
    }
    else {
        return Err("Port slice in ATTP connection string must be integer");
    }

    Ok((host, port))
    
}