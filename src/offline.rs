use rust_ringitem_format;// stubs for offline data sources:
use crate::DataSource;

pub struct FileDataSource {

}
impl FileDataSource {
    pub fn new() -> FileDataSource{ 
        FileDataSource {

        }
    }
}

impl DataSource for FileDataSource {
    fn open(&mut self, uri: &str) -> Result<(), String> {
        Ok(())
    }
    fn read(&mut self) -> Option<rust_ringitem_format::RingItem> {
        None
    }
}