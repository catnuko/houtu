use bevy::{
    pbr::{MaterialPipeline, MaterialPipelineKey},
    prelude::*,
    reflect::TypeUuid,
    render::{
        mesh::{MeshVertexBufferLayout, PrimitiveTopology},
        render_resource::{
            AsBindGroup, PolygonMode, RenderPipelineDescriptor, ShaderRef,
            SpecializedMeshPipelineError,
        },
    },
};
use houtu_scene::{
    GeographicProjection, GeographicTilingScheme, HeightmapTerrainData, IndicesAndEdgesCache,
    TilingScheme,
};
mod terrain_mesh;
use rand::Rng;

pub struct ScenePlugin;

impl bevy::app::Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(MaterialPlugin::<TerrainMeshMaterial>::default())
            .add_startup_system(setup);
    }
}
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut terrain_materials: ResMut<Assets<TerrainMeshMaterial>>,
    mut standard_materials: ResMut<Assets<StandardMaterial>>,
) {
    let tiling_scheme = GeographicTilingScheme::default();
    let level = 2;
    let num_of_x_tiles = tiling_scheme.get_number_of_x_tiles_at_level(level);
    let num_of_y_tiles = tiling_scheme.get_number_of_y_tiles_at_level(level);
    let mut indicesAndEdgesCache = IndicesAndEdgesCache::new();
    let c1 = [Color::WHITE, Color::GREEN];
    for y in 0..num_of_y_tiles {
        for x in 0..num_of_x_tiles {
            let width: u32 = 16;
            let height: u32 = 16;
            let buffer: Vec<f64> = vec![0.; (width * height) as usize];
            let mut heigmapTerrainData = HeightmapTerrainData::new(
                buffer, width, height, None, None, None, None, None, None, None,
            );
            let terrain_mesh = heigmapTerrainData._createMeshSync(
                &tiling_scheme,
                x,
                y,
                level,
                None,
                None,
                &mut indicesAndEdgesCache,
            );
            let center = terrain_mesh.center;
            let mesh = meshes.add(terrain_mesh::TerrainMeshWarp(terrain_mesh).into());
            let mut rng = rand::thread_rng();
            let r: f32 = rng.gen();
            let g: f32 = rng.gen();
            let b: f32 = rng.gen();
            commands.spawn(
                ({
                    MaterialMeshBundle {
                        mesh: mesh,
                        material: terrain_materials.add(TerrainMeshMaterial {
                            color: Color::rgba(r, g, b, 1.0),
                            // color: c1[x as usize],
                            center_3d: Color::rgba(
                                center.x as f32,
                                center.y as f32,
                                center.z as f32,
                                1.,
                            ),
                        }),
                        // material: standard_materials.add(Color::rgba(r, g, b, 1.0).into()),
                        ..Default::default()
                    }
                }),
            );
        }
    }
}
#[derive(Default, AsBindGroup, TypeUuid, Debug, Clone)]
#[uuid = "050ce6ac-080a-4d8c-b6b5-b5bab7560d81"]
pub struct TerrainMeshMaterial {
    #[uniform(0)]
    color: Color,
    #[uniform(0)]
    center_3d: Color,
}
impl Material for TerrainMeshMaterial {
    fn fragment_shader() -> ShaderRef {
        "terrain_material.wgsl".into()
    }
    fn vertex_shader() -> ShaderRef {
        "terrain_material.wgsl".into()
    }
    fn specialize(
        _pipeline: &MaterialPipeline<Self>,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayout,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        // This is the important part to tell bevy to render this material as a line between vertices
        descriptor.primitive.polygon_mode = PolygonMode::Line;
        Ok(())
    }
}
