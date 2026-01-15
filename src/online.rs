use rust_ringitem_format;
use nscldaq_ringbuffer;
use ringmaster_client;

use crate::DataSource;
use url::{Url, ParseError};

/// This represents an online data source that is
/// potentially remote.  At thist time, rather than reading
/// blocks of data, we read one ring item at a time.
/// In the future, we may want to read blocks of data and unpack them like we do for files.
/// 
pub struct TcpDataSource {
    ring : Option<ringmaster_client::RingBufferConsumer>,
}
impl TcpDataSource {
    pub fn new() -> TcpDataSource { 
        TcpDataSource {
            ring : None
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
        None
    }
    fn close(&mut self) {

    }
}