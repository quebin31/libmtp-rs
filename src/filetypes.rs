use libmtp_sys as ffi;
use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::ToPrimitive;
use std::ffi::CStr;
use std::fmt::{self, Display};

#[derive(Debug, Clone, FromPrimitive, ToPrimitive)]
pub enum Filetype {
    Folder = 0,
    Wav,
    Mp3,
    Wma,
    Ogg,
    Audible,
    Mp4,
    UndefAudio,
    Wmv,
    Avi,
    Mpeg,
    Asf,
    Qt,
    UndefVideo,
    Jpeg,
    Jfif,
    Tiff,
    Bmp,
    Gif,
    Pict,
    Png,
    VCalendar1,
    VCalendar2,
    VCard2,
    VCard3,
    WindowsImageFormat,
    WinExec,
    Text,
    Html,
    Firmware,
    Aac,
    MediaCard,
    Flac,
    Mp2,
    M4a,
    Doc,
    Xml,
    Xls,
    Ppt,
    Mht,
    Jp2,
    Jpx,
    Album,
    Playlist,
    Unknown,
}

impl Display for Filetype {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ftype = self.to_u32().unwrap();
        unsafe {
            let description = ffi::LIBMTP_Get_Filetype_Description(ftype);

            let cstr = CStr::from_ptr(description);

            write!(f, "{}", cstr.to_str().unwrap())
        }
    }
}
