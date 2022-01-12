use bevy::prelude::*;
pub use eq_parse;

mod gen_mesh;
mod scaling;

macro_rules! exit {
    ($s:expr) => {{
        println!("{}", $s);
        std::process::exit(1);
    }}
}

fn main() {    
    App::build()
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(ClearColor(Color::NONE))
        .insert_resource(WindowDescriptor {
            title: "Grapher".to_string(),
            width: scaling::DEFAULT_WINDOW_WIDTH,
            height: scaling::DEFAULT_WINDOW_HEIGHT,
            vsync: true,
            ..Default::default()
        }).add_startup_system(setup.system())
        .add_plugins(DefaultPlugins)
        .add_system_set(SystemSet::new()
                    .label("input")
                    .with_system(scaling::zoom_system.system())
                    .with_system(scaling::pan_system.system())
                    .with_system(scaling::window_resize.system())
        ).add_system(scaling::recalculate_graphing_bounds_system.system()
                    .label("calc bounds")
                    .after("input"))
        .add_system(scaling::update_projection_system.system()
                    .after("input"))
        .add_system(gen_mesh::regenerate_meshes_system.system()
                    .after("calc bounds"))
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // parse equation
    let eq = match std::env::args().nth(1).as_deref() {
        None | Some("") => exit!("No equation passed"),
        Some(eq) => match eq_parse::parse_equation(&eq) {
            Ok(eq) => gen_mesh::Equation(eq),
            Err(_) => exit!("Error in expression"),
        },
    };

    // setup bounds
    let (graphing_bounds, view) = {
        use scaling::*;
        let mut camera = OrthographicCameraBundle::new_3d();

        // no need to initialize projection - will be done in scaling::update_projection_system
        camera.orthographic_projection.scaling_mode = bevy::render::camera::ScalingMode::None;
        camera.transform = Transform::from_xyz(0.0, 0.0, 999.0).looking_at(Vec3::ZERO, Vec3::Y);

        commands.spawn_bundle(camera);

        // temporarily remake window descriptor to calc regen bounds.
        // we cannot insert it here and must in main for some reason.
        let window_descriptor = WindowDescriptor {
            title: "Grapher".to_string(),
            width: DEFAULT_WINDOW_WIDTH,
            height: DEFAULT_WINDOW_HEIGHT,
            vsync: true,
            ..Default::default()
        };

        let view = GraphingView {
            centre: Vec2::ZERO,
            scale: DEFAULT_SCALE,
        };

        let graphing_bounds = recalculate_graphing_bounds(&view, &window_descriptor);
        
        (graphing_bounds, view)
    };

    // setup materials
    let (white, light_grey, dark_grey) = {
        let white = materials.add(StandardMaterial {
            base_color: Color::WHITE,
            reflectance: 0.0,
            roughness: 1.0,
            unlit: true,
            ..Default::default()
        });

        let light_grey = materials.add(StandardMaterial {
            base_color: Color::rgb_u8(100, 100, 100),
            reflectance: 0.0,
            roughness: 1.0,
            unlit: true,
            ..Default::default()
        });

        let dark_grey = materials.add(StandardMaterial {
            base_color: Color::rgb_u8(50, 50, 50),
            reflectance: 0.0,
            roughness: 1.0,
            unlit: true,
            ..Default::default()
        });

        (white, light_grey, dark_grey)
    };

    // spawn meshes
    {
        let xbounds = graphing_bounds.xbounds;

        commands.spawn_bundle(PbrBundle {
            mesh: meshes.add(gen_mesh::gen_mesh(&eq, xbounds)),
            material: white.clone(),
            transform: Transform::from_xyz(0.0, 0.0, 0.1),
            ..Default::default()
        }).insert(eq);

        let main_axis = meshes.add(gen_mesh::gen_main_axis(xbounds, xbounds));
        let mid_axis = meshes.add(gen_mesh::gen_mid_axis(xbounds, xbounds, view.scale));
        let min_axis = meshes.add(gen_mesh::gen_min_axis(xbounds, xbounds, view.scale));

        let grid_mesh_handles = gen_mesh::GridMeshHandles {
            main_axis,
            mid_axis,
            min_axis,
        };

        commands.spawn_bundle(PbrBundle {
            mesh: grid_mesh_handles.main_axis.clone(),
            material: white,
            ..Default::default()
        });

        commands.spawn_bundle(PbrBundle {
            mesh: grid_mesh_handles.mid_axis.clone(),
            material: light_grey,
            ..Default::default()
        });

        commands.spawn_bundle(PbrBundle {
            mesh: grid_mesh_handles.min_axis.clone(),
            material: dark_grey,
            ..Default::default()
        });

        commands.insert_resource(grid_mesh_handles);
    }; 

    commands.insert_resource(graphing_bounds);
    commands.insert_resource(view);
}

