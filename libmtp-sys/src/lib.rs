#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

pub mod bindings;
pub use bindings::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init() {
        unsafe {
            LIBMTP_Init();
        }
    }
}
