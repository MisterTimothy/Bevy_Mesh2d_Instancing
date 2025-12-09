use bevy::{
    core_pipeline::{
        core_2d::{CORE_2D_DEPTH_FORMAT, Transparent2d},
        core_3d::Transparent3d,
    },
    ecs::system::lifetimeless::{Read, SRes},
    math::FloatOrd,
    mesh::{PrimitiveTopology, VertexBufferLayout, VertexFormat},
    pbr::{
        MeshPipeline, MeshPipelineKey, RenderMeshInstances, SetMeshBindGroup, SetMeshViewBindGroup,
        SetMeshViewBindingArrayBindGroup,
    },
    prelude::*,
    render::{
        Render, RenderApp, RenderStartup, RenderSystems,
        extract_component::ExtractComponentPlugin,
        extract_resource::ExtractResourcePlugin,
        mesh::{RenderMesh, RenderMeshBufferInfo, allocator::MeshAllocator},
        render_asset::RenderAssets,
        render_phase::{
            AddRenderCommand, DrawFunctions, PhaseItem, RenderCommand, SetItemPipeline,
            ViewSortedRenderPhases,
        },
        render_resource::{
            BindGroup, BindGroupEntries, BindGroupLayout, BindGroupLayoutEntries, BlendState,
            Buffer, BufferInitDescriptor, BufferUsages, ColorTargetState, ColorWrites,
            DepthBiasState, DepthStencilState, FragmentState, FrontFace, PipelineCache,
            PolygonMode, PrimitiveState, RenderPipelineDescriptor, ShaderStages,
            SpecializedMeshPipeline, SpecializedMeshPipelines, SpecializedRenderPipeline,
            SpecializedRenderPipelines, StencilFaceState, StencilState, TextureFormat,
            UniformBuffer, VertexAttribute, VertexState, VertexStepMode,
            binding_types::uniform_buffer,
        },
        renderer::{RenderDevice, RenderQueue},
        sync_world::MainEntity,
        view::{
            ExtractedView, RenderVisibleEntities, ViewTarget, ViewUniform, ViewUniformOffset,
            ViewUniforms,
        },
    },
    sprite_render::{Mesh2dPipeline, Mesh2dPipelineKey, RenderMesh2dInstances},
};

use super::{
    ChangingInstanceData, InstanceMaterialData, InstanceUniformData, SHADER_ASSET_PATH,
    StaticInstanceData,
};

pub(super) struct CustomMaterialPlugin;

impl Plugin for CustomMaterialPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ExtractComponentPlugin::<InstanceMaterialData>::default(),
            ExtractResourcePlugin::<InstanceUniformData>::default(),
        ));

        let render_app = app.sub_app_mut(RenderApp);
        render_app.add_render_command::<Transparent2d, DrawCustom>();
        render_app.init_resource::<SpecializedRenderPipelines<Custom2dPipeline>>();
        render_app.init_resource::<InstanceBuffer>();
        render_app
            // .add_systems(RenderStartup, init_custom_pipeline)
            .add_systems(
                Render,
                (
                    queue_custom.in_set(RenderSystems::QueueMeshes),
                    // prepare_bind_group.in_set(RenderSystems::PrepareBindGroups),
                    prepare_instance_buffers.in_set(RenderSystems::PrepareResources),
                ),
            );
    }

    fn finish(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);
        render_app.init_resource::<Custom2dPipeline>();
    }
}

type DrawCustom = (
    SetItemPipeline,
    SetCustomViewBindGroup<0>,
    DrawMeshInstanced,
);

#[derive(Default, Resource)]
pub struct InstanceBuffer {
    view_bind_group: Option<BindGroup>,
}

#[derive(Component)]
struct InstanceData {
    buffers: [Buffer; 2],
    length: usize,
}

#[derive(PartialEq, Eq, Hash, Clone)]
pub struct Custom2dPipelineKey {
    mesh_key: Mesh2dPipelineKey,
}

#[derive(Resource)]
struct Custom2dPipeline {
    shader: Handle<Shader>,
    view_layout: BindGroupLayout,
    // mesh2d_pipeline: Mesh2dPipeline,
}

impl FromWorld for Custom2dPipeline {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        let shader = asset_server.load(SHADER_ASSET_PATH);

        let render_device = world.resource::<RenderDevice>();
        let view_layout = render_device.create_bind_group_layout(
            "particle_view_layout",
            &BindGroupLayoutEntries::single(
                ShaderStages::VERTEX_FRAGMENT,
                uniform_buffer::<ViewUniform>(true),
            ),
        );

        Custom2dPipeline {
            shader,
            view_layout,
        }
    }
}

impl SpecializedRenderPipeline for Custom2dPipeline {
    type Key = Custom2dPipelineKey;
    fn specialize(&self, key: Self::Key) -> RenderPipelineDescriptor {
        let layout = vec![self.view_layout.clone()];

        let format = if key.mesh_key.contains(Mesh2dPipelineKey::HDR) {
            ViewTarget::TEXTURE_FORMAT_HDR
        } else {
            TextureFormat::bevy_default()
        };

        RenderPipelineDescriptor {
            label: Some("Custom2dRenderPipline".into()),
            layout,
            push_constant_ranges: vec![],
            vertex: VertexState {
                shader: self.shader.clone(),
                shader_defs: vec![],
                entry_point: Some("vs".into()),
                buffers: vec![
                    VertexBufferLayout {
                        step_mode: VertexStepMode::Vertex,
                        array_stride: 2 * 4,
                        attributes: vec![VertexAttribute {
                            shader_location: 0,
                            format: VertexFormat::Float32x2,
                            offset: 0,
                        }],
                    },
                    VertexBufferLayout {
                        step_mode: VertexStepMode::Instance,
                        array_stride: 6 * 4,
                        attributes: vec![
                            VertexAttribute {
                                shader_location: 1,
                                format: VertexFormat::Float32x4,
                                offset: 0,
                            },
                            VertexAttribute {
                                shader_location: 2,
                                format: VertexFormat::Float32x2,
                                offset: 4 * 4,
                            },
                        ],
                    },
                    VertexBufferLayout {
                        step_mode: VertexStepMode::Instance,
                        array_stride: 2 * 4,
                        attributes: vec![VertexAttribute {
                            shader_location: 3,
                            format: VertexFormat::Float32x2,
                            offset: 0,
                        }],
                    },
                ],
            },
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: Some(DepthStencilState {
                format: CORE_2D_DEPTH_FORMAT,
                depth_write_enabled: false,
                depth_compare: bevy::render::render_resource::CompareFunction::GreaterEqual,
                stencil: StencilState {
                    front: StencilFaceState::IGNORE,
                    back: StencilFaceState::IGNORE,
                    read_mask: 0,
                    write_mask: 0,
                },
                bias: DepthBiasState {
                    constant: 0,
                    slope_scale: 0.0,
                    clamp: 0.0,
                },
            }),
            multisample: bevy::render::render_resource::MultisampleState {
                count: key.mesh_key.msaa_samples(),
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(FragmentState {
                shader: self.shader.clone(),
                shader_defs: vec![],
                entry_point: Some("fs".into()),
                targets: vec![Some(ColorTargetState {
                    format,
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            zero_initialize_workgroup_memory: true,
        }
    }
}

pub struct SetCustomViewBindGroup<const I: usize>;
impl<P: PhaseItem, const I: usize> RenderCommand<P> for SetCustomViewBindGroup<I> {
    type Param = SRes<InstanceBuffer>;
    type ViewQuery = Read<ViewUniformOffset>;
    type ItemQuery = ();

    fn render<'w>(
        _item: &P,
        view: bevy::ecs::query::ROQueryItem<'w, '_, Self::ViewQuery>,
        _entity: Option<bevy::ecs::query::ROQueryItem<'w, '_, Self::ItemQuery>>,
        param: bevy::ecs::system::SystemParamItem<'w, '_, Self::Param>,
        pass: &mut bevy::render::render_phase::TrackedRenderPass<'w>,
    ) -> bevy::render::render_phase::RenderCommandResult {
        if let Some(bind_group) = &param.into_inner().view_bind_group {
            pass.set_bind_group(I, bind_group, &[view.offset]);
            bevy::render::render_phase::RenderCommandResult::Success
        } else {
            bevy::render::render_phase::RenderCommandResult::Failure(
                "Failed to prepare bind group!",
            )
        }
    }
}

struct DrawMeshInstanced;
impl<P: PhaseItem> RenderCommand<P> for DrawMeshInstanced {
    type Param = (
        SRes<RenderAssets<RenderMesh>>,
        SRes<RenderMesh2dInstances>,
        SRes<MeshAllocator>,
    );
    type ViewQuery = ();
    type ItemQuery = Read<InstanceData>;

    #[inline]
    fn render<'w>(
        item: &P,
        _view: bevy::ecs::query::ROQueryItem<'w, '_, Self::ViewQuery>,
        instance_data: Option<bevy::ecs::query::ROQueryItem<'w, '_, Self::ItemQuery>>,
        (render_meshes, render_mesh2d_instances, mesh_allocator): bevy::ecs::system::SystemParamItem<
            'w,
            '_,
            Self::Param,
        >,
        pass: &mut bevy::render::render_phase::TrackedRenderPass<'w>,
    ) -> bevy::render::render_phase::RenderCommandResult {
        let mesh_allocator = mesh_allocator.into_inner();

        let Some(mesh_instance) = render_mesh2d_instances.get(&item.main_entity()) else {
            return bevy::render::render_phase::RenderCommandResult::Skip;
        };
        let Some(gpu_mesh) = render_meshes.into_inner().get(mesh_instance.mesh_asset_id) else {
            return bevy::render::render_phase::RenderCommandResult::Skip;
        };
        let Some(vertex_buffer_slice) =
            mesh_allocator.mesh_index_slice(&mesh_instance.mesh_asset_id)
        else {
            return bevy::render::render_phase::RenderCommandResult::Skip;
        };
        let Some(instance_data) = instance_data else {
            return bevy::render::render_phase::RenderCommandResult::Skip;
        };

        pass.set_vertex_buffer(0, vertex_buffer_slice.buffer.slice(..));
        pass.set_vertex_buffer(1, instance_data.buffers[0].slice(..));
        pass.set_vertex_buffer(2, instance_data.buffers[1].slice(..));

        match &gpu_mesh.buffer_info {
            RenderMeshBufferInfo::Indexed {
                count,
                index_format,
            } => {
                let Some(index_buffer_slice) =
                    mesh_allocator.mesh_index_slice(&mesh_instance.mesh_asset_id)
                else {
                    return bevy::render::render_phase::RenderCommandResult::Skip;
                };
                pass.set_index_buffer(index_buffer_slice.buffer.slice(..), 0, *index_format);
                pass.draw_indexed(
                    index_buffer_slice.range.start..(index_buffer_slice.range.start + count),
                    vertex_buffer_slice.range.start as i32,
                    0..instance_data.length as u32,
                );
            }
            RenderMeshBufferInfo::NonIndexed => {
                pass.draw(vertex_buffer_slice.range, 0..instance_data.length as u32);
            }
        }

        bevy::render::render_phase::RenderCommandResult::Success
    }
}

fn queue_custom(
    transparent_2d_draw_functions: Res<DrawFunctions<Transparent2d>>,
    custom_pipline: Res<Custom2dPipeline>,
    mut pipelines: ResMut<SpecializedRenderPipelines<Custom2dPipeline>>,
    pipeline_cache: Res<PipelineCache>,
    material_meshes: Query<(Entity, &MainEntity), With<InstanceMaterialData>>,
    views: Query<(&ExtractedView, &Msaa)>,
    mut render_phases: ResMut<ViewSortedRenderPhases<Transparent2d>>,
) {
    let draw_custom = transparent_2d_draw_functions.read().id::<DrawCustom>();

    for (view, msaa) in &views {
        let Some(transparent_phase) = render_phases.get_mut(&view.retained_view_entity) else {
            continue;
        };

        let mesh_key = Mesh2dPipelineKey::from_msaa_samples(msaa.samples())
            | Mesh2dPipelineKey::from_hdr(view.hdr);

        let key = Custom2dPipelineKey { mesh_key };
        let pipeline = pipelines.specialize(&pipeline_cache, &custom_pipline, key);

        for (entity, main_entity) in &material_meshes {
            transparent_phase.add(Transparent2d {
                sort_key: FloatOrd(0.0),
                entity: (entity, *main_entity),
                pipeline,
                draw_function: draw_custom,
                batch_range: 0..1,
                extracted_index: 0,
                extra_index: bevy::render::render_phase::PhaseItemExtraIndex::None,
                indexed: false,
            });
        }
    }
}

fn prepare_instance_buffers(
    mut commands: Commands,
    query: Query<(Entity, &InstanceMaterialData)>,
    render_device: Res<RenderDevice>,
    view_uniforms: Res<ViewUniforms>,
    custom_pipeline: Res<Custom2dPipeline>,
    mut instance_buffer: ResMut<InstanceBuffer>,
) {
    if let Some(view_binding) = view_uniforms.uniforms.binding() {
        instance_buffer.view_bind_group = Some(render_device.create_bind_group(
            "View_bind_group",
            &custom_pipeline.view_layout,
            &BindGroupEntries::single(view_binding),
        ))
    }

    for (entity, instance_data) in &query {
        info_once!("There are entities i swear!");
        let static_buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("instance data buffer"),
            contents: bytemuck::cast_slice(instance_data.static_data.as_slice()),
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
        });

        let changing_buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("instance data buffer"),
            contents: bytemuck::cast_slice(instance_data.changing_data.as_slice()),
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
        });

        commands.entity(entity).insert(InstanceData {
            buffers: [static_buffer, changing_buffer],
            length: instance_data.static_data.len(),
        });
    }
}
