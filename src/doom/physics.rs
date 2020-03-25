use crate::{
	assets::AssetStorage,
	doom::{
		components::{BoxCollider, MapDynamic, SectorDynamic, Transform, Velocity},
		map::{Linedef, Map},
	},
	geometry::{Line2, Line3, AABB2, AABB3},
};
use lazy_static::lazy_static;
use nalgebra::{Vector2, Vector3};
use specs::{Join, ReadExpect, ReadStorage, RunNow, World, WriteStorage};
use std::time::Duration;

#[derive(Default)]
pub struct PhysicsSystem;

impl<'a> RunNow<'a> for PhysicsSystem {
	fn setup(&mut self, _world: &mut World) {}

	fn run_now(&mut self, world: &'a World) {
		let (
			delta,
			map_storage,
			box_collider_component,
			map_dynamic_component,
			sector_dynamic_component,
			mut transform_component,
			mut velocity_component,
		) = world.system_data::<(
			ReadExpect<Duration>,
			ReadExpect<AssetStorage<Map>>,
			ReadStorage<BoxCollider>,
			ReadStorage<MapDynamic>,
			ReadStorage<SectorDynamic>,
			WriteStorage<Transform>,
			WriteStorage<Velocity>,
		)>();

		let map_dynamic = map_dynamic_component.join().next().unwrap();
		let map = map_storage.get(&map_dynamic.map).unwrap();

		for (box_collider, transform, velocity) in (
			&box_collider_component,
			&mut transform_component,
			&mut velocity_component,
		)
			.join()
		{
			//transform.position += velocity.velocity * delta.as_secs_f32();
			let bbox = AABB3::from_radius_height(box_collider.radius, box_collider.height);

			movement_xy(
				*delta,
				*&map,
				&map_dynamic,
				&sector_dynamic_component,
				&bbox,
				&mut transform.position,
				&mut velocity.velocity,
			);
			movement_z(
				*delta,
				*&map,
				&map_dynamic,
				&sector_dynamic_component,
				&bbox,
				&mut transform.position,
				&mut velocity.velocity,
			);
		}
	}
}

fn movement_xy(
	delta: Duration,
	map: &Map,
	map_dynamic: &MapDynamic,
	sector_dynamic_component: &ReadStorage<SectorDynamic>,
	bbox: &AABB3,
	position: &mut Vector3<f32>,
	velocity: &mut Vector3<f32>,
) {
	if velocity[0] == 0.0 && velocity[1] == 0.0 {
		return;
	}

	let mut new_position = *position;
	let mut new_velocity = *velocity;
	let time_left = delta;

	{
		let mut move_step = Line3::new(new_position, new_velocity * time_left.as_secs_f32());
		move_step.dir[2] = 0.0;

		if let Some(intersect) = trace(
			&move_step,
			&bbox,
			map,
			map_dynamic,
			sector_dynamic_component,
		) {
			// Push back against the collision
			let change = intersect.normal * new_velocity.dot(&intersect.normal) * 1.01;
			new_velocity -= change;

			// Try another move
			let mut move_step = Line3::new(new_position, new_velocity * time_left.as_secs_f32());
			move_step.dir[2] = 0.0;

			if let Some(_intersect) = trace(
				&move_step,
				&bbox,
				map,
				map_dynamic,
				sector_dynamic_component,
			) {
				new_velocity = nalgebra::zero();
			} else {
				new_position += move_step.dir;
			}
		} else {
			new_position += move_step.dir;
		}
	}

	*position = new_position;
	*velocity = new_velocity;
}

#[derive(Clone, Copy, Debug)]
struct Intersect {
	fraction: f32,
	normal: Vector3<f32>,
}

fn trace(
	move_step: &Line3,
	entity_bbox: &AABB3,
	map: &Map,
	map_dynamic: &MapDynamic,
	sector_dynamic_component: &ReadStorage<SectorDynamic>,
) -> Option<Intersect> {
	let move_step2 = Line2::from(move_step);
	let current_bbox = AABB2::from(entity_bbox).offset(move_step2.point);
	let move_bbox = current_bbox.union(&current_bbox.offset(move_step2.dir));

	let bbox_corners = [
		Vector2::new(current_bbox.min[0], current_bbox.min[1]),
		Vector2::new(current_bbox.min[0], current_bbox.max[1]),
		Vector2::new(current_bbox.max[0], current_bbox.max[1]),
		Vector2::new(current_bbox.max[0], current_bbox.min[1]),
	];

	let mut ret: Option<Intersect> = None;

	for linedef in map.linedefs.iter() {
		if let Some(intersect) = intersect_linedef(&move_step2, &move_bbox, &bbox_corners, linedef)
		{
			if intersect.fraction < ret.as_ref().map_or(1.0, |x| x.fraction) {
				if let [Some(front_sidedef), Some(back_sidedef)] = &linedef.sidedefs {
					let front_sector = sector_dynamic_component
						.get(map_dynamic.sectors[front_sidedef.sector_index])
						.unwrap();
					let back_sector = sector_dynamic_component
						.get(map_dynamic.sectors[back_sidedef.sector_index])
						.unwrap();

					if !(front_sector.floor_height <= move_step.point[2] + entity_bbox.min[2]
						&& back_sector.floor_height <= move_step.point[2] + entity_bbox.min[2]
						&& front_sector.ceiling_height >= move_step.point[2] + entity_bbox.max[2]
						&& back_sector.ceiling_height >= move_step.point[2] + entity_bbox.max[2])
					{
						ret = Some(intersect);
					}
				} else {
					ret = Some(intersect);
				}
			}
		}
	}

	ret
}

lazy_static! {
	static ref BBOX_NORMALS: [Vector3<f32>; 4] = [
		Vector3::new(-1.0, 0.0, 0.0),
		Vector3::new(0.0, 1.0, 0.0),
		Vector3::new(1.0, 0.0, 0.0),
		Vector3::new(0.0, -1.0, 0.0),
	];
}

fn intersect_linedef(
	move_step: &Line2,
	move_bbox: &AABB2,
	bbox_corners: &[Vector2<f32>; 4],
	linedef: &Linedef,
) -> Option<Intersect> {
	if !move_bbox.overlaps(&linedef.bbox) {
		return None;
	}

	let mut ret: Option<Intersect> = None;

	for i in 0..4 {
		// Intersect bbox corner with linedef
		if let Some((fraction, linedef_fraction)) =
			Line2::new(bbox_corners[i], move_step.dir).intersect(&linedef.line)
		{
			if fraction >= 0.0
				&& fraction < ret.as_ref().map_or(1.0, |x| x.fraction)
				&& linedef_fraction >= 0.0
				&& linedef_fraction <= 1.0
			{
				ret = Some(Intersect {
					fraction,
					normal: if move_step.dir.dot(&linedef.normal) > 0.0 {
						// Flip the normal if we're on the left side of the linedef
						Vector3::new(-linedef.normal[0], -linedef.normal[1], 0.0)
					} else {
						Vector3::new(linedef.normal[0], linedef.normal[1], 0.0)
					},
				});
			}
		}

		// Intersect linedef vertices with bbox edge
		let bbox_edge = Line2::new(bbox_corners[i], bbox_corners[(i + 1) % 4] - bbox_corners[i]);
		let linedef_vertices = [linedef.line.point, linedef.line.point + linedef.line.dir];

		for vertex in &linedef_vertices {
			if let Some((fraction, edge_fraction)) =
				Line2::new(*vertex, -move_step.dir).intersect(&bbox_edge)
			{
				if fraction >= 0.0
					&& fraction < ret.as_ref().map_or(1.0, |x| x.fraction)
					&& edge_fraction >= 0.0
					&& edge_fraction <= 1.0
				{
					ret = Some(Intersect {
						fraction,
						normal: -BBOX_NORMALS[i],
					});
				}
			}
		}
	}

	ret
}

fn movement_z (
	delta: Duration,
	map: &Map,
	map_dynamic: &MapDynamic,
	sector_dynamic_component: &ReadStorage<SectorDynamic>,
	bbox: &AABB3,
	position: &mut Vector3<f32>,
	velocity: &mut Vector3<f32>,
) {
	if velocity[2] == 0.0 {
		return;
	}

	let mut new_position = *position;
	let mut new_velocity = *velocity;

	let ssect = map.find_subsector(Vector2::new(new_position[0], new_position[1]));
	let sector = sector_dynamic_component
		.get(map_dynamic.sectors[ssect.sector_index])
		.unwrap();

	let min = sector.floor_height;
	let max = sector.ceiling_height;

	new_position[2] += new_velocity[2] * delta.as_secs_f32();

	if new_position[2] <= min - bbox.min[2] {
		new_position[2] = min - bbox.min[2];

		if new_velocity[2] < 0.0 {
			new_velocity[2] = 0.0;
		}
	} else if new_position[2] >= max - bbox.max[2] {
		new_position[2] = max - bbox.max[2];

		if new_velocity[2] > 0.0 {
			new_velocity[2] = 0.0;
		}
	}

	*position = new_position;
	*velocity = new_velocity;
}
