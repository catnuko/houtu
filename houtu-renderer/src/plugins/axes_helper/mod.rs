use anyhow::Result;
use bevy::{
    asset::{AssetLoader, LoadContext, LoadedAsset},
    pbr::{wireframe::Wireframe, MaterialPipeline, MaterialPipelineKey},
    prelude::*,
    reflect::{erased_serde::__private::serde::Deserialize, TypeUuid},
    render::{
        mesh::{MeshVertexBufferLayout, PrimitiveTopology},
        render_resource::{
            AsBindGroup, PolygonMode, RenderPipelineDescriptor, ShaderRef,
            SpecializedMeshPipelineError,
        },
        renderer::RenderDevice,
        texture::{CompressedImageFormats, ImageTextureLoader, ImageType, TextureError},
    },
    utils::BoxedFuture,
};
use houtu_scene::Ellipsoid;

pub struct Plugin;
// impl Plugin {
//     pub fn new(length: f32) -> Self {
//         Self { length: length }
//     }
// }

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(MaterialPlugin::<LineMaterial>::default());
        app.add_startup_system(setup);
    }
}
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<LineMaterial>>,
) {
    let length = (Ellipsoid::WGS84.maximumRadius as f32) + 1000000.0;
    // Spawn a list of lines with start and end points for each lines
    commands.spawn(MaterialMeshBundle {
        mesh: meshes.add(Mesh::from(LineList {
            lines: vec![(Vec3::ZERO, Vec3::new(length, 0.0, 0.0))],
        })),
        material: materials.add(LineMaterial { color: Color::RED }),
        ..default()
    });
    commands.spawn(MaterialMeshBundle {
        mesh: meshes.add(Mesh::from(LineList {
            lines: vec![(Vec3::ZERO, Vec3::new(0.0, length, 0.0))],
        })),
        material: materials.add(LineMaterial {
            color: Color::GREEN,
        }),
        ..default()
    });
    commands.spawn(MaterialMeshBundle {
        mesh: meshes.add(Mesh::from(LineList {
            lines: vec![(Vec3::ZERO, Vec3::new(0.0, 0.0, length))],
        })),
        material: materials.add(LineMaterial { color: Color::BLUE }),
        ..default()
    });
}
/// A list of lines with a start and end position
#[derive(Debug, Clone)]
pub struct LineList {
    pub lines: Vec<(Vec3, Vec3)>,
}

impl From<LineList> for Mesh {
    fn from(line: LineList) -> Self {
        // This tells wgpu that the positions are list of lines
        // where every pair is a start and end point
        let mut mesh = Mesh::new(PrimitiveTopology::LineList);

        let vertices: Vec<_> = line.lines.into_iter().flat_map(|(a, b)| [a, b]).collect();
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
        mesh
    }
}
#[derive(Default, AsBindGroup, TypeUuid, Debug, Clone)]
#[uuid = "050ce6ac-080a-4d8c-b6b5-b5bab7560d8f"]
struct LineMaterial {
    #[uniform(0)]
    color: Color,
}

impl Material for LineMaterial {
    fn fragment_shader() -> ShaderRef {
        "line_material.wgsl".into()
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
