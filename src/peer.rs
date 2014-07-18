extern crate bencode;

use std::io::net::ip::{Ipv4Addr, Ipv6Addr, SocketAddr};
use std::str::from_utf8;
use std::fmt::{Show, Formatter, FormatError};

use bencode::{Bencode, FromBencode, Dict, Key, ByteString};

use super::opt_finder;


pub struct Peer {
    pub address: SocketAddr,
    pub peer_id: Option<[u8, ..20]>,
}


/// Network byte order dword
fn dbyte(hi: u8, lo: u8) -> u16 {
    ((hi as u16 << 8) | lo as u16)
}

impl Peer {
    /// Construct Peer from compact peer list 6-byte representation
    pub fn from_6byte(bytes: &[u8, ..6]) -> Peer {
        Peer {
            address: SocketAddr {
                ip: Ipv4Addr(bytes[0], bytes[1], bytes[2], bytes[3]),
                port: dbyte(bytes[4], bytes[5])
            },
            peer_id: None
        }
    }
    /// Same as from_6byte, except for IPv6 peers
    pub fn from_18byte(bytes: [u8, ..18]) -> Peer {
        use std::iter::count;
        let mut dbytes = [0u16, ..9]; 
        for i in count(0u, 2).take(9) {
            dbytes[i] = dbyte(bytes[i], bytes[i+1]);
        }
        Peer {
            address: SocketAddr {
                ip: Ipv6Addr(dbytes[0], dbytes[1], dbytes[2], dbytes[3], 
                             dbytes[4], dbytes[5], dbytes[6], dbytes[7]),
                port: dbytes[8]
            },
            peer_id: None
        }
    }
}

impl FromBencode for Peer {
    fn from_bencode(bencode: &Bencode) -> Option<Peer> {
        match bencode {
            &Dict(ref dict) => {
                Some(Peer {
                    address: SocketAddr {
                        ip: {
                            let str_ip: String = opt_finder(dict, "ip").expect("failed to find ip in peer");
                            from_str(str_ip.as_slice()).expect("peer ip is not valid address")
                        },
                        port: opt_finder(dict, "port").expect("invalid peer port")
                    },
                    peer_id: {
                        match dict.find(&Key::from_str("peer_id")) {
                            Some(&ByteString(ref vector)) => {
                                let mut vec = [0u8, ..20];
                                for i in range(0, 20) {
                                    vec[i] = *vector.get(i);
                                }
                                Some(vec)
                            },
                            _ => None
                        }
                    }
                })
            },
            _ => None
        }
    }
}

// XXX: check soundness of this
impl PartialEq for Peer {
    fn eq(&self, other: &Peer) -> bool {
        self.address == other.address && 
        // Ugly hack until we have numbers in generics
        self.peer_id.as_ref().map(|x| x.as_slice()) == other.peer_id.as_ref().map(|x| x.as_slice())
    }
}

impl Eq for Peer { }

impl Show for Peer {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FormatError> {
        write!(f, "Peer {{ id: {} address: {} }}", self.peer_id.as_ref().map(|x| x.as_slice()), self.address)
    }
}
