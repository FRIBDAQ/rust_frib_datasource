use rust_ringitem_format;
use nscldaq_ringbuffer;
use crate::DataSource;
use url::{Url, ParseError};
// Stub for online data source:

pub struct TcpDataSource {

}
impl TcpDataSource {
    pub fn new() -> TcpDataSource { 
        TcpDataSource {

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
        Ok(())  
    }
    
    fn read(&mut self) -> Option<rust_ringitem_format::RingItem> {
        None
    }
    fn close(&mut self) {

    }
}