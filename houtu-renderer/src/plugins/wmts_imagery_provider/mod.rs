use bevy::prelude::*;

pub struct Plugin;
impl bevy::prelude::Plugin for Plugin {
    fn name(&self) -> &str {
        "wmts_imagery_service"
    }
    fn build(&self, app: &mut App) {}
}
