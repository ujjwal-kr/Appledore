pub fn encode_resp_simple_string(s: &str) -> Vec<u8> {
    let mut encoded: Vec<u8> = Vec::with_capacity(s.len() + 3);
    encoded.push(b'+');
    encoded.extend(s.as_bytes());
    encoded.extend(&[b'\r', b'\n']);
    encoded
}

pub fn encode_resp_error_string(s: &str) -> Vec<u8> {
    let mut encoded: Vec<u8> = vec![];
    encoded.push(b'-');
    encoded.extend(s.as_bytes());
    encoded.extend(&[b'\r', b'\n']);
    encoded
}

pub fn encode_resp_integer(value: &str) -> Vec<u8> {
    let mut encoded: Vec<u8> = vec![];
    encoded.push(b':');
    encoded.extend(value.as_bytes());
    encoded.extend(&[b'\r', b'\n']);
    encoded
}

pub fn encode_resp_bulk_string(data: String) -> Vec<u8> {
    let mut encoded: Vec<u8> = vec![];
    encoded.push(b'$');
    let len = data.len();
    encoded.extend(len.to_string().as_bytes());
    encoded.extend(&[b'\r', b'\n']);
    encoded.extend(data.as_bytes());
    encoded.extend(&[b'\r', b'\n']);
    encoded
}

pub fn empty_bulk_string() -> Vec<u8> {
    let mut encoded: Vec<u8> = vec![];
    encoded.push(b'$');
    encoded.extend("-1".to_string().as_bytes());
    encoded.extend(&[b'\r', b'\n']);
    encoded
}

pub fn encode_resp_arrays(arr: Vec<String>) -> Vec<u8> {
    let mut encoded: Vec<u8> = vec![];
    encoded.push(b'*');
    encoded.extend(arr.len().to_string().into_bytes());
    encoded.extend(&[b'\r', b'\n']);
    for item in arr {
        encoded.extend(encode_resp_bulk_string(item));
    }
    encoded
}
