use crate::prelude::*;

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
    pub square_yellow: Handle<Image>,
    pub square_pink: Handle<Image>,
    pub horse: Handle<Image>,
    pub tileset: Handle<Image>,
    pub font: Handle<Font>,
    pub word_tag_sprites: HashMap<WordID, Handle<Image>>,
}

fn load_assets(
    asset_server: Res<AssetServer>, 
    mut assets: ResMut<MiscAssets>,
    mut graybox: ResMut<graybox::GrayboxSettings>,
) {
    assets.square_pale = asset_server.load("square_pale.bmp");
    assets.square_yellow = asset_server.load("square_yellow.bmp");
    assets.square_pink = asset_server.load("square_pink.bmp");
    assets.horse = asset_server.load("horse.bmp");
    assets.tileset = asset_server.load("tileset.bmp");

    let font = asset_server.load("fonts/tempfont.ttf");
    assets.font = font.clone();
    graybox.font = font.clone();

    for word in ALL_WORDS {
        let tag_name = format!("{}_tag.bmp", word.forms().filename);
        assets.word_tag_sprites.insert(word, asset_server.load(tag_name));
    }
}
