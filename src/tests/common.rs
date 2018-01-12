pub fn dummy_vector(size: usize) -> Vec<u8> {
    let mut v: Vec<u8> = Vec::new();

    (0..(size / 4)).for_each(|_| 
    {
        v.push(0xBE);
        v.push(0xBA);
        v.push(0xFE);
        v.push(0xCA);
    });

    v
}

pub fn dump_vector(v: &[u8]) -> String {
    v.iter().enumerate()
            .map(|(i, b)| 
            {
                    let mut s = format!("{:02X}", b);
                    if i > 1 && i % 16 == 0 { s += "\n"; }  else { s += " "};
                    s
            }).collect::<Vec<String>>().join("")
}