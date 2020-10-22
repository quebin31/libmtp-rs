use anyhow::Error;
use libmtp_rs::object::filetypes::Filetype;
use libmtp_rs::storage::{files::FileMetadata, Parent};
use libmtp_rs::{
    device::{raw::detect_raw_devices, StorageSort},
    util::CallbackReturn,
};
use std::{fs::File, io::Write, path::Path};

fn main() -> Result<(), Error> {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 2 {
        println!("Error: please provide a path to the file to send");
        return Ok(());
    }

    let raw_devices = detect_raw_devices()?;
    let mtp_device = if let Some(raw) = raw_devices.get(0) {
        raw.open_uncached()
    } else {
        println!("No devices");
        return Ok(());
    };

    if let Some(mut mtp_device) = mtp_device {
        mtp_device.update_storage(StorageSort::ByFreeSpace)?;

        let storage_pool = mtp_device.storage_pool();
        let (_, storage) = storage_pool.iter().next().expect("No storage");

        let path = Path::new(&args[1]);
        let file = File::open(path)?;
        let metadata = file.metadata()?;

        let metadata = FileMetadata {
            file_size: metadata.len(),
            file_name: path.file_name().unwrap().to_str().expect("Invalid UTF-8"),
            file_type: Filetype::Text,
            modification_date: metadata.modified()?.into(),
        };

        storage.send_file_from_path_with_callback(
            path,
            Parent::Root,
            metadata,
            |sent, total| {
                print!("\rProgress {}/{}", sent, total);
                std::io::stdout().lock().flush().expect("Failed to flush");
                CallbackReturn::Continue
            },
        )?;

        println!()
    } else {
        println!("Couldn't open device");
    }

    Ok(())
}
