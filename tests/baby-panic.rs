use baby_shark::triangulation::constrained_delaunay::ConstrainedTriangulation2;
use nalgebra::Point2;

#[test]
fn baby_shark_panics() {
    let points = [
        [-100.0, 0.0],
        [-100.0, 100.0],
        [0.0, 100.0],
        [0.0, 0.0],
        [0.0, -100.0],
        [100.578125, 42.601563],
        [100.0, -100.0],
    ]
    .map(Point2::from);

    let _ = ConstrainedTriangulation2::from_points(&points);

    //let edges = [
    //    (0, 1),
    //    (1, 2),
    //    (2, 3),
    //    (3, 0),
    //    (4, 3),
    //    (3, 5),
    //    (5, 6),
    //    (6, 4),
    //];
    //
    //for (a, b) in edges {
    //    cdt.insert_constrained_edge(a, b);
    //}
    //
    //cdt.triangles();
}
