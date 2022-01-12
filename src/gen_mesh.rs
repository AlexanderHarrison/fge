use eq_parse::blanket_eval;
use super::scaling::{Bounds, GraphingBounds, GraphingView};
use bevy::render::mesh::{Indices, Mesh};
use bevy::render::pipeline::PrimitiveTopology;
use bevy::prelude::*;

const CURVE_WIDTH: f32 = 0.03;
const RESOLUTION: usize = 256;

#[derive(Clone, Debug)]
pub struct GridMeshHandles {
    pub main_axis: Handle<Mesh>,
    pub mid_axis: Handle<Mesh>,
    pub min_axis: Handle<Mesh>,
}

#[derive(Clone, Debug)]
pub struct Equation(pub eq_parse::Equation);

pub fn regenerate_meshes_system(
    graphing_bounds: Res<GraphingBounds>,
    view: Res<GraphingView>,
    mut grid_mesh_handles: ResMut<GridMeshHandles>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut graphs: Query<(&Equation, &mut Handle<Mesh>)>
) {
    if graphing_bounds.is_changed() {
        let xbounds = graphing_bounds.xbounds;
        let ybounds = graphing_bounds.ybounds;
        grid_mesh_handles.main_axis = meshes.set(grid_mesh_handles.main_axis.clone(), gen_main_axis(xbounds, ybounds));
        grid_mesh_handles.mid_axis = meshes.set(grid_mesh_handles.mid_axis.clone(), gen_mid_axis(xbounds, ybounds, view.scale));
        grid_mesh_handles.min_axis = meshes.set(grid_mesh_handles.min_axis.clone(), gen_min_axis(xbounds, ybounds, view.scale));

        for (eq, mut mesh_handle) in graphs.iter_mut() {
            *mesh_handle = meshes.set(mesh_handle.clone(), gen_mesh(eq, xbounds));
        }
    }
}

pub fn gen_mesh(equation: &Equation, bounds: Bounds) -> Mesh {
    gen_2d_tri_strip_mesh(equation, bounds)
}

fn gen_2d_tri_strip_mesh(equation: &Equation, bounds: Bounds) -> Mesh {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleStrip);
    let values = blanket_eval(&equation.0, &[bounds.into()], RESOLUTION);

    let len = values.len() as u16;
    let len_f32 = values.len() as f32;

    let dx = (bounds.end - bounds.start) / len_f32;
    let normals = normals(&values, dx);

    // Two vertices for each value - shifted in positive and negative normal direction
    let vertices = values.into_iter().enumerate()
        .map(|(i, v)| (bounds.start + dx * i as f32, v))
        .zip(normals.into_iter())
        .map(|((x, &y), (nx, ny))| [
            [x + nx * CURVE_WIDTH, y + ny * CURVE_WIDTH, 0.0],
            [x - nx * CURVE_WIDTH, y - ny * CURVE_WIDTH, 0.0]
        ])
        .flatten()
        .collect::<Vec<[f32; 3]>>();

    mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    mesh.set_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0.0, 0.0, 1.0]; 2 * len as usize]);
    mesh.set_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0.0, 0.0, 0.0]; 2 * len as usize]);
    mesh.set_indices(Some(Indices::U16((0..(2*len)).collect::<Vec<u16>>())));

    mesh
}

// Not an actual startup system, as it requires the bounds. Called by scaling::setup_bounds.
// For some reason you can't use added resources in startup systems, even if they were added before
// the startup system ran.
//fn setup_gen_mesh(mut commands: Commands, pregen_bounds: &GraphingBounds, window_bounds: &GraphingView) {
//}

#[allow(dead_code)]
fn gen_2d_line_strip_mesh(equation: &Equation, bounds: Bounds) -> Mesh {
    let mut mesh = Mesh::new(PrimitiveTopology::LineStrip);
    let values = blanket_eval(&equation.0, &[bounds.into()], RESOLUTION);

    let len = values.len() as u16;
    let len_f32 = values.len() as f32;

    let dx = (bounds.end - bounds.start) / len_f32;
    let vertices = values.into_iter().enumerate()
        .map(|(i, v)| (bounds.start + dx * i as f32, v))
        .map(|(i, &v)| [i, v, 0.0])
        .collect::<Vec<[f32; 3]>>();
    mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    mesh.set_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0.0, 0.0, 1.0]; len as usize]);
    mesh.set_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0.0, 0.0, 0.0]; len as usize]);
    mesh.set_indices(Some(Indices::U16((0..len).collect::<Vec<u16>>())));

    mesh
}

pub fn gen_main_axis(xbounds: Bounds, ybounds: Bounds) -> Mesh {
    let Bounds { start: xstart, end: xend } = xbounds;
    let Bounds { start: ystart, end: yend } = ybounds;

    let mut mesh = Mesh::new(PrimitiveTopology::LineList);
    let vertices = vec![
        [xstart, 0.0, 0.0], [xend, 0.0, 0.0],
        [0.0, ystart, 0.0], [0.0, yend, 0.0]
    ];

    let len = vertices.len();
    mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    mesh.set_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0.0, 0.0, 1.0]; len]);
    mesh.set_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0.0, 0.0, 0.0]; len]);
    mesh.set_indices(Some(Indices::U16((0..(len as u16)).collect::<Vec<u16>>())));

    mesh
}

pub fn gen_min_axis(xbounds: Bounds, ybounds: Bounds, scale: f32) -> Mesh {
    gen_mid_axis(xbounds, ybounds, scale / 5.0)
}

pub fn gen_mid_axis(xbounds: Bounds, ybounds: Bounds, scale: f32) -> Mesh {
    let Bounds { start: xstart, end: xend } = xbounds;
    let Bounds { start: ystart, end: yend } = ybounds;

    // Fix for large and small numbers.
    // Good enough for now
    fn mid_axis_diff(scale: f32) -> f32 {
        let l2 = (scale/10.0).log2() as i32 + 1;
        let l5 = (l2+1) / 3;
        2.0f32.powi(l2-2*l5)*5.0f32.powi(l5)
    }

    let diff_between_axis = mid_axis_diff(scale);

    // count on one side of centre of bounds
    // add one more line to fix occasional off by one
    let xline_count = ((xend - xstart) / 2.0 / diff_between_axis) as usize + 1;
    let yline_count = ((yend - ystart) / 2.0 / diff_between_axis) as usize + 1;

    let xcentre = (xstart + xend) / 2.0;
    let ycentre = (ystart + yend) / 2.0;

    let rounded_xcentre = (xcentre / diff_between_axis).round() * diff_between_axis;
    let rounded_ycentre = (ycentre / diff_between_axis).round() * diff_between_axis;

    let mut vertices = Vec::with_capacity(2 + xline_count * 2 + yline_count * 2);

    { 
        let mut add_horiz_line = |n| {
            vertices.push([xstart, n, 0.0]);
            vertices.push([xend, n, 0.0]);
        };

        add_horiz_line(rounded_ycentre);
        for i in 1..yline_count {
            let j = i as f32;
            add_horiz_line(rounded_ycentre + diff_between_axis * j);
            add_horiz_line(rounded_ycentre - diff_between_axis * j);
        }
    }

    { 
        let mut add_vert_line = |n| {
            vertices.push([n, ystart, 0.0]);
            vertices.push([n, yend, 0.0]);
        };

        add_vert_line(rounded_xcentre);
        for i in 1..xline_count {
            let j = i as f32;
            add_vert_line(rounded_xcentre + diff_between_axis * j);
            add_vert_line(rounded_xcentre - diff_between_axis * j);
        }
    }

    let mut mesh = Mesh::new(PrimitiveTopology::LineList);

    let len = vertices.len();
    mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    mesh.set_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0.0, 0.0, 1.0]; len]);
    mesh.set_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0.0, 0.0, 0.0]; len]);
    mesh.set_indices(Some(Indices::U16((0..(len as u16)).collect::<Vec<u16>>())));

    mesh
}

// Return (dx, dy) normalized normal for each value
fn normals(values: &[f32], dv: f32) -> Box<[(f32, f32)]> {
    assert!(values.len() > 1);
    let slopes = (0..(values.len()-1)).map(|i| (values[i+1]-values[i]) / dv);
    let mut normals = slopes.map(|s| {
        let n = (s.powi(2) + 1.0).sqrt().recip();
        (-s*n, n)
    }).collect::<Vec<(f32, f32)>>();

    // Duplicate the last normal so the slice lengths are equal
    normals.push(normals[normals.len() - 1]);
    
    normals.into_boxed_slice()
}
