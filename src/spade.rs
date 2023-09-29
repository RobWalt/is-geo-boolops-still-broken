use bevy::prelude::*;
use geo::{Centroid, Contains};
use polygon_stitching::stitch_multipolygon_from_parts;
use spade::{ConstrainedDelaunayTriangulation, Point2, Triangulation};

pub fn difference(p1: &geo::Polygon<f32>, p2: &geo::Polygon<f32>) -> geo::MultiPolygon<f32> {
    let triangles = difference_triangulation(p1, p2);
    stitch_multipolygon_from_parts(
        &triangles
            .into_iter()
            .map(|tri| tri.to_polygon())
            .collect::<Vec<_>>(),
    )
}

pub fn difference_triangulation(
    p1: &geo::Polygon<f32>,
    p2: &geo::Polygon<f32>,
) -> Vec<geo::Triangle<f32>> {
    let triangles = triangulate(&[p1.clone(), p2.clone()]);
    triangles
        .into_iter()
        .filter(|tri| {
            let p = tri.centroid();
            p1.contains(&p) && !p2.contains(&p)
        })
        .collect::<Vec<_>>()
}

pub fn intersection(p1: &geo::Polygon<f32>, p2: &geo::Polygon<f32>) -> geo::MultiPolygon<f32> {
    let triangles = intersection_triangulation(p1, p2);
    stitch_multipolygon_from_parts(
        &triangles
            .into_iter()
            .map(|tri| tri.to_polygon())
            .collect::<Vec<_>>(),
    )
}

pub fn intersection_triangulation(
    p1: &geo::Polygon<f32>,
    p2: &geo::Polygon<f32>,
) -> Vec<geo::Triangle<f32>> {
    let triangles = triangulate(&[p1.clone(), p2.clone()]);
    triangles
        .into_iter()
        .filter(|tri| [p1, p2].iter().all(|p| p.contains(&tri.centroid())))
        .collect::<Vec<_>>()
}

pub fn general_intersection_triangulation(ps: &[geo::Polygon<f32>]) -> Vec<geo::Triangle<f32>> {
    let triangles = triangulate(ps);
    triangles
        .into_iter()
        .filter(|tri| ps.iter().filter(|p| p.contains(&tri.centroid())).count() >= 2)
        .collect::<Vec<_>>()
}

pub fn triangulate(polys: &[geo::Polygon<f32>]) -> Vec<geo::Triangle<f32>> {
    let mut lines = polys
        .iter()
        .flat_map(|poly| {
            poly.interiors()
                .iter()
                .cloned()
                .chain(std::iter::once(poly.exterior().clone()))
                .collect::<Vec<_>>()
        })
        .flat_map(|mut ls| {
            ls.close();
            ls.lines().collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    let mut loop_count = 10000;

    while let Some(([idx0, idx1], intersection)) =
        lines.iter().enumerate().find_map(|(idx0, line0)| {
            lines
                .iter()
                .enumerate()
                .filter(|(idx1, line1)| *idx1 != idx0 && line0 != *line1)
                .find_map(|(idx1, line1)| {
                    let intersection = geo::line_intersection::line_intersection(*line0, *line1)?;
                    match intersection {
                        geo::LineIntersection::SinglePoint { is_proper, .. } if !is_proper => {
                            return None;
                        }
                        geo::LineIntersection::Collinear { intersection }
                            if intersection.start == intersection.end =>
                        {
                            return None;
                        }
                        _ => {}
                    }
                    let mut idxs = [idx0, idx1];
                    idxs.sort();
                    Some((idxs, intersection))
                })
        })
    {
        loop_count -= 1;
        if loop_count == 0 {
            info!("loop trap lines");
            return vec![];
        }
        let l1 = lines.remove(idx1);
        let l0 = lines.remove(idx0);
        let new_lines = match intersection {
            geo::LineIntersection::SinglePoint { intersection, .. } => [
                (l0.start, intersection),
                (l0.end, intersection),
                (l1.start, intersection),
                (l1.end, intersection),
            ]
            .map(|(a, b)| geo::Line::new(a, b))
            .to_vec(),
            geo::LineIntersection::Collinear { .. } => {
                let mut points = [l0.start, l0.end, l1.start, l1.end];
                points.sort_by(|a, b| {
                    a.x.partial_cmp(&b.x)
                        .unwrap()
                        .then_with(|| a.y.partial_cmp(&b.y).unwrap())
                });
                points
                    .windows(2)
                    .map(|w| geo::Line::new(w[0], w[1]))
                    .collect::<Vec<_>>()
            }
        };
        let mut new_lines = new_lines
            .into_iter()
            .filter(|l| l.start != l.end)
            .filter(|l| !lines.contains(l))
            .filter(|l| !lines.contains(&geo::Line::new(l.end, l.start)))
            .collect::<Vec<_>>();
        new_lines.dedup();

        lines.extend(new_lines);
    }

    let lines = lines
        .into_iter()
        .map(|line| {
            [
                Point2::new(line.start.x, line.start.y),
                Point2::new(line.end.x, line.end.y),
            ]
        })
        .collect::<Vec<_>>();

    let mut cdt = ConstrainedDelaunayTriangulation::<Point2<f32>>::new();

    for [a, b] in lines {
        let a = cdt.insert(a).unwrap();
        let b = cdt.insert(b).unwrap();
        if cdt.can_add_constraint(a, b) {
            cdt.add_constraint(a, b);
        } else {
            error!("yikes!");
        }
    }

    let triangles = cdt
        .inner_faces()
        .map(|a| a.positions().map(|p| geo::Coord { x: p.x, y: p.y }))
        .map(geo::Triangle::from)
        .collect::<Vec<_>>();

    triangles
}
