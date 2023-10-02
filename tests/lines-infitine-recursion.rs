use geo::*;
use geo_svg::ToSvg;

macro_rules! mk_test {
    ($name:ident, $lines:expr) => {
        #[test]
        fn $name() {
            _ = tracing_subscriber::fmt::try_init();
            let lines = $lines;
            let svg = lines
                .iter()
                .map(|line| line.to_svg())
                .reduce(|a, b| a.and(b))
                .map(|svg| {
                    svg.with_stroke_color(geo_svg::Color::Named("#0004"))
                        .to_string()
                })
                .unwrap_or_default();

            bevy::prelude::info!("{svg}");
            panic!();
        }
    };
}

mk_test! {
    lines1,
    [
        Line {
            start: Coord {
                x: 183.58939,
                y: 12.494377,
            },
            end: Coord {
                x: 183.5927,
                y: 12.54799,
            },
        },
        Line {
            start: Coord {
                x: 183.5927,
                y: 12.54799,
            },
            end: Coord {
                x: 183.5311,
                y: 12.468239,
            },
        },
        Line {
            start: Coord {
                x: 183.5311,
                y: 12.468239,
            },
            end: Coord {
                x: 183.58939,
                y: 12.494377,
            },
        },
        Line {
            start: Coord {
                x: 183.5311,
                y: 12.468239,
            },
            end: Coord {
                x: 146.72794,
                y: -4.0353804,
            },
        },
        Line {
            start: Coord {
                x: 146.72794,
                y: -4.0353804,
            },
            end: Coord {
                x: 112.87517,
                y: 18.561363,
            },
        },
        Line {
            start: Coord {
                x: 112.87517,
                y: 18.561363,
            },
            end: Coord {
                x: 128.00903,
                y: -12.429484,
            },
        },
        Line {
            start: Coord {
                x: 128.00903,
                y: -12.429484,
            },
            end: Coord {
                x: 146.72794,
                y: -4.035381,
            },
        },
        Line {
            start: Coord {
                x: 146.72794,
                y: -4.035381,
            },
            end: Coord {
                x: 162.60092,
                y: -14.630608,
            },
        },
        Line {
            start: Coord {
                x: 162.60092,
                y: -14.630608,
            },
            end: Coord {
                x: 183.5927,
                y: 12.54799,
            },
        },
        Line {
            start: Coord {
                x: 183.5927,
                y: 12.54799,
            },
            end: Coord {
                x: 183.5311,
                y: 12.468239,
            },
        },
    ]
}

mk_test! {
    lines2,
[
        Line {
            start: Coord {
                x: 34.648804,
                y: 97.208046,
            },
            end: Coord {
                x: 25.985832,
                y: 77.67871,
            },
        },
        Line {
            start: Coord {
                x: 25.985832,
                y: 77.67871,
            },
            end: Coord {
                x: 93.43891,
                y: 86.491425,
            },
        },
        Line {
            start: Coord {
                x: 93.43891,
                y: 86.491425,
            },
            end: Coord {
                x: 47.59468,
                y: -29.930397,
            },
        },
        Line {
            start: Coord {
                x: 47.59468,
                y: -29.930397,
            },
            end: Coord {
                x: 129.26524,
                y: 127.95375,
            },
        },
        Line {
            start: Coord {
                x: 129.26524,
                y: 127.95375,
            },
            end: Coord {
                x: 34.89432,
                y: 97.76153,
            },
        },
        Line {
            start: Coord {
                x: 34.89432,
                y: 97.76153,
            },
            end: Coord {
                x: 60.506374,
                y: 105.95563,
            },
        },
        Line {
            start: Coord {
                x: 60.506374,
                y: 105.95563,
            },
            end: Coord {
                x: 59.348248,
                y: 105.182045,
            },
        },
        Line {
            start: Coord {
                x: 59.348248,
                y: 105.182045,
            },
            end: Coord {
                x: 34.648804,
                y: 97.208046,
            },
        },
        Line {
            start: Coord {
                x: 10.269012,
                y: 134.0799,
            },
            end: Coord {
                x: 52.478096,
                y: 137.40143,
            },
        },
        Line {
            start: Coord {
                x: 52.478096,
                y: 137.40143,
            },
            end: Coord {
                x: 114.94266,
                y: 142.31693,
            },
        },
        Line {
            start: Coord {
                x: 114.94266,
                y: 142.31693,
            },
            end: Coord {
                x: 60.506374,
                y: 105.95563,
            },
        },
        Line {
            start: Coord {
                x: 60.506374,
                y: 105.95563,
            },
            end: Coord {
                x: 34.89432,
                y: 97.76153,
            },
        },
        Line {
            start: Coord {
                x: 34.89432,
                y: 97.76153,
            },
            end: Coord {
                x: 52.478096,
                y: 137.40143,
            },
        },
        Line {
            start: Coord {
                x: 52.478096,
                y: 137.40143,
            },
            end: Coord {
                x: 53.93119,
                y: 140.6772,
            },
        },
        Line {
            start: Coord {
                x: 53.93119,
                y: 140.6772,
            },
            end: Coord {
                x: 10.269012,
                y: 134.0799,
            },
        },
    ]
        }

mk_test! {
    lines3,
[Line { start: Coord { x: -32.928577, y: 14.782345 }, end: Coord { x: -13.181059, y: 79.938065 } }, Line { start: Coord { x: -13.181059, y: 79.938065 }, end: Coord { x: -17.093695, y: 78.97919 } }, Line { start: Coord { x: -17.093695, y: 78.97919 }, end: Coord { x: -95.73333, y: 34.03808 } }, Line { start: Coord { x: -95.73333, y: 34.03808 }, end: Coord { x: -32.928577, y: 14.782345 } }, Line { start: Coord { x: -17.093695, y: 78.97919}, end: Coord { x: -12.7129, y: 81.48273 } }, Line { start: Coord { x: -12.7129, y: 81.48273 }, end: Coord { x: -13.181059, y: 79.938065 } }, Line { start: Coord { x: -13.181059, y: 79.938065 }, end: Coord { x: 67.44542, y: 99.69739 } }, Line { start: Coord { x: 67.44542, y: 99.69739 }, end: Coord { x: 99.024765, y: 118.352646 } }, Line { start: Coord { x: 99.024765, y: 118.352646 }, end: Coord { x: 84.75231, y: 137.18231 } }, Line { start: Coord { x: 84.75231, y: 137.18231 }, end: Coord { x: -12.7129, y: 81.48273 } }, Line { start: Coord { x: -12.7129, y: 81.48273 }, end: Coord { x: 4.776985, y: 139.18953 }}, Line { start: Coord { x: 4.776985, y: 139.18953 }, end: Coord { x: 121.60872, y: 180.06174 } }, Line { start: Coord { x: 121.60872, y: 180.06174 }, end: Coord { x: -99.87984, y: 256.1396 } }, Line { start: Coord { x: -99.87984, y: 256.1396 }, end: Coord { x: -44.181297, y: 72.34076 } }, Line { start: Coord { x: -44.181297, y: 72.34076 }, end: Coord { x: -13.181059, y: 79.938065 } }]
}
