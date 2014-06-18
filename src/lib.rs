#![crate_id = "tensai#0.1"]

extern crate bencode;
extern crate crypto = "rust-crypto";

use std::io::{File};

use bencode::{FromBencode, Dict, Key, Bencode, List, ByteString, Number};
use crypto::digest::Digest;
use crypto::sha1::Sha1;

pub struct FileInfo {
    pub length: int,
    pub md5sum: Option<String>,
    pub path: Option<Vec<String>>
}

pub struct MetaInfo {
    pub piece_length: int,
    pub pieces: Vec<u8>,
    pub private: bool,

    pub name: String,

    pub single_file: Option<FileInfo>,
    pub multifile: Option<Vec<FileInfo>>
}

pub struct Torrent {
    pub announce: String,
    pub creation_date: Option<int>,
    pub comment: Option<String>,
    pub created_by: Option<String>,
    encoding: Option<String>,
    pub metainfo: MetaInfo,
    infohash: Vec<u8>
}

impl FromBencode for Torrent {
    fn from_bencode(info: &bencode::Bencode) -> Option<Torrent> {
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
                let creation_date = match dict.find(&Key::from_str("creation date")) { Some(value) => FromBencode::from_bencode(value), _ => None };
                let comment = match dict.find(&Key::from_str("comment")) { Some(value) => FromBencode::from_bencode(value), _ => None };
                let created_by = match dict.find(&Key::from_str("creatd by")) { Some(value) => FromBencode::from_bencode(value), _ => None };
                let encoding = match dict.find(&Key::from_str("encoding")) { Some(value) => FromBencode::from_bencode(value), _ => None };

                Some(Torrent {
                    announce: announce,
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

pub fn parse_torrent(path: &Path) -> Option<Torrent> {
    match File::open(path).map(|mut file| { FromBencode::from_bencode(&bencode::from_vec(file.read_to_end().unwrap()).unwrap()) }) {
        Ok(torrentinfo) => torrentinfo,
        Err(e) => None
    }
}
