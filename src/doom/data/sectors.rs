use crate::{
	assets::{AssetHandle, AssetStorage},
	component::EntityTemplate,
	doom::{
		data::FRAME_TIME,
		light::{LightFlash, LightFlashType, LightGlow},
	},
};
use fnv::FnvHashMap;
use legion::prelude::{ResourceSet, Resources, Write};

pub struct SectorTypes {
	pub doomednums: FnvHashMap<u16, AssetHandle<EntityTemplate>>,
}

impl SectorTypes {
	#[rustfmt::skip]
	pub fn new(resources: &mut Resources) -> SectorTypes {
        let mut asset_storage = <Write<AssetStorage>>::fetch_mut(resources);

        let mut doomednums = FnvHashMap::default();

        // Blink random
        let handle = asset_storage.insert({
            EntityTemplate::new()
                .with_component(LightFlash {
                    off_time: 8 * FRAME_TIME,
                    on_time: 64 * FRAME_TIME,
                    ..LightFlash::default()
                })
        });
        doomednums.insert(1, handle);

        // Fast strobe unsynchronised
        let handle = asset_storage.insert({
            EntityTemplate::new()
                .with_component(LightFlash {
                    flash_type: LightFlashType::StrobeUnSync(8 * FRAME_TIME),
                    off_time: 15 * FRAME_TIME,
                    on_time: 5 * FRAME_TIME,
                    ..LightFlash::default()
                })
        });
        doomednums.insert(2, handle);

        // Slow strobe unsynchronised
        let handle = asset_storage.insert({
            EntityTemplate::new()
                .with_component(LightFlash {
                    flash_type: LightFlashType::StrobeUnSync(8 * FRAME_TIME),
                    off_time: 35 * FRAME_TIME,
                    on_time: 5 * FRAME_TIME,
                    ..LightFlash::default()
                })
        });
        doomednums.insert(3, handle);

        // Fast strobe unsynchronised + 20% damage
        let handle = asset_storage.insert({
            EntityTemplate::new()
                .with_component(LightFlash {
                    flash_type: LightFlashType::StrobeUnSync(8 * FRAME_TIME),
                    off_time: 15 * FRAME_TIME,
                    on_time: 5 * FRAME_TIME,
                    ..LightFlash::default()
                })
        });
        doomednums.insert(4, handle);

        // 10% damage
        let handle = asset_storage.insert({
            EntityTemplate::new()
        });
        doomednums.insert(5, handle);

        // 5% damage
        let handle = asset_storage.insert({
            EntityTemplate::new()
        });
        doomednums.insert(7, handle);

        // Glow
        let handle = asset_storage.insert({
            EntityTemplate::new()
                .with_component(LightGlow {
                    speed: 1.09375,
                    ..LightGlow::default()
                })
        });
        doomednums.insert(8, handle);

        // Secret
        let handle = asset_storage.insert({
            EntityTemplate::new()
        });
        doomednums.insert(9, handle);

        // Door close 30 s after level start
        let handle = asset_storage.insert({
            EntityTemplate::new()
        });
        doomednums.insert(10, handle);

        // 20% damage, end map on death
        let handle = asset_storage.insert({
            EntityTemplate::new()
        });
        doomednums.insert(11, handle);

        // Slow strobe
        let handle = asset_storage.insert({
            EntityTemplate::new()
                .with_component(LightFlash {
                    flash_type: LightFlashType::Strobe,
                    off_time: 35 * FRAME_TIME,
                    on_time: 5 * FRAME_TIME,
                    ..LightFlash::default()
                })
        });
        doomednums.insert(12, handle);

        // Fast strobe
        let handle = asset_storage.insert({
            EntityTemplate::new()
                .with_component(LightFlash {
                    flash_type: LightFlashType::Strobe,
                    off_time: 15 * FRAME_TIME,
                    on_time: 5 * FRAME_TIME,
                    ..LightFlash::default()
                })
        });
        doomednums.insert(13, handle);

        // Door open 300 s after level start
        let handle = asset_storage.insert({
            EntityTemplate::new()
        });
        doomednums.insert(14, handle);

        // 20% damage
        let handle = asset_storage.insert({
            EntityTemplate::new()
        });
        doomednums.insert(16, handle);

        // Random flicker
        let handle = asset_storage.insert({
            EntityTemplate::new()
        });
        doomednums.insert(17, handle);

        SectorTypes { doomednums }
    }
}
