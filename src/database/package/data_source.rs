use std::sync::Arc;

use crate::cache::CacheReader;


pub enum DataSource {
	None,
	Partial(CacheReader<u8>),
	Memory(Arc<[u8]>),
}

