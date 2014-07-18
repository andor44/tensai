use std::rand::random;
use std::io;
use std::io::{File, IoResult};

use torrent::{Torrent, TorrentInfo, Stopped, TrafficInfo, SessionInfo, SingleFile, MultiFile};
use super::CLIENT_VERSION;


// XXX: other clients just use random bytes and not a valid utf8 string? need to investigate more
pub struct Client {
    client_rand: String,
    torrents: Vec<Torrent>,
}

impl Client {
    
    pub fn new() -> Client {
        Client {
            torrents: Vec::new(),
            client_rand: format!("{:06u}{:06u}", random::<uint>() % 1000000, random::<uint>() % 1000000)
        }
    }

    /// Initialize a client with the specified `client_ran` value.
    /// This is useful if you want to retain the same "client identiy"
    /// across different instances. The value is expected to be 12 random
    /// ASCII digits.
    pub fn with_client_rand(client_rand: String) -> Client {
        assert!(client_rand.len() == 12)
        Client {
            torrents: Vec::new(),
            client_rand: client_rand
        }
    }

    /// Azureus-style peer identifier generated based on library version
    /// and the client random value
    pub fn peer_id(&self) -> String {
        format!("-TE{:04u}-{:s}", CLIENT_VERSION, self.client_rand)
    }

    /// Add a torrent based on information found in `info`
    /// It'll be set to `Stopped` state by default
    /// It is assumed that `destination_path` is a valid path and a directory
    /// with read/write permissions.
    ///
    /// This method will automatically create the subtree for multifile
    /// torrents
    ///
    /// Note that this also means that it'll overwrite any existing content
    /// under the same path with the same name
    pub fn add_torrent<'a>(&'a mut self, info: &TorrentInfo, destination_path: Path) -> IoResult<&'a Torrent> {
        match info.metainfo.payload {
            MultiFile(ref files) => {
                for file in files.iter() {
                    let path = match file.path {
                        Some(ref path) => path,
                        None => continue
                    };
                    let fullpath = destination_path.join_many(path.as_slice());
                    try!(io::fs::mkdir_recursive(&fullpath.dir_path(), io::UserRWX));
                    try!(File::create(&fullpath));
                }
            }
            SingleFile(_) => {
                let file_destination = destination_path.join(info.metainfo.name.clone());
                try!(File::create(&file_destination));
            }
        }
        self.torrents.push(Torrent {
            info: info.clone(),
            status: Stopped,
            destination_path: destination_path,
            traffic: TrafficInfo { downloaded_bytes: 0, uploaded_bytes: 0 },
            session: SessionInfo { peers: Vec::new() }
        });
        // unwrap because either the call to push fails or it's safe to call it
        // last although i'd prefer if push returned a reference to it
        Ok(self.torrents.last().unwrap())
    }

    /// Get the list of torrents managed by this client
    pub fn get_torrents<'a>(&'a mut self) -> &'a mut Vec<Torrent> {
        &mut self.torrents
    }
}
