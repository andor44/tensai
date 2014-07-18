extern crate bencode;

use std::collections::hashmap::HashMap;

use bencode::{FromBencode, Dict, Key, Bencode};

use super::opt_finder;

#[deriving(Show, Clone)]
pub struct TorrentScrape {
    pub complete: uint,
    pub downloaded: uint,
    pub incomplete: uint,
    pub name: Option<String>
}

#[deriving(Show)]
pub struct ScrapeInfo {
    pub torrents: HashMap<Vec<u8>, TorrentScrape>
}

impl FromBencode for ScrapeInfo {
    fn from_bencode(scrape: &bencode::Bencode) -> Option<ScrapeInfo> {
        let mut torrent_scrapes = HashMap::new();
        match scrape {
            &Dict(ref dict) => {
                let filesdict = match dict.find(&Key::from_str("files")) {
                    Some(&Dict(ref filesdict)) => filesdict,
                    _ => return None
                };
                for (key, value) in filesdict.iter() {
                    let scrape: TorrentScrape = FromBencode::from_bencode(value).unwrap();
                    torrent_scrapes.insert(Vec::from_slice(key.as_slice()), scrape);
                }
            },
            _ => return None
        }
        Some(ScrapeInfo {
            torrents: torrent_scrapes
        })
    }
}

impl FromBencode for TorrentScrape {
    fn from_bencode(scrape: &bencode::Bencode) -> Option<TorrentScrape> {
        match scrape {
            &Dict(ref dict) => {
                Some(TorrentScrape { 
                    complete: opt_finder(dict, "complete").expect("Invalid 'complete' number in TorrentScrape"),
                    downloaded: opt_finder(dict, "downloaded").expect("Invalid 'downloaded' number in TorrentScrape"),
                    incomplete: opt_finder(dict, "incomplete").expect("Invalid 'incomplete' number in TorrentScrape"),
                    name: opt_finder(dict, "name").expect("Invalid 'name' in TorrentScrape")
                })
            },
            _ => None
        }
    }
}
