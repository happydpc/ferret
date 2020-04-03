use crate::{
	assets::AssetHandle,
	audio::Sound,
	doom::map::{MapDynamic, SectorRef},
};
use specs::{
	Component, DenseVecStorage, Entities, Entity, Join, ReadExpect, ReadStorage, RunNow, World,
	WriteExpect, WriteStorage,
};
use specs_derive::Component;
use std::time::Duration;

#[derive(Default)]
pub struct DoorUpdateSystem;

impl<'a> RunNow<'a> for DoorUpdateSystem {
	fn setup(&mut self, _world: &mut World) {}

	fn run_now(&mut self, world: &'a World) {
		let (
			entities,
			delta,
			mut sound_queue,
			sector_ref_component,
			mut door_active_component,
			mut map_dynamic_component,
		) = world.system_data::<(
			Entities,
			ReadExpect<Duration>,
			WriteExpect<Vec<(AssetHandle<Sound>, Entity)>>,
			ReadStorage<SectorRef>,
			WriteStorage<DoorActive>,
			WriteStorage<MapDynamic>,
		)>();

		let mut done = Vec::new();

		for (entity, sector_ref, door_active) in
			(&entities, &sector_ref_component, &mut door_active_component).join()
		{
			let map_dynamic = map_dynamic_component
				.get_mut(sector_ref.map_entity)
				.unwrap();
			let sector_dynamic = &mut map_dynamic.sectors[sector_ref.index];

			match door_active.state {
				DoorState::Closed => {
					door_active.state = DoorState::Opening;

					// Play sound
					sound_queue.push((door_active.open_sound.clone(), entity));
				}
				DoorState::Opening => {
					sector_dynamic.ceiling_height += door_active.speed * delta.as_secs_f32();

					if sector_dynamic.ceiling_height > door_active.open_height {
						sector_dynamic.ceiling_height = door_active.open_height;
						door_active.state = DoorState::Open;
					}
				}
				DoorState::Open => {
					if let Some(new_time) = door_active.time_left.checked_sub(*delta) {
						door_active.time_left = new_time;
					} else {
						door_active.state = DoorState::Closing;

						// Play sound
						sound_queue.push((door_active.close_sound.clone(), entity));
					}
				}
				DoorState::Closing => {
					sector_dynamic.ceiling_height -= door_active.speed * delta.as_secs_f32();

					if sector_dynamic.ceiling_height < door_active.close_height {
						done.push(entity);
					}
				}
			}
		}

		for entity in done {
			door_active_component.remove(entity);
		}
	}
}

#[derive(Clone, Component, Debug)]
pub struct DoorActive {
	pub open_sound: AssetHandle<Sound>,
	pub open_height: f32,

	pub close_sound: AssetHandle<Sound>,
	pub close_height: f32,

	pub state: DoorState,
	pub speed: f32,
	pub time_left: Duration,
}

#[derive(Clone, Copy, Debug)]
pub enum DoorState {
	Closed,
	Opening,
	Open,
	Closing,
}

#[derive(Clone, Component, Debug)]
pub struct DoorUse {
	pub open_sound: AssetHandle<Sound>,
	pub close_sound: AssetHandle<Sound>,
	pub speed: f32,
	pub wait_time: Duration,
}
