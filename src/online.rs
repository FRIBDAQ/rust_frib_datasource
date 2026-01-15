use rust_ringitem_format;
use crate::DataSource;
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
        Ok(())
    }
    fn read(&mut self) -> Option<rust_ringitem_format::RingItem> {
        None
    }
}