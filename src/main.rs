use bevy::prelude::*;
pub use mathjit::expr_parse;

mod gen_mesh;
mod scaling;
mod curve_material;

#[allow(dead_code)]
mod axis_text;

use curve_material::CurveMaterial;

#[derive(Component, Copy, Clone)]
pub struct MainCamera {}
#[derive(Component, Copy, Clone)]
pub struct UICamera {}

macro_rules! exit {
    ($s:expr) => {{
        println!("{}", $s);
        std::process::exit(1);
    }}
}

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::NONE))
        .insert_resource(WindowDescriptor {
            title: "Grapher".to_string(),
            width: scaling::DEFAULT_WINDOW_WIDTH,
            height: scaling::DEFAULT_WINDOW_HEIGHT,
            vsync: true,
            ..Default::default() } )
        .add_startup_system(setup)
        .add_plugins(DefaultPlugins)
        .add_plugin(MaterialPlugin::<CurveMaterial>::default())
        .add_system_set(SystemSet::new()
                    .label("input")
                    .with_system(scaling::zoom_system)
                    .with_system(scaling::pan_system)
                    .with_system(scaling::window_resize))
        .add_system(scaling::update_projection_system.after("input"))
        .add_system(scaling::recalculate_graphing_bounds_system
                    .label("calc bounds").after("input"))
        .add_system(gen_mesh::regenerate_meshes_system.after("calc bounds"))
        //.add_system(axis_text::regenerate_axis_text_system
        //            .label("gen axis text").after("calc bounds"))
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut curve_materials: ResMut<Assets<CurveMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // parse exxpression
    let expr = match std::env::args().nth(1).as_deref() {
        None | Some("") => exit!("No expression passed"),
        Some(expr) => match expr_parse::parse_expression(&expr) {
            Ok(expr) => gen_mesh::Expression(expr),
            Err(_) => exit!("Error in expression"),
        },
    };

    // setup bounds
    let (graphing_bounds, view) = {
        use scaling::*;
        let mut camera = OrthographicCameraBundle::new_3d();
        let mut ui_camera = UiCameraBundle::default();

        // no need to initialize projection - will be done in scaling::update_projection_system
        camera.orthographic_projection.scaling_mode = bevy::render::camera::ScalingMode::None;
        camera.transform = Transform::from_xyz(0.0, 0.0, 999.0).looking_at(Vec3::ZERO, Vec3::Y);
        ui_camera.orthographic_projection.scaling_mode = bevy::render::camera::ScalingMode::None;
        ui_camera.transform = Transform::from_xyz(0.0, 0.0, 999.0).looking_at(Vec3::ZERO, Vec3::Y);

        commands.spawn_bundle(camera)
            .insert(MainCamera {});
        commands.spawn_bundle(ui_camera)
            .insert(UICamera {});

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
    let (const_width_curve, white, light_grey, dark_grey) = {
        let white = materials.add(StandardMaterial {
            base_color: Color::WHITE,
            reflectance: 0.0,
            perceptual_roughness: 1.0,
            unlit: true,
            ..Default::default()
        });

        let const_width_curve = curve_materials.add(CurveMaterial {
            //color: Color::WHITE,
        });

        let light_grey = materials.add(StandardMaterial {
            base_color: Color::rgb_u8(100, 100, 100),
            reflectance: 0.0,
            perceptual_roughness: 1.0,
            unlit: true,
            ..Default::default()
        });

        let dark_grey = materials.add(StandardMaterial {
            base_color: Color::rgb_u8(50, 50, 50),
            reflectance: 0.0,
            perceptual_roughness: 1.0,
            unlit: true,
            ..Default::default()
        });

        (const_width_curve, white, light_grey, dark_grey)
    };

    let mid_axis_info = { // Text info
        axis_text::recalculate_mid_axis_info(&graphing_bounds, &view)
    };

    // spawn meshes
    {
        let xbounds = graphing_bounds.xbounds;

        commands.spawn_bundle(MaterialMeshBundle {
            mesh: meshes.add(gen_mesh::gen_expr_mesh(&expr, xbounds)),
            material: const_width_curve,
            transform: Transform::from_xyz(0.0, 0.0, 0.1),
            visibility: Visibility { is_visible: true },
            ..Default::default()
        }).insert(expr);

        let main_axis = meshes.add(gen_mesh::gen_main_axis(xbounds, xbounds));
        let mid_axis = meshes.add(gen_mesh::gen_mid_axis(mid_axis_info, &graphing_bounds));
        let min_axis = meshes.add(gen_mesh::gen_min_axis(mid_axis_info.calculate_min_axis_info(), &graphing_bounds));

        let grid_mesh_handles = gen_mesh::GridMeshHandles {
            main_axis,
            mid_axis,
            min_axis,
        };

        commands.spawn_bundle(PbrBundle {
            mesh: grid_mesh_handles.main_axis.clone(),
            material: white,
            visibility: Visibility { is_visible: true },
            ..Default::default()
        });

        commands.spawn_bundle(PbrBundle {
            mesh: grid_mesh_handles.mid_axis.clone(),
            material: light_grey,
            visibility: Visibility { is_visible: true },
            ..Default::default()
        });

        commands.spawn_bundle(PbrBundle {
            mesh: grid_mesh_handles.min_axis.clone(),
            material: dark_grey,
            visibility: Visibility { is_visible: false },
            ..Default::default()
        });

        commands.insert_resource(grid_mesh_handles);
    }; 

    { // axis info
        let font = asset_server.load("fonts/Lato-Light.ttf");
        let text_style = TextStyle {
            font,
            font_size: 60.0,
            color: Color::WHITE,
        };

        let axis_text_info = axis_text::AxisTextInfo {
            text_style,
        };

        commands.insert_resource(axis_text_info);

    }

    commands.insert_resource(graphing_bounds);
    commands.insert_resource(view);
    commands.insert_resource(mid_axis_info);
}

