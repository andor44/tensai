#![crate_id = "tensai#0.1"]

extern crate bencode;

use std::io::{File};

use bencode::{FromBencode, Dict, Key, Bencode, List, ByteString};

pub struct FileInfo {
    pub length: int,
    pub md5sum: Option<~str>,
    pub path: Option<Vec<~str>>
}

pub struct MetaInfo {
    pub piece_length: int,
    pub pieces: Vec<u8>,
    pub private: bool,

    pub name: ~str,

    pub single_file: Option<FileInfo>,
    pub multifile: Option<Vec<FileInfo>>
}

pub struct TorrentInfo {
    pub announce: ~str,
    pub creation_date: Option<int>,
    pub comment: Option<~str>,
    pub created_by: Option<~str>,
    encoding: Option<~str>,
    pub metainfo: MetaInfo
}

impl FromBencode for TorrentInfo {
    fn from_bencode(info: &bencode::Bencode) -> Option<TorrentInfo> {
        match info {
            &Dict(ref dict) => {
                let metainfo = match dict.find(&Key::from_str("info")) {
                    Some(&Dict(ref info)) => {
                        let piece_length = FromBencode::from_bencode(info.find(&Key::from_str("piece length")).unwrap()).unwrap();
                        let pieces = match info.find(&Key::from_str("pieces")) {
                            Some(&ByteString(ref vec)) => vec, _ => fail!("pieces is not a bytestring")
                        };
                        let name = FromBencode::from_bencode(info.find(&Key::from_str("name")).unwrap()).unwrap();

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
                            MetaInfo {
                                piece_length: piece_length,
                                pieces: pieces.clone(),
                                private: false,
                                name: name,
                                single_file: None,
                                multifile: Some(files)
                            }
                        }
                        else {
                            let length = FromBencode::from_bencode(info.find(&Key::from_str("length")).unwrap()).unwrap();
                            let md5sum = match dict.find(&Key::from_str("md5sum")) { Some(value) => FromBencode::from_bencode(value), _ => None };
                            let file = FileInfo {
                                length: length,
                                md5sum: md5sum,
                                path: None
                            };
                            MetaInfo {
                                piece_length: piece_length,
                                pieces: pieces.clone(),
                                private: false,
                                name: name,
                                single_file: Some(file),
                                multifile: None
                            }
                        }
                    },
                    _ => fail!("Torrent is missing metainfo dict!")
                };

                let announce: ~str = FromBencode::from_bencode(dict.find(&Key::from_str("announce")).unwrap()).unwrap();
                let creation_date = match dict.find(&Key::from_str("creation date")) { Some(value) => FromBencode::from_bencode(value), _ => None };
                let comment = match dict.find(&Key::from_str("comment")) { Some(value) => FromBencode::from_bencode(value), _ => None };
                let created_by = match dict.find(&Key::from_str("creatd by")) { Some(value) => FromBencode::from_bencode(value), _ => None };
                let encoding = match dict.find(&Key::from_str("encoding")) { Some(value) => FromBencode::from_bencode(value), _ => None };

                Some(TorrentInfo {
                    announce: announce,
                    creation_date: creation_date,
                    comment: comment,
                    created_by: created_by,
                    encoding: encoding,
                    metainfo: metainfo
                })
            }
            _ => None
        }
    }
}

pub fn parse_torrent(path: &Path) -> Option<TorrentInfo> {
    match File::open(path).map(|mut file| { FromBencode::from_bencode(&bencode::from_vec(file.read_to_end().unwrap()).unwrap()) }) {
        Ok(torrentinfo) => torrentinfo,
        Err(e) => None
    }
}
