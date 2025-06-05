use super::Cache;

static ITEMS: &[u64] = &[0, 2, 4, 8];

#[test]
fn test() {
	let cache = Cache::<u64>::new();

	cache.extend(ITEMS.iter().copied());

	for _ in 0..2 {
		let mut reader = cache.reader();
		let mut dst = Vec::new();
		assert_eq!(reader.read_into_vec(&mut dst), ITEMS.len());
		assert_eq!(&dst, ITEMS);
	}

	let mut reader = cache.reader();
	let mut dst = Vec::new();
	drop(cache);

	while reader.read_into_vec(&mut dst) > 0 {}
	assert_eq!(&dst, ITEMS);
}
