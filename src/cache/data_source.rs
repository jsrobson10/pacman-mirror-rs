use std::sync::Arc;

use super::PartialCacheReader;


#[derive(Clone)]
pub enum DataSource {
	Empty,
	Partial(PartialCacheReader<u8>),
	Memory(Arc<[u8]>),
}

