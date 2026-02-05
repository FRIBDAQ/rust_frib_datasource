pub mod online;
pub mod offline;
use rust_ringitem_format;
use url::Url;
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

pub trait DataSink {
    /// Open the data source to the specified URI.
    /// if a data source is already open, it should be closed.
    /// 
    /// ### Parameters:
    ///    uri - the URI of the data source. Note if the data_sink_factory is used,
    ///      This will be a suitable URI for the type of source.
    /// 
    fn open(&mut self, uri: &str) -> Result<(), String>;
    ///
    /// Write a ring item to the sink.
    ///
    fn write(&mut self, item : &rust_ringitem_format::RingItem) -> Result<(), String>;
    ///
    /// Close the sink.  After this is done, all write's will fail.
    /// 
    fn close(&mut self);
    ///
    /// If supported by the sink, flush any buffered data to the sink.
    /// 
    fn flush(&mut self) {}
}


///
/// Data source factory, givne A URI, returns the 
/// appropriate type of data source as a boxed dynamic DataSource implementing
/// object.
/// 
pub fn data_source_factory(uri: &str) -> Result<Box<dyn DataSource>, String> {
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

 ///
 /// data source factory for sinks.  Given a URI, return a boxed data sink
 /// of the appropriate type.  The only exposed methods are the DataSink
 /// trait methods.
 /// 
 /// Note we let the online sink worry about the host being local.
 ///
 /// ### Parameters:
 /// * uri - the URI of the desired data source ```file``` for a file and
 /// ```tcp``` for a ringbuffer.
 pub fn data_sink_factory(uri: &str) -> Result<Box<dyn DataSink>, String> {
    let sink_url = Url::parse(uri);
    if let Err(e) = sink_url {
        return Err(format!("Failed to parse the sink specification as a URI: {}", e));
    }
    let sink_url = sink_url.unwrap();
    match sink_url.scheme() {
        "tcp" => {
            let mut sink = online::RingDataSink::new();
            let status = sink.open(uri);
            if let Err(e) = status {
                return Err(format!("Unable to open ringbuffer sink {}", e));
            }
            Ok(Box::new(sink))
        },
        "file" => {
            let mut sink = offline::FileDataSink::new();
            if let Err(e) = sink.open(uri) {
                return Err(format!("Could not open file data sink: {}", e));
            }
            Ok(Box::new(sink))
        },
        _ => Err(format!("unsupported URI Scheme: {}", sink_url.scheme())),
    }

    
 }