use std::io::prelude::*;
use std::net::TcpStream;
use std::sync::{Arc, RwLock};

use std::collections::VecDeque;

fn main() {
    // Open a TCP connection with INDI server
    let mut stream = TcpStream::connect("127.0.0.1:7624").unwrap();

    // Create the "queue" to store INDI messages, since the queue will be shared between threads
    // it needs to be ARCed.
    let queue: Arc<RwLock<VecDeque<([u8; 2048], usize)>>> = Arc::new(RwLock::new(VecDeque::new()));

    // Clone the queue for the reader thread
    let q = Arc::clone(&queue);

    // Say hello to INDI server
    stream.write(b"<getProperties version='1.7'/>").unwrap();
    
    std::thread::spawn(move || {
	loop {
	    let mut rqueue = q.write().unwrap();
	    match rqueue.pop_front() {
		Some((m, s)) => {
		    match roxmltree::Document::parse(std::str::from_utf8(&m[0..s]).unwrap()) {
			Ok(doc) => println!("XML Doc: {:?}\n\n\n", &doc),
			Err(_) => {
			    println!("Bad XML");
			    // TODO: if we cannot convert to XML we likely have a partial message,
			    // the next one needs to be parsed and joined with the previous one to
			    // check if it's now valid. Continue until the XML doc is valid.
			},
		    };
		},
		None => (),
	    }
	};
    });
    
    loop {
	let mut buf = [0; 2048];
	let read_bytes = stream.read(&mut buf).unwrap();
	let mut wqueue = queue.write().unwrap();
	wqueue.push_back((buf, read_bytes));
    };
}
