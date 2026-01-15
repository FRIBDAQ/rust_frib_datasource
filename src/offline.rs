use rust_ringitem_format;// stubs for offline data sources:
use crate::DataSource;
use std::io;
use std::fs;
use url::{Url, ParseError};
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
    source : Option<Box<dyn io::Read>>,    // Some readble sourcde of data.
}
impl FileDataSource {
    // True if a data source is open.
    fn readable(&self) -> bool {
        self.source.is_some()
    }
    pub fn new() -> FileDataSource{ 
        FileDataSource {
            buffer: Vec::new(),
            cursor: 0,
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
        None
    }
    fn close(&mut self) {
        self.source = None;
    }
}