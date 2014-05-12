extern crate tensai;

fn main() {
	println!("Hello world!");

    let torrent = tensai::parse_torrent(&Path::new("/home/andor/test.torrent")).unwrap();
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
