use crate::prelude::*;

pub mod fan;
pub mod lock_zone;
pub mod player_spawner;
pub mod word_tag;
pub use word_tag::*;
pub use lock_zone::*;
pub use player_spawner::*;
pub use fan::*;

pub trait WorldObject: Component {
    type Bundle: Bundle;
    type InWorld;

    fn bundle(in_world: &Self::InWorld, assets: &MiscAssets) -> Self::Bundle;
}
