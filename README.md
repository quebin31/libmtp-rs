# libmtp-rs 
This crate aims to provide a flexible and high-level interface to the `libmtp` library, at
its current state is alpha software and may not be used in production since some features
are still missing, said this contributions are welcome.

## Requirements
You need to have `libmtp` installed (minimum version 1.1.15), and have `pkg-config`
properly configured (`pkg-config --libs libmtp` should output something reasonable).

## Available APIs 
- [x] Internals API
- [x] Device properties API 
- [x] Object management API
    - [x] Get properties/attributes 
    - [x] Set properties/attributes 
    - [x] Move, copy, delete object
    - [ ] Get/send partial object 
    - [ ] Truncate object
- [ ] Storage API
    - [x] Format storage
    - [ ] File management
        - [x] List files 
        - [x] Send files 
        - [x] Receive files 
        - [x] Rename files
        - [ ] Sample data
        - [ ] Events 
        - [ ] Thumbnails
    - [x] Folder management 
        - [x] List folders
        - [x] Create folder 
        - [x] Rename folder
    - [ ] Track management
        - [ ] List tracks
        - [ ] Send tracks
        - [ ] Receive tracks 
        - [ ] Rename track
        - [ ] Update metadata
    - [ ] Album management
        - [ ] List albums
        - [ ] Create album 
        - [ ] Update album
        - [ ] Rename album
    - [ ] Playlist management
        - [ ] List playlists
        - [ ] Create playlist 
        - [ ] Update playlist
        - [ ] Rename playlist
- [ ] Custom operations API (c_variadic)

    
## Contributing 
`libmtp-rs` is an open source project! If you'd like to contribute, check any 
open issue or create one, current API design is open to discussion. Note that 
the code you submit in PRs is assumed to be licensed under the MIT License.

## License 
This crate is licensed under the terms of the MIT License.

See [LICENSE](LICENSE) to see the full text.