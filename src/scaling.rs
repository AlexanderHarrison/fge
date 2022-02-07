use bevy::prelude::*;
use bevy::input::mouse::{MouseWheel, MouseMotion};
use bevy::render::camera::{Camera, OrthographicProjection};
use bevy::window::WindowResized;

use crate::axis_text::{recalculate_mid_axis_info, MidAxisInfo};

/// Only renders functions within these xbounds.
/// May have y bounds in the future.
#[derive(Clone, Debug)]
pub struct GraphingBounds {
    pub xbounds: Bounds,
    pub ybounds: Bounds,
}

#[derive(Clone, Debug)]
pub struct GraphingView {
    pub centre: Vec2,
    pub scale: f32,
}

#[derive(Copy, Clone, Debug)]
pub struct Bounds {
    pub start: f32,
    pub end: f32,
}

pub const DEFAULT_WINDOW_WIDTH: f32 = 640.0;
pub const DEFAULT_WINDOW_HEIGHT: f32 = 640.0;

/// The scale is half the magnitude of the total x range visible.
/// E.g x is between -5 and 5 when starting if the default scale is 5
pub const DEFAULT_SCALE: f32 = 5.0;

// Pregenerate meshes this factor outside the current window view.
pub const PREGENERATE_DISTANCE_FACTOR: f32 = 2.0;

pub const ZOOM_FACTOR: f32 = 1.1;

pub fn recalculate_graphing_bounds_system(
    view: Res<GraphingView>,
    window_descriptor: Res<WindowDescriptor>,
    mut graphing_bounds: ResMut<GraphingBounds>,
    mut mid_axis_info: ResMut<MidAxisInfo>,
) {
    if view.is_changed() || window_descriptor.is_changed() {
        let visible_xbounds = view.visible_xbounds(&window_descriptor);
        let visible_ybounds = view.visible_ybounds(&window_descriptor);

        let should_recalculate = 
            visible_xbounds.start < graphing_bounds.xbounds.start
            || visible_xbounds.end > graphing_bounds.xbounds.end
            || (visible_xbounds.end - visible_xbounds.start) * PREGENERATE_DISTANCE_FACTOR 
                < graphing_bounds.xbounds.end - graphing_bounds.xbounds.start
            || visible_ybounds.start < graphing_bounds.ybounds.start
            || visible_ybounds.end > graphing_bounds.ybounds.end
            || (visible_ybounds.end - visible_ybounds.start) * PREGENERATE_DISTANCE_FACTOR 
                < graphing_bounds.ybounds.end - graphing_bounds.ybounds.start;

        if should_recalculate {
            *graphing_bounds = recalculate_graphing_bounds(&view, &window_descriptor);
            *mid_axis_info = recalculate_mid_axis_info(&graphing_bounds, &view);
        }
    }
}

pub fn zoom_system(
    mut scroll: EventReader<MouseWheel>,
    mut view: ResMut<GraphingView>,
) {
    let total_y_scroll = scroll.iter().map(|s| s.y).sum::<f32>();

    match total_y_scroll {
        n if n > 0.0 => {
            view.scale *= ZOOM_FACTOR.powi(-n as i32);
        }
        n if n < 0.0 => {
            view.scale /= ZOOM_FACTOR.powi(n as i32);
        }
        _ => (),
    }
}

pub fn pan_system(
    mouse_click: Res<Input<MouseButton>>,
    mut mouse_motion: EventReader<MouseMotion>,
    mut view: ResMut<GraphingView>,
    window_descriptor: Res<WindowDescriptor>,
) {
    let mut delta = mouse_motion.iter().map(|motion| &motion.delta).sum::<Vec2>();

    // Mouse y coordinate is positive downwards - opposite of world space.
    delta.y = -delta.y;

    if mouse_click.pressed(MouseButton::Left) && delta.length_squared() > 0.01 {
        let scale = view.scale;
        view.centre -= delta * 2.0 * scale / window_descriptor.width;
    }
}

/// B: rouevy doesn't update WindowDescriptor on window resize for some reason
pub fn window_resize(
    mut resize_event: EventReader<WindowResized>,
    mut window: ResMut<WindowDescriptor>
) {
     if let Some(resize) = resize_event.iter().last() {
        window.width = resize.width.try_into().unwrap();
        window.height = resize.height.try_into().unwrap();
    }
}

pub fn update_projection_system(
    mut projection: Query<(&mut Transform, &mut Camera, &mut OrthographicProjection)>,
    view: Res<GraphingView>,
    window_descriptor: Res<WindowDescriptor>,
) {
    if view.is_changed() || window_descriptor.is_changed() {
        let proj_x = view.scale;
        let proj_y = view.scale * window_descriptor.height / window_descriptor.width;

        for (mut transform, mut camera, mut proj) in projection.iter_mut() {
            use bevy::render::camera::CameraProjection;

            proj.left = -proj_x;
            proj.right = proj_x;
            proj.top = proj_y;
            proj.bottom = -proj_y;
            camera.projection_matrix = proj.get_projection_matrix();
            camera.depth_calculation = proj.depth_calculation();
            transform.translation = Vec3::new(
                view.centre.x,
                view.centre.y, 
                1.0
            );
        }

    }
}

pub fn recalculate_graphing_bounds(view: &GraphingView, window: &WindowDescriptor) -> GraphingBounds {
    let vis_x = view.visible_xbounds(window);
    let centre_x = vis_x.centre();
    let dx = (vis_x.end - vis_x.start) * PREGENERATE_DISTANCE_FACTOR / 2.0;

    let vis_y = view.visible_ybounds(window);
    let centre_y = vis_y.centre();
    let dy = (vis_y.end - vis_y.start) * PREGENERATE_DISTANCE_FACTOR / 2.0;
    GraphingBounds {
        xbounds: Bounds {
            start: centre_x - dx,
            end: centre_x + dx,
        },
        ybounds: Bounds {
            start: centre_y - dy,
            end: centre_y + dy,
        }
    }
}

impl Bounds {
    pub fn centre(&self) -> f32 {
        (self.start + self.end) / 2.0
    }
}

impl From<std::ops::Range<f32>> for Bounds {
    fn from(range: std::ops::Range<f32>) -> Self {
        Self {
            start: range.start,
            end: range.end,
        }
    }
}

impl Into<std::ops::Range<f32>> for Bounds {
    fn into(self) -> std::ops::Range<f32> {
        self.start..self.end
    }
}

impl GraphingView {
    pub fn visible_xbounds(&self, _window: &WindowDescriptor) -> Bounds {
        Bounds {
            start: self.centre.x - self.scale,
            end: self.centre.x + self.scale,
        }
    }

    pub fn visible_ybounds(&self, window: &WindowDescriptor) -> Bounds {
        let dy = self.scale * window.height / window.width;
        Bounds {
            start: self.centre.y - dy,
            end: self.centre.y + dy,
        }
    }
}
