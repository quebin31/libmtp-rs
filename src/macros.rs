macro_rules! cstr_to_u8vec {
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

macro_rules! prim_array_ptr_to_vec {
    ($ptr:expr, $to:ty, $len:expr) => {{
        let mut vec = Vec::new();

        for offset in 0..($len as isize) {
            let item = *$ptr.offset(offset);
            vec.push(item as $to);
        }

        vec
    }};
}
