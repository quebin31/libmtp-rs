use num_derive::{FromPrimitive, ToPrimitive};

/// Possible device capabilities.
#[derive(Debug, Clone, FromPrimitive, ToPrimitive)]
pub enum DeviceCapability {
    /// This capability tells whether you can get partial objects.
    GetPartialObject = 0,
    /// This capability tells whether you can send partial objects.
    SendPartialObject,
    /// This capability tells whether you can edit objects in-place on a device.
    EditObjects,
    /// This capability tells whether you can move an object.
    MoveObject,
    /// This capability tells whether you can copy an object.
    CopyObject,
}
