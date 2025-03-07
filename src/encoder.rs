use std::io::Write;

pub fn encode_resp_simple_string(s: &str) -> Vec<u8> {
    let mut encoded: Vec<u8> = Vec::with_capacity(s.len() + 3);
    encoded.push(b'+');
    encoded.extend(s.as_bytes());
    encoded.extend(&[b'\r', b'\n']);
    encoded
}

pub fn encode_resp_error_string(s: &str) -> Vec<u8> {    
    let mut encoded = Vec::with_capacity(s.len() + 3);
    encoded.push(b'-');    
    encoded.extend_from_slice(s.as_bytes());
    encoded.extend_from_slice(b"\r\n");
    encoded    
}

pub fn encode_resp_integer(value: &str) -> Vec<u8> {
    let mut encoded = Vec::with_capacity(value.len() + 3);
    encoded.push(b':');    
    encoded.extend_from_slice(value.as_bytes());
    encoded.extend_from_slice(b"\r\n");
    encoded    
}

pub fn encode_resp_bulk_string(data: String) -> Vec<u8> {
    let len = data.len();    
    let mut encoded = Vec::with_capacity(len + len.to_string().len() + 5);
    encoded.push(b'$');    
    write!(&mut encoded, "{}", len).unwrap();
    encoded.extend_from_slice(b"\r\n");
    encoded.extend_from_slice(data.as_bytes());
    encoded.extend_from_slice(b"\r\n");
    encoded    
}

pub fn empty_bulk_string() -> Vec<u8> {
    let mut encoded = Vec::with_capacity(5);
    encoded.push(b'$');    
    encoded.extend_from_slice(b"-1");
    encoded.extend_from_slice(b"\r\n");
    encoded    
}

pub fn encode_resp_arrays(arr: Vec<String>) -> Vec<u8> {
    let mut encoded = Vec::with_capacity(arr.len() * 5);
    encoded.push(b'*');    
    write!(&mut encoded, "{}", arr.len()).unwrap();
    encoded.extend_from_slice(b"\r\n");
    for item in arr {
        encoded.extend(encode_resp_bulk_string(item));
    }
    encoded
}

pub fn encode_resp_empty_array() -> Vec<u8> {
    let encoded: Vec<u8> = vec![b'*', b'0', b'\r', b'\n'];    
    encoded    
}
