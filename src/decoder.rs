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
