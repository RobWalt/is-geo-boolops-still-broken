use bevy::prelude::*;
use geo::{Centroid, Contains, EuclideanDistance, LineIntersection};
use polygon_stitching::stitch_multipolygon_from_parts;
use spade::{ConstrainedDelaunayTriangulation, Point2, Triangulation};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SpadeBoolopsError {
    #[error("Hit an infinite loop and exited after a certain amount of iterations\n\n{0:?}")]
    LoopTrap(Vec<geo::Line<f32>>),
    #[error("Couldn't add a constraint even after preprocessing")]
    ConstraintFailure,
    #[error("Internal spade error: {0:?}")]
    SpadeError(#[from] spade::InsertionError),
}

pub fn difference(
    p1: &geo::Polygon<f32>,
    p2: &geo::Polygon<f32>,
) -> Result<geo::MultiPolygon<f32>, SpadeBoolopsError> {
    let triangles = difference_triangulation(p1, p2)?;
    Ok(stitch_multipolygon_from_parts(
        &triangles
            .into_iter()
            .map(|tri| tri.to_polygon())
            .collect::<Vec<_>>(),
    ))
}

pub fn difference_triangulation(
    p1: &geo::Polygon<f32>,
    p2: &geo::Polygon<f32>,
) -> Result<Vec<geo::Triangle<f32>>, SpadeBoolopsError> {
    let triangles = triangulate_polys(&[p1.clone(), p2.clone()])?;
    Ok(triangles
        .into_iter()
        .filter(|tri| {
            let p = tri.centroid();
            p1.contains(&p) && !p2.contains(&p)
        })
        .collect::<Vec<_>>())
}

pub fn intersection(
    p1: &geo::Polygon<f32>,
    p2: &geo::Polygon<f32>,
) -> Result<geo::MultiPolygon<f32>, SpadeBoolopsError> {
    let triangles = intersection_triangulation(p1, p2)?;
    Ok(stitch_multipolygon_from_parts(
        &triangles
            .into_iter()
            .map(|tri| tri.to_polygon())
            .collect::<Vec<_>>(),
    ))
}

pub fn intersection_triangulation(
    p1: &geo::Polygon<f32>,
    p2: &geo::Polygon<f32>,
) -> Result<Vec<geo::Triangle<f32>>, SpadeBoolopsError> {
    let triangles = triangulate_polys(&[p1.clone(), p2.clone()])?;
    Ok(triangles
        .into_iter()
        .filter(|tri| [p1, p2].iter().all(|p| p.contains(&tri.centroid())))
        .collect::<Vec<_>>())
}

pub fn general_intersection_triangulation(
    ps: &[geo::Polygon<f32>],
) -> Result<Vec<geo::Triangle<f32>>, SpadeBoolopsError> {
    let triangles = triangulate_polys(ps)?;
    Ok(triangles
        .into_iter()
        .filter(|tri| ps.iter().filter(|p| p.contains(&tri.centroid())).count() >= 2)
        .collect::<Vec<_>>())
}

pub fn triangulate_polys(
    polys: &[geo::Polygon<f32>],
) -> Result<Vec<geo::Triangle<f32>>, SpadeBoolopsError> {
    let (mut known_points, mut lines) = polys_to_lines(polys);

    // safety net. We can't prove that the `while let` loop isn't going to run infinitely, so
    // we abort after a fixed amount of iterations
    let mut loop_count = 1000;
    // in case of an error we have something to return (the scenario that triggered the infinite
    // loop)
    let original_lines = lines.clone();
    let mut loop_check = || {
        loop_count -= 1;
        (loop_count == 0)
            .then_some(())
            .ok_or_else(|| SpadeBoolopsError::LoopTrap(original_lines.clone()))
    };

    while let Some((indices, intersection)) = {
        let mut iter = iter_line_pairs(&lines);
        iter.find_map(find_intersecting_lines_fn)
    } {
        loop_check()?;
        let [l0, l1] = remove_lines_by_index(indices, &mut lines);
        let new_lines = split_lines([l0, l1], intersection);
        let new_lines = cleanup_filter_lines(new_lines, &lines, &mut known_points);

        lines.extend(new_lines);
    }

    triangulate_lines(lines)
}

/// snap point to the nearest existing point if it's close enough
fn snap_or_register_point(
    point: geo::Coord<f32>,
    known_points: &mut Vec<geo::Coord<f32>>,
) -> geo::Coord<f32> {
    const EPSILON_RANGE: f32 = 0.0001;
    known_points
        .iter()
        // find closest
        .min_by(|a, b| {
            a.euclidean_distance(&point)
                .partial_cmp(&b.euclidean_distance(&point))
                .expect("Couldn't compare coordinate distances")
        })
        // only snap if closest is within epsilone range
        .filter(|nearest_point| nearest_point.euclidean_distance(&point) < EPSILON_RANGE)
        .cloned()
        // otherwise register and use input point
        .unwrap_or_else(|| {
            known_points.push(point);
            point
        })
}

/// convert geo::Polygons to geo::Lines
///
/// the lines are somewhat snapped and duduplicated
///
/// the function also returns a vector including all the unique points of the collection of lines
fn polys_to_lines(polys: &[geo::Polygon<f32>]) -> (Vec<geo::Coord<f32>>, Vec<geo::Line<f32>>) {
    let mut known_points: Vec<geo::Coord<f32>> = vec![];

    let lines = polys
        .iter()
        .flat_map(|poly| {
            std::iter::once(poly.exterior().clone())
                .chain(poly.interiors().iter().cloned())
                .collect::<Vec<_>>()
        })
        .flat_map(|mut ls| {
            // make sure the linestring is closed
            ls.close();
            // get all lines of the linestring
            ls.lines().collect::<Vec<_>>()
        })
        .fold(vec![], |mut lines, mut line| {
            // deduplicating:

            // 1. snap points of lines to existing points
            line.start = snap_or_register_point(line.start, &mut known_points);
            line.end = snap_or_register_point(line.end, &mut known_points);
            if
            // 2. make sure line isn't degenerate (no length when start == end)
            line.start != line.end
                // 3. make sure line or flipped line wasn't already added
                && !lines.contains(&line)
                && !lines.contains(&geo::Line::new(line.end, line.start))
            {
                lines.push(line)
            }

            lines
        });

    (known_points, lines)
}

/// iterates over all combinations (a,b) of lines in a vector where a != b
fn iter_line_pairs(
    lines: &[geo::Line<f32>],
) -> impl Iterator<Item = ((usize, &geo::Line<f32>), (usize, &geo::Line<f32>))> {
    lines.iter().enumerate().flat_map(|(idx0, line0)| {
        lines
            .iter()
            .enumerate()
            .filter(move |(idx1, _)| *idx1 != idx0)
            .map(move |(idx1, line1)| ((idx0, line0), (idx1, line1)))
    })
}

/// checks whether two lines are intersecting and if so, checks the intersection to not be ill
/// formed
///
/// returns
/// - [usize;2] : sorted indexes of lines, smaller one comes first
/// - intersection : type of intersection
fn find_intersecting_lines_fn(
    ((idx0, line0), (idx1, line1)): ((usize, &geo::Line<f32>), (usize, &geo::Line<f32>)),
) -> Option<([usize; 2], LineIntersection<f32>)> {
    geo::line_intersection::line_intersection(*line0, *line1)
        .filter(|intersection| {
            match intersection {
                // intersection is not located in both lines
                geo::LineIntersection::SinglePoint { is_proper, .. } if !is_proper => false,
                // collinear intersection is length zero line
                geo::LineIntersection::Collinear { intersection }
                    if intersection.start == intersection.end =>
                {
                    false
                }
                _ => true,
            }
        })
        .map(|intersection| ([idx0, idx1], intersection))
}

/// removes two lines by index in a safe way since the second index can be invalidated after
/// the first line was removed (remember `.remove(idx)` returns the element and shifts the tail
/// of the vector in direction of its start to close the gap)
fn remove_lines_by_index(
    mut indices: [usize; 2],
    lines: &mut Vec<geo::Line<f32>>,
) -> [geo::Line<f32>; 2] {
    indices.sort();
    let [idx0, idx1] = indices;
    let l1 = lines.remove(idx1);
    let l0 = lines.remove(idx0);
    [l0, l1]
}

/// split lines based on intersection kind:
///
/// - intersection point: create 4 new lines from the existing line's endpoints to the intersection
/// point
/// - collinear: create 3 new lines (before overlap, overlap, after overlap)
fn split_lines(
    [l0, l1]: [geo::Line<f32>; 2],
    intersection: geo::LineIntersection<f32>,
) -> Vec<geo::Line<f32>> {
    match intersection {
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
            // sort points by their coordinate values to resolve ambiguities
            points.sort_by(|a, b| {
                a.x.partial_cmp(&b.x)
                    .unwrap()
                    .then_with(|| a.y.partial_cmp(&b.y).unwrap())
            });
            // since all points are on one line we can just create new lines from consecutive
            // points after sorting
            points
                .windows(2)
                .map(|win| geo::Line::new(win[0], win[1]))
                .collect::<Vec<_>>()
        }
    }
}

/// new lines from the `split_lines` function may contain a variety of ill formed lines, this
/// function cleans all of these cases up
fn cleanup_filter_lines(
    lines_need_check: Vec<geo::Line<f32>>,
    existing_lines: &[geo::Line<f32>],
    known_points: &mut Vec<geo::Coord<f32>>,
) -> Vec<geo::Line<f32>> {
    lines_need_check
        .into_iter()
        .map(|mut line| {
            line.start = snap_or_register_point(line.start, known_points);
            line.end = snap_or_register_point(line.end, known_points);
            line
        })
        .filter(|l| l.start != l.end)
        .filter(|l| !existing_lines.contains(l))
        .filter(|l| !existing_lines.contains(&geo::Line::new(l.end, l.start)))
        .collect::<Vec<_>>()
}

/// convertes geo::Line to something somewhat similar in the spade world
fn to_spade_line(line: geo::Line<f32>) -> [Point2<f32>; 2] {
    [
        Point2::new(line.start.x, line.start.y),
        Point2::new(line.end.x, line.end.y),
    ]
}

/// given some geo lines (! NON-INTERSECTING !) create the triangulation resulting from a
/// constrained delauny triangulation of those lines
fn triangulate_lines(
    lines: Vec<geo::Line<f32>>,
) -> Result<Vec<geo::Triangle<f32>>, SpadeBoolopsError> {
    lines
        .into_iter()
        .map(to_spade_line)
        .try_fold(
            ConstrainedDelaunayTriangulation::<Point2<f32>>::new(),
            |mut cdt, [start, end]| {
                let start = cdt.insert(start)?;
                let end = cdt.insert(end)?;
                // safety check (to prevent panic) whether we can add the line
                if !cdt.can_add_constraint(start, end) {
                    return Err(SpadeBoolopsError::ConstraintFailure);
                }
                cdt.add_constraint(start, end);
                Ok(cdt)
            },
        )
        .map(|cdt| {
            // collect triangles if everything went fine
            cdt.inner_faces()
                .map(|a| a.positions().map(|p| geo::Coord { x: p.x, y: p.y }))
                .map(geo::Triangle::from)
                .collect::<Vec<_>>()
        })
}
