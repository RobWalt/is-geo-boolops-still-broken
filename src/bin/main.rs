use bevy::input::common_conditions::input_toggle_active;
use bevy::prelude::*;
use bevy_mod_picking::prelude::*;
use bevy_prototype_lyon::prelude::*;
use geo::*;
use rand::seq::IteratorRandom;
use rand::thread_rng;

use geo::SpadeBoolops as MySpade;

// put your own implementation of a safe intersection algorithm here
fn intersection(
    p1: &geo::Polygon<f32>,
    p2: &geo::Polygon<f32>,
) -> anyhow::Result<geo::MultiPolygon<f32>> {
    MySpade::intersection(p1, p2).map_err(anyhow::Error::from)
}

// put your own implementation of a safe difference algorithm here
fn difference(
    p1: &geo::Polygon<f32>,
    p2: &geo::Polygon<f32>,
) -> anyhow::Result<geo::MultiPolygon<f32>> {
    MySpade::difference(p1, p2).map_err(anyhow::Error::from)
}

fn main() {
    let mut app = App::new();
    app.add_plugins((
        DefaultPlugins.set(low_latency_window_plugin()),
        DefaultPickingPlugins
            .build()
            .disable::<DebugPickingPlugin>(),
    ));
    #[cfg(debug)]
    {
        use bevy_inspector_egui::quick::WorldInspectorPlugin;
        app.add_plugins(WorldInspectorPlugin::default());
    }
    app.add_plugins(ShapePlugin)
        .add_systems(Startup, (setup_camera, setup_shapes))
        .add_systems(
            Update,
            (
                update_shape,
                visualize_intersection,
                visualize_triangulation.run_if(input_toggle_active(false, KeyCode::Return)),
                delete_triangulation.run_if(input_toggle_active(true, KeyCode::Return)),
            ),
        )
        .add_systems(
            Update,
            moving.run_if(input_toggle_active(false, KeyCode::Space)),
        )
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

#[derive(Debug, Clone, Component, Default, Reflect)]
pub struct Triangle;

fn delete_triangulation(
    mut commands: Commands,

    q_previous_triangulation: Query<Entity, With<Triangle>>,
) {
    q_previous_triangulation.iter().for_each(|tri| {
        commands.entity(tri).despawn_recursive();
    });
}

fn visualize_triangulation(
    mut commands: Commands,
    q_changed_shape: Query<(), (Changed<Path>, With<ShapeMarker>)>,
    q_shapes: Query<&Children, With<ShapeMarker>>,
    q_vertices: Query<&GlobalTransform, With<VertexMarker>>,
    q_previous_triangulation: Query<Entity, With<Triangle>>,
) {
    if q_changed_shape.is_empty() {
        return;
    }

    for old in q_previous_triangulation.iter() {
        commands.entity(old).despawn_recursive();
    }

    let shapes = q_shapes
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

    fn general_intersection_triangulation(
        ps: &[geo::Polygon<f32>],
    ) -> Result<Vec<geo::Triangle<f32>>, ()> {
        let triangles = ps.constrained_triangulation().map_err(drop)?;
        Ok(triangles
            .into_iter()
            .filter(|tri| ps.iter().filter(|p| p.contains(&tri.centroid())).count() >= 2)
            .collect::<Vec<_>>())
    }

    let Ok(triangles) = general_intersection_triangulation(&shapes) else {
        return;
    };

    for poly in triangles.iter().map(|tri| tri.to_polygon()) {
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
                transform: Transform::from_translation(Vec2::ZERO.extend(0.5)),
                ..default()
            },
            Fill::color(Color::GREEN),
            Stroke::new(Color::BLACK, 1.0),
            Triangle,
        ));
    }
}

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

    let mut loop_count = 1000;
    let mut intersections = vec![];
    while let Some(res) = {
        let mut iter = shapes.iter().enumerate().flat_map(|(idx0, shape0)| {
            shapes
                .iter()
                .enumerate()
                .filter(move |&(idx, shape1)| idx != idx0 && shape0 != shape1)
                .map(move |(idx1, shape1)| ((idx0, shape0), (idx1, shape1)))
        });
        iter.find_map(|((idx0, shape0), (idx1, shape1))| {
            let intersection = intersection(shape0, shape1);
            match intersection {
                Ok(intersection) => (!intersection.0.is_empty()).then(|| {
                    let mut idxs = [idx0, idx1];
                    idxs.sort();
                    Ok((idxs, intersection))
                }),
                Err(e) => Some(Err(e)),
            }
        })
    } {
        let ([idx0, idx1], intersection) = match res {
            Ok(ok) => ok,
            Err(e) => panic!("{e:?}"),
        };
        loop_count -= 1;
        if loop_count == 0 {
            info!("loop trap shapes");
            return;
        }
        // order matters! removing the bigger one first to prevent index invalidation
        let shape1 = shapes.remove(idx1);
        let shape0 = shapes.remove(idx0);

        let d1 = match difference(&shape0, &shape1) {
            Ok(ok) => ok,
            Err(e) => panic!("{e:?}"),
        };
        let d2 = match difference(&shape1, &shape0) {
            Ok(ok) => ok,
            Err(e) => panic!("{e:?}"),
        };

        intersections.extend(intersection);
        shapes.extend(d1);
        shapes.extend(d2);
    }

    let differences = shapes;

    for (poly, (depth, color)) in intersections
        .iter()
        .zip(std::iter::repeat((0.4, Color::BLACK)))
        .chain(
            differences
                .iter()
                .zip(std::iter::repeat((0.3, Color::WHITE))),
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
                transform: Transform::from_translation(Vec2::ZERO.extend(depth)),
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
    commands.spawn((TextBundle::from_section(
        "
        Press SPACEBAR to toggle wiggle!\n
        Press ENTER to toggle triangulation vis!\n
        Drag and Drop points
        ",
        TextStyle::default(),
    )
    .with_style(Style {
        position_type: PositionType::Absolute,
        top: Val::Px(50.0),
        right: Val::Percent(40.0),
        ..default()
    }),));
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
                    Fill::color(Color::ORANGE),
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
                                        let new_translation = new_translation;
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
