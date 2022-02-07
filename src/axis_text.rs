use bevy::prelude::*;
use crate::scaling::{GraphingView, GraphingBounds};
use crate::gen_mesh::{mid_axis_count, mid_axis_diff};

#[derive(Copy, Clone, Debug)]
pub struct MidAxisInfo {
    pub separation: f32,
    pub xline_count: usize,
    pub yline_count: usize,
    pub rounded_xcentre: f32,
    pub rounded_ycentre: f32,
}

#[derive(Clone, Debug)]
pub struct AxisTextInfo {
    pub text_style: TextStyle,
}

#[derive(Bundle, Clone, Debug)]
pub struct AxisTextBundle {
    #[bundle]
    pub text_bundle: Text2dBundle,
    pub axis_text: AxisText,
}

#[derive(Component, Copy, Clone, Debug)]
pub enum AxisText {
    X(f32),
    Y(f32),
    Origin,
}

pub fn generate_text_bundle(
    axis_text: AxisText,
    text_style: TextStyle,
) -> AxisTextBundle {
    //let position = match axis_text {
    //    AxisText::X(n) => Rect { bottom: Val::Px(0.0), left: Val::Px(n), ..Default::default()},
    //    AxisText::Y(n) => Rect { bottom: Val::Px(n), left: Val::Px(0.0), ..Default::default()},
    //    AxisText::Origin => Rect::default(),
    //};

    let n = match axis_text {
        AxisText::X(n) => n,
        AxisText::Y(n) => n,
        AxisText::Origin => 0.0,
    };
    
    AxisTextBundle {
        axis_text,
        text_bundle: Text2dBundle {
            text: Text::with_section(
                n.to_string(),
                text_style,
                TextAlignment::default(),
            ),
            //style: Style {
            //    position_type: PositionType::Absolute,
            //    position,
            //    ..Default::default()
            //},
            //transform: Transform::from_scale(Vec3::new(0.1, 0.1, 1.0)),
            ..Default::default()
        }
    }
}

pub fn regenerate_axis_text_system(
    mut commands: Commands,
    graphing_bounds: Res<GraphingBounds>,
    view: Res<GraphingView>,
    axis_text_info: Res<AxisTextInfo>,
    prev_text: Query<(Entity, With<AxisText>)>,
) {
    if graphing_bounds.is_changed() {
        for (entity, _) in prev_text.iter() {
            commands.entity(entity).despawn();
        }

        let xbounds = graphing_bounds.xbounds;
        let ybounds = graphing_bounds.ybounds;

        let axis_separation = mid_axis_diff(view.scale);
        let (xline_count, yline_count) = mid_axis_count(xbounds, ybounds, axis_separation);

        let main_x_axis_rendered = ybounds.start < 0.0 && 0.0 < ybounds.end;
        let main_y_axis_rendered = xbounds.start < 0.0 && 0.0 < xbounds.end;
        
        if main_y_axis_rendered {
            let rounded_ycentre = (view.centre.y / axis_separation).round() * axis_separation;
            let text_style = axis_text_info.text_style.clone();

            let bundles_iter = (1..yline_count).map(move |i| {
                let n1 = rounded_ycentre + axis_separation * i as f32;
                let n2 = rounded_ycentre - axis_separation * i as f32;
                [
                    generate_text_bundle(AxisText::Y(n1), text_style.clone()),
                    generate_text_bundle(AxisText::Y(n2), text_style.clone())
                ]
            }).flatten();
            commands.spawn_batch(bundles_iter);
        }

        if main_x_axis_rendered {
            let rounded_xcentre = (view.centre.x / axis_separation).round() * axis_separation;
            let text_style = axis_text_info.text_style.clone();

            let bundles_iter = (1..xline_count).map(move |i| {
                let n1 = rounded_xcentre + axis_separation * i as f32;
                let n2 = rounded_xcentre - axis_separation * i as f32;
                [
                    generate_text_bundle(AxisText::X(n1), text_style.clone()),
                    generate_text_bundle(AxisText::X(n2), text_style.clone())
                ]
            }).flatten();
            commands.spawn_batch(bundles_iter);
        }

        if main_x_axis_rendered && main_y_axis_rendered {
            commands.spawn_bundle(generate_text_bundle(
                AxisText::Origin, 
                axis_text_info.text_style.clone()
            ));
        }
    }
}

pub fn recalculate_mid_axis_info(bounds: &GraphingBounds, view: &GraphingView) -> MidAxisInfo {
    use crate::gen_mesh;
    let xbounds = bounds.xbounds;
    let separation = gen_mesh::mid_axis_diff(view.scale);
    let (xline_count, yline_count) = gen_mesh::mid_axis_count(xbounds, xbounds, separation);
    let rounded_ycentre = (view.centre.y / separation).round() * separation;
    let rounded_xcentre = (view.centre.x / separation).round() * separation;

    MidAxisInfo {
        separation,
        xline_count,
        yline_count,
        rounded_xcentre,
        rounded_ycentre,
    }
}

//pub fn keep_axis_text_on_screen_system(
//    view: Res<GraphingView>,
//    axis_text_x: Query<mut Transform, AxisTextX>,
//    axis_text_y: Query<mut Transform, AxisTextY>,
//) {
//    if view.is_changed() {
//        let xbounds = view.visible_xbounds();
//
//        if 0.0 > xbounds.start
//
//        let ybounds = view.visible_ybounds();
//    }
//}

pub type MinAxisInfo = MidAxisInfo;
impl MidAxisInfo {
    pub fn calculate_min_axis_info(&self) -> MinAxisInfo {
        MinAxisInfo {
            separation: self.separation / 5.0,
            xline_count: self.xline_count * 5,
            yline_count: self.yline_count * 5,
            ..*self
        }
    }
}
