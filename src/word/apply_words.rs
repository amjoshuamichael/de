use crate::prelude::*;

use super::spawn::*;

pub fn apply_wide(
    mut wides: Query<(&mut Collider, &GlobalTransform, &mut Transform, Entity), &WideMark>,
    all_transforms: Query<&GlobalTransform, Without<WideMark>>,
    phys_context: Res<RapierContext>,
) {
    const WIDEN_SPEED: f32 = 0.15;
    const MAX_WIDTH: f32 = 4.;

    for mut wide in &mut wides {
        if wide.2.scale.x >= MAX_WIDTH - 0.01 {
            continue;
        }

        let old_scale = wide.2.scale.x;
        let scale_diff = 1. + (MAX_WIDTH - old_scale) * WIDEN_SPEED;

        let current_collider_rect = wide.0.as_typed_shape();
        let widened_shape = current_collider_rect
            .raw_scale_by(Vec2::new(scale_diff, 0.99), 0)
            .unwrap();
        let widened_col = Collider::from(widened_shape);

        let (_, rotation, translation) = wide.1.to_scale_rotation_translation();

        let mut pushback_vector = Vec2::ZERO;

        phys_context.intersections_with_shape(
            translation.xy(),
            rotation.x, // TODO: get rotation here
            &widened_col,
            QueryFilter {
                exclude_collider: Some(wide.3),
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

        wide.2.translation += pushback_vector.extend(0.);
        wide.2.scale.x = new_scale;
    }
}

pub fn apply_tall(
    mut wides: Query<(&mut Collider, &GlobalTransform, &mut Transform, Entity), &TallMark>,
    all_transforms: Query<&GlobalTransform, Without<WideMark>>,
    phys_context: Res<RapierContext>,
) {
    const HEIGHTEN_SPEED: f32 = 0.1;
    const MAX_HEIGHT: f32 = 4.;

    for mut wide in &mut wides {
        if wide.2.scale.y >= MAX_HEIGHT - 0.01 {
            continue;
        }

        let old_scale = wide.2.scale.y;
        let scale_diff = 1. + (MAX_HEIGHT - old_scale) * HEIGHTEN_SPEED;

        let current_collider_rect = wide.0.as_typed_shape();
        let widened_shape = current_collider_rect
            .raw_scale_by(Vec2::new(0.99, scale_diff), 0)
            .unwrap();
        let widened_col = Collider::from(widened_shape);

        let (_, rotation, translation) = wide.1.to_scale_rotation_translation();

        let mut pushback_vector = Vec2::ZERO;

        phys_context.intersections_with_shape(
            translation.xy(),
            rotation.x, // TODO: get rotation here
            &widened_col,
            QueryFilter {
                exclude_collider: Some(wide.3),
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

        wide.2.translation += pushback_vector.extend(0.);
        wide.2.scale.y = new_scale;
    }
}

pub fn apply_fluttering(
    mut flutters: Query<(&FlutteringMark, Entity)>,
    parents: Query<&Parent>,
    mut velocities: Query<&mut Velocity>,
) {
    for flutter in &mut flutters {
        for ancestor in parents.iter_ancestors(flutter.1) {
            if let Ok(mut velocity) = velocities.get_mut(ancestor) {
                let dir_vector = match flutter.0.direction {
                    FlutteringDirection::Up => Vec2::new(0., 2.),
                    FlutteringDirection::Down => todo!(),
                    FlutteringDirection::Left => todo!(),
                    FlutteringDirection::Right => Vec2::new(10., 0.),
                };

                if (velocity.linvel * dir_vector).length() < dir_vector.length() * 100. {
                    velocity.linvel += dir_vector;
                } else {
                    // entity is already moving at a speed higher than 100 times the
                    // direction of the fan, we don't have to do anything.
                }
            }
            break;
        }
    }
}
