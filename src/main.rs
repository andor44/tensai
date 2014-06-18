extern crate tensai;
extern crate debug;

use std::os;

fn main() {
	println!("Hello world!");
    let filename = os::args().get(1).to_string();
    let torrent = tensai::parse_torrent(&Path::new(filename)).unwrap();
    match torrent.metainfo.multifile {
        Some(ref files) => {
            for file in files.iter() {
                println!("{:?}", file);
                let filepath = file.path.clone();
                println!("{:?}", filepath.unwrap().get(0));
            }
        },
        _ => ()
    }
    println!("{:?}", torrent);
}
