use std::sync::Arc;

use bevy::prelude::*;
use houtu_scene::{GeographicTilingScheme, TilingScheme};

pub struct Plugin;
impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup);
        app.add_system(system);
    }
}
#[derive(Component)]
pub struct TileDatasourceMark;

#[derive(Component, Debug)]
pub struct TilingSchemeWrap<T: TilingScheme>(pub Arc<T>);
impl<T: TilingScheme> Clone for TilingSchemeWrap<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

#[derive(Component)]
pub struct QuadTreeTileDatasourceMark;

#[derive(Component)]
pub struct Ready(pub bool);

#[derive(Bundle)]
pub struct TileDatasource<T: TilingScheme + Sync + Send + 'static> {
    pub mark: TileDatasourceMark,
    pub tiling_scheme: TilingSchemeWrap<T>,
    pub ready: Ready,
}
fn setup(mut commands: Commands) {
    commands.spawn((
        TileDatasource {
            mark: TileDatasourceMark,
            tiling_scheme: TilingSchemeWrap(Arc::new(GeographicTilingScheme::default())),
            ready: Ready(false),
        },
        QuadTreeTileDatasourceMark,
    ));
}
fn system(mut commands: Commands, mut query: Query<(Entity), With<QuadTreeTileDatasourceMark>>) {
    if query.iter().len() != 1 {
        panic!("根数据源只能有1个")
    }
    for (entity) in &mut query {}
}
