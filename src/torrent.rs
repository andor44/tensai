extern crate bencode;
extern crate crypto = "rust-crypto";
extern crate url;
extern crate curl;

use std::io::{File};
use std::num::ToStrRadix;
use std::str::raw::from_utf8_owned;
use std::iter::AdditiveIterator;
use std::rand::task_rng;
use url::Url;

use bencode::{FromBencode, Dict, Key, Bencode, List, ByteString, Number};
use crypto::digest::Digest;
use crypto::sha1::Sha1;

use super::{random_string, opt_finder};
use scrape::{TorrentScrape, ScrapeInfo};
use peer::Peer;
use announce::{AnnounceResponse, AnnounceResult};


#[deriving(Clone, Show)]
pub struct FileInfo {
    pub length: uint,
    pub md5sum: Option<String>,
    pub path: Option<Vec<String>>
}

#[deriving(Clone, Show)]
pub enum Payload {
    SingleFile(FileInfo),
    MultiFile(Vec<FileInfo>)
}

#[deriving(Clone, Show)]
pub struct MetaInfo {
    pub piece_length: int,
    pub pieces: Vec<u8>,
    pub private: bool,

    pub name: String,

    pub payload: Payload,
}

#[deriving(Clone, Show)]
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

#[allow(dead_code)]
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
                                payload: MultiFile(files)
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
                                payload: SingleFile(file)
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

    pub fn payload_size(&self) -> uint {
        match self.metainfo.payload {
            SingleFile(ref file) => file.length,
            MultiFile(ref files) => files.iter().map(|file| file.length).sum()
        }
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

pub struct Torrent {
    pub info: TorrentInfo,
    pub status: Status,
    pub destination_path: Path,
    pub traffic: TrafficInfo,
    pub session: SessionInfo,
}

pub struct TrafficInfo {
    pub uploaded_bytes: uint,
    pub downloaded_bytes: uint,
}

pub struct SessionInfo {
    pub peers: Vec<Peer>,
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
        let response = match curl::http::handle().get(scrape_url.as_slice()).exec() {
            Ok(response) => response,
            _ => return None
        };
        let body = response.get_body().clone();
        let data = match bencode::from_vec(Vec::from_slice(body)) {
            Ok(data) => data,
            _ => return None
        };
        let scrape: ScrapeInfo = match FromBencode::from_bencode(&data) {
            Some(scrape) => scrape,
            _ => return None
        };
        scrape.torrents.find(&self.info.infohash).map(|x| (*x).clone())
    }
    pub fn announce(&self, peer_id: String) -> Option<AnnounceResponse> {
        use announce::{Failure};
        let mut query = String::from_str("?");
        for &(key, ref value) in vec![("info_hash", self.info.urlencoded_hash()),
                                      ("peer_id", peer_id),
                                      ("port", 44000u.to_str()), 
                                      ("uploaded", self.traffic.uploaded_bytes.to_str()),
                                      ("downloaded", self.traffic.downloaded_bytes.to_str()),
                                      ("left", self.info.payload_size().to_str()),
                                      ("event", String::from_str("started")),
                                      ("key", "BqNcyuLEsZ".to_str()),//random_string(10)),
                                      ("compact", 1u.to_str())].iter() {
            query.push_str(format!("{}={}&", key, value).as_slice());
        }
        let url = self.info.announce.clone().append(query.as_slice());
        println!("{}", url);
        let response = match curl::http::handle().get(url.as_slice()).exec() {
            Ok(response) => response,
            _ => return None
        };
        let body = response.get_body().clone();
        let announce_response: AnnounceResponse = match bencode::from_vec(Vec::from_slice(body)) {
            Ok(bencode) => {
                match FromBencode::from_bencode(&bencode) {
                    Some(result) => result,
                    _ => Failure("error".to_str())
                }
            },
            _ => return None
        };
        Some(announce_response)
    }
}
