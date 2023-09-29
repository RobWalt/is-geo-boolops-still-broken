use bevy::input::common_conditions::input_pressed;
use bevy::prelude::*;
use bevy_eventlistener::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_mod_picking::prelude::*;
use bevy_prototype_lyon::prelude::*;
use geo::{BooleanOps, CheckedBooleanOps};
use rand::seq::IteratorRandom;
use rand::thread_rng;

fn intersection(p1: &geo::Polygon<f32>, p2: &geo::Polygon<f32>) -> geo::MultiPolygon<f32> {
    p1.checked_intersection(p2).unwrap()
}

fn difference(p1: &geo::Polygon<f32>, p2: &geo::Polygon<f32>) -> geo::MultiPolygon<f32> {
    p1.checked_difference(p2).unwrap()
}

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(low_latency_window_plugin()),
            DefaultPickingPlugins
                .build()
                .disable::<DebugPickingPlugin>(),
        ))
        .add_plugins(WorldInspectorPlugin::default())
        .add_plugins(ShapePlugin)
        .add_systems(Startup, (setup_camera, setup_shapes))
        .add_systems(Update, (update_shape, visualize_intersection))
        .add_systems(Update, moving.run_if(input_pressed(KeyCode::Space)))
        //.add_systems(Startup, setup_plugin)
        //.add_systems(Update, spin)
        .register_type::<ShapeVertexChildren>()
        .register_type::<VertexMarker>()
        .register_type::<ShapeMarker>()
        .run();
}

#[derive(Debug, Clone, Component, Deref, DerefMut, Default, Reflect)]
pub struct ShapeVertexChildren(pub Vec<Entity>);

#[derive(Debug, Clone, Component, Default, Reflect)]
pub struct VertexMarker;

#[derive(Debug, Clone, Component, Default, Reflect)]
pub struct ShapeMarker;

#[derive(Debug, Clone, Component, Default, Reflect)]
pub struct IntersectionMarker;

#[derive(Debug, Clone, Component, Default, Reflect)]
pub struct Moving;

fn visualize_intersection(
    mut commands: Commands,
    q_changed_shape: Query<(), (Changed<Path>, With<ShapeMarker>)>,
    q_shapes: Query<&Children, With<ShapeMarker>>,
    q_vertices: Query<&GlobalTransform, With<VertexMarker>>,
    q_previous_intersection: Query<Entity, With<IntersectionMarker>>,
) {
    if q_changed_shape.is_empty() {
        return;
    }

    for old in q_previous_intersection.iter() {
        commands.entity(old).despawn_recursive();
    }

    let mut shapes = q_shapes
        .iter()
        .map(|shape| {
            shape
                .iter()
                .filter_map(|&child| q_vertices.get(child).ok())
                .map(|transform| transform.translation().truncate())
                .collect::<Vec<_>>()
        })
        .map(|shape| {
            geo::Polygon::new(
                geo::LineString::new(
                    shape
                        .iter()
                        .map(|v| geo::Coord { x: v.x, y: v.y })
                        .collect::<Vec<_>>(),
                ),
                vec![],
            )
        })
        .collect::<Vec<_>>();

    let mut intersections = vec![];
    while let Some(([idx0, idx1], intersection)) =
        shapes.iter().enumerate().find_map(|(idx0, shape0)| {
            shapes
                .iter()
                .enumerate()
                .filter(|&(idx, _)| idx != idx0)
                .find_map(|(idx1, shape1)| {
                    let intersection = intersection(shape0, shape1);
                    (!intersection.0.is_empty()).then(|| {
                        let mut idxs = [idx0, idx1];
                        idxs.sort();
                        (idxs, intersection)
                    })
                })
        })
    {
        // order matters! removing the bigger one first to prevent index invalidation
        let shape1 = shapes.remove(idx1);
        let shape0 = shapes.remove(idx0);

        intersections.extend(intersection);
        shapes.extend(difference(&shape0, &shape1));
        shapes.extend(difference(&shape1, &shape0));
    }

    let differences = shapes;

    for (poly, color) in intersections
        .iter()
        .zip(std::iter::repeat(Color::BLACK.with_a(0.8)))
        .chain(
            differences
                .iter()
                .zip(std::iter::repeat(Color::WHITE.with_a(0.8))),
        )
    {
        let shape = poly
            .exterior()
            .0
            .iter()
            .map(|c| Vec2::from(c.x_y()))
            .collect::<Vec<_>>();
        commands.spawn((
            ShapeBundle {
                path: GeometryBuilder::build_as(&shapes::Polygon {
                    points: shape,
                    closed: true,
                }),
                transform: Transform::from_translation(Vec2::ZERO.extend(0.1)),
                ..default()
            },
            Fill::color(color),
            IntersectionMarker,
        ));
    }
}

fn update_shape(
    q_changed_shape: Query<&Parent, (Changed<Transform>, With<VertexMarker>)>,
    q_vertices: Query<&Transform, With<VertexMarker>>,
    q_shape_vertex_children: Query<&ShapeVertexChildren>,
    mut q_shapes_children: Query<(&mut Path, &Children), With<ShapeMarker>>,
) {
    for parent in q_changed_shape.iter() {
        let Ok((_, children)) = q_shapes_children.get(parent.get()) else {
            return;
        };

        let Some(vertex_children) = children
            .iter()
            .find_map(|&child| q_shape_vertex_children.get(child).ok())
        else {
            return;
        };

        let new_positions = vertex_children
            .iter()
            .filter_map(|&entity| q_vertices.get(entity).ok())
            .map(|transform| transform.translation.truncate())
            .collect::<Vec<_>>();

        let Ok((mut path, _)) = q_shapes_children.get_mut(parent.get()) else {
            return;
        };

        *path = GeometryBuilder::build_as(&shapes::Polygon {
            points: new_positions,
            closed: true,
        });
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn setup_shapes(mut commands: Commands) {
    let rectangle = [(0, 0), (0, 1), (1, 1), (1, 0)]
        .map(|(a, b)| (a as f32, b as f32))
        .map(Vec2::from)
        .map(|v| v * 100.0);

    [Vec2::X, -Vec2::X, -Vec2::Y, Vec2::Y]
        .map(|v| v * 100.0)
        .into_iter()
        .for_each(|offset| {
            let color = Color::rgb(offset.x * 0.005 + 0.5, offset.y * 0.005 + 0.5, 0.0).with_a(0.5);
            info!("{offset:?}->{color:?}");
            commands
                .spawn((
                    ShapeBundle {
                        path: GeometryBuilder::build_as(&shapes::Polygon {
                            points: rectangle.to_vec(),
                            closed: true,
                        }),
                        transform: Transform::from_translation(offset.extend(0.0)),
                        ..default()
                    },
                    Fill::color(color),
                    ShapeMarker,
                ))
                .with_children(|children| {
                    let mut shape = vec![];
                    rectangle.into_iter().for_each(|vertex| {
                        shape.push({
                            let child = children.spawn((
                                ShapeBundle {
                                    path: GeometryBuilder::build_as(&shapes::Circle {
                                        center: Vec2::ZERO,
                                        radius: 10.0,
                                    }),
                                    transform: Transform::from_translation(vertex.extend(1.0)),
                                    ..default()
                                },
                                Fill::color(Color::YELLOW),
                                VertexMarker,
                                PickableBundle::default(),
                                On::<Pointer<DragStart>>::target_insert(Pickable::IGNORE),
                                On::<Pointer<DragEnd>>::target_insert(Pickable::default()),
                                On::<Pointer<Drag>>::target_component_mut::<Transform>(
                                    |drag, transform| {
                                        let new_translation = transform.translation
                                            + (drag.delta * Vec2::new(1.0, -1.0)).extend(0.0);
                                        let new_translation = new_translation.trunc();
                                        transform.translation = new_translation;
                                    },
                                ),
                                On::<Pointer<Drop>>::commands_mut(|_, _| {}),
                                Moving,
                            ));
                            child.id()
                        });
                    });
                    children.spawn(ShapeVertexChildren(shape));
                });
        });
}

fn moving(mut q: Query<&mut Transform, With<Moving>>) {
    let mut rng = thread_rng();
    if let Some(mut t) = q.iter_mut().choose(&mut rng) {
        t.translation += ((rand::random::<Vec2>() - Vec2::ONE * 0.5) * 5.0).extend(0.0);
    }
}
