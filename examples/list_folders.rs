use anyhow::Error;
use libmtp_rs::{
    device::{raw::detect_raw_devices, StorageSort},
    object::filetypes::Filetype,
    object::Object,
    storage::files::File,
    storage::folders::Folder,
    storage::Parent,
    storage::Storage,
};

fn print_folder_tree_wfolder(folder: Option<Folder>, level: usize) {
    if let Some(folder) = folder {
        println!("{:>level$}{}", "", folder.name(), level = level);
        print_folder_tree_wfolder(folder.child(), level + 1);
        while let Some(sibling) = folder.sibling() {
            print_folder_tree_wfolder(Some(sibling), level);
        }
    }
}

fn print_folder_tree_wfiles(storage: &Storage, files: Vec<File>, level: usize) {
    for file in files {
        match file.ftype() {
            Filetype::Folder => {
                println!("{:>level$}{}", "", file.name(), level = level);
                let this_contents = storage.files_and_folders(Parent::Folder(file.id()));
                print_folder_tree_wfiles(storage, this_contents, level + 1);
            }

            _ => continue,
        }
    }
}

fn main() -> Result<(), Error> {
    let raw_devices = detect_raw_devices()?;
    let mtp_devices = raw_devices.into_iter().map(|raw| raw.open_uncached());

    for (i, mtp_device) in mtp_devices.enumerate() {
        if let Some(mut mtp_device) = mtp_device {
            mtp_device.update_storage(StorageSort::ByFreeSpace)?;
            let storage_pool = mtp_device.storage_pool();
            let (_, storage) = storage_pool.iter().next().expect("No storages");

            println!("{:#?}", storage);

            let root = storage.folder_list();
            if let Some(root) = root {
                print_folder_tree_wfolder(Some(root), 0);
            } else {
                let root_contents = storage.files_and_folders(Parent::Root);
                println!("/");
                print_folder_tree_wfiles(storage, root_contents, 1);
            }
        } else {
            println!("Couldn't open device {}", i + 1);
        }
    }

    Ok(())
}
