use crate::prelude::*;

pub struct AssetPlugin;

impl Plugin for AssetPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<DeAssets>()
            .add_systems(PreStartup, load_assets);
    }
}

#[derive(Resource, Default)]
pub struct DeAssets {
    pub square_pale: Handle<Image>,
    pub tileset: Handle<Image>,
    pub font: Handle<Font>,
}

fn load_assets(
    asset_server: Res<AssetServer>, 
    mut assets: ResMut<DeAssets>,
    mut graybox: ResMut<graybox::GrayboxSettings>,
) {
    assets.square_pale = asset_server.load("square_pale.bmp");
    assets.tileset = asset_server.load("tileset.bmp");
    let font = asset_server.load("fonts/tempfont.ttf");
    assets.font = font.clone();
    graybox.font = font.clone();
}
