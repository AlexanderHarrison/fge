use crate::scaling::Bounds;
use super::Expression;
use bevy::render::mesh::{Indices, Mesh};
use bevy::render::render_resource::PrimitiveTopology;

const RESOLUTION: usize = 256;

pub fn gen_expr_mesh(expression: &Expression, bounds: Bounds) -> Mesh {
    gen_2d_tri_strip_mesh(expression, bounds)
}

fn gen_2d_tri_strip_mesh(expression: &Expression, bounds: Bounds) -> Mesh {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleStrip);
    //let values = blanket_eval(&expression.0, &[bounds.into()], RESOLUTION);
    let compiled_expr = mathjit::CompiledExpression::new(&expression.0).expect("error compiling equation.");
    let dx = (bounds.end - bounds.start) / RESOLUTION as f32;
    let input_vals = (0..RESOLUTION).map(|n| bounds.start + n as f32 * dx).collect::<Vec<f32>>();
    let values = compiled_expr.eval(&input_vals);

    let len = values.len() as u16;

    let normals = normals(&values, dx);

    // Two vertices for each value - shifted in positive and negative normal direction
    let vertices = values.into_iter().enumerate()
        .map(|(i, v)| (bounds.start + dx * i as f32, v))
        .map(|(x, &y)| [
            [x, y, 0.0],
            [x, y, 0.0]
        ])
        .flatten()
        .collect::<Vec<[f32; 3]>>();

    let vertex_normals = normals.into_iter()
        .map(|[nx, ny]| [[*nx, *ny, 0.0], [-*nx, -*ny, 0.0]]).flatten().collect::<Vec<[f32; 3]>>();

    mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    mesh.set_attribute(Mesh::ATTRIBUTE_NORMAL, vertex_normals);
    mesh.set_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0.0, 0.0]; 2 * len as usize]);
    mesh.set_indices(Some(Indices::U16((0..(2*len)).collect::<Vec<u16>>())));

    mesh
}

// Return (dx, dy) normalized normal for each value
fn normals(values: &[f32], dx: f32) -> Box<[[f32; 2]]> {
    assert!(values.len() > 1);
    let mut differences = vec![0.0; values.len()];
    difference(&values, &mut differences);

    let slopes = (0..(values.len()-1)).map(|i| (values[i+1]-values[i]) / dx);
    let mut normals = slopes.map(|s| {
        let n = (s.powi(2) + 1.0).sqrt().recip();
        [-s*n, n]
    }).collect::<Vec<[f32; 2]>>();

    // Duplicate the last normal so the slice lengths are equal
    normals.push(normals[normals.len() - 1]);
    
    normals.into_boxed_slice()
}

// not quite the derivative.
// divide by the x diff for each val if you want derivative.
fn difference(values: &[f32], out: &mut [f32]) {
    assert!(values.len() > 1);
    assert!(values.len() == out.len());

    for i in 1..(values.len() - 1) {
        out[i] = values[i+1] - values[i];
    }

    out[values.len()-1] = values[values.len()-2];
}
