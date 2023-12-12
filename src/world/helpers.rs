use crate::prelude::*;
use crate::world::*;

/// Finds the level that contains a given position.
pub fn level_is_in_position(
    level: (&LoadedLevel, &Transform),
    position: Vec2,
) -> Option<Rect> {
    let size = grid_size(&level.0.tiles) * 16.;

    let level_rect = Rect::from_corners(
        level.1.translation.xy() - Vec2::splat(8.),
        level.1.translation.xy() + size - Vec2::splat(8.),
    );

    level_rect.contains(position).then_some(level_rect)
}
