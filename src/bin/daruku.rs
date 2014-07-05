extern crate tensai;
extern crate http;

use std::os;

use tensai::torrent::{Torrent, TorrentInfo};
use tensai::torrent::{Stopped};
use tensai::client::Client;

fn main() {
	println!("Daruku start");
    println!("Tensai version {}", tensai::CLIENT_VERSION);
    let filename = os::args().get(1).to_string();
    let torrent = TorrentInfo::read(&Path::new(filename)).unwrap();
    println!("{}", torrent.hash_string());
    println!("{}", torrent.urlencoded_hash());
    let t = Torrent { info: torrent.clone(), status: Stopped };
    println!("{}", t.scrape().unwrap());
    println!("{}", torrent.comment.clone().map_or(String::from_str("no comment"), |comment| comment));
    let c = Client::new();
    let c2 = Client::with_client_rand("foobar");
    println!("{}", c.peer_id());
}
