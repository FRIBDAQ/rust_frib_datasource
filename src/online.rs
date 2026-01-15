use rust_ringitem_format;
use nscldaq_ringbuffer;
use ringmaster_client;

use crate::DataSource;
use url::{Url, ParseError};

const BUFFER_SIZE :usize = 4096*1024*1024;    // Size of buffer to read at a time.  Should be big enough to hold multiple ring items.

/// This represents an online data source that is
/// potentially remote.  At thist time, rather than reading
/// blocks of data, we read one ring item at a time.
/// In the future, we may want to read blocks of data and unpack them like we do for files.
/// 
pub struct TcpDataSource {
    ring : Option<ringmaster_client::RingBufferConsumer>,
    buffer : [u8; BUFFER_SIZE],
    bytes_in_buffer : usize,
    cursor : usize,
}
impl TcpDataSource {
    pub fn new() -> TcpDataSource { 
        
        TcpDataSource {
            ring : None,
            buffer : [0; BUFFER_SIZE],
            bytes_in_buffer : 0,
            cursor : 0,
        }

    }
}
impl DataSource for TcpDataSource {
    fn open(&mut self, uri: &str) -> Result<(), String> {

        let url = Url::parse(uri);
        if let Err(e) = url {
            return Err(format!("Failed to parse URI {}: {}", uri, e));
        }
        let url = url.unwrap();            // Won't file.
        if url.scheme() != "tcp" {
            return Err(format!("Invalid URI scheme for TcpDataSource: {}", url.scheme()));
        }
        let mut ring = ringmaster_client::RingBufferConsumer::attach(uri);
        if let Err(e) = ring {
            return Err(format!("Failed to connect to ring buffer at {}: {}", uri, e));  
        }
        self.ring = Some(ring.unwrap());
        Ok(())
    }
    
    fn read(&mut self) -> Option<rust_ringitem_format::RingItem> {
        if let Some(src) = self.ring.as_mut() {
            let ring  = &mut src.consumer;

            return None;

        } else {
            return None;                      // Data sourc ie not open.
        }
    }
    fn close(&mut self) {

    }
}