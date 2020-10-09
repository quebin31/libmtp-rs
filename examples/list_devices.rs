use anyhow::Error;
use libmtp_rs::device::raw::detect_raw_devices;

fn main() -> Result<(), Error> {
    let raw_devices = detect_raw_devices()?;
    let mtp_devices = raw_devices
        .into_iter()
        .inspect(|raw| println!("Found a device with an mtp descriptor:\n{:#?}", raw))
        .map(|raw| raw.open_uncached());

    for (i, mtp_device) in mtp_devices.enumerate() {
        if let Some(mtp_device) = mtp_device {
            let name = if let Ok(fname) = mtp_device.get_friendly_name() {
                fname
            } else {
                format!(
                    "{} {}",
                    mtp_device.manufacturer_name()?,
                    mtp_device.model_name()?
                )
            };

            println!("Device {}: {}", i + 1, name);
        } else {
            println!("Coulnd't open device {}", i + 1);
        }
    }

    Ok(())
}
