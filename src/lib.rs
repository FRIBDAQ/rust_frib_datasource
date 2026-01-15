pub mod online;
pub mod offline;
use rust_ringitem_format;
use url::{Url, ParseError};
pub trait DataSource {
    fn open(&mut self, uri: &str) -> Result<(), String>;
    fn read(&mut self) -> Option<rust_ringitem_format::RingItem>;
}




///
/// Data source factory, givne A URI, returns the 
/// appropriate type of data source as a boxed dynamic DataSource implementing
/// object.
/// 
fn data_source_factory(uri: &str) -> Result<Box<dyn DataSource>, String> {
    let source_url = Url::parse(uri);
    if let Err(e) = source_url {
        return Err(format!("Failed to parse URI {}: {}", uri, e));
    }
    let source_url = source_url.unwrap();

    // The scheme must be either tcp or file:

    match source_url.scheme() {
        "tcp" => {
            let mut ds = online::TcpDataSource::new();
            let status = ds.open(uri);
            if let Err(e) = status {   
                return Err(format!("Failed to open data source {}: {}", uri, e));
            }
            Ok(Box::new(ds))
        },
        "file" => {
            let mut ds = offline::FileDataSource::new();
            let status = ds.open(uri);
            if let Err(e) = status {    
                return Err(format!("Failed to open data source {}: {}", uri, e));
            }
            Ok(Box::new(ds))
        },
        _ => Err(format!("Unsupported URI scheme: {}", source_url.scheme())),
    }
 }
