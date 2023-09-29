use baby_shark::triangulation::constrained_delaunay::ConstrainedTriangulation2;
use geo::{Centroid, Contains};
use nalgebra::Point2;
use polygon_stitching::stitch_multipolygon_from_parts;

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
            (p1.contains(&p) && !p2.contains(&p)) || (!p1.contains(&p) && p2.contains(&p))
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
    let vertices = polys
        .iter()
        .flat_map(|poly| {
            poly.interiors()
                .iter()
                .cloned()
                .chain(std::iter::once(poly.exterior().clone()))
                .collect::<Vec<_>>()
        })
        .map(|mut ls| {
            ls.close();
            let points =
                ls.0.iter()
                    .take(ls.0.len().saturating_sub(1))
                    .map(|c| Point2::new(c.x, c.y))
                    .collect::<Vec<_>>();
            let edges = points
                .iter()
                .zip(points.iter().cycle().skip(1))
                .map(|(a, b)| (*a, *b))
                .collect::<Vec<_>>();
            (points, edges)
        })
        .collect::<Vec<_>>();

    let mut points = vec![];
    let mut edges = vec![];

    for (ps, es) in vertices {
        for p in ps {
            if !points.iter().any(|(o, _)| *o == p) {
                points.push((p, points.len()));
            }
        }
        let es = es
            .iter()
            .map(|(a, b)| {
                let idx_a = points
                    .iter()
                    .find(|(o, _)| o == a)
                    .map(|(_, idx)| idx)
                    .cloned()
                    .unwrap();
                let idx_b = points
                    .iter()
                    .find(|(o, _)| o == b)
                    .map(|(_, idx)| idx)
                    .cloned()
                    .unwrap();
                (idx_a, idx_b)
            })
            .collect::<Vec<_>>();
        edges.extend(es);
    }

    let points = points.into_iter().map(|(p, _)| p).collect::<Vec<_>>();

    let mut cdt = ConstrainedTriangulation2::from_points(&points);
    for (a, b) in edges {
        cdt.insert_constrained_edge(a, b);
    }

    let tri = cdt.triangles();
    tri.chunks(3)
        .map(|w| [w[0], w[1], w[2]])
        .map(|ps| {
            ps.map(|idx| {
                let p = cdt.points()[idx];
                geo::Coord { x: p.x, y: p.y }
            })
        })
        .map(geo::Triangle::from)
        .collect::<Vec<_>>()
}
