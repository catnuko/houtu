use bevy::{prelude::*, utils::HashMap};

#[derive(Debug, Component, Hash, PartialEq, Eq, Clone)]
pub struct Label(pub &'static str);
#[derive(Debug, Resource)]
pub struct LabelToEntity {
    map: HashMap<Label, Entity>,
}
impl LabelToEntity {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }
}
pub struct Plugin;
impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(LabelToEntity::new());
        app.add_startup_system(setup);
        app.add_system(update_label);
    }
}
fn update_label(
    mut commands: Commands,
    mut label_to_entity: ResMut<LabelToEntity>,
    query: Query<(Entity, &Label), Added<Label>>,
) {
    for (entity, label) in &query {
        label_to_entity.map.insert(label.clone(), entity);
    }
}
fn setup(
    mut commands: Commands,
    mut label_to_entity: ResMut<LabelToEntity>,
    query: Query<(Entity, &Label)>,
) {
    for (entity, label) in &query {
        label_to_entity.map.insert(label.clone(), entity);
    }
}
