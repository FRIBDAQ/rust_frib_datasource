pub mod online;
pub mod offline;
use rust_ringitem_format;
use url::{Url, ParseError};
pub trait DataSource {
    /// Open the data source to the specified URI.
    /// If a data source is already opened it should be closed.
    /// If the open fails Err(String::from("some error message")) should be returned.
    /// 
    /// ### Paramters
    /// uri - The URI of the dats source to e opened.  Note that the data_source_factory
    /// function will only call this method with a URI that has the appropriate scheme for the
    /// data source type.  For example, the file data source will only be called with a URI that has the "file" scheme.
    /// 
    /// However since it is legal to explicitly create and open a data source, the scheme should be
    /// checked for legality.
    /// 
    fn open(&mut self, uri: &str) -> Result<(), String>;
    /// Read the next ring item from the data sourcde.  For finite sources,
    /// this returns None.  For infinite sources like Online and pipes or FIFOs, this could
    /// block for a significant amount of time until the next item becomes available.
    fn read(&mut self) -> Option<rust_ringitem_format::RingItem>;
    /// Disconnect from the data source.  If not connected, this is a No-op.
    /// 
    fn close(&mut self);
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
