use crate::prelude::*;

use super::spawn::*;

#[derive(WorldQuery)]
pub struct QWordObject {
    pub words: &'static WordObject,
    pub entity: Entity,
}

pub fn apply_scalers(
    mut word_objects: Query<(QWordObject, &mut Collider, &GlobalTransform, &mut Transform)>,
    transforms: Query<&GlobalTransform>,
    phys_context: Res<RapierContext>,
){
    const SHRINK_SPEED: f32 = 0.15;

    for mut object in &mut word_objects {
        let target_scale = {
            let mut target_scale = Vec2::ONE;

            let adjectives = &object.0.words.adjectives;

            if adjectives.tall { target_scale.y *= 4.; }
            if adjectives.wide { target_scale.x *= 4.; }
            if adjectives.baby { target_scale *= 0.5; }

            target_scale
        };

        let old_scale = object.3.scale.xy();
        if (old_scale - target_scale).length() <= 0.01 {
            continue;
        }


        let scale_diff = Vec2::splat(1.) + (target_scale - old_scale) * SHRINK_SPEED;

        let babied_shape = object.1.as_typed_shape()
            .raw_scale_by(scale_diff, 0)
            .unwrap();
        let babied_col = Collider::from(babied_shape);

        let (_, rotation, translation) = object.2.to_scale_rotation_translation();

        let mut pushback_vector = Vec2::ZERO;
        phys_context.intersections_with_shape(
            translation.xy(),
            rotation.z,
            &babied_col,
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

        let new_scale = old_scale * scale_diff;

        pushback_vector = pushback_vector.normalize_or_zero() * 0.1;

        object.3.translation += pushback_vector.extend(0.);
        object.3.scale.x = new_scale.x;
        object.3.scale.y = new_scale.y;
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
                    FlutteringDirection::Right => Vec2::new(16., 0.),
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
