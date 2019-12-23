use derivative::Derivative;
use specs::{System, Write};
use std::{
	clone::Clone,
	collections::{HashMap, VecDeque},
	error::Error,
	marker::PhantomData,
	sync::Arc,
};

pub trait Asset: Send + Sync + 'static {
	type Data: Send + Sync + 'static;
}

#[derive(Derivative)]
#[derivative(
	Clone(bound = ""),
	Eq(bound = ""),
	Hash(bound = ""),
	PartialEq(bound = ""),
	Debug(bound = "")
)]
pub struct AssetHandle<A: ?Sized> {
	id: Arc<u32>,
	marker: PhantomData<A>,
}

impl<A> AssetHandle<A> {
	/*pub fn downgrade(&self) -> WeakHandle<A> {
		let id = Arc::downgrade(&self.id);

		WeakHandle {
			id,
			marker: PhantomData,
		}
	}*/

	fn id(&self) -> u32 {
		*self.id.as_ref()
	}

	fn is_unique(&self) -> bool {
		Arc::strong_count(&self.id) == 1
	}
}

/*#[derive(Derivative)]
#[derivative(Clone(bound = ""))]
pub struct WeakHandle<A> {
	id: Weak<u32>,
	marker: PhantomData<A>,
}

impl<A> WeakHandle<A> {
	pub fn upgrade(&self) -> Option<AssetHandle<A>> {
		self.id.upgrade().map(|id| AssetHandle {
			id,
			marker: PhantomData,
		})
	}
}*/

/*#[derive(Derivative)]
#[derivative(Default(bound = ""))]
pub struct AssetCache<A> {
	map: HashMap<String, WeakHandle<A>>,
}

impl<A> AssetCache<A> {
	pub fn new() -> Self {
		Default::default()
	}

	pub fn insert<K: Into<String>>(
		&mut self,
		key: K,
		asset: &AssetHandle<A>,
	) -> Option<WeakHandle<A>> {
		self.map.insert(key.into(), asset.downgrade())
	}

	pub fn get<K>(&self, key: &K) -> Option<AssetHandle<A>>
	where
		K: ?Sized + Hash + Eq,
		String: Borrow<K>,
	{
		self.map.get(key).and_then(WeakHandle::upgrade)
	}
}*/

#[derive(Derivative)]
#[derivative(Default(bound = ""))]
pub struct AssetStorage<A: Asset> {
	assets: HashMap<u32, A>,
	unbuilt: Vec<(AssetHandle<A>, A::Data, String)>,
	handles: Vec<AssetHandle<A>>,
	highest_id: u32,
	unused_ids: VecDeque<u32>,
}

impl<A: Asset> AssetStorage<A> {
	pub fn get(&self, handle: &AssetHandle<A>) -> Option<&A> {
		self.assets.get(&handle.id())
	}

	fn allocate_handle(&mut self) -> AssetHandle<A> {
		let id = self.unused_ids.pop_front().unwrap_or_else(|| {
			self.highest_id += 1;
			self.highest_id
		});

		AssetHandle {
			id: Arc::new(id),
			marker: PhantomData,
		}
	}

	pub fn insert(&mut self, asset: A) -> AssetHandle<A> {
		let handle = self.allocate_handle();
		self.assets.insert(handle.id(), asset);
		self.handles.push(handle.clone());
		handle
	}

	pub fn load(
		&mut self,
		name: &str,
		format: impl AssetFormat<Asset = A::Data>,
		source: &mut impl DataSource,
	) -> Result<AssetHandle<A>, Box<dyn Error>> {
		let data = format.import(name, source)?;
		let handle = self.allocate_handle();
		self.unbuilt.push((handle.clone(), data, name.to_owned()));
		Ok(handle)
	}

	pub fn clear_unused(&mut self) {
		let assets = &mut self.assets;
		let unused_ids = &mut self.unused_ids;
		let old_len = self.handles.len();

		self.handles.retain(|handle| {
			if handle.is_unique() {
				assets.remove(&handle.id());
				unused_ids.push_back(handle.id());
				false
			} else {
				true
			}
		});

		let count = old_len - self.handles.len();

		if count > 0 {
			trace!(
				"Freed {} assets of type {}",
				count,
				std::any::type_name::<A>()
			);
		}
	}

	pub fn build_waiting<F: FnMut(A::Data) -> Result<A, Box<dyn Error>>>(
		&mut self,
		mut build_func: F,
	) {
		for (handle, data, name) in self.unbuilt.drain(..) {
			let asset = match build_func(data) {
				Ok(asset) => {
					trace!("Asset '{}' loaded successfully", name);
					asset
				}
				Err(e) => {
					error!("Asset '{}' could not be loaded: {}", name, e);
					continue;
				}
			};

			self.assets.insert(handle.id(), asset);
			self.handles.push(handle);
		}
	}
}

pub trait DataSource {
	fn load(&mut self, path: &str) -> Result<Vec<u8>, Box<dyn Error>>;
	fn names<'a>(&'a self) -> Box<dyn Iterator<Item = &str> + 'a>;
}

pub trait AssetFormat: Clone {
	type Asset;

	fn import(
		&self,
		name: &str,
		source: &mut impl DataSource,
	) -> Result<Self::Asset, Box<dyn Error>>;
}

pub struct AssetMaintenanceSystem<A>(PhantomData<A>);

impl<A> AssetMaintenanceSystem<A> {
	pub fn new() -> AssetMaintenanceSystem<A> {
		AssetMaintenanceSystem(PhantomData)
	}
}

impl<'a, A: Asset> System<'a> for AssetMaintenanceSystem<A> {
	type SystemData = Write<'a, AssetStorage<A>>;

	fn run(&mut self, mut storage: Self::SystemData) {
		storage.clear_unused();
	}
}