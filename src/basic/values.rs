use libmtp_sys as ffi;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

#[derive(Debug, Clone, Copy, FromPrimitive)]
pub enum DataType {
    I8 = 0,
    U8,
    I16,
    U16,
    I32,
    U32,
    I64,
    U64,
}

#[derive(Debug, Clone)]
pub struct Values<T: Copy> {
    max: T,
    min: T,
    step: T,
    vals: Vec<T>,
}

impl<T: Copy> Values<T> {
    pub fn max(&self) -> T {
        self.max
    }

    pub fn min(&self) -> T {
        self.min
    }

    pub fn step(&self) -> T {
        self.step
    }

    pub fn vals(&self) -> &[T] {
        &self.vals
    }
}

#[derive(Debug, Clone)]
pub struct AllowedValues {
    u8_values: Option<Values<u8>>,
    i8_values: Option<Values<i8>>,
    u16_values: Option<Values<u16>>,
    i16_values: Option<Values<i16>>,
    u32_values: Option<Values<u32>>,
    i32_values: Option<Values<i32>>,
    u64_values: Option<Values<u64>>,
    i64_values: Option<Values<i64>>,
    datatype: DataType,
    is_range: bool,
}

impl AllowedValues {
    pub fn is_range(&self) -> bool {
        self.is_range
    }

    pub fn datatype(&self) -> DataType {
        self.datatype
    }

    pub fn u8_values(&self) -> Option<&Values<u8>> {
        self.u8_values.as_ref()
    }

    pub fn i8_values(&self) -> Option<&Values<i8>> {
        self.i8_values.as_ref()
    }

    pub fn u16_values(&self) -> Option<&Values<u16>> {
        self.u16_values.as_ref()
    }

    pub fn i16_values(&self) -> Option<&Values<i16>> {
        self.i16_values.as_ref()
    }

    pub fn u32_values(&self) -> Option<&Values<u32>> {
        self.u32_values.as_ref()
    }

    pub fn i32_values(&self) -> Option<&Values<i32>> {
        self.i32_values.as_ref()
    }

    pub fn u64_values(&self) -> Option<&Values<u64>> {
        self.u64_values.as_ref()
    }

    pub fn i64_values(&self) -> Option<&Values<i64>> {
        self.i64_values.as_ref()
    }
}

impl Default for AllowedValues {
    fn default() -> Self {
        AllowedValues {
            u8_values: None,
            i8_values: None,
            u16_values: None,
            i16_values: None,
            u32_values: None,
            i32_values: None,
            u64_values: None,
            i64_values: None,
            datatype: DataType::I8,
            is_range: false,
        }
    }
}

impl AllowedValues {
    pub(crate) unsafe fn from_raw(raw: *mut ffi::LIBMTP_allowed_values_t) -> Option<Self> {
        if raw.is_null() {
            None
        } else {
            let len = (*raw).num_entries;
            let datatype = DataType::from_u32((*raw).datatype).unwrap();
            let is_range = (*raw).is_range != 0;

            let base = Self::default();
            let base = match datatype {
                DataType::I8 => Self {
                    datatype,
                    is_range,
                    i8_values: Some(Values {
                        max: (*raw).i8max,
                        min: (*raw).i8min,
                        step: (*raw).i8step,
                        vals: prim_array_ptr_to_vec!((*raw).i8vals, i8, len),
                    }),
                    ..base
                },

                DataType::U8 => Self {
                    datatype,
                    is_range,
                    u8_values: Some(Values {
                        max: (*raw).u8max,
                        min: (*raw).u8min,
                        step: (*raw).u8step,
                        vals: prim_array_ptr_to_vec!((*raw).u8vals, u8, len),
                    }),
                    ..base
                },

                DataType::I16 => Self {
                    datatype,
                    is_range,
                    i16_values: Some(Values {
                        max: (*raw).i16max,
                        min: (*raw).i16min,
                        step: (*raw).i16step,
                        vals: prim_array_ptr_to_vec!((*raw).i16vals, i16, len),
                    }),
                    ..base
                },

                DataType::U16 => Self {
                    datatype,
                    is_range,
                    u16_values: Some(Values {
                        max: (*raw).u16max,
                        min: (*raw).u16min,
                        step: (*raw).u16step,
                        vals: prim_array_ptr_to_vec!((*raw).u16vals, u16, len),
                    }),
                    ..base
                },

                DataType::I32 => Self {
                    datatype,
                    is_range,
                    i32_values: Some(Values {
                        max: (*raw).i32max,
                        min: (*raw).i32min,
                        step: (*raw).i32step,
                        vals: prim_array_ptr_to_vec!((*raw).i32vals, i32, len),
                    }),
                    ..base
                },

                DataType::U32 => Self {
                    datatype,
                    is_range,
                    u32_values: Some(Values {
                        max: (*raw).u32max,
                        min: (*raw).u32min,
                        step: (*raw).u32step,
                        vals: prim_array_ptr_to_vec!((*raw).u32vals, u32, len),
                    }),
                    ..base
                },

                DataType::I64 => Self {
                    datatype,
                    is_range,
                    i64_values: Some(Values {
                        max: (*raw).i64max,
                        min: (*raw).i64min,
                        step: (*raw).i64step,
                        vals: prim_array_ptr_to_vec!((*raw).i64vals, i64, len),
                    }),
                    ..base
                },

                DataType::U64 => Self {
                    datatype,
                    is_range,
                    u64_values: Some(Values {
                        max: (*raw).u64max,
                        min: (*raw).u64min,
                        step: (*raw).u64step,
                        vals: prim_array_ptr_to_vec!((*raw).u64vals, u64, len),
                    }),
                    ..base
                },
            };

            Some(base)
        }
    }
}
