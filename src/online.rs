use rust_ringitem_format;
use nscldaq_ringbuffer;
use ringmaster_client;

use crate::DataSource;
use url::Url;
use std::time::Duration;

const BUFFER_SIZE :usize = 4096*1024*1024;    // Size of buffer to read at a time.  Should be big enough to hold multiple ring items.

/// This represents an online data source that is
/// potentially remote. 
pub struct TcpDataSource {
    ring : Option<ringmaster_client::RingBufferConsumer>,
    buffer : Vec<u8>,
    bytes_in_buffer : usize,
    cursor : usize,
}
impl TcpDataSource {
    // Put data into the data buffer.
    // before doing so, the data already in is slid down to the beginning.
    // On exit the bytes_in_buffer will be updated but may not be
    // BUFFER_SIZE-1 as we might not have the full buffer worth of data available.

    fn fill(&mut self) {
        
        if self.cursor > 0{
            // Slide any remaining data down to the beginning of the buffer:
            let remaining = self.bytes_in_buffer - self.cursor;
            self.buffer.copy_within(self.cursor..self.bytes_in_buffer, 0);
            self.bytes_in_buffer = remaining;
            self.cursor = 0;
        }
        if let Some(src) = self.ring.as_mut() {
            let ring  = &mut src.consumer;
            match ring.timed_get(&mut self.buffer[self.bytes_in_buffer..], Duration::from_secs(1)) {
                Ok(n) => {
                    self.bytes_in_buffer += n;
                },
                Err(e) => {
                    if let nscldaq_ringbuffer::ringbuffer::consumer::Error::Timeout = e {
                        // No data available, just return and try again later.
                        return;
                    } else {
                        eprintln!("Error reading from data source: {:?}", e);
                       self.close();
                    }
                }
            }
        } else {
            panic!("Data source is not open");
        }

    }
    /// Create a new data source.  Once done, one needs to open it to
    /// connect it to an actual ringbuffer.
    /// Note that the data source factory will call this method to create a new data source before calling open on it.  
    /// However, it is legal to explicitly create and open a data source, so this method should be public.
    pub fn new() -> TcpDataSource { 
        
        TcpDataSource {
            ring : None,
            buffer : Vec::new(),
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
        let ring = ringmaster_client::RingBufferConsumer::attach(uri);
        if let Err(e) = ring {
            return Err(format!("Failed to connect to ring buffer at {}: {}", uri, e));  
        }
        self.ring = Some(ring.unwrap());
        Ok(())
    }
    
    fn read(&mut self) -> Option<rust_ringitem_format::RingItem> {
        if let Some(_) = self.ring.as_mut() {
            
            // If I don't have enough data for a header, try to fill:

            let mut remaining  = self.bytes_in_buffer - self.cursor;
            while remaining < 2*size_of::<u32>() {
                self.fill();
                remaining = self.bytes_in_buffer - self.cursor;
            }                               // Might loop a while.
            // Get the size and type .. without consuming the data from the buffer.

            let size = u32::from_le_bytes(
                self.buffer[self.cursor..self.cursor+size_of::<u32>()].try_into().unwrap()) as usize;
            while remaining < size {
                self.fill();
                remaining = self.bytes_in_buffer - self.cursor;
            }                                            // Loop until we have a ring item.
            let item_type = u32::from_le_bytes(
                self.buffer[self.cursor+size_of::<u32>()..self.cursor+size_of::<u32>()*2].try_into().unwrap());
            //Make and fill in the ring item:

            let mut item = rust_ringitem_format::RingItem::new(item_type);
            let bodysize = size - 2*size_of::<u32>();
            self.cursor += 2*size_of::<u32>();   // Move past the header.

            for _i in 0..bodysize {
                item.add(self.buffer[self.cursor]);
                self.cursor += 1;
            }
            


            return Some(item);

        } else {
            return None;                      // Data source is not open.
        }
    }
    fn close(&mut self) {
        self.ring = None;             // Dropping the consumer unregisters us with the ringmaster.
    }
}

// tests require the ring master
#[cfg(test)]
mod online_tests {
    use super::*;
    #[test]
    fn create_1() {
        // Creating a data source does not overflow stack:

        let mut x = TcpDataSource::new();

        // Open a nonexisting ring will faile:

        let result = x.open("tcp://localhost/no_such_ring");
        assert!(result.is_err());
    }
}