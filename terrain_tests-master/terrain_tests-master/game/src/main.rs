//! Demonstrates how to use the fly camera
#[macro_use]
extern crate amethyst_derive;
#[macro_use]
extern crate log;




use amethyst::{
    assets::{
        Completion,
        PrefabLoader, PrefabLoaderSystemDesc, ProgressCounter, RonFormat,
        HotReloadBundle, HotReloadStrategy
    },
    controls::{
        ArcBallControlBundle, ArcBallControlTag, FlyControlBundle, FlyControlTag,
    },
    core::{
        math::{Unit, UnitQuaternion, Vector3, Point3},
        transform::{Transform, TransformBundle},
        Time,
    },
    ecs::prelude::*,
    input::{is_key_down, InputBundle, StringBindings},
    prelude::*,
    gltf::GltfSceneLoaderSystemDesc,
    utils::{
        application_root_dir,
        auto_fov::{AutoFov, AutoFovSystem},
    },
    winit::{VirtualKeyCode},
    renderer::{
        camera::{ActiveCamera, Camera},
        debug_drawing::{DebugLinesComponent, DebugLinesParams},
        palette::{Srgb, Srgba},
        plugins::{RenderToWindow, RenderDebugLines, RenderSkybox, RenderPbr3D, RenderShaded3D},
        RenderingBundle,
        types::{DefaultBackend},
    }
};
use amethyst_terrain::*;
use std::path::Path;

// use gfx_core::format::ChannelType;

use prefab_data::{Scene, ScenePrefabData};

use overfly::{Overfly, OverflySystem, Coord3};
use flo_curves::bezier::Curve;
mod prefab_data;
mod overfly;


struct Orbit {
    axis: Unit<Vector3<f32>>,
    time_scale: f32,
    center: Vector3<f32>,
    radius: f32,
    height: f32,
}

impl Component for Orbit {
    type Storage = DenseVecStorage<Self>;
}

struct OrbitSystem;

impl<'a> System<'a> for OrbitSystem {
    type SystemData = (
        Read<'a, Time>,
        ReadStorage<'a, Orbit>,
        WriteStorage<'a, Transform>,
    );

    fn run(&mut self, (time, orbits, mut transforms): Self::SystemData) {
        for (orbit, transform) in (&orbits, &mut transforms).join() {
            let angle = time.absolute_time_seconds() as f32 * orbit.time_scale;
            let cross = orbit.axis.cross(&Vector3::z()).normalize() * orbit.radius;
            let rot = UnitQuaternion::from_axis_angle(&orbit.axis, angle);
            let final_pos = (rot * cross) + orbit.center;
            transform.set_translation(final_pos);
            transform.set_translation_y(orbit.height);
            transform.face_towards(orbit.center, [0., 1., 0.].into());
        }
    }
}
#[derive(Debug)]
enum ViewMode {
    Overfly,
    FlyControlTag
}
struct Example {
    progress: Option<ProgressCounter>,
    entity: Option<Entity>,
    initialised: bool,
    camera: Option<Entity>,
    view_mode: ViewMode,
    dispatcher: Dispatcher<'static, 'static>,

}
impl Example {
    pub fn new() -> Self {
        Self {
            entity: None,
            initialised: false,
            progress: None,
            camera: None,
            view_mode: ViewMode::Overfly,
            dispatcher: DispatcherBuilder::new().with(OverflySystem::default(), "overfly", &[]).build()
        }
    }

    fn get_curve() -> Overfly {
        let curves = vec![
            Curve { 
                start_point: Coord3(512.0, 100.0, 0.0),
                control_points: (Coord3(768.0, 150.0, 0.0), Coord3(1024.0, 100.0, 256.0),),
                end_point: Coord3(1024.0, 200.0, 512.0),
            },
            Curve { 
                start_point: Coord3(1024.0, 200.0, 512.0),
                control_points: (Coord3(1024.0, 300.0, 768.0), Coord3(768.0, 300.0, 1024.0),),
                end_point: Coord3(512.0, 300.0, 1024.0),
            },
            Curve { 
                start_point: Coord3(512.0, 300.0, 1024.0),
                control_points: (Coord3(256.0, 300.0, 1024.0), Coord3(0.0, 400.0, 768.0),),
                end_point: Coord3(0.0, 200.0, 512.0),
            },
            Curve { 
                start_point: Coord3(0.0, 200.0, 512.0),
                control_points: (Coord3(0.0, 0.0, 256.0), Coord3(256.0, 50.0, 0.0),),
                end_point: Coord3(512.0, 100.0, 0.0),
            }
        ];
        Overfly {
            curves,
            time_scale: 0.5
        }
    }
}

impl SimpleState for Example {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let StateData { world, .. } = data;
        world.insert(TerrainConfig {
            view_mode: TerrainViewMode::Wireframe,
            ..Default::default()
        });

        // world.register::<Transform>();
        world.register::<Orbit>();
        world.register::<Overfly>();


        self.progress = Some(ProgressCounter::default());

        world.exec(
            |(loader, mut scene): (PrefabLoader<'_, ScenePrefabData>, Write<'_, Scene>)| {
                println!("Start loading test.ron...");
                scene.handle = Some(
                    loader.load(
                        Path::new("prefab")
                            .join("test.ron")
                            .to_string_lossy(),
                        RonFormat,
                        self.progress.as_mut().unwrap(),
                    ),
                );
            },
        );

        

        // Configure width of lines. Optional step
        world.insert(DebugLinesParams {
            line_width: 1.0 / 50.0,
        });
        let mut debug_lines_component = DebugLinesComponent::with_capacity(100);

        let width: u32 = 200;
        let main_color = Srgba::new(0.4, 0.4, 0.4, 1.0);
        let primary_color = Srgba::new(0., 1.0, 0., 1.0);

        // Center
        debug_lines_component.add_direction(
            [-500., 0., 0.].into(),
            [1000., 0., 0.].into(),
            primary_color,
        );
        debug_lines_component.add_direction(
            [0., -500., 0.].into(),
            [0., 1000., 0.].into(),
            primary_color,
        );
        debug_lines_component.add_direction(
            [0., 0., -500.].into(),
            [0., 0., 1000.].into(),
            primary_color,
        );

        // Grid lines in X-axis
        for x in 0..=width {
            let (x, width) = (x as f32, width as f32);

            let position = Point3::new(x, 0.0, 0.);

            let direction = Vector3::new(0.0, 0.0, width);
            debug_lines_component.add_direction(position, direction, primary_color);
            let direction = Vector3::new(0.0, 0.0, -width);
            debug_lines_component.add_direction(position, direction, main_color);

            let position = Point3::new(-x, 0.0, 0.);

            let direction = Vector3::new(0.0, 0.0, width);
            debug_lines_component.add_direction(position, direction, main_color);
            let direction = Vector3::new(0.0, 0.0, -width);
            debug_lines_component.add_direction(position, direction, main_color);

            let position = Point3::new(0., 0.0, x);
            let direction = Vector3::new(width, 0.0, 0.0);
            debug_lines_component.add_direction(position, direction, primary_color);
            let direction = Vector3::new(-width, 0.0, 0.0);
            debug_lines_component.add_direction(position, direction, main_color);

            let position = Point3::new(0.0, 0.0, -x);
            let direction = Vector3::new(width, 0.0, 0.0);
            debug_lines_component.add_direction(position, direction, main_color);
            let direction = Vector3::new(-width, 0.0, 0.0);
            debug_lines_component.add_direction(position, direction, main_color);
        }
        world.register::<DebugLinesComponent>();
        world
            .create_entity()
            .with(debug_lines_component)
            .build();

        let mut transform = Transform::default();
        transform.set_translation_xyz(512.0, 50.0, 512.0);

        let _center_entity = world.create_entity().with(transform).build();

        let mut pos = Transform::default();
        pos.set_translation_xyz(212., 200., 512.);
        
        let mut auto_fov = AutoFov::default();
        auto_fov.set_base_fovx(std::f32::consts::FRAC_PI_3);
        auto_fov.set_base_aspect_ratio(1, 1);

        

        self.camera = Some(world
            .create_entity()
            .with(Camera::standard_3d(16.0, 9.0))
            .with(pos)
            .with(auto_fov)
            // .with(FlyControlTag)
            .with(Self::get_curve())
            // .with(ArcBallControlTag {target: center_entity, distance: 100.})
            // .with(Orbit {
            //     axis: Unit::new_normalize(Vector3::y()),
            //     time_scale: 0.5,
            //     Vector3::new(512.0, 50., 512.),
            //     radius: 300.,
            //     height: 150.,
            // })
            .build());

        world.insert(ActiveCamera { entity: self.camera })
    }
    fn update(&mut self, data: &mut StateData<GameData>) -> SimpleTrans {
        if !self.initialised {
            let remove = match self.progress.as_ref().map(|p| p.complete()) {
                None | Some(Completion::Loading) => false,

                Some(Completion::Complete) => {
                    let scene_handle = data
                        .world
                        .read_resource::<Scene>()
                        .handle
                        .as_ref()
                        .unwrap()
                        .clone();
                    println!("Loading complete.");
                    data.world.create_entity().with(scene_handle).build();
                    true
                }

                Some(Completion::Failed) => {
                    println!("Error: {:?}", self.progress.as_ref().unwrap().errors());
                    return Trans::Quit;
                }
            };
            if remove {
                self.progress = None;
            }
        }
        Trans::None
    }

    fn fixed_update(&mut self, data: StateData<GameData>) -> SimpleTrans {
        self.dispatcher.dispatch(&data.world);
        Trans::None
    }

    fn handle_event(
        &mut self,
        data: StateData<'_, GameData<'_, '_>>,
        event: StateEvent,
    ) -> SimpleTrans {
        if let StateEvent::Window(event) = event {
            if is_key_down(&event, VirtualKeyCode::Escape) {
                Trans::Quit
            } else if is_key_down(&event, VirtualKeyCode::R) {
                if let Some(camera) = self.camera {
                    self.view_mode = match self.view_mode {
                        ViewMode::FlyControlTag => {
                            data.world.exec(|(mut overfly, mut fly): (WriteStorage<Overfly>, WriteStorage<FlyControlTag>)| {
                                fly.remove(camera);
                                overfly.insert(camera, Self::get_curve());
                            });
                            ViewMode::Overfly},
                        ViewMode::Overfly => {
                            data.world.exec(|(mut overfly, mut fly): (WriteStorage<Overfly>, WriteStorage<FlyControlTag>)| {
                                overfly.remove(camera);
                                fly.insert(camera, FlyControlTag);
                            });
                            ViewMode::FlyControlTag
                        },
                    };
                    debug!("Switching to {:?}", &self.view_mode);
                }    
                Trans::None
            } else if is_key_down(&event, VirtualKeyCode::F11)
                || is_key_down(&event, VirtualKeyCode::Return)
                    && (is_key_down(&event, VirtualKeyCode::LAlt)
                        || is_key_down(&event, VirtualKeyCode::RAlt))
            {
                // Todo : Add Fullscreen Toggle
                Trans::None
            } else if is_key_down(&event, VirtualKeyCode::F) {
                data.world.exec(|mut terrain_config: Write<TerrainConfig>| {
                    // Todo : Reimplement ViewMode support
                    terrain_config.view_mode = match terrain_config.view_mode {
                        TerrainViewMode::Wireframe => TerrainViewMode::Color,
                        TerrainViewMode::Color => TerrainViewMode::LOD,
                        TerrainViewMode::LOD => TerrainViewMode::Wireframe,
                    };
                    debug!("Switching to {:?}", &terrain_config.view_mode);
                });
                Trans::None
            } else {
                Trans::None
            }
        } else {
            Trans::None
        }
    }
}

fn main() -> amethyst::Result<()> {
    // amethyst::Logger::from_config(amethyst::LoggerConfig{level_filter: log::LevelFilter::Trace, ..Default::default()})
    //     .level_for("gfx_device_vulkan", log::LevelFilter::Trace)
    //     .level_for("amethyst_assets", log::LevelFilter::Warn)
    //     .start();

    amethyst::Logger::from_config(amethyst::LoggerConfig {
        // log_file: Some("terrain.log".into()),
        // level_filter: log::LevelFilter::Trace,
        level_filter: log::LevelFilter::Warn,
        ..Default::default()
    })
    .level_for("amethyst_utils::fps_counter", log::LevelFilter::Debug)
    .level_for("gfx_backend_vulkan", log::LevelFilter::Warn)
    .level_for("amethyst_assets", log::LevelFilter::Warn)
    // .level_for("rendy_factory", log::LevelFilter::Trace)
    // .level_for("rendy_resource", log::LevelFilter::Trace)
    // .level_for("rendy_descriptor", log::LevelFilter::Trace)
    .start();

    let app_root = application_root_dir()?;
    println!("Application Root: {:?}", app_root);

    let resources = app_root.join("resources");
    let display_config = resources.join("display.ron");


    let key_bindings_path = resources.join("input.ron");


    let game_data = GameDataBuilder::default()
        // .with(TerrainSystem::default(), "terrain_system", &["prefab"])
        .with(OrbitSystem, "orbit", &[])
        .with(AutoFovSystem::new(), "auto_fov", &[])
        .with_system_desc(
            PrefabLoaderSystemDesc::<ScenePrefabData>::default(),
            "scene_loader",
            &[],
        )
        .with_system_desc(
            GltfSceneLoaderSystemDesc::default(),
            "gltf_loader",
            &["scene_loader"], // This is important so that entity instantiation is performed in a single frame.
        )
        .with_bundle(
            InputBundle::<StringBindings>::new().with_bindings_from_file(&key_bindings_path)?,
        )?
        .with_bundle(HotReloadBundle::new(HotReloadStrategy::every(2)))?
        .with_bundle(
            FlyControlBundle::<StringBindings>::new(
                Some("horizontal".into()),
                None,
                Some("vertical".into()),
            )
            .with_sensitivity(0.1, 0.1)
            .with_speed(50.),
        )?
        .with_bundle(TransformBundle::new().with_dep(&[
            "orbit",
            "fly_movement"
        ]))?
        .with_bundle(
            RenderingBundle::<DefaultBackend>::new()
                .with_plugin(
                    RenderToWindow::from_config_path(display_config)
                        .with_clear([0.34, 0.36, 0.52, 1.0]),
                )
                .with_plugin(RenderTerrain::default())
                .with_plugin(RenderPbr3D::default())
                .with_plugin(RenderDebugLines::default())
                .with_plugin(RenderSkybox::with_colors(
                    Srgb::new(0.82, 0.51, 0.50),
                    Srgb::new(0.18, 0.11, 0.85),
                )),
        )?;

    let mut game = Application::new(&resources, Example::new(), game_data)?;

    game.run();

    Ok(())
}