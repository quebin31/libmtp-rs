//! Describes the device capabilities, some devices may not necessarily implement
//! or support certain capabilities, like copying or moving objects.

use num_derive::{FromPrimitive, ToPrimitive};

/// Supported `libmtp` device capabilities, you can test if an MTP device supports
/// one of those with [`MtpDevice::check_capability`](../struct.MtpDevice.html#method.check_capability)
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
