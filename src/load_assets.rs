use crate::{prelude::*, word::Words};

pub struct AssetPlugin;

impl Plugin for AssetPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<MiscAssets>()
            .add_systems(PreStartup, load_assets);
    }
}

#[derive(Resource, Default)]
pub struct MiscAssets {
    pub square_pale: Handle<Image>,
    pub tileset: Handle<Image>,
    pub font: Handle<Font>,
}

fn load_assets(
    asset_server: Res<AssetServer>, 
    mut assets: ResMut<MiscAssets>,
    mut graybox: ResMut<graybox::GrayboxSettings>,
    mut words: ResMut<Words>,
) {
    assets.square_pale = asset_server.load("square_pale.bmp");
    assets.tileset = asset_server.load("tileset.bmp");
    let font = asset_server.load("fonts/tempfont.ttf");
    assets.font = font.clone();
    graybox.font = font.clone();


    for word_data in words.0.values_mut() {
        let tag_name = format!("{}_tag.bmp", word_data.basic.to_lowercase());
        word_data.tag_handle = asset_server.load(tag_name);
    }
}
