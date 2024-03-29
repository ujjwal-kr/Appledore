use std::num::ParseIntError;

pub fn decode_get_pure_command(cmd: Vec<&str>) -> Vec<String> {
    let mut pure_cmds: Vec<String> = vec![];
    let mut i = 0;
    for c in cmd {
        if i % 2 == 0 && i != 0 {
            pure_cmds.push(c.to_owned())
        }
        i += 1;
    }
    pure_cmds
}

pub fn decode_array_indices(s: &str, e: &str, len: usize) -> Result<Vec<usize>, ParseIntError> {
    let mut decoded: Vec<usize> = vec![];
    let start: usize = s.parse()?;
    let mut endi: i64 = e.parse()?;
    if endi < 0 {
        endi = len as i64 + endi + 1
    }
    let end: usize = endi.try_into().unwrap();
    decoded.extend([start, end]);
    Ok(decoded)
}

pub fn parse_u64(s: &str) -> Result<u64, ParseIntError> {
    s.parse::<u64>()
}
