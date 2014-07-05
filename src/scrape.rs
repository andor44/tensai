use std::collections::hashmap::HashMap;


#[deriving(Show)]
pub struct TorrentScrape {
    pub complete: int,
    pub downloaded: int,
    pub incomplete: int,
    pub name: Option<String>
}

#[deriving(Show)]
pub struct ScrapeInfo {
    files: HashMap<Vec<u8>, TorrentScrape>
}
