extern crate tensai;
extern crate debug;

use std::os;
use std::num::ToStrRadix;

use tensai::torrent::TorrentInfo;

fn main() {
	println!("Hello world!");
    let filename = os::args().get(1).to_string();
    let torrent = TorrentInfo::read(&Path::new(filename)).unwrap();
    println!("{}", to_hex(torrent.infohash.as_slice()));
    /*match torrent.metainfo.multifile {
        Some(ref files) => {
            for file in files.iter() {
                println!("{:?}", file);
                let filepath = file.path.clone();
                println!("{:?}", filepath.unwrap().get(0));
            }
        },
        _ => ()
    }*/
    println!("{}", torrent.comment.clone().map_or(String::from_str("no comment"), |comment| comment));
    println!("{:?}", torrent);
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
