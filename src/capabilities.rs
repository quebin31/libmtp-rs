use num_derive::{FromPrimitive, ToPrimitive};

#[derive(Debug, Clone, FromPrimitive, ToPrimitive)]
pub enum DeviceCap {
    GetPartialObject = 0,
    SendPartialObject,
    EditObjects,
    MoveObject,
    CopyObject,
}
