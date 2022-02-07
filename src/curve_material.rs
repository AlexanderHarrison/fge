use bevy::prelude::*;
use bevy::render::render_resource::*;
use bevy::render::renderer::RenderDevice;
use bevy::render::render_asset::{PrepareAssetError, RenderAsset};
use bevy::pbr::MaterialPipeline;
use bevy::reflect::TypeUuid;
use bevy::ecs::system::lifetimeless::SRes;
use bevy::ecs::system::SystemParamItem;

const CURVE_WIDTH: f32 = 0.03;

#[derive(Copy, Clone, Debug, TypeUuid)]
#[uuid = "0000002a-000c-0005-0c03-0938362b0809"]
pub struct CurveMaterial {
    //pub color: Color,
}

#[derive(Clone)]
pub struct GpuCurveMaterial {
    _buffer: Buffer,
    bind_group: BindGroup,
}

impl RenderAsset for CurveMaterial {
    type ExtractedAsset = Self;
    type PreparedAsset = GpuCurveMaterial;
    type Param = (SRes<RenderDevice>, SRes<MaterialPipeline<Self>>);
    
    fn extract_asset(&self) -> Self::ExtractedAsset {
        *self
    }

    fn prepare_asset(
        _extracted_asset: Self::ExtractedAsset,
        (render_device, material_pipeline): &mut SystemParamItem<Self::Param>,
    ) -> Result<Self::PreparedAsset, PrepareAssetError<Self::ExtractedAsset>> {
        let buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            contents: &CURVE_WIDTH.to_ne_bytes(),
            label: None,
            usage: BufferUsages::UNIFORM,
        });
        let bind_group = render_device.create_bind_group(&BindGroupDescriptor {
            entries: &[BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
            label: None,
            layout: &material_pipeline.material_layout,
        });

        Ok(GpuCurveMaterial {
            _buffer: buffer,
            bind_group,
        })
    }
}

impl Material for CurveMaterial {
    fn vertex_shader(asset_server: &AssetServer) -> Option<Handle<Shader>> {
        Some(asset_server.load("curve_shader.wgsl"))
    }
    
    fn fragment_shader(asset_server: &AssetServer) -> Option<Handle<Shader>> {
        Some(asset_server.load("curve_shader.wgsl"))
    }

    fn bind_group(render_asset: &<Self as RenderAsset>::PreparedAsset) -> &BindGroup {
        &render_asset.bind_group
    }

    fn bind_group_layout(render_device: &RenderDevice) -> BindGroupLayout {
        render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: BufferSize::new(4),
                },
                count: None,
            }],
            label: None,
        })
    }
}
