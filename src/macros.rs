#[macro_export]
macro_rules! c_charp_to_u8v {
    ($ptr:expr) => {{
        let mut u8_vec = Vec::new();

        let mut offset = 0;
        let mut ch = *$ptr.offset(offset);
        while ch as u8 != 0x0 {
            u8_vec.push(ch as u8);
            offset += 1;
            ch = *$ptr.offset(offset);
        }

        u8_vec
    }};
}
