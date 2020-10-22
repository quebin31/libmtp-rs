use anyhow::Error;
use libmtp_rs::{
    device::{raw::detect_raw_devices, StorageSort},
    object::filetypes::Filetype,
    storage::Parent,
    util::CallbackReturn,
};
use std::io::Write;
use text_io::read;

fn main() -> Result<(), Error> {
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

        let root_contents: Vec<_> = storage
            .files_and_folders(Parent::Root)
            .into_iter()
            .filter(|file| !matches!(file.ftype(), Filetype::Folder))
            .collect();

        let no_digits = root_contents.len().to_string().len();

        for (idx, file) in root_contents.iter().enumerate() {
            println!("{:>d$}) {}", idx, file.name(), d = no_digits);
        }

        print!("Choose a file (type a number): ");
        std::io::stdout().lock().flush()?;
        let choosen: usize = read!();

        if let Some(file) = root_contents.get(choosen) {
            storage.get_file_to_path_with_callback(file, file.name(), |sent, total| {
                print!("\rProgress {}/{}", sent, total);
                std::io::stdout().lock().flush().expect("Failed to flush");
                CallbackReturn::Continue
            })?;

            println!();
        } else {
            println!("Invalid selection!");
        }
    } else {
        println!("Couldn't open device");
    }

    Ok(())
}
