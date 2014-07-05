extern crate bencode;
extern crate crypto = "rust-crypto";
extern crate url;
extern crate curl;

use std::io::{File};
use std::num::ToStrRadix;
use std::io::net::ip::SocketAddr;
use std::io::net::tcp::{TcpStream, TcpListener, TcpAcceptor};
use std::str::raw::from_utf8_owned;
use url::Url;

use bencode::{FromBencode, Dict, Key, Bencode, List, ByteString, Number};
use crypto::digest::Digest;
use crypto::sha1::Sha1;

use scrape::{TorrentScrape, ScrapeInfo};


#[deriving(Clone)]
pub struct FileInfo {
    pub length: int,
    pub md5sum: Option<String>,
    pub path: Option<Vec<String>>
}

#[deriving(Clone)]
pub struct MetaInfo {
    pub piece_length: int,
    pub pieces: Vec<u8>,
    pub private: bool,

    pub name: String,

    pub single_file: Option<FileInfo>,
    pub multifile: Option<Vec<FileInfo>>
}

#[deriving(Clone)]
pub struct TorrentInfo {
    pub announce: String,
    pub announce_list: Option<Vec<String>>,
    pub creation_date: Option<int>,
    pub comment: Option<String>,
    pub created_by: Option<String>,
    #[allow(dead_code)]
    encoding: Option<String>,
    pub metainfo: MetaInfo,
    pub infohash: Vec<u8>
}

fn opt_finder<T: FromBencode>(dict: &Dict, key: &str) -> Option<T> {
    match dict.find(&Key::from_str(key)) {
        Some(value) => FromBencode::from_bencode(value),
        _ => None
    }
}

fn opt_finder_key<T: FromBencode>(dict: &Dict, key: &Key) -> Option<T> {
    match dict.find(key) {
        Some(value) => FromBencode::from_bencode(value),
        _ => None
    }
}

impl FromBencode for TorrentInfo {
    fn from_bencode(info: &bencode::Bencode) -> Option<TorrentInfo> {
        match info {
            &Dict(ref dict) => {
                let (metainfo, infohash) = match dict.find(&Key::from_str("info")) {
                    Some(ref infodict) => {
                        let info = match infodict {
                            &&Dict(ref info) => info,
                            _ => fail!("`info` not a dict")
                        };
                        let mut hasher = Sha1::new();
                        hasher.input(infodict.to_bytes().unwrap().as_slice());
                        let mut hash = [0u8, ..20];
                        hasher.result(hash);
                        let infohash = hash;
                        let piece_length = FromBencode::from_bencode(info.find(&Key::from_str("piece length")).unwrap()).unwrap();
                        let pieces = match info.find(&Key::from_str("pieces")) {
                            Some(&ByteString(ref vec)) => vec, _ => fail!("`pieces` is not a bytestring")
                        };
                        let name = FromBencode::from_bencode(info.find(&Key::from_str("name")).unwrap()).unwrap();
                        let private = match info.find(&Key::from_str("private")) {
                            Some(&Number(x)) if x > 0 => true,
                            _ => false
                        };

                        if info.contains_key(&Key::from_str("files")) {
                            let files = match info.find(&Key::from_str("files")) {
                                Some(&List(ref filelist)) => {
                                    let mut files = Vec::new();
                                    for file in filelist.iter() {
                                        let file = match file { &Dict(ref file) => file, _ => fail!("non-dict in file list") };
                                        let length = FromBencode::from_bencode(file.find(&Key::from_str("length")).unwrap()).unwrap();
                                        let path = FromBencode::from_bencode(file.find(&Key::from_str("path")).unwrap()).unwrap();
                                        let md5sum = match dict.find(&Key::from_str("md5sum")) { Some(value) => FromBencode::from_bencode(value), _ => None };
                                        let fileinfo = FileInfo {
                                            length: length,
                                            path: path,
                                            md5sum: md5sum
                                        };
                                        files.push(fileinfo);
                                    }
                                    files
                                }
                                _ => fail!("'files' is not a list")
                            };
                            (MetaInfo {
                                piece_length: piece_length,
                                pieces: pieces.clone(),
                                private: private,
                                name: name,
                                single_file: None,
                                multifile: Some(files)
                            }, infohash)
                        }
                        else {
                            let length = FromBencode::from_bencode(info.find(&Key::from_str("length")).unwrap()).unwrap();
                            let md5sum = match dict.find(&Key::from_str("md5sum")) { Some(value) => FromBencode::from_bencode(value), _ => None };
                            let file = FileInfo {
                                length: length,
                                md5sum: md5sum,
                                path: None
                            };
                            (MetaInfo {
                                piece_length: piece_length,
                                pieces: pieces.clone(),
                                private: private,
                                name: name,
                                single_file: Some(file),
                                multifile: None
                            }, infohash)
                        }
                    },
                    _ => fail!("Torrent is missing metainfo dict!")
                };

                let announce: String = FromBencode::from_bencode(dict.find(&Key::from_str("announce")).unwrap()).unwrap();
                let announce_list = opt_finder(dict, "announce-list");
                let creation_date = opt_finder(dict, "creation date");
                let comment = opt_finder(dict, "comment");
                let created_by = opt_finder(dict, "created by");
                let encoding = opt_finder(dict, "encoding");

                Some(TorrentInfo {
                    announce: announce,
                    announce_list: announce_list,
                    creation_date: creation_date,
                    comment: comment,
                    created_by: created_by,
                    encoding: encoding,
                    metainfo: metainfo,
                    infohash: Vec::from_slice(infohash)
                })
            }
            _ => None
        }
    }
}

impl TorrentInfo {
    pub fn read(path: &Path) -> Option<TorrentInfo> {
        match File::open(path).map(|mut file| { FromBencode::from_bencode(&bencode::from_vec(file.read_to_end().unwrap()).unwrap()) }) {
            Ok(torrentinfo) => torrentinfo,
            Err(_) => None
        }
    }

    pub fn hash_string(&self) -> String {
        to_hex(self.infohash.as_slice())
    }

    pub fn urlencoded_hash(&self) -> String {
        unsafe { url::encode_component(from_utf8_owned(self.infohash.clone()).as_slice()) }
    }

}

fn to_hex(rr: &[u8]) -> String {
    let mut s = String::new();
    for b in rr.iter() {
        let hex = (*b as uint).to_str_radix(16u);
        if hex.len() == 1 {
            s.push_char('0');
        }
        s.push_str(hex.as_slice());
    }
    return s;
}

pub enum Status {
    Stopped, // The torrent is completely stopped, no TX/RX
    Downloading, // The torrent is downloading
    Seeding // The torrent is downloading
}

pub struct Peer {
    pub address: SocketAddr
}

pub struct Torrent {
    pub info: TorrentInfo,
    pub status: Status,
}


impl Torrent {
    // oh god, i hope this goes away soon
    pub fn scrape_url(&self) -> Url {
        from_str(self.info.announce.replace("announce", "scrape").as_slice()).unwrap()
    }
    fn _scrape_url(&self) -> String {
        self.info.announce.replace("announce", "scrape")
    }
    pub fn scrape(&self) -> Option<TorrentScrape> {
        let mut scrape_url = self._scrape_url();
        scrape_url = scrape_url.append(String::from_str("?info_hash=").append(self.info.urlencoded_hash().as_slice()).as_slice());
        let response = curl::http::handle().get(scrape_url.as_slice()).exec().unwrap();
        let body = response.get_body().clone();
        let scrape_result = match bencode::from_vec(Vec::from_slice(body)) {
            Ok(Dict(ref dict)) => {
                let filesdict = match dict.find(&Key::from_str("files")) {
                    Some(&Dict(ref filesdict)) => filesdict,
                    _ => return None
                };
                match filesdict.find(&Key::from_slice(self.info.infohash.as_slice())) {
                    Some(&Dict(ref torrentinfo)) => {
                        Some(TorrentScrape {
                            complete: opt_finder(torrentinfo, "complete").unwrap(),
                            downloaded: opt_finder(torrentinfo, "downloaded").unwrap(),
                            incomplete: opt_finder(torrentinfo, "incomplete").unwrap(),
                            name: opt_finder(torrentinfo, "name")
                        })
                    },
                    _ => None
                }
            },
            _ => None
        };
        scrape_result
    }
}
