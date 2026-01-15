pub mod online;
pub mod offline;
use rust_ringitem_format;

pub trait DataSource {
    fn open(&mut self, uri: &str) -> Result<Box<dyn DataSource>, String>;
    fn read(&mut self) -> Option<rust_ringitem_format::RingItem>;
}