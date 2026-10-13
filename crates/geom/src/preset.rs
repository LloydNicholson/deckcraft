//! Preset shape geometry.
//!
//! Each preset is identified by its Office Open XML name (`rect`, `roundRect`, `rightArrow`…) so
//! files round-trip, but every outline here is our own construction in Rust: we re-derive the shapes
//! from what they look like and from the meaning of their adjust values, not from any reference
//! geometry table. Adjust values use the file convention: ratios in 1/100 000 of the shorter side (or
//! of the width/height), angles in 1/60 000 degree.

use kurbo::{Arc, BezPath, Point, Rect, Shape as _, Vec2};
use serde::{Deserialize, Serialize};

mod extra;

/// How a sub-path is filled. `Lighten`/`Darken` variants shade the shape's fill (3-D looking faces).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum FillMode {
    Norm,
    None,
    Lighten,
    LightenLess,
    Darken,
    DarkenLess,
}

#[derive(Clone, Debug)]
pub struct SubPath {
    pub path: BezPath,
    pub fill: FillMode,
    pub stroke: bool,
    /// Even-odd fill (holes in donuts and frames).
    pub even_odd: bool,
}

/// A draggable adjust handle (the yellow dot). `pos = origin + dir * value`; dragging projects
/// the pointer onto `dir`.
#[derive(Clone, Debug, PartialEq)]
pub struct Handle {
    pub index: usize,
    pub pos: Point,
    pub origin: Point,
    pub dir: Vec2,
    pub min: f64,
    pub max: f64,
}

impl Handle {
    fn new(index: usize, origin: Point, dir: Vec2, value: f64, min: f64, max: f64) -> Self {
        Handle { index, pos: origin + dir * value, origin, dir, min, max }
    }
    /// The adjust value for a pointer at `p` (shape-local space).
    pub fn value_at(&self, p: Point) -> f64 {
        let l2 = self.dir.hypot2();
        if l2 < 1e-18 {
            return self.min;
        }
        let v = (p - self.origin).dot(self.dir) / l2;
        if !v.is_finite() {
            return self.min;
        }
        v.clamp(self.min.min(self.max), self.max.max(self.min)).round()
    }
}

#[derive(Clone, Debug)]
pub struct Geometry {
    pub paths: Vec<SubPath>,
    /// Where text goes, in shape-local space.
    pub text_rect: Rect,
    pub handles: Vec<Handle>,
    /// Connector glue points, shape-local.
    pub sites: Vec<Point>,
}

impl Geometry {
    /// The union outline used for hit testing and selection (all filled or stroked sub-paths).
    pub fn outline(&self) -> BezPath {
        let mut p = BezPath::new();
        for s in &self.paths {
            p.extend(s.path.iter());
        }
        p
    }
    /// Whether nothing is filled (lines, connectors, arcs, brackets).
    pub fn is_open(&self) -> bool {
        self.paths.iter().all(|s| s.fill == FillMode::None)
    }
}

/// Gallery category, as in the Shapes menu.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Category {
    Lines,
    Rectangles,
    BasicShapes,
    BlockArrows,
    EquationShapes,
    Flowchart,
    StarsAndBanners,
    Callouts,
    ActionButtons,
}

impl Category {
    pub const ALL: [Category; 9] = [
        Category::Lines,
        Category::Rectangles,
        Category::BasicShapes,
        Category::BlockArrows,
        Category::EquationShapes,
        Category::Flowchart,
        Category::StarsAndBanners,
        Category::Callouts,
        Category::ActionButtons,
    ];
    pub fn label(self) -> &'static str {
        match self {
            Category::Lines => "Lines",
            Category::Rectangles => "Rectangles",
            Category::BasicShapes => "Basic Shapes",
            Category::BlockArrows => "Block Arrows",
            Category::EquationShapes => "Equation Shapes",
            Category::Flowchart => "Flowchart",
            Category::StarsAndBanners => "Stars and Banners",
            Category::Callouts => "Callouts",
            Category::ActionButtons => "Action Buttons",
        }
    }
}

pub struct PresetInfo {
    pub name: &'static str,
    pub label: &'static str,
    pub category: Category,
    /// Default adjust values.
    pub defaults: &'static [f64],
}

macro_rules! p {
    ($n:literal, $l:literal, $c:ident, [$($d:expr),*]) => {
        PresetInfo { name: $n, label: $l, category: Category::$c, defaults: &[$($d as f64),*] }
    };
}

/// Every preset we generate, in gallery order.
pub static CATALOG: &[PresetInfo] = &[
    p!("line", "Line", Lines, []),
    p!("straightConnector1", "Arrow", Lines, []),
    p!("bentConnector3", "Connector: Elbow", Lines, [50000]),
    p!("curvedConnector3", "Connector: Curved", Lines, [50000]),
    p!("bentConnector2", "Connector: Elbow (one bend)", Lines, []),
    p!("lineInv", "Line: Rising", Lines, []),
    p!("rect", "Rectangle", Rectangles, []),
    p!("roundRect", "Rectangle: Rounded Corners", Rectangles, [16667]),
    p!("snip1Rect", "Rectangle: Single Corner Snipped", Rectangles, [16667]),
    p!("snip2SameRect", "Rectangle: Top Corners Snipped", Rectangles, [16667, 0]),
    p!("snip2DiagRect", "Rectangle: Diagonal Corners Snipped", Rectangles, [0, 16667]),
    p!("snipRoundRect", "Rectangle: Top Corners One Rounded and One Snipped", Rectangles, [16667, 16667]),
    p!("round1Rect", "Rectangle: Single Corner Rounded", Rectangles, [16667]),
    p!("round2SameRect", "Rectangle: Top Corners Rounded", Rectangles, [16667, 0]),
    p!("round2DiagRect", "Rectangle: Diagonal Corners Rounded", Rectangles, [16667, 0]),
    p!("textBox", "Text Box", BasicShapes, []),
    p!("ellipse", "Oval", BasicShapes, []),
    p!("triangle", "Isosceles Triangle", BasicShapes, [50000]),
    p!("rtTriangle", "Right Triangle", BasicShapes, []),
    p!("parallelogram", "Parallelogram", BasicShapes, [25000]),
    p!("trapezoid", "Trapezoid", BasicShapes, [25000]),
    p!("nonIsoscelesTrapezoid", "Trapezoid: Uneven Sides", BasicShapes, [25000, 25000]),
    p!("diamond", "Diamond", BasicShapes, []),
    p!("pentagon", "Regular Pentagon", BasicShapes, []),
    p!("hexagon", "Hexagon", BasicShapes, [25000]),
    p!("heptagon", "Heptagon", BasicShapes, []),
    p!("octagon", "Octagon", BasicShapes, [29289]),
    p!("decagon", "Decagon", BasicShapes, []),
    p!("dodecagon", "Dodecagon", BasicShapes, []),
    p!("pie", "Partial Circle", BasicShapes, [0, 16200000]),
    p!("pieWedge", "Quarter Wedge", BasicShapes, []),
    p!("chord", "Chord", BasicShapes, [2700000, 16200000]),
    p!("teardrop", "Teardrop", BasicShapes, [100000]),
    p!("frame", "Frame", BasicShapes, [12500]),
    p!("cornerTabs", "Corner Tabs: Triangles", BasicShapes, []),
    p!("squareTabs", "Corner Tabs: Squares", BasicShapes, []),
    p!("plaqueTabs", "Corner Tabs: Quarter Circles", BasicShapes, []),
    p!("halfFrame", "Half Frame", BasicShapes, [33333, 33333]),
    p!("corner", "L-Shape", BasicShapes, [50000, 50000]),
    p!("diagStripe", "Diagonal Stripe", BasicShapes, [50000]),
    p!("plus", "Cross", BasicShapes, [25000]),
    p!("plaque", "Plaque", BasicShapes, [16667]),
    p!("can", "Cylinder", BasicShapes, [25000]),
    p!("cube", "Cube", BasicShapes, [25000]),
    p!("bevel", "Bevel", BasicShapes, [12500]),
    p!("donut", "Donut", BasicShapes, [25000]),
    p!("noSmoking", "\"Not Allowed\" Symbol", BasicShapes, [18750]),
    p!("blockArc", "Block Arc", BasicShapes, [10800000, 0, 25000]),
    p!("foldedCorner", "Folded Corner", BasicShapes, [16667]),
    p!("smileyFace", "Smiley Face", BasicShapes, [4653]),
    p!("heart", "Heart", BasicShapes, []),
    p!("lightningBolt", "Lightning Bolt", BasicShapes, []),
    p!("sun", "Sun", BasicShapes, [25000]),
    p!("moon", "Moon", BasicShapes, [50000]),
    p!("cloud", "Cloud", BasicShapes, []),
    p!("gear6", "Gear: 6 Teeth", BasicShapes, [15000, 3526]),
    p!("gear9", "Gear: 9 Teeth", BasicShapes, [10000, 1763]),
    p!("funnel", "Funnel", BasicShapes, []),
    p!("arc", "Arc", BasicShapes, [16200000, 0]),
    p!("bracketPair", "Double Bracket", BasicShapes, [16667]),
    p!("bracePair", "Double Brace", BasicShapes, [8333]),
    p!("leftBracket", "Left Bracket", BasicShapes, [8333]),
    p!("rightBracket", "Right Bracket", BasicShapes, [8333]),
    p!("leftBrace", "Left Brace", BasicShapes, [8333, 50000]),
    p!("rightBrace", "Right Brace", BasicShapes, [8333, 50000]),
    p!("chartX", "Chart Marker: Cross Lines", BasicShapes, []),
    p!("chartStar", "Chart Marker: Star Lines", BasicShapes, []),
    p!("chartPlus", "Chart Marker: Plus Lines", BasicShapes, []),
    p!("rightArrow", "Arrow: Right", BlockArrows, [50000, 50000]),
    p!("leftArrow", "Arrow: Left", BlockArrows, [50000, 50000]),
    p!("upArrow", "Arrow: Up", BlockArrows, [50000, 50000]),
    p!("downArrow", "Arrow: Down", BlockArrows, [50000, 50000]),
    p!("leftRightArrow", "Arrow: Left-Right", BlockArrows, [50000, 50000]),
    p!("upDownArrow", "Arrow: Up-Down", BlockArrows, [50000, 50000]),
    p!("quadArrow", "Arrow: Quad", BlockArrows, [22500, 22500, 22500]),
    p!("leftRightUpArrow", "Arrow: Left-Right-Up", BlockArrows, [25000, 25000, 25000]),
    p!("bentArrow", "Arrow: Bent", BlockArrows, [25000, 25000, 25000, 43750]),
    p!("uturnArrow", "Arrow: U-Turn", BlockArrows, [25000, 25000, 25000, 43750, 75000]),
    p!("curvedRightArrow", "Arrow: Curved Right", BlockArrows, [25000, 50000, 25000]),
    p!("curvedLeftArrow", "Arrow: Curved Left", BlockArrows, [25000, 50000, 25000]),
    p!("curvedUpArrow", "Arrow: Curved Up", BlockArrows, [25000, 50000, 25000]),
    p!("curvedDownArrow", "Arrow: Curved Down", BlockArrows, [25000, 50000, 25000]),
    p!("leftUpArrow", "Arrow: Left-Up", BlockArrows, [25000, 25000, 25000]),
    p!("bentUpArrow", "Arrow: Bent-Up", BlockArrows, [25000, 25000, 25000]),
    p!("stripedRightArrow", "Arrow: Striped Right", BlockArrows, [50000, 50000]),
    p!("notchedRightArrow", "Arrow: Notched Right", BlockArrows, [50000, 50000]),
    p!("homePlate", "Arrow: Pentagon", BlockArrows, [50000]),
    p!("chevron", "Arrow: Chevron", BlockArrows, [50000]),
    p!("circularArrow", "Arrow: Circular", BlockArrows, [12500, 1142319, 20457681, 10800000, 12500]),
    p!("leftCircularArrow", "Arrow: Circular Counter-clockwise", BlockArrows, [12500, -1142319, 1142319, 10800000, 12500]),
    p!("leftRightCircularArrow", "Arrow: Circular Two-headed", BlockArrows, [12500, 1142319, 20457681, 11942319, 12500]),
    p!("swooshArrow", "Arrow: Swoosh", BlockArrows, [25000, 16667]),
    p!("rightArrowCallout", "Callout: Right Arrow", BlockArrows, [25000, 25000, 25000, 64977]),
    p!("downArrowCallout", "Callout: Down Arrow", BlockArrows, [25000, 25000, 25000, 64977]),
    p!("leftArrowCallout", "Callout: Left Arrow", BlockArrows, [25000, 25000, 25000, 64977]),
    p!("upArrowCallout", "Callout: Up Arrow", BlockArrows, [25000, 25000, 25000, 64977]),
    p!("leftRightArrowCallout", "Callout: Left-Right Arrow", BlockArrows, [25000, 25000, 25000, 48123]),
    p!("upDownArrowCallout", "Callout: Up-Down Arrow", BlockArrows, [25000, 25000, 25000, 48123]),
    p!("quadArrowCallout", "Callout: Quad Arrow", BlockArrows, [18515, 18515, 18515, 48123]),
    p!("mathPlus", "Plus Sign", EquationShapes, [23520]),
    p!("mathMinus", "Minus Sign", EquationShapes, [23520]),
    p!("mathMultiply", "Multiplication Sign", EquationShapes, [23520]),
    p!("mathDivide", "Division Sign", EquationShapes, [23520, 5880, 11760]),
    p!("mathEqual", "Equals", EquationShapes, [23520, 11760]),
    p!("mathNotEqual", "Not Equal", EquationShapes, [23520, 6600000, 11760]),
    p!("flowChartProcess", "Flowchart: Process", Flowchart, []),
    p!("flowChartAlternateProcess", "Flowchart: Alternate Process", Flowchart, []),
    p!("flowChartDecision", "Flowchart: Decision", Flowchart, []),
    p!("flowChartInputOutput", "Flowchart: Data", Flowchart, []),
    p!("flowChartPredefinedProcess", "Flowchart: Predefined Process", Flowchart, []),
    p!("flowChartInternalStorage", "Flowchart: Internal Storage", Flowchart, []),
    p!("flowChartDocument", "Flowchart: Document", Flowchart, []),
    p!("flowChartMultidocument", "Flowchart: Multidocument", Flowchart, []),
    p!("flowChartTerminator", "Flowchart: Terminator", Flowchart, []),
    p!("flowChartPreparation", "Flowchart: Preparation", Flowchart, []),
    p!("flowChartManualInput", "Flowchart: Manual Input", Flowchart, []),
    p!("flowChartManualOperation", "Flowchart: Manual Operation", Flowchart, []),
    p!("flowChartConnector", "Flowchart: Connector", Flowchart, []),
    p!("flowChartOffpageConnector", "Flowchart: Off-page Connector", Flowchart, []),
    p!("flowChartPunchedCard", "Flowchart: Card", Flowchart, []),
    p!("flowChartPunchedTape", "Flowchart: Punched Tape", Flowchart, []),
    p!("flowChartSummingJunction", "Flowchart: Summing Junction", Flowchart, []),
    p!("flowChartOr", "Flowchart: Or", Flowchart, []),
    p!("flowChartCollate", "Flowchart: Collate", Flowchart, []),
    p!("flowChartSort", "Flowchart: Sort", Flowchart, []),
    p!("flowChartExtract", "Flowchart: Extract", Flowchart, []),
    p!("flowChartMerge", "Flowchart: Merge", Flowchart, []),
    p!("flowChartOnlineStorage", "Flowchart: Stored Data", Flowchart, []),
    p!("flowChartDelay", "Flowchart: Delay", Flowchart, []),
    p!("flowChartMagneticTape", "Flowchart: Sequential Access Storage", Flowchart, []),
    p!("flowChartMagneticDisk", "Flowchart: Magnetic Disk", Flowchart, []),
    p!("flowChartMagneticDrum", "Flowchart: Direct Access Storage", Flowchart, []),
    p!("flowChartDisplay", "Flowchart: Display", Flowchart, []),
    p!("flowChartOfflineStorage", "Flowchart: Offline Storage", Flowchart, []),
    p!("irregularSeal1", "Explosion: 8 Points", StarsAndBanners, []),
    p!("irregularSeal2", "Explosion: 14 Points", StarsAndBanners, []),
    p!("star4", "Star: 4 Points", StarsAndBanners, [12500]),
    p!("star5", "Star: 5 Points", StarsAndBanners, [19098]),
    p!("star6", "Star: 6 Points", StarsAndBanners, [28868]),
    p!("star7", "Star: 7 Points", StarsAndBanners, [34601]),
    p!("star8", "Star: 8 Points", StarsAndBanners, [37500]),
    p!("star10", "Star: 10 Points", StarsAndBanners, [42533]),
    p!("star12", "Star: 12 Points", StarsAndBanners, [37500]),
    p!("star16", "Star: 16 Points", StarsAndBanners, [37500]),
    p!("star24", "Star: 24 Points", StarsAndBanners, [37500]),
    p!("star32", "Star: 32 Points", StarsAndBanners, [37500]),
    p!("ribbon2", "Ribbon: Tilted Up", StarsAndBanners, [16667, 50000]),
    p!("ribbon", "Ribbon: Tilted Down", StarsAndBanners, [16667, 50000]),
    p!("ellipseRibbon2", "Ribbon: Curved and Tilted Up", StarsAndBanners, [25000, 50000, 12500]),
    p!("ellipseRibbon", "Ribbon: Curved and Tilted Down", StarsAndBanners, [25000, 50000, 12500]),
    p!("leftRightRibbon", "Ribbon: Left-Right Fold", StarsAndBanners, [50000, 50000, 16667]),
    p!("verticalScroll", "Scroll: Vertical", StarsAndBanners, [12500]),
    p!("horizontalScroll", "Scroll: Horizontal", StarsAndBanners, [12500]),
    p!("wave", "Wave", StarsAndBanners, [12500, 0]),
    p!("doubleWave", "Double Wave", StarsAndBanners, [6250, 0]),
    p!("wedgeRectCallout", "Speech Bubble: Rectangle", Callouts, [-20833, 62500]),
    p!("wedgeRoundRectCallout", "Speech Bubble: Rectangle with Corners Rounded", Callouts, [-20833, 62500, 16667]),
    p!("wedgeEllipseCallout", "Speech Bubble: Oval", Callouts, [-20833, 62500]),
    p!("cloudCallout", "Thought Bubble: Cloud", Callouts, [-20833, 62500]),
    p!("borderCallout1", "Callout: Line", Callouts, [18750, -8333, 112500, -38333]),
    p!("borderCallout2", "Callout: Bent Line", Callouts, [18750, -8333, 18750, -16667, 112500, -46667]),
    p!("borderCallout3", "Callout: Double Bent Line", Callouts, [18750, -8333, 18750, -16667, 100000, -16667, 112963, -8333]),
    p!("accentCallout1", "Callout: Line with Accent Bar", Callouts, [18750, -8333, 112500, -38333]),
    p!("accentCallout2", "Callout: Bent Line with Accent Bar", Callouts, [18750, -8333, 18750, -16667, 112500, -46667]),
    p!("accentCallout3", "Callout: Double Bent Line with Accent Bar", Callouts, [18750, -8333, 18750, -16667, 100000, -16667, 112963, -8333]),
    p!("callout1", "Callout: Line with No Border", Callouts, [18750, -8333, 112500, -38333]),
    p!("callout2", "Callout: Bent Line with No Border", Callouts, [18750, -8333, 18750, -16667, 112500, -46667]),
    p!("callout3", "Callout: Double Bent Line with No Border", Callouts, [18750, -8333, 18750, -16667, 100000, -16667, 112963, -8333]),
    p!("accentBorderCallout1", "Callout: Line with Border and Accent Bar", Callouts, [18750, -8333, 112500, -38333]),
    p!("accentBorderCallout2", "Callout: Bent Line with Border and Accent Bar", Callouts, [18750, -8333, 18750, -16667, 112500, -46667]),
    p!(
        "accentBorderCallout3",
        "Callout: Double Bent Line with Border and Accent Bar",
        Callouts,
        [18750, -8333, 18750, -16667, 100000, -16667, 112963, -8333]
    ),
    p!("actionButtonBackPrevious", "Action Button: Go Back or Previous", ActionButtons, []),
    p!("actionButtonForwardNext", "Action Button: Go Forward or Next", ActionButtons, []),
    p!("actionButtonBeginning", "Action Button: Go to Beginning", ActionButtons, []),
    p!("actionButtonEnd", "Action Button: Go to End", ActionButtons, []),
    p!("actionButtonHome", "Action Button: Go Home", ActionButtons, []),
    p!("actionButtonInformation", "Action Button: Get Information", ActionButtons, []),
    p!("actionButtonReturn", "Action Button: Go Back", ActionButtons, []),
    p!("actionButtonMovie", "Action Button: Video", ActionButtons, []),
    p!("actionButtonDocument", "Action Button: Document", ActionButtons, []),
    p!("actionButtonSound", "Action Button: Sound", ActionButtons, []),
    p!("actionButtonHelp", "Action Button: Help", ActionButtons, []),
    p!("actionButtonBlank", "Action Button: Custom", ActionButtons, []),
];

pub fn info(name: &str) -> Option<&'static PresetInfo> {
    CATALOG.iter().find(|p| p.name == name)
}

pub fn is_line_like(name: &str) -> bool {
    matches!(name, "line" | "straightConnector1" | "bentConnector2" | "bentConnector3" | "curvedConnector3" | "arc")
        || name.starts_with("bentConnector")
        || name.starts_with("curvedConnector")
}

// ---------- path builder ----------

struct Pb {
    p: BezPath,
    cur: Point,
}

impl Pb {
    fn new() -> Self {
        Pb { p: BezPath::new(), cur: Point::ZERO }
    }
    fn m(&mut self, x: f64, y: f64) -> &mut Self {
        self.cur = Point::new(x, y);
        self.p.move_to(self.cur);
        self
    }
    fn l(&mut self, x: f64, y: f64) -> &mut Self {
        self.cur = Point::new(x, y);
        self.p.line_to(self.cur);
        self
    }
    fn c(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, x: f64, y: f64) -> &mut Self {
        self.cur = Point::new(x, y);
        self.p.curve_to(Point::new(x1, y1), Point::new(x2, y2), self.cur);
        self
    }
    fn q(&mut self, x1: f64, y1: f64, x: f64, y: f64) -> &mut Self {
        self.cur = Point::new(x, y);
        self.p.quad_to(Point::new(x1, y1), self.cur);
        self
    }
    /// Elliptic arc continuing from the current point, which lies on the ellipse at parametric
    /// angle `st` (degrees); sweeps `sw` degrees (positive = clockwise on screen).
    fn arc(&mut self, wr: f64, hr: f64, st: f64, sw: f64) -> &mut Self {
        let (s, e) = (st.to_radians(), (st + sw).to_radians());
        let c = Point::new(self.cur.x - wr * s.cos(), self.cur.y - hr * s.sin());
        let a = Arc::new(c, Vec2::new(wr.abs(), hr.abs()), s, e - s, 0.0);
        a.to_cubic_beziers(0.05, |p1, p2, p| {
            self.p.curve_to(p1, p2, p);
        });
        self.cur = Point::new(c.x + wr * e.cos(), c.y + hr * e.sin());
        self
    }
    fn z(&mut self) -> &mut Self {
        self.p.close_path();
        self
    }
    fn poly(&mut self, pts: &[(f64, f64)]) -> &mut Self {
        for (i, (x, y)) in pts.iter().enumerate() {
            if i == 0 {
                self.m(*x, *y);
            } else {
                self.l(*x, *y);
            }
        }
        self.z()
    }
    fn ellipse(&mut self, cx: f64, cy: f64, rx: f64, ry: f64) -> &mut Self {
        self.m(cx + rx, cy);
        self.arc(rx, ry, 0.0, 360.0);
        self.z()
    }
    fn take(&mut self) -> BezPath {
        std::mem::take(&mut self.p)
    }
}

fn fill(p: BezPath) -> SubPath {
    SubPath { path: p, fill: FillMode::Norm, stroke: true, even_odd: false }
}
fn fill_eo(p: BezPath) -> SubPath {
    SubPath { path: p, fill: FillMode::Norm, stroke: true, even_odd: true }
}
fn shade(p: BezPath, m: FillMode) -> SubPath {
    SubPath { path: p, fill: m, stroke: true, even_odd: false }
}
fn stroke_only(p: BezPath) -> SubPath {
    SubPath { path: p, fill: FillMode::None, stroke: true, even_odd: false }
}
fn fill_only(p: BezPath) -> SubPath {
    SubPath { path: p, fill: FillMode::Norm, stroke: false, even_odd: false }
}

/// Parametric angle on an rx×ry ellipse for a visual angle `deg` (file angles are visual).
fn vis(deg: f64, rx: f64, ry: f64) -> f64 {
    let t = deg.to_radians();
    (rx * t.sin()).atan2(ry * t.cos()).to_degrees()
}

fn polygon_regular(n: usize, w: f64, h: f64, phase: f64) -> Vec<(f64, f64)> {
    let pts: Vec<(f64, f64)> = (0..n)
        .map(|i| {
            let a = (phase + 360.0 * i as f64 / n as f64).to_radians();
            (a.cos(), a.sin())
        })
        .collect();
    normalize(&pts, w, h)
}

/// Scale `pts` so their bounding box fills 0..w × 0..h.
fn normalize(pts: &[(f64, f64)], w: f64, h: f64) -> Vec<(f64, f64)> {
    let (mut x0, mut y0, mut x1, mut y1) = (f64::MAX, f64::MAX, f64::MIN, f64::MIN);
    for (x, y) in pts {
        x0 = x0.min(*x);
        y0 = y0.min(*y);
        x1 = x1.max(*x);
        y1 = y1.max(*y);
    }
    let sx = if x1 - x0 > 1e-12 { w / (x1 - x0) } else { 0.0 };
    let sy = if y1 - y0 > 1e-12 { h / (y1 - y0) } else { 0.0 };
    pts.iter().map(|(x, y)| ((x - x0) * sx, (y - y0) * sy)).collect()
}

fn star(n: usize, w: f64, h: f64, inner: f64, normalized: bool) -> Vec<(f64, f64)> {
    let mut pts = Vec::with_capacity(n * 2);
    for i in 0..n * 2 {
        let a = (-90.0 + 180.0 * i as f64 / n as f64).to_radians();
        let r = if i % 2 == 0 { 1.0 } else { inner };
        pts.push((r * a.cos(), r * a.sin()));
    }
    if normalized { normalize(&pts, w, h) } else { pts.iter().map(|(x, y)| (w / 2.0 + x * w / 2.0, h / 2.0 + y * h / 2.0)).collect() }
}

#[derive(Clone, Copy)]
enum Corner {
    Square,
    Round(f64),
    Snip(f64),
}

/// A rectangle with each corner (TL, TR, BR, BL) square, rounded or snipped.
fn corner_rect(w: f64, h: f64, c: [Corner; 4]) -> BezPath {
    let mut b = Pb::new();
    let size = |k: Corner| match k {
        Corner::Square => 0.0,
        Corner::Round(r) | Corner::Snip(r) => r.max(0.0),
    };
    let s0 = size(c[0]);
    b.m(s0, 0.0);
    // top edge → TR
    let s1 = size(c[1]);
    b.l(w - s1, 0.0);
    match c[1] {
        Corner::Round(r) if r > 0.0 => {
            b.arc(r, r, 270.0, 90.0);
        }
        Corner::Snip(_) => {
            b.l(w, s1);
        }
        _ => {}
    }
    let s2 = size(c[2]);
    b.l(w, h - s2);
    match c[2] {
        Corner::Round(r) if r > 0.0 => {
            b.arc(r, r, 0.0, 90.0);
        }
        Corner::Snip(_) => {
            b.l(w - s2, h);
        }
        _ => {}
    }
    let s3 = size(c[3]);
    b.l(s3, h);
    match c[3] {
        Corner::Round(r) if r > 0.0 => {
            b.arc(r, r, 90.0, 90.0);
        }
        Corner::Snip(_) => {
            b.l(0.0, h - s3);
        }
        _ => {}
    }
    b.l(0.0, s0);
    match c[0] {
        Corner::Round(r) if r > 0.0 => {
            b.arc(r, r, 180.0, 90.0);
        }
        Corner::Snip(_) => {
            b.l(s0, 0.0);
        }
        _ => {}
    }
    b.z();
    b.take()
}

fn mid_sites(w: f64, h: f64) -> Vec<Point> {
    vec![Point::new(w / 2.0, 0.0), Point::new(0.0, h / 2.0), Point::new(w / 2.0, h), Point::new(w, h / 2.0)]
}

/// Orientation for single-axis arrows built along +x.
#[derive(Clone, Copy)]
enum Dir {
    Right,
    Left,
    Up,
    Down,
}

/// Map canonical (u along the arrow's length L, v across thickness T) to shape space.
fn orient(d: Dir, w: f64, h: f64, pts: &[(f64, f64)]) -> Vec<(f64, f64)> {
    pts.iter()
        .map(|&(u, v)| match d {
            Dir::Right => (u, v),
            Dir::Left => (w - u, v),
            Dir::Up => (v, h - u),
            Dir::Down => (v, u),
        })
        .collect()
}

fn len_thick(d: Dir, w: f64, h: f64) -> (f64, f64) {
    match d {
        Dir::Right | Dir::Left => (w, h),
        Dir::Up | Dir::Down => (h, w),
    }
}

fn arrow(d: Dir, w: f64, h: f64, a1: f64, a2: f64) -> Geometry {
    let (l, t) = len_thick(d, w, h);
    let ss = l.min(t);
    let a1 = a1.clamp(0.0, 100000.0) / 100000.0;
    let dx = (ss * a2.clamp(0.0, 100000.0 * l / ss.max(1e-9)) / 100000.0).min(l);
    let y1 = t / 2.0 - t * a1 / 2.0;
    let y2 = t / 2.0 + t * a1 / 2.0;
    let pts = [(0.0, y1), (l - dx, y1), (l - dx, 0.0), (l, t / 2.0), (l - dx, t), (l - dx, y2), (0.0, y2)];
    let pts = orient(d, w, h, &pts);
    let tr = orient(d, w, h, &[(0.0, y1), (l - dx + dx * (y1 / (t / 2.0).max(1e-9)), y2)]);
    let mut text = Rect::from_points(tr[0], tr[1]);
    text = text.abs();
    let mut g = Geometry { paths: vec![fill(Pb::new().poly(&pts).take())], text_rect: text, handles: vec![], sites: mid_sites(w, h) };
    // handle 0: shaft thickness on the head base; handle 1: head length.
    let o = orient(d, w, h, &[(l - dx, t / 2.0), (l, t / 2.0)]);
    let (o0, o1) = (Point::new(o[0].0, o[0].1), Point::new(o[1].0, o[1].1));
    let across = match d {
        Dir::Right | Dir::Left => Vec2::new(0.0, -t / 2.0 / 100000.0),
        Dir::Up | Dir::Down => Vec2::new(-t / 2.0 / 100000.0, 0.0),
    };
    let along = (o0 - o1) * (ss / 100000.0 / dx.max(1e-9));
    g.handles.push(Handle::new(0, o0, across, a1 * 100000.0, 0.0, 100000.0));
    if dx > 1e-9 {
        g.handles.push(Handle::new(1, o1, along, a2, 0.0, 100000.0 * l / ss.max(1e-9)));
    }
    g
}

fn double_arrow(vertical: bool, w: f64, h: f64, a1: f64, a2: f64) -> Geometry {
    let (l, t) = if vertical { (h, w) } else { (w, h) };
    let ss = l.min(t);
    let a1 = a1.clamp(0.0, 100000.0) / 100000.0;
    let dx = (ss * a2.max(0.0) / 100000.0).min(l / 2.0);
    let y1 = t / 2.0 - t * a1 / 2.0;
    let y2 = t / 2.0 + t * a1 / 2.0;
    let pts = [(0.0, t / 2.0), (dx, 0.0), (dx, y1), (l - dx, y1), (l - dx, 0.0), (l, t / 2.0), (l - dx, t), (l - dx, y2), (dx, y2), (dx, t)];
    let pts = orient(if vertical { Dir::Down } else { Dir::Right }, w, h, &pts);
    let r = if vertical { Rect::new(y1, dx, y2, h - dx) } else { Rect::new(dx, y1, w - dx, y2) };
    Geometry { paths: vec![fill(Pb::new().poly(&pts).take())], text_rect: r, handles: vec![], sites: mid_sites(w, h) }
}

fn simple(path: BezPath, text: Rect, w: f64, h: f64) -> Geometry {
    Geometry { paths: vec![fill(path)], text_rect: text, handles: vec![], sites: mid_sites(w, h) }
}

/// Builds the geometry for preset `name` at size `w`×`h` with adjust values `adj` (missing
/// values use the preset's defaults). Unknown names return `None`; the caller falls back to `rect`.
pub fn build(name: &str, w: f64, h: f64, adj: &[f64]) -> Option<Geometry> {
    if !(w.is_finite() && h.is_finite()) {
        return None;
    }
    let w = w.max(0.0);
    let h = h.max(0.0);
    let defaults = info(name).map(|i| i.defaults).unwrap_or(&[]);
    let a = |i: usize| -> f64 {
        let v = adj.get(i).copied().or_else(|| defaults.get(i).copied()).unwrap_or(0.0);
        if v.is_finite() { v } else { 0.0 }
    };
    let ss = w.min(h);
    let full = Rect::new(0.0, 0.0, w, h);
    let mut b = Pb::new();
    let g = match name {
        "rect" | "textBox" | "flowChartProcess" => simple(corner_rect(w, h, [Corner::Square; 4]), full, w, h),
        "actionButtonBlank"
        | "actionButtonBackPrevious"
        | "actionButtonForwardNext"
        | "actionButtonBeginning"
        | "actionButtonEnd"
        | "actionButtonHome"
        | "actionButtonInformation"
        | "actionButtonReturn"
        | "actionButtonMovie"
        | "actionButtonDocument"
        | "actionButtonSound"
        | "actionButtonHelp" => action_button(name, w, h),
        "roundRect" => {
            let v = a(0).clamp(0.0, 50000.0);
            let r = ss * v / 100000.0;
            let mut g = simple(corner_rect(w, h, [Corner::Round(r); 4]), full.inset(-r * 0.29289), w, h);
            g.handles.push(Handle::new(0, Point::ZERO, Vec2::new(ss / 100000.0, 0.0), v, 0.0, 50000.0));
            g
        }
        "flowChartAlternateProcess" => simple(corner_rect(w, h, [Corner::Round(ss / 6.0); 4]), full.inset(-ss / 6.0 * 0.29289), w, h),
        "snip1Rect" => {
            let v = a(0).clamp(0.0, 50000.0);
            let d = ss * v / 100000.0;
            let mut g = simple(
                corner_rect(w, h, [Corner::Square, Corner::Snip(d), Corner::Square, Corner::Square]),
                Rect::new(0.0, d / 2.0, w - d / 2.0, h),
                w,
                h,
            );
            g.handles.push(Handle::new(0, Point::new(w, 0.0), Vec2::new(-ss / 100000.0, 0.0), v, 0.0, 50000.0));
            g
        }
        "snip2SameRect" => {
            let (d1, d2) = (ss * a(0).clamp(0.0, 50000.0) / 1e5, ss * a(1).clamp(0.0, 50000.0) / 1e5);
            simple(corner_rect(w, h, [Corner::Snip(d1), Corner::Snip(d1), Corner::Snip(d2), Corner::Snip(d2)]), full.inset(-d1.max(d2) / 2.0), w, h)
        }
        "snip2DiagRect" => {
            let (d1, d2) = (ss * a(0).clamp(0.0, 50000.0) / 1e5, ss * a(1).clamp(0.0, 50000.0) / 1e5);
            simple(corner_rect(w, h, [Corner::Snip(d1), Corner::Snip(d2), Corner::Snip(d1), Corner::Snip(d2)]), full.inset(-d1.max(d2) / 2.0), w, h)
        }
        "snipRoundRect" => {
            let (r, d) = (ss * a(0).clamp(0.0, 50000.0) / 1e5, ss * a(1).clamp(0.0, 50000.0) / 1e5);
            simple(corner_rect(w, h, [Corner::Round(r), Corner::Snip(d), Corner::Square, Corner::Square]), full.inset(-r.max(d) / 2.0), w, h)
        }
        "round1Rect" => {
            let r = ss * a(0).clamp(0.0, 50000.0) / 1e5;
            simple(corner_rect(w, h, [Corner::Square, Corner::Round(r), Corner::Square, Corner::Square]), full.inset(-r * 0.29289), w, h)
        }
        "round2SameRect" => {
            let (r1, r2) = (ss * a(0).clamp(0.0, 50000.0) / 1e5, ss * a(1).clamp(0.0, 50000.0) / 1e5);
            simple(
                corner_rect(w, h, [Corner::Round(r1), Corner::Round(r1), Corner::Round(r2), Corner::Round(r2)]),
                full.inset(-r1.max(r2) * 0.29289),
                w,
                h,
            )
        }
        "round2DiagRect" => {
            let (r1, r2) = (ss * a(0).clamp(0.0, 50000.0) / 1e5, ss * a(1).clamp(0.0, 50000.0) / 1e5);
            simple(
                corner_rect(w, h, [Corner::Round(r1), Corner::Round(r2), Corner::Round(r1), Corner::Round(r2)]),
                full.inset(-r1.max(r2) * 0.29289),
                w,
                h,
            )
        }
        "ellipse" | "flowChartConnector" => {
            let mut g =
                simple(b.ellipse(w / 2.0, h / 2.0, w / 2.0, h / 2.0).take(), Rect::new(w * 0.14645, h * 0.14645, w * 0.85355, h * 0.85355), w, h);
            g.sites = ellipse_sites(w, h);
            g
        }
        "triangle" | "flowChartExtract" => {
            let v = if name == "triangle" { a(0).clamp(0.0, 100000.0) } else { 50000.0 };
            let ax = w * v / 100000.0;
            let mut g = simple(b.poly(&[(0.0, h), (ax, 0.0), (w, h)]).take(), Rect::new(ax / 2.0, h / 2.0, (ax + w) / 2.0, h), w, h);
            g.sites = vec![
                Point::new(ax, 0.0),
                Point::new(ax / 2.0, h / 2.0),
                Point::new(0.0, h),
                Point::new(w / 2.0, h),
                Point::new(w, h),
                Point::new((ax + w) / 2.0, h / 2.0),
            ];
            if name == "triangle" {
                g.handles.push(Handle::new(0, Point::ZERO, Vec2::new(w / 100000.0, 0.0), v, 0.0, 100000.0));
            }
            g
        }
        "flowChartMerge" => simple(b.poly(&[(0.0, 0.0), (w, 0.0), (w / 2.0, h)]).take(), Rect::new(w / 4.0, 0.0, w * 0.75, h / 2.0), w, h),
        "rtTriangle" => {
            simple(b.poly(&[(0.0, 0.0), (0.0, h), (w, h)]).take(), Rect::new(w / 12.0, h * 7.0 / 12.0, w * 7.0 / 12.0, h * 11.0 / 12.0), w, h)
        }
        "parallelogram" | "flowChartInputOutput" => {
            let v = if name == "parallelogram" { a(0).clamp(0.0, 100000.0 * w / ss.max(1e-9)) } else { 20000.0 * w / ss.max(1e-9) };
            let off = ss * v / 100000.0;
            let mut g = simple(
                b.poly(&[(0.0, h), (off, 0.0), (w, 0.0), (w - off, h)]).take(),
                Rect::new(off / 2.0 + off * 0.2, h * 0.08, w - off / 2.0 - off * 0.2, h * 0.92),
                w,
                h,
            );
            if name == "parallelogram" {
                g.handles.push(Handle::new(0, Point::ZERO, Vec2::new(ss / 100000.0, 0.0), v, 0.0, 100000.0 * w / ss.max(1e-9)));
            }
            g
        }
        "trapezoid" => {
            let v = a(0).clamp(0.0, 50000.0 * w / ss.max(1e-9));
            let off = ss * v / 100000.0;
            let mut g =
                simple(b.poly(&[(0.0, h), (off, 0.0), (w - off, 0.0), (w, h)]).take(), Rect::new(off * 0.66, h * 0.2, w - off * 0.66, h), w, h);
            g.handles.push(Handle::new(0, Point::ZERO, Vec2::new(ss / 100000.0, 0.0), v, 0.0, 50000.0 * w / ss.max(1e-9)));
            g
        }
        "flowChartManualOperation" => {
            simple(b.poly(&[(0.0, 0.0), (w, 0.0), (w * 0.8, h), (w * 0.2, h)]).take(), Rect::new(w * 0.2, 0.0, w * 0.8, h), w, h)
        }
        "diamond" | "flowChartDecision" => simple(
            b.poly(&[(w / 2.0, 0.0), (w, h / 2.0), (w / 2.0, h), (0.0, h / 2.0)]).take(),
            Rect::new(w / 4.0, h / 4.0, w * 0.75, h * 0.75),
            w,
            h,
        ),
        "flowChartSort" => {
            let d = b.poly(&[(w / 2.0, 0.0), (w, h / 2.0), (w / 2.0, h), (0.0, h / 2.0)]).take();
            let line = Pb::new().m(0.0, h / 2.0).l(w, h / 2.0).take();
            Geometry {
                paths: vec![fill(d), stroke_only(line)],
                text_rect: Rect::new(w / 4.0, h / 4.0, w * 0.75, h * 0.75),
                handles: vec![],
                sites: mid_sites(w, h),
            }
        }
        "pentagon" => simple(b.poly(&polygon_regular(5, w, h, -90.0)).take(), Rect::new(w * 0.2, h * 0.27, w * 0.8, h * 0.9), w, h),
        "heptagon" => simple(b.poly(&polygon_regular(7, w, h, -90.0)).take(), Rect::new(w * 0.16, h * 0.2, w * 0.84, h * 0.86), w, h),
        "decagon" => simple(b.poly(&polygon_regular(10, w, h, 0.0)).take(), Rect::new(w * 0.1, h * 0.17, w * 0.9, h * 0.83), w, h),
        "dodecagon" => simple(b.poly(&polygon_regular(12, w, h, 15.0)).take(), Rect::new(w * 0.13, h * 0.13, w * 0.87, h * 0.87), w, h),
        "hexagon" | "flowChartPreparation" => {
            let v = if name == "hexagon" { a(0).clamp(0.0, 50000.0 * w / ss.max(1e-9)) } else { 20000.0 * w / ss.max(1e-9) };
            let off = ss * v / 100000.0;
            let mut g = simple(
                b.poly(&[(0.0, h / 2.0), (off, 0.0), (w - off, 0.0), (w, h / 2.0), (w - off, h), (off, h)]).take(),
                Rect::new(off * 0.6, h * 0.14, w - off * 0.6, h * 0.86),
                w,
                h,
            );
            if name == "hexagon" {
                g.handles.push(Handle::new(0, Point::ZERO, Vec2::new(ss / 100000.0, 0.0), v, 0.0, 50000.0 * w / ss.max(1e-9)));
            }
            g
        }
        "octagon" => {
            let v = a(0).clamp(0.0, 50000.0);
            let d = ss * v / 100000.0;
            let mut g = simple(
                b.poly(&[(d, 0.0), (w - d, 0.0), (w, d), (w, h - d), (w - d, h), (d, h), (0.0, h - d), (0.0, d)]).take(),
                full.inset(-d / 2.0),
                w,
                h,
            );
            g.handles.push(Handle::new(0, Point::ZERO, Vec2::new(ss / 100000.0, 0.0), v, 0.0, 50000.0));
            g
        }
        "plus" => {
            let v = a(0).clamp(0.0, 50000.0);
            let d = ss * v / 100000.0;
            let mut g = simple(
                b.poly(&[
                    (d, 0.0),
                    (w - d, 0.0),
                    (w - d, d),
                    (w, d),
                    (w, h - d),
                    (w - d, h - d),
                    (w - d, h),
                    (d, h),
                    (d, h - d),
                    (0.0, h - d),
                    (0.0, d),
                    (d, d),
                ])
                .take(),
                Rect::new(0.0, d, w, h - d),
                w,
                h,
            );
            g.handles.push(Handle::new(0, Point::ZERO, Vec2::new(ss / 100000.0, 0.0), v, 0.0, 50000.0));
            g
        }
        "frame" => {
            let v = a(0).clamp(0.0, 50000.0);
            let t = ss * v / 100000.0;
            b.poly(&[(0.0, 0.0), (w, 0.0), (w, h), (0.0, h)]);
            b.poly(&[(t, t), (w - t, t), (w - t, h - t), (t, h - t)]);
            let mut g = Geometry { paths: vec![fill_eo(b.take())], text_rect: full.inset(-t), handles: vec![], sites: mid_sites(w, h) };
            g.handles.push(Handle::new(0, Point::ZERO, Vec2::new(ss / 100000.0, 0.0), v, 0.0, 50000.0));
            g
        }
        "halfFrame" => {
            let (t1, t2) = (ss * a(0).clamp(0.0, 50000.0) / 1e5, ss * a(1).clamp(0.0, 50000.0) / 1e5);
            simple(b.poly(&[(0.0, 0.0), (w, 0.0), (w - t1, t1), (t2, t1), (t2, h - t2), (0.0, h)]).take(), full, w, h)
        }
        "corner" => {
            let (t1, t2) = (ss * a(0).clamp(0.0, 100000.0) / 1e5, ss * a(1).clamp(0.0, 100000.0) / 1e5);
            simple(b.poly(&[(0.0, 0.0), (t2, 0.0), (t2, h - t1), (w, h - t1), (w, h), (0.0, h)]).take(), Rect::new(0.0, h - t1, w, h), w, h)
        }
        "diagStripe" => {
            let k = a(0).clamp(0.0, 100000.0) / 1e5;
            simple(b.poly(&[(0.0, h * k), (w * k, 0.0), (w, 0.0), (0.0, h)]).take(), Rect::new(0.0, 0.0, w / 2.0, h / 2.0), w, h)
        }
        "plaque" => {
            let v = a(0).clamp(0.0, 50000.0);
            let r = ss * v / 100000.0;
            b.m(0.0, r)
                .arc(r, r, 90.0, -90.0)
                .l(w - r, 0.0)
                .arc(r, r, 180.0, -90.0)
                .l(w, h - r)
                .arc(r, r, 270.0, -90.0)
                .l(r, h)
                .arc(r, r, 0.0, -90.0)
                .z();
            let mut g = simple(b.take(), full.inset(-r * 0.7), w, h);
            g.handles.push(Handle::new(0, Point::ZERO, Vec2::new(ss / 100000.0, 0.0), v, 0.0, 50000.0));
            g
        }
        "can" | "flowChartMagneticDisk" => {
            let v = if name == "can" { a(0).clamp(0.0, 50000.0 * h / ss.max(1e-9)) } else { 33333.0 * h / ss.max(1e-9) / 2.0 };
            let eh = ss * v / 100000.0;
            let r = eh / 2.0;
            let body = Pb::new().m(0.0, r).l(0.0, h - r).arc(w / 2.0, r, 180.0, -180.0).l(w, r).arc(w / 2.0, r, 0.0, -180.0).z().take();
            let top = Pb::new().ellipse(w / 2.0, r, w / 2.0, r).take();
            let mut g = Geometry {
                paths: vec![fill(body), shade(top, if name == "can" { FillMode::Lighten } else { FillMode::Norm })],
                text_rect: Rect::new(0.0, eh, w, h - r),
                handles: vec![],
                sites: mid_sites(w, h),
            };
            if name == "can" {
                g.handles.push(Handle::new(0, Point::new(w / 2.0, 0.0), Vec2::new(0.0, ss / 100000.0), v, 0.0, 50000.0 * h / ss.max(1e-9)));
            }
            g
        }
        "flowChartMagneticDrum" => {
            let r = w / 6.0 / 2.0;
            let body = Pb::new().m(r, 0.0).l(w - r, 0.0).arc(r, h / 2.0, 270.0, 180.0).l(r, h).arc(r, h / 2.0, 90.0, 180.0).z().take();
            let side = Pb::new().m(w - r, 0.0).arc(r, h / 2.0, 270.0, -180.0).take();
            Geometry {
                paths: vec![fill(body), stroke_only(side)],
                text_rect: Rect::new(r, 0.0, w - 2.0 * r, h),
                handles: vec![],
                sites: mid_sites(w, h),
            }
        }
        "cube" => {
            let v = a(0).clamp(0.0, 100000.0);
            let d = ss * v / 100000.0;
            let front = Pb::new().poly(&[(0.0, d), (w - d, d), (w - d, h), (0.0, h)]).take();
            let top = Pb::new().poly(&[(0.0, d), (d, 0.0), (w, 0.0), (w - d, d)]).take();
            let side = Pb::new().poly(&[(w - d, d), (w, 0.0), (w, h - d), (w - d, h)]).take();
            let mut g = Geometry {
                paths: vec![fill(front), shade(top, FillMode::LightenLess), shade(side, FillMode::DarkenLess)],
                text_rect: Rect::new(0.0, d, w - d, h),
                handles: vec![],
                sites: mid_sites(w, h),
            };
            g.handles.push(Handle::new(0, Point::new(0.0, 0.0), Vec2::new(0.0, ss / 100000.0), v, 0.0, 100000.0));
            g
        }
        "bevel" => {
            let v = a(0).clamp(0.0, 50000.0);
            let d = ss * v / 100000.0;
            let inner = Pb::new().poly(&[(d, d), (w - d, d), (w - d, h - d), (d, h - d)]).take();
            let top = Pb::new().poly(&[(0.0, 0.0), (w, 0.0), (w - d, d), (d, d)]).take();
            let left = Pb::new().poly(&[(0.0, 0.0), (d, d), (d, h - d), (0.0, h)]).take();
            let right = Pb::new().poly(&[(w, 0.0), (w, h), (w - d, h - d), (w - d, d)]).take();
            let bottom = Pb::new().poly(&[(0.0, h), (d, h - d), (w - d, h - d), (w, h)]).take();
            let mut g = Geometry {
                paths: vec![
                    fill(inner),
                    shade(top, FillMode::LightenLess),
                    shade(left, FillMode::Lighten),
                    shade(right, FillMode::DarkenLess),
                    shade(bottom, FillMode::Darken),
                ],
                text_rect: full.inset(-d),
                handles: vec![],
                sites: mid_sites(w, h),
            };
            g.handles.push(Handle::new(0, Point::ZERO, Vec2::new(ss / 100000.0, 0.0), v, 0.0, 50000.0));
            g
        }
        "donut" => {
            let v = a(0).clamp(0.0, 50000.0);
            let t = ss * v / 100000.0;
            b.ellipse(w / 2.0, h / 2.0, w / 2.0, h / 2.0);
            b.ellipse(w / 2.0, h / 2.0, (w / 2.0 - t).max(0.0), (h / 2.0 - t).max(0.0));
            let mut g = Geometry {
                paths: vec![fill_eo(b.take())],
                text_rect: Rect::new(w * 0.14645, h * 0.14645, w * 0.85355, h * 0.85355),
                handles: vec![],
                sites: ellipse_sites(w, h),
            };
            g.handles.push(Handle::new(0, Point::new(0.0, h / 2.0), Vec2::new(ss / 100000.0, 0.0), v, 0.0, 50000.0));
            g
        }
        "noSmoking" => {
            let v = a(0).clamp(0.0, 50000.0);
            let t = ss * v / 100000.0;
            let (rx, ry) = ((w / 2.0 - t).max(0.0), (h / 2.0 - t).max(0.0));
            b.ellipse(w / 2.0, h / 2.0, w / 2.0, h / 2.0);
            b.ellipse(w / 2.0, h / 2.0, rx, ry);
            let ring = b.take();
            let th = (h).atan2(w);
            let dir = Vec2::new(th.cos(), th.sin());
            let r = if rx > 0.0 && ry > 0.0 { rx * ry / ((ry * th.cos()).powi(2) + (rx * th.sin()).powi(2)).sqrt() } else { 0.0 };
            let perp = Vec2::new(-dir.y, dir.x) * (t / 2.0);
            let c = Point::new(w / 2.0, h / 2.0);
            let p = [c - dir * r - perp, c + dir * r - perp, c + dir * r + perp, c - dir * r + perp];
            let bar = Pb::new().poly(&p.map(|q| (q.x, q.y))).take();
            Geometry {
                paths: vec![fill_eo(ring), fill(bar)],
                text_rect: Rect::new(w * 0.14645, h * 0.14645, w * 0.85355, h * 0.85355),
                handles: vec![Handle::new(0, Point::new(0.0, h / 2.0), Vec2::new(ss / 100000.0, 0.0), v, 0.0, 50000.0)],
                sites: ellipse_sites(w, h),
            }
        }
        "blockArc" => {
            let st = a(0) / 60000.0;
            let en = a(1) / 60000.0;
            let t = ss * a(2).clamp(0.0, 50000.0) / 100000.0;
            let mut sw = en - st;
            if sw <= 0.0 {
                sw += 360.0;
            }
            let (rx, ry) = (w / 2.0, h / 2.0);
            let (irx, iry) = ((rx - t).max(0.0), (ry - t).max(0.0));
            let s0 = vis(st, rx, ry);
            let e0 = vis(st + sw, rx, ry);
            let mut sweep = e0 - s0;
            if sweep <= 0.0 {
                sweep += 360.0;
            }
            let si = vis(st + sw, irx, iry);
            let ei = vis(st, irx, iry);
            let mut isweep = ei - si;
            if isweep >= 0.0 {
                isweep -= 360.0;
            }
            let p0 = Point::new(w / 2.0 + rx * s0.to_radians().cos(), h / 2.0 + ry * s0.to_radians().sin());
            b.m(p0.x, p0.y).arc(rx, ry, s0, sweep);
            let pi = Point::new(w / 2.0 + irx * si.to_radians().cos(), h / 2.0 + iry * si.to_radians().sin());
            b.l(pi.x, pi.y).arc(irx, iry, si, isweep).z();
            simple(b.take(), full, w, h)
        }
        "pie" | "chord" | "arc" => {
            let (rx, ry) = (w / 2.0, h / 2.0);
            let s = vis(a(0) / 60000.0, rx, ry);
            let e = vis(a(1) / 60000.0, rx, ry);
            let mut sw = e - s;
            if sw <= 0.0 {
                sw += 360.0;
            }
            let p0 = Point::new(rx + rx * s.to_radians().cos(), ry + ry * s.to_radians().sin());
            b.m(p0.x, p0.y).arc(rx, ry, s, sw);
            let text = Rect::new(w * 0.14645, h * 0.14645, w * 0.85355, h * 0.85355);
            match name {
                "pie" => {
                    b.l(rx, ry).z();
                    simple(b.take(), text, w, h)
                }
                "chord" => {
                    b.z();
                    simple(b.take(), text, w, h)
                }
                _ => {
                    let open = b.take();
                    let mut f = BezPath::from_vec(open.elements().to_vec());
                    f.line_to(Point::new(rx, ry));
                    f.close_path();
                    Geometry { paths: vec![fill_only(f), stroke_only(open)], text_rect: text, handles: vec![], sites: mid_sites(w, h) }
                }
            }
        }
        "teardrop" => {
            let k = a(0).clamp(0.0, 200000.0) / 100000.0;
            let tip = (w / 2.0 + w / 2.0 * k, h / 2.0 - h / 2.0 * k);
            b.m(0.0, h / 2.0).arc(w / 2.0, h / 2.0, 180.0, 90.0);
            b.q(w / 2.0 + w / 4.0 * k, 0.0, tip.0, tip.1).q(w, h / 2.0 - h / 4.0 * k, w, h / 2.0);
            b.arc(w / 2.0, h / 2.0, 0.0, 180.0).z();
            simple(b.take(), Rect::new(w * 0.14645, h * 0.14645, w * 0.85355, h * 0.85355), w, h)
        }
        "foldedCorner" => {
            let v = a(0).clamp(0.0, 50000.0);
            let d = ss * v / 100000.0;
            let body = Pb::new().poly(&[(0.0, 0.0), (w, 0.0), (w, h - d), (w - d, h), (0.0, h)]).take();
            let fold = Pb::new().m(w - d, h).l(w - d + d * 0.2, h - d + d * 0.2).l(w, h - d).z().take();
            Geometry {
                paths: vec![fill(body), shade(fold, FillMode::DarkenLess)],
                text_rect: Rect::new(0.0, 0.0, w, h - d),
                handles: vec![],
                sites: mid_sites(w, h),
            }
        }
        "smileyFace" => {
            let k = a(0).clamp(-4653.0, 4653.0) / 100000.0;
            let face = Pb::new().ellipse(w / 2.0, h / 2.0, w / 2.0, h / 2.0).take();
            let mut eyes = Pb::new();
            eyes.ellipse(w * 0.35, h * 0.37, w * 0.05, h * 0.05);
            eyes.ellipse(w * 0.65, h * 0.37, w * 0.05, h * 0.05);
            let ys = h * (0.72 - k);
            let smile = Pb::new().m(w * 0.22, ys).q(w / 2.0, ys + h * k * 4.0 + h * 0.0, w * 0.78, ys).take();
            Geometry {
                paths: vec![fill(face), shade(eyes.take(), FillMode::DarkenLess), stroke_only(smile)],
                text_rect: Rect::new(w * 0.14645, h * 0.14645, w * 0.85355, h * 0.85355),
                handles: vec![],
                sites: ellipse_sites(w, h),
            }
        }
        "heart" => {
            b.m(w / 2.0, h * 0.25);
            b.c(w * 0.62, h * -0.02, w, h * 0.0, w, h * 0.32);
            b.c(w, h * 0.6, w * 0.62, h * 0.78, w / 2.0, h);
            b.c(w * 0.38, h * 0.78, 0.0, h * 0.6, 0.0, h * 0.32);
            b.c(0.0, h * 0.0, w * 0.38, h * -0.02, w / 2.0, h * 0.25);
            b.z();
            simple(b.take(), Rect::new(w * 0.2, h * 0.25, w * 0.8, h * 0.7), w, h)
        }
        "lightningBolt" => {
            let pts = normalize(&[(0.36, 0.0), (0.68, 0.0), (0.53, 0.34), (0.79, 0.34), (0.24, 1.0), (0.39, 0.53), (0.15, 0.53)], w, h);
            simple(b.poly(&pts).take(), Rect::new(w * 0.3, h * 0.35, w * 0.65, h * 0.6), w, h)
        }
        "sun" => {
            let k = a(0).clamp(12500.0, 46875.0) / 100000.0;
            let (cx, cy) = (w / 2.0, h / 2.0);
            let cr = 0.5 - k;
            let core = Pb::new().ellipse(cx, cy, w * cr * 0.72, h * cr * 0.72).take();
            let mut rays = Pb::new();
            for i in 0..8 {
                let ang = (i as f64 * 45.0).to_radians();
                let half = 9f64.to_radians();
                let rb = cr * 0.72 + 0.06;
                let tip = (cx + w * 0.5 * ang.cos(), cy + h * 0.5 * ang.sin());
                let b1 = (cx + w * rb * (ang - half).cos(), cy + h * rb * (ang - half).sin());
                let b2 = (cx + w * rb * (ang + half).cos(), cy + h * rb * (ang + half).sin());
                rays.poly(&[tip, b1, b2]);
            }
            Geometry {
                paths: vec![fill(rays.take()), fill(core)],
                text_rect: Rect::new(w * 0.3, h * 0.3, w * 0.7, h * 0.7),
                handles: vec![],
                sites: mid_sites(w, h),
            }
        }
        "moon" => {
            let k = a(0).clamp(0.0, 87500.0) / 100000.0;
            let g0 = w * k;
            b.m(w, 0.0).arc(w, h / 2.0, 270.0, -180.0).arc((w - g0).max(0.0), h / 2.0, 90.0, 180.0).z();
            simple(b.take(), Rect::new(w * 0.1, h * 0.25, w * (1.0 - k).max(0.2) * 0.8, h * 0.75), w, h)
        }
        "cloud" | "cloudCallout" => cloud(name, w, h, a(0), a(1)),
        "bracketPair" => {
            let r = ss * a(0).clamp(0.0, 50000.0) / 1e5;
            let mut l = Pb::new();
            l.m(r, h).arc(r, r, 90.0, 90.0).l(0.0, r).arc(r, r, 180.0, 90.0);
            l.m(w - r, 0.0).arc(r, r, 270.0, 90.0).l(w, h - r).arc(r, r, 0.0, 90.0);
            let f = corner_rect(w, h, [Corner::Round(r); 4]);
            Geometry {
                paths: vec![fill_only(f), stroke_only(l.take())],
                text_rect: full.inset(-r * 0.29289),
                handles: vec![],
                sites: mid_sites(w, h),
            }
        }
        "bracePair" => {
            let r = ss * a(0).clamp(0.0, 25000.0) / 1e5;
            let mut l = Pb::new();
            l.m(2.0 * r, h).arc(r, r, 90.0, 90.0).l(r, h / 2.0 + r).arc(r, r, 0.0, -90.0).arc(r, r, 90.0, -90.0).l(r, r).arc(r, r, 180.0, 90.0);
            l.m(w - 2.0 * r, 0.0)
                .arc(r, r, 270.0, 90.0)
                .l(w - r, h / 2.0 - r)
                .arc(r, r, 180.0, -90.0)
                .arc(r, r, 270.0, -90.0)
                .l(w - r, h - r)
                .arc(r, r, 0.0, 90.0);
            let f = corner_rect(w, h, [Corner::Round(r); 4]);
            Geometry { paths: vec![fill_only(f), stroke_only(l.take())], text_rect: full.inset(-r * 1.2), handles: vec![], sites: mid_sites(w, h) }
        }
        "leftBracket" | "rightBracket" => {
            let r = (ss * a(0).clamp(0.0, 50000.0 * h / ss.max(1e-9)) / 1e5).min(h / 2.0);
            let mut l = Pb::new();
            l.m(w, h).arc(w, r, 90.0, 90.0).l(0.0, r).arc(w, r, 180.0, 90.0);
            let mut path = l.take();
            if name == "rightBracket" {
                path.apply_affine(kurbo::Affine::new([-1.0, 0.0, 0.0, 1.0, w, 0.0]));
            }
            Geometry { paths: vec![stroke_only(path)], text_rect: full, handles: vec![], sites: mid_sites(w, h) }
        }
        "leftBrace" | "rightBrace" => {
            let r = (ss * a(0).clamp(0.0, 25000.0 * h / ss.max(1e-9)) / 1e5).min(h / 4.0);
            let my = h * a(1).clamp(0.0, 100000.0) / 1e5;
            let hw = w / 2.0;
            let mut l = Pb::new();
            l.m(w, h).arc(hw, r, 90.0, 90.0).l(hw, my + r).arc(hw, r, 0.0, -90.0).arc(hw, r, 90.0, -90.0).l(hw, r).arc(hw, r, 180.0, 90.0);
            let mut path = l.take();
            if name == "rightBrace" {
                path.apply_affine(kurbo::Affine::new([-1.0, 0.0, 0.0, 1.0, w, 0.0]));
            }
            Geometry { paths: vec![stroke_only(path)], text_rect: full, handles: vec![], sites: mid_sites(w, h) }
        }
        "rightArrow" => arrow(Dir::Right, w, h, a(0), a(1)),
        "leftArrow" => arrow(Dir::Left, w, h, a(0), a(1)),
        "upArrow" => arrow(Dir::Up, w, h, a(0), a(1)),
        "downArrow" => arrow(Dir::Down, w, h, a(0), a(1)),
        "leftRightArrow" => double_arrow(false, w, h, a(0), a(1)),
        "upDownArrow" => double_arrow(true, w, h, a(0), a(1)),
        "notchedRightArrow" | "stripedRightArrow" => {
            let mut g = arrow(Dir::Right, w, h, a(0), a(1));
            let k = a(0).clamp(0.0, 100000.0) / 1e5;
            let (y1, y2) = (h / 2.0 - h * k / 2.0, h / 2.0 + h * k / 2.0);
            let dx = (ss * a(1).max(0.0) / 1e5).min(w);
            if name == "notchedRightArrow" {
                let n = dx * (k / 2.0) * 0.9;
                let pts = [(0.0, y1), (w - dx, y1), (w - dx, 0.0), (w, h / 2.0), (w - dx, h), (w - dx, y2), (0.0, y2), (n, h / 2.0)];
                g.paths = vec![fill(Pb::new().poly(&pts).take())];
            } else {
                let s = ss / 32.0;
                let start = 5.0 * s;
                let pts = [(start, y1), (w - dx, y1), (w - dx, 0.0), (w, h / 2.0), (w - dx, h), (w - dx, y2), (start, y2)];
                let mut stripes = Pb::new();
                stripes.poly(&[(0.0, y1), (s, y1), (s, y2), (0.0, y2)]);
                stripes.poly(&[(2.0 * s, y1), (4.0 * s, y1), (4.0 * s, y2), (2.0 * s, y2)]);
                g.paths = vec![fill(stripes.take()), fill(Pb::new().poly(&pts).take())];
            }
            g
        }
        "homePlate" | "chevron" => {
            let v = a(0).clamp(0.0, 100000.0 * w / ss.max(1e-9));
            let d = (ss * v / 1e5).min(w);
            let pts: Vec<(f64, f64)> = if name == "homePlate" {
                vec![(0.0, 0.0), (w - d, 0.0), (w, h / 2.0), (w - d, h), (0.0, h)]
            } else {
                vec![(0.0, 0.0), (w - d, 0.0), (w, h / 2.0), (w - d, h), (0.0, h), (d, h / 2.0)]
            };
            let tx = if name == "homePlate" { 0.0 } else { d };
            let mut g = simple(b.poly(&pts).take(), Rect::new(tx, 0.0, w - d / 2.0, h), w, h);
            g.handles.push(Handle::new(0, Point::new(w, 0.0), Vec2::new(-ss / 1e5, 0.0), v, 0.0, 100000.0 * w / ss.max(1e-9)));
            g
        }
        "quadArrow" => {
            let (t, hw, hl) = (ss * a(0).clamp(0.0, 50000.0) / 1e5, ss * a(1).clamp(0.0, 50000.0) / 1e5, ss * a(2).clamp(0.0, 50000.0) / 1e5);
            let (cx, cy) = (w / 2.0, h / 2.0);
            let (t2, hw2) = (t / 2.0, hw);
            let pts = [
                (0.0, cy),
                (hl, cy - hw2),
                (hl, cy - t2),
                (cx - t2, cy - t2),
                (cx - t2, hl),
                (cx - hw2, hl),
                (cx, 0.0),
                (cx + hw2, hl),
                (cx + t2, hl),
                (cx + t2, cy - t2),
                (w - hl, cy - t2),
                (w - hl, cy - hw2),
                (w, cy),
                (w - hl, cy + hw2),
                (w - hl, cy + t2),
                (cx + t2, cy + t2),
                (cx + t2, h - hl),
                (cx + hw2, h - hl),
                (cx, h),
                (cx - hw2, h - hl),
                (cx - t2, h - hl),
                (cx - t2, cy + t2),
                (hl, cy + t2),
                (hl, cy + hw2),
            ];
            simple(b.poly(&pts).take(), Rect::new(cx - t2, cy - t2, cx + t2, cy + t2), w, h)
        }
        "leftRightUpArrow" => {
            let (t, hw, hl) = (ss * a(0).clamp(0.0, 50000.0) / 1e5, ss * a(1).clamp(0.0, 50000.0) / 1e5, ss * a(2).clamp(0.0, 50000.0) / 1e5);
            let cx = w / 2.0;
            let by = h - hw; // horizontal shaft centre
            let pts = [
                (0.0, by),
                (hl, by - hw),
                (hl, by - t / 2.0),
                (cx - t / 2.0, by - t / 2.0),
                (cx - t / 2.0, hl),
                (cx - hw, hl),
                (cx, 0.0),
                (cx + hw, hl),
                (cx + t / 2.0, hl),
                (cx + t / 2.0, by - t / 2.0),
                (w - hl, by - t / 2.0),
                (w - hl, by - hw),
                (w, by),
                (w - hl, h),
                (w - hl, by + t / 2.0),
                (hl, by + t / 2.0),
                (hl, h),
            ];
            simple(b.poly(&pts).take(), Rect::new(hl, by - t / 2.0, w - hl, by + t / 2.0), w, h)
        }
        "leftUpArrow" | "bentUpArrow" => {
            let (t, hw, hl) = (ss * a(0).clamp(0.0, 50000.0) / 1e5, ss * a(1).clamp(0.0, 50000.0) / 1e5, ss * a(2).clamp(0.0, 50000.0) / 1e5);
            let pts: Vec<(f64, f64)> = if name == "leftUpArrow" {
                let cx = w - hw;
                let cy = h - hw;
                vec![
                    (0.0, cy),
                    (hl, cy - hw),
                    (hl, cy - t / 2.0),
                    (cx - t / 2.0, cy - t / 2.0),
                    (cx - t / 2.0, hl),
                    (cx - hw, hl),
                    (cx, 0.0),
                    (w, hl),
                    (cx + t / 2.0, hl),
                    (cx + t / 2.0, cy + t / 2.0),
                    (hl, cy + t / 2.0),
                    (hl, h),
                ]
            } else {
                let cx = w - hw;
                vec![
                    (0.0, h - t),
                    (cx - t / 2.0, h - t),
                    (cx - t / 2.0, hl),
                    (cx - hw, hl),
                    (cx, 0.0),
                    (w, hl),
                    (cx + t / 2.0, hl),
                    (cx + t / 2.0, h),
                    (0.0, h),
                ]
            };
            simple(b.poly(&pts).take(), full, w, h)
        }
        "bentArrow" => {
            let (t, hw, hl, r) = (
                ss * a(0).clamp(0.0, 50000.0) / 1e5,
                ss * a(1).clamp(0.0, 50000.0) / 1e5,
                ss * a(2).clamp(0.0, 50000.0) / 1e5,
                ss * a(3).clamp(0.0, 100000.0) / 1e5,
            );
            let cy = hw; // head centre line
            let ro = r.max(t);
            let ri = (ro - t).max(0.0);
            b.m(0.0, h)
                .l(0.0, cy - t / 2.0 + ro)
                .arc(ro, ro, 180.0, 90.0)
                .l(w - hl, cy - t / 2.0)
                .l(w - hl, 0.0)
                .l(w, cy)
                .l(w - hl, 2.0 * hw)
                .l(w - hl, cy + t / 2.0);
            b.l(t + ri, cy + t / 2.0);
            if ri > 0.0 {
                b.arc(ri, ri, 270.0, -90.0);
            }
            b.l(t, h).z();
            simple(b.take(), full, w, h)
        }
        "uturnArrow" => {
            let (t, hw, hl, r) = (
                ss * a(0).clamp(0.0, 25000.0) / 1e5,
                ss * a(1).clamp(0.0, 25000.0) / 1e5,
                ss * a(2).clamp(0.0, 25000.0) / 1e5,
                ss * a(3).clamp(0.0, 100000.0) / 1e5,
            );
            let hy = (h * a(4).clamp(0.0, 100000.0) / 1e5).max(hl + t);
            let ax = w - hw; // arrow shaft centre x
            let ro = r.max(t).min((ax + t / 2.0) / 2.0);
            let ri = (ro - t).max(0.0);
            b.m(0.0, h).l(0.0, ro).arc(ro, ro, 180.0, 90.0).l(ax + t / 2.0 - ro, 0.0).arc(ro, ro, 270.0, 90.0);
            b.l(ax + t / 2.0, hy - hl).l(w, hy - hl).l(ax, hy).l(ax - hw, hy - hl).l(ax - t / 2.0, hy - hl).l(ax - t / 2.0, ro);
            if ri > 0.0 {
                b.arc(ri, ri, 0.0, -90.0);
            }
            b.l(t + ri, t);
            if ri > 0.0 {
                b.arc(ri, ri, 270.0, -90.0);
            }
            b.l(t, h).z();
            simple(b.take(), full, w, h)
        }
        "rightArrowCallout" | "leftArrowCallout" | "upArrowCallout" | "downArrowCallout" => {
            let d = match name {
                "rightArrowCallout" => Dir::Right,
                "leftArrowCallout" => Dir::Left,
                "upArrowCallout" => Dir::Up,
                _ => Dir::Down,
            };
            let (l, t) = len_thick(d, w, h);
            let ss2 = l.min(t);
            let shaft = ss2 * a(0).clamp(0.0, 100000.0) / 1e5;
            let head = ss2 * a(1).clamp(0.0, 100000.0) / 1e5;
            let hl = ss2 * a(2).clamp(0.0, 100000.0) / 1e5;
            let boxl = (l * a(3).clamp(0.0, 100000.0) / 1e5).min(l - hl);
            let c = t / 2.0;
            let pts = [
                (0.0, 0.0),
                (boxl, 0.0),
                (boxl, c - shaft / 2.0),
                (l - hl, c - shaft / 2.0),
                (l - hl, c - head),
                (l, c),
                (l - hl, c + head),
                (l - hl, c + shaft / 2.0),
                (boxl, c + shaft / 2.0),
                (boxl, t),
                (0.0, t),
            ];
            let pts = orient(d, w, h, &pts);
            let tr = orient(d, w, h, &[(0.0, 0.0), (boxl, t)]);
            simple(b.poly(&pts).take(), Rect::from_points(Point::new(tr[0].0, tr[0].1), Point::new(tr[1].0, tr[1].1)).abs(), w, h)
        }
        "mathPlus" | "mathMinus" | "mathEqual" | "mathDivide" | "mathMultiply" | "mathNotEqual" => math(name, w, h, a(0), a(1), a(2)),
        "flowChartPredefinedProcess" => {
            let r = Pb::new().poly(&[(0.0, 0.0), (w, 0.0), (w, h), (0.0, h)]).take();
            let mut l = Pb::new();
            l.m(w / 8.0, 0.0).l(w / 8.0, h).m(w * 7.0 / 8.0, 0.0).l(w * 7.0 / 8.0, h);
            Geometry {
                paths: vec![fill(r), stroke_only(l.take())],
                text_rect: Rect::new(w / 8.0, 0.0, w * 7.0 / 8.0, h),
                handles: vec![],
                sites: mid_sites(w, h),
            }
        }
        "flowChartInternalStorage" => {
            let r = Pb::new().poly(&[(0.0, 0.0), (w, 0.0), (w, h), (0.0, h)]).take();
            let mut l = Pb::new();
            l.m(w / 8.0, 0.0).l(w / 8.0, h).m(0.0, h / 8.0).l(w, h / 8.0);
            Geometry {
                paths: vec![fill(r), stroke_only(l.take())],
                text_rect: Rect::new(w / 8.0, h / 8.0, w, h),
                handles: vec![],
                sites: mid_sites(w, h),
            }
        }
        "flowChartDocument" => {
            b.m(0.0, 0.0).l(w, 0.0).l(w, h * 0.8).c(w * 0.7, h * 0.62, w * 0.35, h * 1.08, 0.0, h * 0.88).z();
            simple(b.take(), Rect::new(0.0, 0.0, w, h * 0.78), w, h)
        }
        "flowChartMultidocument" => {
            let (dx, dy) = (w * 0.1, h * 0.1);
            let mut back = Pb::new();
            back.poly(&[(2.0 * dx, 0.0), (w, 0.0), (w, h * 0.7), (w - dx * 0.5, h * 0.7), (w - dx * 0.5, dy * 0.5), (2.0 * dx, dy * 0.5)]);
            back.poly(&[(dx, dy * 0.5), (w - dx * 0.5, dy * 0.5), (w - dx * 0.5, h * 0.78), (w - dx, h * 0.78), (w - dx, dy), (dx, dy)]);
            let front = Pb::new().m(0.0, dy).l(w - dx, dy).l(w - dx, h * 0.84).c(w * 0.62, h * 0.7, w * 0.32, h * 1.08, 0.0, h * 0.9).z().take();
            Geometry {
                paths: vec![fill(back.take()), fill(front)],
                text_rect: Rect::new(0.0, dy, w - dx, h * 0.8),
                handles: vec![],
                sites: mid_sites(w, h),
            }
        }
        "flowChartTerminator" => {
            let rx = (w * 0.16).min(w / 2.0);
            b.m(rx, 0.0).l(w - rx, 0.0).arc(rx, h / 2.0, 270.0, 180.0).l(rx, h).arc(rx, h / 2.0, 90.0, 180.0).z();
            simple(b.take(), Rect::new(rx * 0.3, h * 0.15, w - rx * 0.3, h * 0.85), w, h)
        }
        "flowChartManualInput" => simple(b.poly(&[(0.0, h / 5.0), (w, 0.0), (w, h), (0.0, h)]).take(), Rect::new(0.0, h / 5.0, w, h), w, h),
        "flowChartOffpageConnector" => {
            simple(b.poly(&[(0.0, 0.0), (w, 0.0), (w, h * 0.8), (w / 2.0, h), (0.0, h * 0.8)]).take(), Rect::new(0.0, 0.0, w, h * 0.8), w, h)
        }
        "flowChartPunchedCard" => {
            simple(b.poly(&[(w * 0.2, 0.0), (w, 0.0), (w, h), (0.0, h), (0.0, h * 0.2)]).take(), Rect::new(0.0, h * 0.2, w, h), w, h)
        }
        "flowChartPunchedTape" => {
            let a1 = h * 0.1;
            b.m(0.0, a1).c(w * 0.25, a1 * 3.0, w * 0.25, -a1, w / 2.0, a1).c(w * 0.75, a1 * 3.0, w * 0.75, -a1, w, a1).l(w, h - a1);
            b.c(w * 0.75, h - 3.0 * a1, w * 0.75, h + a1, w / 2.0, h - a1).c(w * 0.25, h - 3.0 * a1, w * 0.25, h + a1, 0.0, h - a1).z();
            simple(b.take(), Rect::new(0.0, h * 0.2, w, h * 0.8), w, h)
        }
        "flowChartSummingJunction" | "flowChartOr" => {
            let e = Pb::new().ellipse(w / 2.0, h / 2.0, w / 2.0, h / 2.0).take();
            let mut l = Pb::new();
            if name == "flowChartOr" {
                l.m(w / 2.0, 0.0).l(w / 2.0, h).m(0.0, h / 2.0).l(w, h / 2.0);
            } else {
                let (dx, dy) = (w / 2.0 * std::f64::consts::FRAC_1_SQRT_2, h / 2.0 * std::f64::consts::FRAC_1_SQRT_2);
                l.m(w / 2.0 - dx, h / 2.0 - dy).l(w / 2.0 + dx, h / 2.0 + dy).m(w / 2.0 + dx, h / 2.0 - dy).l(w / 2.0 - dx, h / 2.0 + dy);
            }
            Geometry {
                paths: vec![fill(e), stroke_only(l.take())],
                text_rect: Rect::new(w * 0.14645, h * 0.14645, w * 0.85355, h * 0.85355),
                handles: vec![],
                sites: ellipse_sites(w, h),
            }
        }
        "flowChartCollate" => simple(b.poly(&[(0.0, 0.0), (w, 0.0), (0.0, h), (w, h)]).take(), Rect::new(w / 4.0, h / 4.0, w * 0.75, h * 0.75), w, h),
        "flowChartOnlineStorage" => {
            let rx = w / 6.0;
            b.m(rx, 0.0).l(w, 0.0).arc(rx, h / 2.0, 270.0, -180.0).l(rx, h).arc(rx, h / 2.0, 90.0, 180.0).z();
            simple(b.take(), Rect::new(rx, 0.0, w - rx, h), w, h)
        }
        "flowChartDelay" => {
            b.m(0.0, 0.0).l(w / 2.0, 0.0).arc(w / 2.0, h / 2.0, 270.0, 180.0).l(0.0, h).z();
            simple(b.take(), Rect::new(0.0, h * 0.15, w * 0.85, h * 0.85), w, h)
        }
        "flowChartMagneticTape" => {
            b.m(w / 2.0, h).arc(w / 2.0, h / 2.0, 90.0, 90.0).arc(w / 2.0, h / 2.0, 180.0, 90.0).arc(w / 2.0, h / 2.0, 270.0, 90.0).arc(
                w / 2.0,
                h / 2.0,
                0.0,
                45.0,
            );
            let p = b.cur;
            b.l(w, p.y).l(w, h).z();
            simple(b.take(), Rect::new(w * 0.14645, h * 0.14645, w * 0.85355, h * 0.85355), w, h)
        }
        "flowChartDisplay" => {
            let rx = w / 6.0;
            b.m(0.0, h / 2.0).l(rx, 0.0).l(w - rx, 0.0).arc(rx, h / 2.0, 270.0, 180.0).l(rx, h).z();
            simple(b.take(), Rect::new(rx, 0.0, w - rx, h), w, h)
        }
        "star4" | "star5" | "star6" | "star7" | "star8" | "star10" | "star12" | "star16" | "star24" | "star32" => {
            let n: usize = name.trim_start_matches("star").parse().unwrap_or(5);
            let v = a(0).clamp(0.0, 50000.0);
            let inner = v / 50000.0;
            let normalized = matches!(n, 5 | 7);
            let mut g = simple(b.poly(&star(n, w, h, inner, normalized)).take(), full.inset(-ss * (0.5 - inner * 0.5) * 0.5), w, h);
            g.handles.push(Handle::new(0, Point::new(w / 2.0, h / 2.0), Vec2::new(0.0, -h / 2.0 / 50000.0), v, 0.0, 50000.0));
            g
        }
        "irregularSeal1" | "irregularSeal2" => {
            let pts: &[(f64, f64)] = if name == "irregularSeal1" {
                &[
                    (0.48, 0.18),
                    (0.62, 0.0),
                    (0.64, 0.2),
                    (0.86, 0.1),
                    (0.78, 0.3),
                    (1.0, 0.36),
                    (0.82, 0.5),
                    (0.94, 0.72),
                    (0.7, 0.66),
                    (0.68, 0.9),
                    (0.54, 0.72),
                    (0.42, 1.0),
                    (0.36, 0.74),
                    (0.14, 0.86),
                    (0.22, 0.62),
                    (0.0, 0.6),
                    (0.18, 0.44),
                    (0.04, 0.24),
                    (0.3, 0.3),
                    (0.26, 0.06),
                ]
            } else {
                &[
                    (0.5, 0.16),
                    (0.58, 0.0),
                    (0.63, 0.18),
                    (0.78, 0.06),
                    (0.76, 0.24),
                    (0.96, 0.2),
                    (0.86, 0.38),
                    (1.0, 0.5),
                    (0.84, 0.58),
                    (0.92, 0.76),
                    (0.74, 0.72),
                    (0.76, 0.94),
                    (0.6, 0.8),
                    (0.5, 1.0),
                    (0.44, 0.8),
                    (0.28, 0.94),
                    (0.28, 0.74),
                    (0.08, 0.82),
                    (0.16, 0.62),
                    (0.0, 0.54),
                    (0.14, 0.44),
                    (0.02, 0.28),
                    (0.22, 0.3),
                    (0.2, 0.1),
                    (0.36, 0.22),
                    (0.4, 0.04),
                ]
            };
            let pts: Vec<(f64, f64)> = pts.iter().map(|(x, y)| (x * w, y * h)).collect();
            simple(b.poly(&pts).take(), Rect::new(w * 0.25, h * 0.3, w * 0.75, h * 0.7), w, h)
        }
        "ribbon" | "ribbon2" => ribbon(name == "ribbon2", w, h, a(0), a(1)),
        "verticalScroll" | "horizontalScroll" => scroll(name == "verticalScroll", w, h, a(0)),
        "wave" | "doubleWave" => {
            let amp = h * a(0).clamp(0.0, 20000.0) / 1e5;
            let shift = w * a(1).clamp(-10000.0, 10000.0) / 1e5;
            if name == "wave" {
                b.m(shift.max(0.0), amp).c(w / 3.0, -amp, 2.0 * w / 3.0, 3.0 * amp, w + shift.min(0.0), amp);
                b.l(w - shift.max(0.0), h - amp).c(2.0 * w / 3.0, h - 3.0 * amp, w / 3.0, h + amp, -shift.min(0.0), h - amp).z();
            } else {
                b.m(0.0, amp).c(w / 6.0, -amp, w / 3.0, 3.0 * amp, w / 2.0, amp).c(2.0 * w / 3.0, -amp, 5.0 * w / 6.0, 3.0 * amp, w, amp);
                b.l(w, h - amp)
                    .c(5.0 * w / 6.0, h - 3.0 * amp, 2.0 * w / 3.0, h + amp, w / 2.0, h - amp)
                    .c(w / 3.0, h - 3.0 * amp, w / 6.0, h + amp, 0.0, h - amp)
                    .z();
            }
            simple(b.take(), Rect::new(0.0, amp * 2.0, w, h - amp * 2.0), w, h)
        }
        "wedgeRectCallout" | "wedgeRoundRectCallout" => {
            let r = if name == "wedgeRoundRectCallout" { ss * a(2).clamp(0.0, 50000.0) / 1e5 } else { 0.0 };
            wedge_rect(w, h, a(0), a(1), r)
        }
        "wedgeEllipseCallout" => {
            let tip = Point::new(w / 2.0 + w * a(0) / 1e5, h / 2.0 + h * a(1) / 1e5);
            let ang = (tip.y - h / 2.0).atan2(tip.x - w / 2.0).to_degrees();
            let s = vis(ang + 10.0, w / 2.0, h / 2.0);
            let p0 = Point::new(w / 2.0 + w / 2.0 * s.to_radians().cos(), h / 2.0 + h / 2.0 * s.to_radians().sin());
            let e = vis(ang - 10.0, w / 2.0, h / 2.0);
            let mut sw = e - s;
            if sw <= 0.0 {
                sw += 360.0;
            }
            b.m(p0.x, p0.y).arc(w / 2.0, h / 2.0, s, sw).l(tip.x, tip.y).z();
            let mut g = simple(b.take(), Rect::new(w * 0.14645, h * 0.14645, w * 0.85355, h * 0.85355), w, h);
            g.handles.push(Handle::new(0, Point::new(w / 2.0, h / 2.0), Vec2::new(w / 1e5, 0.0), a(0), -1e6, 1e6));
            g.handles.push(Handle::new(1, Point::new(w / 2.0, h / 2.0), Vec2::new(0.0, h / 1e5), a(1), -1e6, 1e6));
            g.handles.retain(|_| false);
            g.handles.push(tip_handle(w, h, tip));
            g
        }
        "borderCallout1" => {
            let r = Pb::new().poly(&[(0.0, 0.0), (w, 0.0), (w, h), (0.0, h)]).take();
            let p1 = (w * a(1) / 1e5, h * a(0) / 1e5);
            let p2 = (w * a(3) / 1e5, h * a(2) / 1e5);
            let l = Pb::new().m(p1.0, p1.1).l(p2.0, p2.1).take();
            Geometry { paths: vec![fill(r), stroke_only(l)], text_rect: full, handles: vec![], sites: mid_sites(w, h) }
        }
        "line" | "straightConnector1" => {
            let l = Pb::new().m(0.0, 0.0).l(w, h).take();
            Geometry { paths: vec![stroke_only(l)], text_rect: full, handles: vec![], sites: vec![Point::ZERO, Point::new(w, h)] }
        }
        "bentConnector2" => {
            let l = Pb::new().m(0.0, 0.0).l(w, 0.0).l(w, h).take();
            Geometry { paths: vec![stroke_only(l)], text_rect: full, handles: vec![], sites: vec![Point::ZERO, Point::new(w, h)] }
        }
        "bentConnector3" | "bentConnector4" | "bentConnector5" => {
            let x = w * a(0) / 1e5;
            let l = Pb::new().m(0.0, 0.0).l(x, 0.0).l(x, h).l(w, h).take();
            let mut g = Geometry { paths: vec![stroke_only(l)], text_rect: full, handles: vec![], sites: vec![Point::ZERO, Point::new(w, h)] };
            g.handles.push(Handle::new(0, Point::new(0.0, h / 2.0), Vec2::new(w / 1e5, 0.0), a(0), -1e6, 1e6));
            g
        }
        "curvedConnector2" | "curvedConnector3" | "curvedConnector4" | "curvedConnector5" => {
            let x = w * if name == "curvedConnector2" { 100000.0 } else { a(0) } / 1e5;
            let l = Pb::new().m(0.0, 0.0).c(x / 2.0, 0.0, x, h / 4.0, x, h / 2.0).c(x, h * 0.75, (x + w) / 2.0, h, w, h).take();
            Geometry { paths: vec![stroke_only(l)], text_rect: full, handles: vec![], sites: vec![Point::ZERO, Point::new(w, h)] }
        }
        _ => return extra::build(name, w, h, &a),
    };
    Some(g)
}

fn ellipse_sites(w: f64, h: f64) -> Vec<Point> {
    (0..8)
        .map(|i| {
            let a = (-90.0 - 45.0 * i as f64).to_radians();
            Point::new(w / 2.0 + w / 2.0 * a.cos(), h / 2.0 + h / 2.0 * a.sin())
        })
        .collect()
}

fn tip_handle(w: f64, h: f64, tip: Point) -> Handle {
    // A free 2-D handle is represented as index 0 with zero direction; callers move both values.
    Handle { index: 0, pos: tip, origin: Point::new(w / 2.0, h / 2.0), dir: Vec2::new(w / 1e5, h / 1e5), min: -1e6, max: 1e6 }
}

fn wedge_rect(w: f64, h: f64, a0: f64, a1: f64, r: f64) -> Geometry {
    let tip = Point::new(w / 2.0 + w * a0 / 1e5, h / 2.0 + h * a1 / 1e5);
    let inside = tip.x > 0.0 && tip.x < w && tip.y > 0.0 && tip.y < h;
    let (dx, dy) = ((tip.x - w / 2.0) / w.max(1e-9), (tip.y - h / 2.0) / h.max(1e-9));
    let mut b = Pb::new();
    let r = r.min(w / 2.0).min(h / 2.0);
    // Walk clockwise from the top-left; insert the wedge on the edge facing the tip.
    let horiz = dx.abs() > dy.abs();
    let (bx0, bx1) = (w * 5.0 / 12.0, w * 7.0 / 12.0);
    let (by0, by1) = (h * 5.0 / 12.0, h * 7.0 / 12.0);
    let (bx0, bx1) = if tip.x < w / 2.0 { (w * 2.0 / 12.0, w * 5.0 / 12.0) } else { (bx0 + w / 12.0 * 2.0 - w / 12.0 * 2.0, bx1 + w * 2.0 / 12.0) };
    let (by0, by1) = if tip.y < h / 2.0 { (h * 2.0 / 12.0, h * 5.0 / 12.0) } else { (by0, by1 + h * 2.0 / 12.0) };
    b.m(r, 0.0);
    if !inside && !horiz && dy < 0.0 {
        b.l(bx0, 0.0).l(tip.x, tip.y).l(bx1, 0.0);
    }
    b.l(w - r, 0.0);
    if r > 0.0 {
        b.arc(r, r, 270.0, 90.0);
    }
    if !inside && horiz && dx > 0.0 {
        b.l(w, by0).l(tip.x, tip.y).l(w, by1);
    }
    b.l(w, h - r);
    if r > 0.0 {
        b.arc(r, r, 0.0, 90.0);
    }
    if !inside && !horiz && dy > 0.0 {
        b.l(bx1, h).l(tip.x, tip.y).l(bx0, h);
    }
    b.l(r, h);
    if r > 0.0 {
        b.arc(r, r, 90.0, 90.0);
    }
    if !inside && horiz && dx < 0.0 {
        b.l(0.0, by1).l(tip.x, tip.y).l(0.0, by0);
    }
    b.l(0.0, r);
    if r > 0.0 {
        b.arc(r, r, 180.0, 90.0);
    }
    b.z();
    let mut g = simple(b.take(), Rect::new(0.0, 0.0, w, h).inset(-r * 0.29289), w, h);
    g.handles.push(tip_handle(w, h, tip));
    g
}

fn cloud(name: &str, w: f64, h: f64, a0: f64, a1: f64) -> Geometry {
    let n = 10;
    let pts: Vec<Point> = (0..n)
        .map(|i| {
            let t = (i as f64 / n as f64 * 360.0 - 90.0).to_radians();
            let wob = if i % 2 == 0 { 1.0 } else { 0.92 };
            Point::new(0.5 + 0.40 * wob * t.cos(), 0.5 + 0.36 * wob * t.sin())
        })
        .collect();
    let mut b = Pb::new();
    let first = pts.first().copied().unwrap_or(Point::ZERO);
    b.m(first.x, first.y);
    for i in 0..n {
        let p = pts.get(i).copied().unwrap_or(Point::ZERO);
        let q = pts.get((i + 1) % n).copied().unwrap_or(Point::ZERO);
        let mid = p.midpoint(q);
        let out = (mid - Point::new(0.5, 0.5)).normalize();
        let bulge = (q - p).hypot() * 0.75;
        let c1 = p + (mid - p) * 0.2 + out * bulge;
        let c2 = q + (mid - q) * 0.2 + out * bulge;
        b.c(c1.x, c1.y, c2.x, c2.y, q.x, q.y);
    }
    b.z();
    let mut path = b.take();
    let bb = path.bounding_box();
    let sx = if bb.width() > 0.0 { w / bb.width() } else { 1.0 };
    let sy = if bb.height() > 0.0 { h / bb.height() } else { 1.0 };
    path.apply_affine(kurbo::Affine::scale_non_uniform(sx, sy) * kurbo::Affine::translate(-bb.origin().to_vec2()));
    let mut paths = vec![fill(path)];
    let text_rect = Rect::new(w * 0.2, h * 0.22, w * 0.8, h * 0.78);
    let mut handles = vec![];
    if name == "cloudCallout" {
        let tip = Point::new(w / 2.0 + w * a0 / 1e5, h / 2.0 + h * a1 / 1e5);
        let c = Point::new(w / 2.0, h / 2.0);
        let mut bubbles = Pb::new();
        for (k, r) in [(0.55, 0.06), (0.75, 0.045), (0.95, 0.03)] {
            let p = c + (tip - c) * k;
            if !(p.x > w * 0.1 && p.x < w * 0.9 && p.y > h * 0.1 && p.y < h * 0.9) || k > 0.9 {
                bubbles.ellipse(p.x, p.y, w * r, h * r);
            }
        }
        paths.push(fill(bubbles.take()));
        handles.push(tip_handle(w, h, tip));
    }
    Geometry { paths, text_rect, handles, sites: mid_sites(w, h) }
}

fn math(name: &str, w: f64, h: f64, a0: f64, a1: f64, a2: f64) -> Geometry {
    let ss = w.min(h);
    let t = ss * a0.clamp(0.0, 50000.0) / 1e5;
    let (cx, cy) = (w / 2.0, h / 2.0);
    let mx = w * 0.0735;
    let my = h * 0.0735;
    let mut b = Pb::new();
    match name {
        "mathPlus" => {
            let (hx, hy) = (cx - mx, cy - my);
            b.poly(&[
                (cx - t / 2.0, cy - hy),
                (cx + t / 2.0, cy - hy),
                (cx + t / 2.0, cy - t / 2.0),
                (cx + hx, cy - t / 2.0),
                (cx + hx, cy + t / 2.0),
                (cx + t / 2.0, cy + t / 2.0),
                (cx + t / 2.0, cy + hy),
                (cx - t / 2.0, cy + hy),
                (cx - t / 2.0, cy + t / 2.0),
                (cx - hx, cy + t / 2.0),
                (cx - hx, cy - t / 2.0),
                (cx - t / 2.0, cy - t / 2.0),
            ]);
        }
        "mathMinus" => {
            b.poly(&[(mx, cy - t / 2.0), (w - mx, cy - t / 2.0), (w - mx, cy + t / 2.0), (mx, cy + t / 2.0)]);
        }
        "mathEqual" => {
            let gap = h * a1.clamp(0.0, 100000.0) / 1e5;
            b.poly(&[(mx, cy - gap / 2.0 - t), (w - mx, cy - gap / 2.0 - t), (w - mx, cy - gap / 2.0), (mx, cy - gap / 2.0)]);
            b.poly(&[(mx, cy + gap / 2.0), (w - mx, cy + gap / 2.0), (w - mx, cy + gap / 2.0 + t), (mx, cy + gap / 2.0 + t)]);
        }
        "mathNotEqual" => {
            let gap = h * a2.clamp(0.0, 100000.0) / 1e5;
            b.poly(&[(mx, cy - gap / 2.0 - t), (w - mx, cy - gap / 2.0 - t), (w - mx, cy - gap / 2.0), (mx, cy - gap / 2.0)]);
            b.poly(&[(mx, cy + gap / 2.0), (w - mx, cy + gap / 2.0), (w - mx, cy + gap / 2.0 + t), (mx, cy + gap / 2.0 + t)]);
            let ang = (a1 / 60000.0).to_radians();
            let dir = Vec2::new(ang.cos(), ang.sin());
            let len = (h / 2.0 - my) / ang.sin().abs().max(0.2);
            let perp = Vec2::new(-dir.y, dir.x) * (t / 2.0);
            let c = Point::new(cx, cy);
            let p = [c - dir * len - perp, c + dir * len - perp, c + dir * len + perp, c - dir * len + perp];
            b.poly(&p.map(|q| (q.x, q.y)));
        }
        "mathDivide" => {
            let gap = h * a1.clamp(0.0, 100000.0) / 1e5;
            let r = ss * a2.clamp(0.0, 50000.0) / 1e5 / 2.0;
            b.poly(&[(mx, cy - t / 2.0), (w - mx, cy - t / 2.0), (w - mx, cy + t / 2.0), (mx, cy + t / 2.0)]);
            b.ellipse(cx, cy - t / 2.0 - gap - r, r, r);
            b.ellipse(cx, cy + t / 2.0 + gap + r, r, r);
        }
        _ => {
            // mathMultiply: two bars crossing at the centre along the diagonals.
            let len = ((cx - mx).powi(2) + (cy - my).powi(2)).sqrt();
            let c = Point::new(cx, cy);
            for d in [Vec2::new(cx - mx, cy - my), Vec2::new(cx - mx, -(cy - my))] {
                let dir = d / d.hypot().max(1e-9);
                let perp = Vec2::new(-dir.y, dir.x) * (t / 2.0);
                let p = [c - dir * len - perp, c + dir * len - perp, c + dir * len + perp, c - dir * len + perp];
                b.poly(&p.map(|q| (q.x, q.y)));
            }
        }
    }
    Geometry { paths: vec![fill(b.take())], text_rect: Rect::new(mx, cy - t / 2.0, w - mx, cy + t / 2.0), handles: vec![], sites: mid_sites(w, h) }
}

fn ribbon(up: bool, w: f64, h: f64, a0: f64, a1: f64) -> Geometry {
    let fold = h * a0.clamp(0.0, 33333.0) / 1e5;
    let cw = w * a1.clamp(25000.0, 75000.0) / 1e5; // centre band width
    let x0 = (w - cw) / 2.0;
    let x1 = w - x0;
    let ear = w / 8.0;
    let (top, bot) = (0.0, h);
    let mut b = Pb::new();
    let mut shade_p = Pb::new();
    if up {
        // centre band sits high; ends hang low.
        b.poly(&[(x0, top), (x1, top), (x1, bot - fold), (x0, bot - fold)]);
        let mut ends = Pb::new();
        ends.poly(&[(0.0, fold), (x0 + ear * 0.5, fold), (x0 + ear * 0.5, bot), (0.0, bot), (ear, (fold + bot) / 2.0)]);
        ends.poly(&[(w, fold), (x1 - ear * 0.5, fold), (x1 - ear * 0.5, bot), (w, bot), (w - ear, (fold + bot) / 2.0)]);
        shade_p.poly(&[(x0, bot - fold), (x0 + ear * 0.5, bot - fold), (x0 + ear * 0.5, bot)]);
        shade_p.poly(&[(x1, bot - fold), (x1 - ear * 0.5, bot - fold), (x1 - ear * 0.5, bot)]);
        let text = Rect::new(x0, top, x1, bot - fold);
        return Geometry {
            paths: vec![fill(ends.take()), shade(shade_p.take(), FillMode::DarkenLess), fill(b.take())],
            text_rect: text,
            handles: vec![],
            sites: mid_sites(w, h),
        };
    }
    b.poly(&[(x0, top + fold), (x1, top + fold), (x1, bot), (x0, bot)]);
    let mut ends = Pb::new();
    ends.poly(&[(0.0, top), (x0 + ear * 0.5, top), (x0 + ear * 0.5, bot - fold), (0.0, bot - fold), (ear, (bot - fold) / 2.0)]);
    ends.poly(&[(w, top), (x1 - ear * 0.5, top), (x1 - ear * 0.5, bot - fold), (w, bot - fold), (w - ear, (bot - fold) / 2.0)]);
    shade_p.poly(&[(x0, top + fold), (x0 + ear * 0.5, top + fold), (x0 + ear * 0.5, top)]);
    shade_p.poly(&[(x1, top + fold), (x1 - ear * 0.5, top + fold), (x1 - ear * 0.5, top)]);
    let text = Rect::new(x0, top + fold, x1, bot);
    Geometry {
        paths: vec![fill(ends.take()), shade(shade_p.take(), FillMode::DarkenLess), fill(b.take())],
        text_rect: text,
        handles: vec![],
        sites: mid_sites(w, h),
    }
}

fn scroll(vertical: bool, w: f64, h: f64, a0: f64) -> Geometry {
    let ss = w.min(h);
    let r = ss * a0.clamp(0.0, 25000.0) / 1e5 / 2.0;
    let mut b = Pb::new();
    let mut curl = Pb::new();
    if vertical {
        b.poly(&[(r, r * 2.0), (w - r, r * 2.0), (w - r, h - r), (r, h - r)]);
        b.ellipse(w / 2.0, r, w / 2.0 - r * 0.0, r);
        curl.ellipse(r, h - r, r, r);
        curl.ellipse(w - r * 2.0, r, r * 0.5, r * 0.5);
        let text = Rect::new(r, r * 2.0, w - r, h - r);
        return Geometry {
            paths: vec![fill(b.take()), shade(curl.take(), FillMode::DarkenLess)],
            text_rect: text,
            handles: vec![],
            sites: mid_sites(w, h),
        };
    }
    b.poly(&[(r * 2.0, r), (w - r, r), (w - r, h - r), (r * 2.0, h - r)]);
    b.ellipse(r, h / 2.0, r, h / 2.0);
    curl.ellipse(w - r, r, r, r);
    curl.ellipse(r, h - r * 2.0, r * 0.5, r * 0.5);
    let text = Rect::new(r * 2.0, r, w - r, h - r);
    Geometry { paths: vec![fill(b.take()), shade(curl.take(), FillMode::DarkenLess)], text_rect: text, handles: vec![], sites: mid_sites(w, h) }
}

fn action_button(name: &str, w: f64, h: f64) -> Geometry {
    let body = corner_rect(w, h, [Corner::Square; 4]);
    let ss = w.min(h);
    let (cx, cy) = (w / 2.0, h / 2.0);
    let s = ss * 0.3;
    let mut g = Pb::new();
    match name {
        "actionButtonBackPrevious" => {
            g.poly(&[(cx + s, cy - s), (cx + s, cy + s), (cx - s, cy)]);
        }
        "actionButtonForwardNext" => {
            g.poly(&[(cx - s, cy - s), (cx - s, cy + s), (cx + s, cy)]);
        }
        "actionButtonBeginning" => {
            g.poly(&[(cx + s, cy - s), (cx + s, cy + s), (cx - s * 0.6, cy)]);
            g.poly(&[(cx - s, cy - s), (cx - s * 0.7, cy - s), (cx - s * 0.7, cy + s), (cx - s, cy + s)]);
        }
        "actionButtonEnd" => {
            g.poly(&[(cx - s, cy - s), (cx - s, cy + s), (cx + s * 0.6, cy)]);
            g.poly(&[(cx + s * 0.7, cy - s), (cx + s, cy - s), (cx + s, cy + s), (cx + s * 0.7, cy + s)]);
        }
        "actionButtonHome" => {
            g.poly(&[
                (cx, cy - s),
                (cx + s, cy),
                (cx + s * 0.7, cy),
                (cx + s * 0.7, cy + s),
                (cx - s * 0.7, cy + s),
                (cx - s * 0.7, cy),
                (cx - s, cy),
            ]);
        }
        "actionButtonInformation" | "actionButtonHelp" => {
            g.ellipse(cx, cy, s, s);
        }
        "actionButtonReturn" => {
            g.m(cx + s, cy - s * 0.6).l(cx + s, cy + s * 0.2).q(cx + s, cy + s * 0.8, cx + s * 0.4, cy + s * 0.8).l(cx - s * 0.5, cy + s * 0.8);
            g.l(cx - s * 0.5, cy + s * 0.4)
                .l(cx - s, cy + s * 0.9)
                .l(cx - s * 0.5, cy + s * 1.4)
                .l(cx - s * 0.5, cy + s)
                .l(cx + s * 0.6, cy + s)
                .l(cx + s * 0.6, cy - s * 0.6)
                .z();
        }
        "actionButtonMovie" => {
            g.poly(&[(cx - s, cy - s * 0.6), (cx + s * 0.4, cy - s * 0.6), (cx + s * 0.4, cy + s * 0.6), (cx - s, cy + s * 0.6)]);
            g.poly(&[(cx + s * 0.4, cy), (cx + s, cy - s * 0.5), (cx + s, cy + s * 0.5)]);
        }
        "actionButtonDocument" => {
            g.poly(&[(cx - s * 0.7, cy - s), (cx + s * 0.3, cy - s), (cx + s * 0.7, cy - s * 0.6), (cx + s * 0.7, cy + s), (cx - s * 0.7, cy + s)]);
        }
        "actionButtonSound" => {
            g.poly(&[
                (cx - s, cy - s * 0.35),
                (cx - s * 0.5, cy - s * 0.35),
                (cx, cy - s),
                (cx, cy + s),
                (cx - s * 0.5, cy + s * 0.35),
                (cx - s, cy + s * 0.35),
            ]);
        }
        _ => {}
    }
    Geometry {
        paths: vec![fill(body), shade(g.take(), FillMode::DarkenLess)],
        text_rect: Rect::new(0.0, 0.0, w, h),
        handles: vec![],
        sites: mid_sites(w, h),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kurbo::Shape;

    #[test]
    fn every_catalog_preset_builds_within_bounds() {
        for p in CATALOG {
            for (w, h) in [(200.0, 100.0), (100.0, 200.0), (50.0, 50.0), (0.0, 0.0), (1.0, 300.0)] {
                let g = build(p.name, w, h, &[]).unwrap_or_else(|| panic!("{} did not build", p.name));
                assert!(!g.paths.is_empty(), "{} has no paths", p.name);
                for s in &g.paths {
                    let bb = s.path.bounding_box();
                    if w > 0.0 && h > 0.0 && !p.name.contains("Connector") && !p.name.contains("allout") {
                        let tol = w.max(h) * 0.35 + 1.0;
                        assert!(bb.x0 >= -tol && bb.y0 >= -tol && bb.x1 <= w + tol && bb.y1 <= h + tol, "{} {w}x{h} out of bounds: {bb:?}", p.name);
                    }
                    assert!(bb.x0.is_finite() && bb.y1.is_finite(), "{} produced non-finite geometry", p.name);
                }
                assert!(g.text_rect.x0.is_finite() && g.text_rect.y1.is_finite(), "{}", p.name);
            }
        }
    }

    #[test]
    fn hostile_adjust_values_never_panic() {
        for p in CATALOG {
            for v in [f64::NAN, f64::INFINITY, -1e12, 1e12, -1.0, 0.0] {
                let adj = vec![v; 6];
                let g = build(p.name, 120.0, 80.0, &adj);
                assert!(g.is_some(), "{}", p.name);
            }
        }
        assert!(build("rect", f64::NAN, 10.0, &[]).is_none());
        assert!(build("noSuchShape", 10.0, 10.0, &[]).is_none());
    }

    #[test]
    fn catalog_names_are_unique() {
        let mut names: Vec<&str> = CATALOG.iter().map(|p| p.name).collect();
        names.sort_unstable();
        let n = names.len();
        names.dedup();
        assert_eq!(n, names.len());
        assert!(n >= 140, "catalog has {n} presets");
    }

    #[test]
    fn round_rect_handle_maps_back() {
        let g = build("roundRect", 200.0, 100.0, &[16667.0]).unwrap();
        let h = g.handles.first().unwrap();
        assert!((h.value_at(h.pos) - 16667.0).abs() <= 1.0);
        assert_eq!(h.value_at(Point::new(-50.0, 0.0)), 0.0);
        assert_eq!(h.value_at(Point::new(500.0, 0.0)), 50000.0);
    }

    #[test]
    fn rect_area_and_ellipse_area() {
        let g = build("rect", 100.0, 50.0, &[]).unwrap();
        assert!((g.paths[0].path.area().abs() - 5000.0).abs() < 1e-6);
        let e = build("ellipse", 100.0, 50.0, &[]).unwrap();
        let expect = std::f64::consts::PI * 50.0 * 25.0;
        assert!((e.paths[0].path.area().abs() - expect).abs() / expect < 0.01);
    }

    #[test]
    fn lines_are_open() {
        assert!(build("line", 10.0, 10.0, &[]).unwrap().is_open());
        assert!(!build("rect", 10.0, 10.0, &[]).unwrap().is_open());
    }
}
