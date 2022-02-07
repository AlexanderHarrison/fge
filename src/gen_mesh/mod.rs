use super::scaling::{Bounds, GraphingBounds};
use bevy::render::mesh::{Indices, Mesh};
use bevy::render::render_resource::PrimitiveTopology;
use bevy::prelude::*;
use crate::axis_text::{MinAxisInfo, MidAxisInfo};

mod gen_expr_mesh;
pub use gen_expr_mesh::gen_expr_mesh;

#[derive(Clone, Debug)]
pub struct GridMeshHandles {
    pub main_axis: Handle<Mesh>,
    pub mid_axis: Handle<Mesh>,
    pub min_axis: Handle<Mesh>,
}

#[derive(Component, Clone, Debug)]
pub struct Expression(pub mathjit::expr_parse::Expression);

pub fn regenerate_meshes_system(
    graphing_bounds: Res<GraphingBounds>,
    mid_axis_info: Res<MidAxisInfo>,
    mut grid_mesh_handles: ResMut<GridMeshHandles>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut graphs: Query<(&Expression, &mut Handle<Mesh>)>
) {
    if graphing_bounds.is_changed() {
        let xbounds = graphing_bounds.xbounds;
        let ybounds = graphing_bounds.ybounds;
        grid_mesh_handles.main_axis = meshes.set(grid_mesh_handles.main_axis.clone(), gen_main_axis(xbounds, ybounds));
        grid_mesh_handles.mid_axis = meshes.set(
            grid_mesh_handles.mid_axis.clone(),
            gen_mid_axis(*mid_axis_info, &graphing_bounds)
        );
        grid_mesh_handles.min_axis = meshes.set(
            grid_mesh_handles.min_axis.clone(),
            gen_min_axis(mid_axis_info.calculate_min_axis_info(), &graphing_bounds)
        );

        for (expr, mut mesh_handle) in graphs.iter_mut() {
            *mesh_handle = meshes.set(mesh_handle.clone(), gen_expr_mesh(expr, xbounds));
        }
    }
}

pub fn gen_main_axis(xbounds: Bounds, ybounds: Bounds) -> Mesh {
    let Bounds { start: xstart, end: xend } = xbounds;
    let Bounds { start: ystart, end: yend } = ybounds;

    let vertices = vec![
        [xstart, 0.0, 0.0], [xend, 0.0, 0.0],
        [0.0, ystart, 0.0], [0.0, yend, 0.0]
    ];

    let len = vertices.len();
    let mut mesh = Mesh::new(PrimitiveTopology::LineList);
    mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    mesh.set_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0.0, 1.0, 0.0], [0.0, 1.0, 0.0], [1.0, 0.0, 0.0], [1.0, 0.0, 0.0]]);
    mesh.set_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0.0, 0.0]; len]);
    mesh.set_indices(Some(Indices::U16((0..(len as u16)).collect::<Vec<u16>>())));

    mesh
}

// Fix for large and small numbers.
// Good enough for now
pub fn mid_axis_diff(scale: f32) -> f32 {
    let l2 = (scale/10.0).log2() as i32 + 1;
    let l5 = (l2+1) / 3;
    2.0f32.powi(l2-2*l5)*5.0f32.powi(l5)
}

pub fn mid_axis_count(xbounds: Bounds, ybounds: Bounds, axis_separation: f32) -> (usize, usize) {
    let Bounds { start: xstart, end: xend } = xbounds;
    let Bounds { start: ystart, end: yend } = ybounds;

    // count on one side of centre of bounds
    // add one more line to fix occasional off by one
    let xline_count = ((xend - xstart) / (2.0 * axis_separation)) as usize + 1;
    let yline_count = ((yend - ystart) / (2.0 * axis_separation)) as usize + 1;

    (xline_count, yline_count)
}

pub fn gen_min_axis(info: MinAxisInfo, bounds: &GraphingBounds) -> Mesh {
    gen_mid_axis(info, bounds)
}

pub fn gen_mid_axis(info: MidAxisInfo, bounds: &GraphingBounds) -> Mesh {
    let Bounds { start: xstart, end: xend } = bounds.xbounds;
    let Bounds { start: ystart, end: yend } = bounds.ybounds;

    let MidAxisInfo {
        separation,
        xline_count,
        yline_count,
        rounded_xcentre,
        rounded_ycentre,
    } = info;

    let vertex_count = 2 + xline_count * 2 + yline_count * 2;
    let mut vertices = Vec::with_capacity(vertex_count);
    let mut normals = Vec::with_capacity(vertex_count);

    { 
        let mut add_horiz_line = |n| {
            vertices.push([xstart, n, 0.0]);
            vertices.push([xend, n, 0.0]);
        };

        add_horiz_line(rounded_ycentre);
        for i in 1..yline_count {
            let j = i as f32;
            add_horiz_line(rounded_ycentre + separation * j);
            add_horiz_line(rounded_ycentre - separation * j);
        }

        normals.extend_from_slice(&vec![[0.0, 1.0, 0.0]; yline_count * 4 - 2])
    }

    { 
        let mut add_vert_line = |n| {
            vertices.push([n, ystart, 0.0]);
            vertices.push([n, yend, 0.0]);
        };

        add_vert_line(rounded_xcentre);
        for i in 1..xline_count {
            let j = i as f32;
            add_vert_line(rounded_xcentre + separation * j);
            add_vert_line(rounded_xcentre - separation * j);
        }

        normals.extend_from_slice(&vec![[1.0, 0.0, 0.0]; xline_count * 4 - 2])
    }

    let mut mesh = Mesh::new(PrimitiveTopology::LineList);

    let len = vertices.len();
    mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    mesh.set_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.set_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0.0, 0.0]; len]);
    mesh.set_indices(Some(Indices::U16((0..(len as u16)).collect::<Vec<u16>>())));

    mesh
}
