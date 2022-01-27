use tokio::net::TcpStream;
use tokio::io::AsyncWriteExt;
use std::error::Error;
use std::{io, str};
use tokio::task;



fn process_xml(bytes: usize, content: &[u8]) {
    println!("read {} bytes", bytes);
    let s = str::from_utf8(content).unwrap();
    let s = format!("<fake>{}</fake>", s);
    println!("result: {}", s);
    let doc = roxmltree::Document::parse(&s).unwrap();
    println!("doc {:?}", doc);
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Connect to a peer
    let mut stream = TcpStream::connect("127.0.0.1:7624").await?;
 
    // Say welcome to indiserver, otherwise we are not getting any message
    stream.write_all(b"<getProperties version='1.7'/>").await?;
    
    loop {
        // Wait for the socket to be readable
        stream.readable().await?;

        // Creating the buffer **after** the `await` prevents it from
        // being stored in the async task.
	let mut buf = Vec::with_capacity(16384);

        // Try to read data, this may still fail with `WouldBlock`
        // if the readiness event is a false positive.
        match stream.try_read_buf(&mut buf) {
            Ok(0) => break,
            Ok(n) => task::spawn_blocking(move || {
		process_xml(n, &buf);
            }).await?,
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
		continue;
            }
            Err(e) => {
                return Err(e.into());
            }
        }
    }

    Ok(())
}
