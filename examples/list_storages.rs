use anyhow::Error;
use libmtp_rs::device::{raw::detect_raw_devices, StorageSort};

fn main() -> Result<(), Error> {
    let raw_devices = detect_raw_devices()?;
    let mtp_devices = raw_devices.into_iter().map(|raw| raw.open_uncached());

    for (i, mtp_device) in mtp_devices.enumerate() {
        if let Some(mut mtp_device) = mtp_device {
            mtp_device.update_storage(StorageSort::ByFreeSpace)?;
            let storage_pool = mtp_device.storage_pool();

            for (i, (_id, storage)) in storage_pool.iter().enumerate() {
                println!("Storage {}:", i + 1);
                println!(
                    "  Description: {}",
                    storage.description().unwrap_or_else(|| "Unknown")
                );
                println!(
                    "  Max. capacity: {}",
                    bytefmt::format(storage.maximum_capacity())
                );
                println!(
                    "  Free space: {}",
                    bytefmt::format(storage.free_space_in_bytes())
                );
            }
        } else {
            println!("Coulnd't open device {}", i + 1);
        }
    }

    Ok(())
}
