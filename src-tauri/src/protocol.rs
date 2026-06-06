use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::io::{Read, Write};

const PROTOCOL_VERSION_V1: u8 = 0b0001;
const CLIENT_FULL_REQUEST: u8 = 0b0001;
const CLIENT_AUDIO_ONLY_REQUEST: u8 = 0b0010;
const SERVER_FULL_RESPONSE: u8 = 0b1001;
const SERVER_ERROR_RESPONSE: u8 = 0b1111;
const POS_SEQUENCE: u8 = 0b0001;
const NEG_WITH_SEQUENCE: u8 = 0b0011;
const JSON_SERIALIZATION: u8 = 0b0001;
const GZIP_COMPRESSION: u8 = 0b0001;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ParsedResponse {
    pub code: i32,
    pub is_last_package: bool,
    pub sequence: i32,
    pub payload_size: u32,
    pub payload_msg: Option<Value>,
}

pub fn build_full_request(payload: &Value, seq: i32) -> Result<Vec<u8>, String> {
    let payload_bytes = gzip_compress(
        serde_json::to_string(payload)
            .map_err(|err| format!("序列化首包失败: {}", err))?
            .as_bytes(),
    )?;
    let mut request = Vec::new();
    request.extend(build_header(CLIENT_FULL_REQUEST, POS_SEQUENCE));
    request.extend(seq.to_be_bytes());
    request.extend((payload_bytes.len() as u32).to_be_bytes());
    request.extend(payload_bytes);
    Ok(request)
}

pub fn build_audio_request(seq: i32, audio_chunk: &[u8], is_last: bool) -> Result<Vec<u8>, String> {
    let payload = gzip_compress(audio_chunk)?;
    let flags = if is_last {
        NEG_WITH_SEQUENCE
    } else {
        POS_SEQUENCE
    };
    let signed_seq = if is_last { -seq } else { seq };

    let mut request = Vec::new();
    request.extend(build_header(CLIENT_AUDIO_ONLY_REQUEST, flags));
    request.extend(signed_seq.to_be_bytes());
    request.extend((payload.len() as u32).to_be_bytes());
    request.extend(payload);
    Ok(request)
}

pub fn parse_response(message: &[u8]) -> Result<ParsedResponse, String> {
    if message.len() < 4 {
        return Err("响应长度不足".to_string());
    }

    let mut response = ParsedResponse::default();
    let header_size = (message[0] & 0x0f) as usize * 4;
    if message.len() < header_size {
        return Err("响应 header 长度异常".to_string());
    }

    let message_type = message[1] >> 4;
    let flags = message[1] & 0x0f;
    let serialization = message[2] >> 4;
    let compression = message[2] & 0x0f;
    let mut offset = header_size;

    if flags & 0x01 != 0 {
        response.sequence = read_i32(message, offset)?;
        offset += 4;
    }
    if flags & 0x02 != 0 {
        response.is_last_package = true;
    }

    if message_type == SERVER_FULL_RESPONSE {
        response.payload_size = read_u32(message, offset)?;
        offset += 4;
    } else if message_type == SERVER_ERROR_RESPONSE {
        response.code = read_i32(message, offset)?;
        offset += 4;
        response.payload_size = read_u32(message, offset)?;
        offset += 4;
    }

    if offset >= message.len() {
        return Ok(response);
    }

    let mut payload = message[offset..].to_vec();
    if compression == GZIP_COMPRESSION {
        payload = gzip_decompress(&payload)?;
    }
    if serialization == JSON_SERIALIZATION {
        response.payload_msg = Some(
            serde_json::from_slice(&payload)
                .map_err(|err| format!("解析响应 JSON 失败: {}", err))?,
        );
    }
    Ok(response)
}

fn build_header(message_type: u8, flags: u8) -> [u8; 4] {
    [
        (PROTOCOL_VERSION_V1 << 4) | 1,
        (message_type << 4) | flags,
        (JSON_SERIALIZATION << 4) | GZIP_COMPRESSION,
        0,
    ]
}

fn gzip_compress(data: &[u8]) -> Result<Vec<u8>, String> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder
        .write_all(data)
        .map_err(|err| format!("GZIP 压缩失败: {}", err))?;
    encoder
        .finish()
        .map_err(|err| format!("GZIP 结束失败: {}", err))
}

fn gzip_decompress(data: &[u8]) -> Result<Vec<u8>, String> {
    let mut decoder = GzDecoder::new(data);
    let mut output = Vec::new();
    decoder
        .read_to_end(&mut output)
        .map_err(|err| format!("GZIP 解压失败: {}", err))?;
    Ok(output)
}

fn read_i32(message: &[u8], offset: usize) -> Result<i32, String> {
    let bytes = message
        .get(offset..offset + 4)
        .ok_or_else(|| "响应 i32 字段长度不足".to_string())?;
    Ok(i32::from_be_bytes(
        bytes
            .try_into()
            .map_err(|_| "响应 i32 字段异常".to_string())?,
    ))
}

fn read_u32(message: &[u8], offset: usize) -> Result<u32, String> {
    let bytes = message
        .get(offset..offset + 4)
        .ok_or_else(|| "响应 u32 字段长度不足".to_string())?;
    Ok(u32::from_be_bytes(
        bytes
            .try_into()
            .map_err(|_| "响应 u32 字段异常".to_string())?,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn full_request_has_expected_header_and_sequence() {
        let packet = build_full_request(&json!({"audio": {"format": "pcm"}}), 7).unwrap();
        assert_eq!(packet[0], 0x11);
        assert_eq!(packet[1], 0x11);
        assert_eq!(packet[2], 0x11);
        assert_eq!(i32::from_be_bytes(packet[4..8].try_into().unwrap()), 7);
    }

    #[test]
    fn last_audio_request_uses_negative_sequence() {
        let packet = build_audio_request(9, b"pcm", true).unwrap();
        assert_eq!(packet[1], 0x23);
        assert_eq!(i32::from_be_bytes(packet[4..8].try_into().unwrap()), -9);
    }

    #[test]
    fn parses_server_full_response() {
        let payload = gzip_compress(br#"{"result":{"text":"hello"}}"#).unwrap();
        let mut packet = Vec::new();
        packet.extend([0x11, 0x91, 0x11, 0x00]);
        packet.extend(3_i32.to_be_bytes());
        packet.extend((payload.len() as u32).to_be_bytes());
        packet.extend(payload);

        let parsed = parse_response(&packet).unwrap();
        assert_eq!(parsed.sequence, 3);
        assert_eq!(parsed.payload_msg.unwrap()["result"]["text"], "hello");
    }
}
