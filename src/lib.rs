#![crate_name = "tensai"]
extern crate bencode;
extern crate crypto = "rust-crypto";
extern crate curl;
extern crate url;

use std::rand::{Rng, task_rng};

use bencode::{FromBencode, Key, Dict};

pub mod torrent;
pub mod scrape;
pub mod client;
pub mod announce;
pub mod peer;

pub static CLIENT_VERSION: uint = 1;

fn opt_finder<T: FromBencode>(dict: &Dict, key: &str) -> Option<T> {
    match dict.find(&Key::from_str(key)) {
        Some(value) => FromBencode::from_bencode(value),
        _ => None
    }
}

fn random_string(count: uint) -> String {
    task_rng().gen_ascii_chars().take(count).collect::<String>()
}
