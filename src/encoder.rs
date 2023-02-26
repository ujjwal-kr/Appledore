pub fn encode_resp_simple_string(s: &str) -> Vec<u8> {
    let mut encoded: Vec<u8> = vec![];
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

pub fn encode_resp_integer(number: i64) -> Vec<u8> {
    let mut encoded: Vec<u8> = vec![];
    encoded.push(b':');
    encoded.extend(number.to_be_bytes());
    encoded.extend(&[b'\r', b'\n']);
    encoded
}

pub fn encode_resp_bulk_string(bytes: Vec<u8>) -> Vec<u8> {
    let mut encoded: Vec<u8> = vec![];
    encoded.push(b'$');
    let len = String::from_utf8(bytes.clone()).unwrap().len();
    encoded.extend(len.to_be_bytes());
    encoded.extend(&[b'\r', b'\n']);
    encoded.extend(bytes);
    encoded.extend(&[b'\r', b'\n']);
    encoded
}

pub fn encode_resp_array(arr: Vec<String>) -> Vec<u8> {
    let mut encoded: Vec<u8> = vec![];
    encoded.push(b'*');
    encoded.extend(arr.len().to_be_bytes());
    encoded.extend(&[b'\r', b'\n']);
    for item in arr {
        encoded.extend(encode_resp_bulk_string(item.into_bytes()));
    }
    encoded
}
