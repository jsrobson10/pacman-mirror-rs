use std::sync::Arc;

use super::partial;


#[derive(Clone)]
pub enum DataSource {
    Empty,
    Partial(Arc<partial::Source>),
    Memory(Arc<[u8]>),
}

