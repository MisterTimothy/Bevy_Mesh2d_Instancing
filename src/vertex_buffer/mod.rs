mod instance_material;

use bevy::{
    asset::RenderAssetUsages,
    camera::visibility::NoFrustumCulling,
    mesh::{MeshVertexAttribute, VertexFormat},
    prelude::*,
    render::{
        extract_component::ExtractComponent, extract_resource::ExtractResource,
        render_resource::ShaderType,
    },
    window::PrimaryWindow,
};
use rand::{Rng, SeedableRng};

const SHADER_ASSET_PATH: &str = "shaders/instancing.wgsl";

const ATTRIBUTE_CUSTOM_POSITION: MeshVertexAttribute =
    MeshVertexAttribute::new("Position", 988540917, VertexFormat::Float32x2);

pub struct InstancingPlugin;

impl Plugin for InstancingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(instance_material::CustomMaterialPlugin);
        app.add_systems(Startup, setup);
    }
}

fn setup(
    mut commands: Commands,
    window: Single<&Window, With<PrimaryWindow>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let aspect = window.width() / window.height();

    let mut mesh = Mesh::new(
        bevy::mesh::PrimitiveTopology::TriangleList,
        RenderAssetUsages::all(),
    );

    mesh.insert_attribute(
        ATTRIBUTE_CUSTOM_POSITION,
        create_circle_vertices(0.5, 24, 0.25, 0.0, std::f32::consts::PI * 2.0),
    );

    let mesh_handle = meshes.add(mesh);

    let mut rand = rand_chacha::ChaCha8Rng::from_os_rng();
    // Spawning 1 entity
    commands.spawn((
        Mesh2d(mesh_handle),
        Transform::default(),
        Visibility::default(),
        InstanceMaterialData {
            static_data: (1..=100)
                .flat_map(|x| (1..10).map(move |y| (x as f32 / 10.0, y as f32 / 10.)))
                .map(|(x, y)| StaticInstanceData {
                    offset: Vec2::new(
                        rand.random_range(-1_f32..1.0),
                        rand.random_range(-1_f32..1.0),
                    ),
                    color: LinearRgba::from(Color::hsla(x * 360., y, 0.5, 1.0)).to_f32_array(),
                })
                .collect(),
            changing_data: (1..=100)
                .flat_map(|x| (1..10).map(move |y| (x as f32 / 10.0, y as f32 / 10.)))
                .map(|(x, y)| ChangingInstanceData {
                    scale: Vec2::splat(rand.random_range(0.25..1_f32)) / aspect,
                })
                .collect(),
        },
        NoFrustumCulling,
    ));

    commands.insert_resource(InstanceUniformData { instance: 0 });

    commands.spawn((
        Camera2d,
        // Transform::from_xyz(0.0, 0.0, 15.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

// TO-DO:
// - Split into two structs: StaticData vs ChangingData
#[derive(Component)]
struct InstanceMaterialData {
    static_data: Vec<StaticInstanceData>,
    changing_data: Vec<ChangingInstanceData>,
}

impl ExtractComponent for InstanceMaterialData {
    type QueryData = &'static InstanceMaterialData;
    type QueryFilter = ();
    type Out = Self;

    fn extract_component(
        item: bevy::ecs::query::QueryItem<'_, '_, Self::QueryData>,
    ) -> Option<Self::Out> {
        Some(InstanceMaterialData {
            changing_data: item.changing_data.clone(),
            static_data: item.static_data.clone(),
        })
    }
}

#[derive(Debug, Clone, Resource, Reflect, ExtractResource, ShaderType)]
struct InstanceUniformData {
    instance: u32,
}

#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
struct StaticInstanceData {
    color: [f32; 4],
    offset: Vec2,
}

#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
struct ChangingInstanceData {
    scale: Vec2,
}

/// Default values:
///
/// radius: 1.0,
///
/// num_subdivisions: 24,
///
/// inner_radius: 0.0,
///
/// start_angle: 0.0,
///   
/// end_angle: std::f32::consts::PI * 2,
fn create_circle_vertices(
    radius: f32,
    num_subdivisions: usize,
    inner_radius: f32,
    start_angle: f32,
    end_angle: f32,
) -> Vec<[f32; 2]> {
    let num_vertices = num_subdivisions * 3 * 2;
    let mut vertex_data = Vec::with_capacity(num_vertices);

    let mut add_vertex = |x, y| {
        // Bevy requires data to be given as array
        vertex_data.push([x, y]);
    };

    for i in 0..num_subdivisions {
        let angle_1 =
            start_angle + (i as f32 + 0.0) * (end_angle - start_angle) / num_subdivisions as f32;
        let angle_2 =
            start_angle + (i as f32 + 1.0) * (end_angle - start_angle) / num_subdivisions as f32;

        let c1 = angle_1.cos();
        let s1 = angle_1.sin();
        let c2 = angle_2.cos();
        let s2 = angle_2.sin();

        // First triangle
        add_vertex(c1 * radius, s1 * radius);
        add_vertex(c2 * radius, s2 * radius);
        add_vertex(c1 * inner_radius, s1 * inner_radius);

        // Second triangle
        add_vertex(c1 * inner_radius, s1 * inner_radius);
        add_vertex(c2 * radius, s2 * radius);
        add_vertex(c2 * inner_radius, s2 * inner_radius);
    }

    vertex_data
}
