extern crate tensai;
extern crate time;

use std::os;
use std::io::MemWriter;

use time::precise_time_ns;

use tensai::torrent::{TorrentInfo, SingleFile, MultiFile};
use tensai::client::{Client};
use tensai::peer::Peer;
use tensai::announce::Success;

fn usage() {
    println!("{} <torrent file> <dest path>", os::args().get(0));
}

fn main() {
	println!("Daruku start");
    println!("Tensai version {}", tensai::CLIENT_VERSION);
    if os::args().len() < 3 {
        usage(); return;
    }
    let (file, destination_path) = (Path::new(os::args().get(1).clone()), 
                                    Path::new(os::args().get(2).clone()));
    if !file.is_file() {
        fail!("cannot open torrent file");
    }
    if !destination_path.is_dir() {
        fail!("destination directory doesn't exist");
    }
    let torrentinfo = TorrentInfo::read(&file).unwrap();
    //println!("{}", torrentinfo);
    //println!("{}", torrentinfo.metainfo.pieces.len());
    //return;
    let rand = "123456654321";
    let mut c = Client::with_client_rand(rand.to_string());
    let peer_id = c.peer_id();
    let (fsize, fpath, infohash, result, psize) = {
        let mut torrent = c.add_torrent(&torrentinfo, destination_path).unwrap();
        let (fsize, fpath) = match torrent.info.metainfo.payload {
            SingleFile(ref file) => (file.length, torrent.destination_path.join(torrent.info.metainfo.name.clone())),
            MultiFile(ref files) => (files.get(0).length, torrent.destination_path.join_many(files.get(0).path.as_ref().unwrap().as_slice())) 
        };
        let infohash = torrent.info.infohash.clone();
        let result = torrent.announce(peer_id.clone());
        (fsize, fpath, infohash, result, torrent.info.metainfo.piece_length)
    };
    match result {
        Some(Success(ref announce)) => {
            use std::num::pow;
            use std::io::net::tcp::{TcpStream};
            use std::io::BufferedReader;
            use std::io::fs::File;
            let peer = announce.peers.get(0);
            println!("Trying to connect to {}", peer);
            let mut stream = TcpStream::connect_timeout(peer.address, 10000).unwrap();
            println!("connected, sending handshake");
            stream.write_u8(19);
            stream.write(b"BitTorrent protocol");
            stream.write([0u8, ..8]);
            stream.write(infohash.as_slice());
            stream.write(peer_id.as_bytes());
            println!("done sending handshake");
            let mut reader = BufferedReader::new(stream.clone());
            let pstrlen = reader.read_byte().unwrap();
            let pstr = reader.read_exact(pstrlen as uint).unwrap();
            println!("peer pstr: {}", String::from_utf8(pstr).unwrap());
            let reserved_bytes = reader.read_exact(8u).unwrap();
            let info_hash = reader.read_exact(20u).unwrap();
            let peer_id = reader.read_exact(20u).unwrap();
            println!("reserved bytes: {}", reserved_bytes);
            println!("info hash: {}", info_hash);
            println!("peer_id: {}", peer_id);

            // write empty bitfield
            //stream.write_be_u32(114);
            //stream.write_u8(5);
            //stream.write([0u8, ..114]);
            //println!("wrote empty bitfield");

            // write unchoke 
            stream.write_be_u32(1);
            stream.write_u8(1);
            println!("wrote unchoke");
            // write interest
            stream.write_be_u32(1);
            stream.write_u8(2);
            println!("wrote interest");

            // read bitfield
            // size
            let bf_size = reader.read_be_u32().unwrap();
            // bf id
            reader.read_u8();
            // bf itself
            reader.read_exact((bf_size-1) as uint);

            // umm... no idea what this is?
            assert!(reader.read_exact(5) == Ok(vec![0u8, 0, 0, 1, 1]));

            struct Piece {
                data: MemWriter,
                offset: uint
            }
            let psize = psize as uint;
            let mut recv_bytes = 0u;
            let num_pieces = (fsize as f64 / psize as f64).ceil() as uint;
            let mut pieces = Vec::from_fn(num_pieces, |_| {
                Piece {            
                    data: MemWriter::with_capacity(psize),
                    offset: 0
                }
            });

            let s2 = stream.clone();
            let n2 = num_pieces;
            let p2 = psize;
            spawn(proc() {
                let mut stream = s2;
                let num_pieces = n2;
                let blocksize = pow(2u, 12);
                let psize = p2;
                let blocks_per_piece = psize / blocksize;
                for i in range(0, num_pieces) {
                    use std::iter::range_step;
                    for j in range_step(0u, psize, blocksize) {
                        // write message length
                        stream.write_be_u32(13);
                        // packet type is "request"
                        stream.write_u8(6);

                        // index of piece we're currently in
                        stream.write_be_u32(i as u32);
                        // offset
                        stream.write_be_u32(j as u32);
                        // length
                        stream.write_be_u32(blocksize as u32);
                    }
                }
            });
            
            let blocksize = pow(2u, 12);
            let begin = precise_time_ns();
            while recv_bytes < fsize {
                use std::io::SeekSet;
                println!("enter loop recv data: {} < {}", recv_bytes, fsize);
                let msgsize = reader.read_be_u32().unwrap();
                // keepalive msg
                if msgsize == 0 {
                    stream.write_be_u32(0);
                    stream.flush();
                    continue;
                }
                let msgtype = reader.read_u8().unwrap();

                // it's not a piece msg
                if msgtype != 7 { reader.read_exact((msgsize-1) as uint); continue;  }

                let recv_pindex = reader.read_be_u32().unwrap();
                let recv_offset = reader.read_be_u32().unwrap();
                let recv_data = reader.read_exact((msgsize - 9) as uint).unwrap();
                println!("its a piece message pindex {} poffset {}", recv_pindex, recv_offset);
                let mut m_piece = pieces.get_mut(recv_pindex as uint);
                //println!("got correct data, writing it into the buffer");
                // seek to the offset within the piece
                m_piece.data.seek(recv_offset as i64, SeekSet);
                // write data into the piece
                m_piece.data.write(recv_data.as_slice());
                // update offset
                m_piece.offset += recv_data.len();
                recv_bytes += recv_data.len();
            }
            let end = precise_time_ns();
            let elapsed = end-begin;
            let elapsed_seconds = (elapsed as f64) / 1_000_000_000f64;
            let mb: f64 = recv_bytes as f64 / 1024f64 / 1024f64;
            println!("elapsed nanoseconds {} copied bytes {}", elapsed, recv_bytes);
            println!("{} mb/s", mb/elapsed_seconds);

            let mut file = File::create(&Path::new("test.png"));
            for ref piece in pieces.iter() {
                file.write(piece.data.get_ref());
            }
    
        },
        _=>()
    }
    println!("{}", result);
}
