fn empty_rdb() -> Vec<u8> {
    let data_string = "524544495330303131fa0972656469732d76657205372e322e30fa0a72656469732d62697473c040fa056374696d65c26d08bc65fa08757365642d6d656dc2b0c41000fa08616f662d62617365c000fff06e3bfec0ff5aa2".to_string();
    let mut data = Vec::new();

    for i in 0..data_string.len() / 2 {
        let byte = u8::from_str_radix(&data_string[i * 2..i * 2 + 2], 16).unwrap();
        data.push(byte);
    }

    data
}
