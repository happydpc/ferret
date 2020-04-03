use crate::{
	assets::AssetStorage,
	doom::map::{Map, MapDynamic, SectorDynamic},
};
use rand::Rng;
use rand_pcg::Pcg64Mcg;
use specs::{
	Component, DenseVecStorage, Join, ReadExpect, ReadStorage, RunNow, World, WriteExpect,
	WriteStorage,
};
use specs_derive::Component;
use std::time::Duration;

#[derive(Default)]
pub struct LightUpdateSystem;

impl<'a> RunNow<'a> for LightUpdateSystem {
	fn setup(&mut self, _world: &mut World) {}

	fn run_now(&mut self, world: &'a World) {
		let (
			map_storage,
			delta,
			mut light_flash_component,
			mut light_glow_component,
			map_dynamic_component,
			mut sector_dynamic_component,
			mut rng,
		) = world.system_data::<(
			ReadExpect<AssetStorage<Map>>,
			ReadExpect<Duration>,
			WriteStorage<LightFlash>,
			WriteStorage<LightGlow>,
			ReadStorage<MapDynamic>,
			WriteStorage<SectorDynamic>,
			WriteExpect<Pcg64Mcg>,
		)>();

		for (sector_dynamic, light_flash) in
			(&mut sector_dynamic_component, &mut light_flash_component).join()
		{
			if let Some(new_time) = light_flash.time_left.checked_sub(*delta) {
				light_flash.time_left = new_time;
			} else {
				light_flash.state = !light_flash.state;
				let map_dynamic = map_dynamic_component
					.get(sector_dynamic.map_entity)
					.expect("map_entity does not have MapDynamic component");
				let map = map_storage.get(&map_dynamic.map).unwrap();
				let sector = &map.sectors[sector_dynamic.index];

				let max_light = sector.light_level;
				let min_light = sector
					.neighbours
					.iter()
					.map(|index| map.sectors[*index].light_level)
					.min_by(|x, y| x.partial_cmp(y).unwrap())
					.unwrap_or(0.0);

				match light_flash.flash_type {
					LightFlashType::Broken => {
						if light_flash.state {
							light_flash.time_left = light_flash.on_time
								* (rng.gen::<bool>() as u32)
								+ crate::doom::FRAME_TIME;
							sector_dynamic.light_level = max_light;
						} else {
							light_flash.time_left = light_flash.off_time.mul_f64(rng.gen::<f64>())
								+ crate::doom::FRAME_TIME;
							sector_dynamic.light_level = min_light;
						}
					}
					LightFlashType::Strobe => {
						if light_flash.state {
							light_flash.time_left = light_flash.on_time;
							sector_dynamic.light_level = max_light;
						} else {
							light_flash.time_left = light_flash.off_time;
							sector_dynamic.light_level = if min_light == max_light {
								0.0
							} else {
								min_light
							};
						}
					}
					LightFlashType::StrobeUnSync(time) => {
						light_flash.time_left =
							time.mul_f64(rng.gen::<f64>()) + crate::doom::FRAME_TIME;
						light_flash.flash_type = LightFlashType::Strobe;
					}
				}
			}
		}

		for (sector_dynamic, light_glow) in
			(&mut sector_dynamic_component, &mut light_glow_component).join()
		{
			let map_dynamic = map_dynamic_component
				.get(sector_dynamic.map_entity)
				.expect("map_entity does not have MapDynamic component");
			let map = map_storage.get(&map_dynamic.map).unwrap();
			let sector = &map.sectors[sector_dynamic.index];
			let speed = light_glow.speed * delta.as_secs_f32();

			if light_glow.state {
				sector_dynamic.light_level += speed;
				let max_light = sector.light_level;

				if sector_dynamic.light_level > max_light {
					sector_dynamic.light_level = 2.0 * max_light - sector_dynamic.light_level;
					light_glow.state = !light_glow.state;
				}
			} else {
				sector_dynamic.light_level -= speed;
				let min_light = sector
					.neighbours
					.iter()
					.map(|index| map.sectors[*index].light_level)
					.min_by(|x, y| x.partial_cmp(y).unwrap())
					.unwrap_or(0.0);

				if sector_dynamic.light_level < min_light {
					sector_dynamic.light_level = 2.0 * min_light - sector_dynamic.light_level;
					light_glow.state = !light_glow.state;
				}
			}
		}
	}
}

#[derive(Clone, Component, Copy, Debug, Default)]
pub struct LightFlash {
	pub on_time: Duration,
	pub off_time: Duration,
	pub time_left: Duration,
	pub state: bool,
	pub flash_type: LightFlashType,
}

#[derive(Clone, Copy, Debug)]
pub enum LightFlashType {
	Broken,
	Strobe,
	StrobeUnSync(Duration),
}

impl Default for LightFlashType {
	fn default() -> LightFlashType {
		LightFlashType::Broken
	}
}

#[derive(Clone, Component, Copy, Debug, Default)]
pub struct LightGlow {
	pub speed: f32,
	pub state: bool,
}
