use crate::prelude::*;

use super::spawn::*;

#[derive(WorldQuery)]
pub struct QWordObject {
    pub words: &'static WordObject,
    pub entity: Entity,
}

pub fn apply_wide(
    mut word_objects: Query<(QWordObject, &mut Collider, &GlobalTransform, &mut Transform)>,
    transforms: Query<&GlobalTransform>,
    phys_context: Res<RapierContext>,
) {
    const WIDEN_SPEED: f32 = 0.15;

    for mut object in &mut word_objects {
        let max_width = if object.0.words.adjectives.wide { 4. } else { 1. };
        if (object.3.scale.x - max_width).abs() <= 0.01 {
            continue;
        }

        let old_scale = object.3.scale.x;
        let scale_diff = 1. + (max_width - old_scale) * WIDEN_SPEED;

        let widened_shape = object.1.as_typed_shape()
            .raw_scale_by(Vec2::new(scale_diff, 0.99), 0)
            .unwrap();
        let widened_col = Collider::from(widened_shape);

        let (_, rotation, translation) = object.2.to_scale_rotation_translation();

        let mut pushback_vector = Vec2::ZERO;

        phys_context.intersections_with_shape(
            translation.xy(),
            rotation.z,
            &widened_col,
            QueryFilter {
                exclude_collider: Some(object.0.entity),
                ..default()
            },
            |colliding_shape| {
                let col_pos = transforms.get(colliding_shape).unwrap().translation();
                pushback_vector += translation.xy() - col_pos.xy();
                true
            },
        );

        let new_scale = (old_scale * scale_diff).min(4.);

        pushback_vector = pushback_vector.normalize_or_zero() * 0.1;

        object.3.translation += pushback_vector.extend(0.);
        object.3.scale.x = new_scale;
    }
}

pub fn apply_tall(
    mut word_objects: Query<(QWordObject, &mut Collider, &GlobalTransform, &mut Transform)>,
    all_transforms: Query<&GlobalTransform>,
    phys_context: Res<RapierContext>,
) {
    const HEIGHTEN_SPEED: f32 = 0.1;

    for mut object in &mut word_objects {
        let max_height = if object.0.words.adjectives.tall { 4. } else { 1. };
        if (object.3.scale.y - max_height).abs() <= 0.01 {
            continue;
        }

        let old_scale = object.3.scale.y;
        let scale_diff = 1. + (max_height - old_scale) * HEIGHTEN_SPEED;

        let heightened_shape = object.1.as_typed_shape()
            .raw_scale_by(Vec2::new(0.99, scale_diff), 0)
            .unwrap();
        let heightened_col = Collider::from(heightened_shape);

        let (_, rotation, translation) = object.2.to_scale_rotation_translation();

        let mut pushback_vector = Vec2::ZERO;

        phys_context.intersections_with_shape(
            translation.xy(),
            rotation.z,
            &heightened_col,
            QueryFilter {
                exclude_collider: Some(object.0.entity),
                ..default()
            },
            |colliding_shape| {
                let col_pos = all_transforms.get(colliding_shape).unwrap().translation().xy();
                pushback_vector += translation.xy() - col_pos;
                true
            },
        );

        let new_scale = (old_scale * scale_diff).min(4.);

        pushback_vector = pushback_vector.normalize_or_zero() * 0.1;

        object.3.translation += pushback_vector.extend(0.);
        object.3.scale.y = new_scale;
    }
}

pub fn apply_fluttering(
    mut flutters: Query<QWordObject>,
    parents: Query<&Parent>,
    mut velocities: Query<&mut Velocity>,
    time: Res<Time>,
) {
    for flutter in &mut flutters {
        let Some(direction) = flutter.words.adjectives.fluttering else { continue };

        for ancestor in parents.iter_ancestors(flutter.entity) {
            if let Ok(mut velocity) = velocities.get_mut(ancestor) {
                let dir_vector = match direction {
                    FlutteringDirection::Up => Vec2::new(0., 4.),
                    FlutteringDirection::Down => todo!(),
                    FlutteringDirection::Left => todo!(),
                    FlutteringDirection::Right => Vec2::new(8., 0.),
                };

                if (velocity.linvel * dir_vector).length() < dir_vector.length() * 100. {
                    velocity.linvel += dir_vector * time.delta_seconds() * 60.;
                } else {
                    // entity is already moving at a speed higher than 100 times the
                    // direction of the fan, we don't have to do anything.
                }
            }
            break;
        }
    }
}
