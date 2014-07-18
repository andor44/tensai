extern crate bencode;

use super::opt_finder;
use peer::Peer;

use bencode::{FromBencode, Dict, Key, List, ByteString, Bencode};


#[deriving(Show)]
pub enum AnnounceResponse {
    Failure(String),
    Success(AnnounceResult)
}

#[deriving(Show)]
pub struct AnnounceResult {
    pub warning_message: Option<String>,
    pub interval: uint,
    pub min_interval: Option<uint>,
    pub tracker_id: Option<String>,
    pub complete: uint,
    pub incomplete: uint,
    pub peers: Vec<Peer>,
}

impl FromBencode for AnnounceResponse {
    fn from_bencode(bencode: &Bencode) -> Option<AnnounceResponse> {
        match bencode {
            &Dict(ref dict) => {
                match dict.find(&Key::from_str("failure reason")) {
                    Some(&ByteString(ref message)) => return Some(Failure(String::from_utf8((message.clone())).ok().expect("unknown error"))),
                    Some(_) => return Some(Failure("unknown error".to_str())),
                    None => ()
                }
                let mut peers = Vec::new();
                let peerlist: Option<&Bencode> = dict.find(&Key::from_str("peers"));
                match peerlist {
                    Some(&List(ref peervec)) => {
                        for bencode in peervec.iter() {
                            match FromBencode::from_bencode(bencode) {
                                Some(peer) => peers.push(peer),
                                _ => ()
                            }
                        }
                    },
                    Some(&ByteString(ref peervec)) => {
                        for bytes in peervec.as_slice().chunks(6) {
                            let mut v = [0u8, ..6]; v.copy_from(bytes);
                            peers.push(Peer::from_6byte(&v));
                        }
                    },
                    _ => ()
                }
                Some(Success(AnnounceResult {
                    warning_message: opt_finder(dict, "warning message"),
                    // default to 10 minutes
                    interval: opt_finder(dict, "interval").unwrap_or(600u),
                    min_interval: opt_finder(dict, "min interval"),
                    tracker_id: opt_finder(dict, "tracker id"),
                    complete: opt_finder(dict, "complete").unwrap_or(0u),
                    incomplete: opt_finder(dict, "incomplete").unwrap_or(0u),
                    peers: peers
                }))
            },
            _ => None
        }
    }
}
