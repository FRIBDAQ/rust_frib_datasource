use rust_ringitem_format;// stubs for offline data sources:
use crate::DataSource;
use std::io;
use std::fs;
use url::Url;

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
            // Read more data to fill the buffer -- or to EOF.
            while self.bytes_in_buffer < READ_SIZE {
                let mut temp_buffer = vec![0; 4096];
                let status = {source.read(&mut temp_buffer)};
                match status {
                    Ok(n) => {
                        if n == 0 {
                            // EOF
                            break;
                        }   
                        self.buffer.extend_from_slice(&temp_buffer[..n]);
                        self.bytes_in_buffer += n;
                    },
                    Err(e) => {
                        eprintln!("Error reading from data source: {}", e);
                        
                    }
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
        let mut item_data = self.buffer[self.cursor+2*size_of::<u32>()..self.cursor+item_size].to_vec();

        // Figure out if there's a body header... this will be the case if the next u32 in the buffer
        // is at least the size of a body header. If so, we need to pull out the timestamp, source id and
        // barrier type and make a ring item with a body header, otherwise not.
        // Note that the body header struct might not be packe so we use 5*size_of::<u32>
        // the size, timestamp (u64), source, and lastly barrier type = 5 u32s.
        let bhdr_size = u32::from_le_bytes(item_data[0..4].try_into().unwrap());
        
        let mut ring_item = if bhdr_size as usize  == 5*size_of::<u32>() {
            let timestamp = u64::from_le_bytes(item_data[4..12].try_into().unwrap());
            let source_id = u32::from_le_bytes(item_data[12..16].try_into().unwrap());
            let barrier_type = u32::from_le_bytes(item_data[16..20].try_into().unwrap());
            // Remove the body header from the item_data and make the item.

            item_data.drain(0..20);

            rust_ringitem_format::RingItem::new_with_body_header(item_type, timestamp, source_id, barrier_type)

            
        } else {
            // Remove the null body header size from the data and make the ring item.
            item_data.drain(0..4);                           // There's always a dummy size.

            rust_ringitem_format::RingItem::new(item_type)

        };


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