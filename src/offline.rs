use rust_ringitem_format;// stubs for offline data sources:
use crate::DataSource;
use std::io;
use std::fs;
use url::{Url, ParseError};

const READ_SIZE: usize = 4096*1024*1024;    // Size of buffer to read at a time.  Should be big enough to hold multiple ring items.
///
/// A file data source represents a data source that is a disk file.
/// Rather than reading a ring item at a time we read in blocks of data.
/// We then sequentially  unpack ring items from that block.  When we get to
/// a partial ring item at the end of the file, we slide it to the beginning of the buffer
/// and read more data  to fil the remainder of the buffer.
/// 
/// Since we implement the DataSource trait, we can box this up and use it as an
/// abtract data sourc.  In fact, we can read data from anything with a read trait,
/// but in fact, the open method of the DataSource trait is what restricts us to
/// files and stdin.
pub struct FileDataSource {
    buffer: Vec<u8>,
    cursor: usize,
    bytes_in_buffer: usize,
    source : Option<Box<dyn io::Read>>,    // Some readble sourcde of data.
}
impl FileDataSource {
    // True if a data source is open.
    fn readable(&self) -> bool {
        self.source.is_some()
    }
    fn fill_buffer(&mut self) {
        if let Some(source) = &mut self.source {
            // Slide any partial ring item to the beginning of the buffer:
            if self.cursor < self.bytes_in_buffer {
                let remaining = self.bytes_in_buffer - self.cursor;
                self.buffer.copy_within(self.cursor..self.bytes_in_buffer, 0);
                self.bytes_in_buffer = remaining;
                self.buffer.truncate(remaining);
            } else {
                self.bytes_in_buffer = 0;
            }
            self.cursor = 0;
            // Read more data to fill the buffer:
            let mut temp_buffer = vec![0; 4096];
            match source.read(&mut temp_buffer) {
                Ok(n) => {
                    self.buffer.extend_from_slice(&temp_buffer[..n]);
                    self.bytes_in_buffer += n;
                },
                Err(e) => {
                    eprintln!("Error reading from data source: {}", e);
                    self.close();
                }
            }
        }
    }
    pub fn new() -> FileDataSource{ 
        FileDataSource {
            buffer: Vec::new(),
            cursor: 0,
            bytes_in_buffer: 0,
            source: None
        }
    }
}

impl DataSource for FileDataSource {
    fn open(&mut self, uri: &str) -> Result<(), String> {
        let url = Url::parse(uri);
        if let Err(e) = url {
            return Err(format!("Failed to parse URI {}: {}", uri, e));
        }
        let url = url.unwrap();            // Won't fail.
        if url.scheme() != "file" {
            return Err(format!(
                "Invalid URI scheme for FileDataSource {}: {} must be 'file'", uri, url.scheme()
            ));
        }  
        let p = url.path();
        if p == "-" {
            self.source = Some(Box::new(io::stdin()));
        } else {
            let f = fs::File::open(p);
            if let Err(e) = f {
                return Err(format!("Failed to open file {}: {}", p, e));
            }
            self.source = Some(Box::new(f.unwrap()));
        }
        Ok(())
    }
    fn read(&mut self) -> Option<rust_ringitem_format::RingItem> {
        if !self.readable() {
            return None;                             // No data source open.
        }

        if self.cursor >= self.bytes_in_buffer {
            self.fill_buffer();
            if self.bytes_in_buffer == 0 {
                self.close();
                return None;                             // Exhausted data.
            }
        } 
        // Might also need to load the buffer if we only have a partial ring item:

        if self.bytes_in_buffer - self.cursor < size_of::<u32>() {  // Do we even have a ring item size?
            self.fill_buffer();
            if self.bytes_in_buffer - self.cursor < size_of::<u32>() {   // EOF in the middle of stuff.
                self.close();
                return None;                             // Exhausted data.
            }
        }
        // Get the ring item size:
        let item_size = u32::from_le_bytes(self.buffer[self.cursor..self.cursor+size_of::<u32>()].try_into().unwrap()) as usize;
        if self.bytes_in_buffer - self.cursor < item_size {   // Do we have the whole ring item?
            self.fill_buffer();
            if self.bytes_in_buffer - self.cursor < item_size {   // EOF in the middle of stuff.
                // Should we really panic because this could mean taht we just can't do a big enough
                // read to get the full ring item into our buffer.
                self.close();
                return None;                             // Exhausted data. 
            }
        }
        // Construct a ring item from the point of the cursor

        let item_type = u32::from_le_bytes(self.buffer[self.cursor+size_of::<u32>()..self.cursor+2*size_of::<u32>()].try_into().unwrap());
        let item_data = self.buffer[self.cursor+2*size_of::<u32>()..self.cursor+item_size].to_vec();
        let mut ring_item = rust_ringitem_format::RingItem::new(item_type);
        ring_item.add_byte_vec(&item_data);
    
        // advance the cursor.

        self.cursor += item_size;
        
        // Returnt he ring item.

        Some(ring_item)
    }
    fn close(&mut self) {
        self.source = None;
    }
}