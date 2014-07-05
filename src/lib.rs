#![crate_id = "tensai#0.1"]
extern crate bencode;
extern crate crypto = "rust-crypto";
extern crate curl;
extern crate url;

pub mod torrent;
pub mod scrape;
pub mod client;

pub static CLIENT_VERSION: uint = 1;
