//! Charts: an embedded data grid plus type and element options.

use serde::{Deserialize, Serialize};

use crate::style::{ColorRef, Fill, Line};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ChartType {
    #[default]
    Column,
    StackedColumn,
    PercentColumn,
    Bar,
    StackedBar,
    PercentBar,
    Line,
    LineMarkers,
    StackedLine,
    Area,
    StackedArea,
    Pie,
    Doughnut,
    Scatter,
    Bubble,
    Radar,
    FilledRadar,
    Stock,
    Surface,
    Histogram,
    Pareto,
    BoxWhisker,
    Waterfall,
    Funnel,
    Treemap,
    Sunburst,
    Combo,
}

impl ChartType {
    pub const MAIN: [ChartType; 12] = [
        ChartType::Column,
        ChartType::Bar,
        ChartType::Line,
        ChartType::Area,
        ChartType::Pie,
        ChartType::Doughnut,
        ChartType::Scatter,
        ChartType::Radar,
        ChartType::Waterfall,
        ChartType::Funnel,
        ChartType::Histogram,
        ChartType::Treemap,
    ];
    pub fn label(self) -> &'static str {
        match self {
            ChartType::Column => "Clustered Column",
            ChartType::StackedColumn => "Stacked Column",
            ChartType::PercentColumn => "100% Stacked Column",
            ChartType::Bar => "Clustered Bar",
            ChartType::StackedBar => "Stacked Bar",
            ChartType::PercentBar => "100% Stacked Bar",
            ChartType::Line => "Line",
            ChartType::LineMarkers => "Line with Markers",
            ChartType::StackedLine => "Stacked Line",
            ChartType::Area => "Area",
            ChartType::StackedArea => "Stacked Area",
            ChartType::Pie => "Pie",
            ChartType::Doughnut => "Doughnut",
            ChartType::Scatter => "Scatter",
            ChartType::Bubble => "Bubble",
            ChartType::Radar => "Radar",
            ChartType::FilledRadar => "Filled Radar",
            ChartType::Stock => "Stock",
            ChartType::Surface => "Surface",
            ChartType::Histogram => "Histogram",
            ChartType::Pareto => "Pareto",
            ChartType::BoxWhisker => "Box and Whisker",
            ChartType::Waterfall => "Waterfall",
            ChartType::Funnel => "Funnel",
            ChartType::Treemap => "Treemap",
            ChartType::Sunburst => "Sunburst",
            ChartType::Combo => "Combo",
        }
    }
    pub fn is_bar_like(self) -> bool {
        matches!(
            self,
            ChartType::Column
                | ChartType::StackedColumn
                | ChartType::PercentColumn
                | ChartType::Bar
                | ChartType::StackedBar
                | ChartType::PercentBar
                | ChartType::Histogram
                | ChartType::Pareto
                | ChartType::Waterfall
        )
    }
    pub fn horizontal(self) -> bool {
        matches!(self, ChartType::Bar | ChartType::StackedBar | ChartType::PercentBar | ChartType::Funnel)
    }
    pub fn stacked(self) -> bool {
        matches!(
            self,
            ChartType::StackedColumn
                | ChartType::StackedBar
                | ChartType::StackedLine
                | ChartType::StackedArea
                | ChartType::PercentColumn
                | ChartType::PercentBar
        )
    }
    pub fn percent(self) -> bool {
        matches!(self, ChartType::PercentColumn | ChartType::PercentBar)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Series {
    pub name: String,
    pub values: Vec<Option<f64>>,
    /// X values for scatter/bubble.
    pub x_values: Vec<Option<f64>>,
    pub sizes: Vec<Option<f64>>,
    pub fill: Option<Fill>,
    pub line: Option<Line>,
    /// Per-point fill overrides (pie slices).
    pub point_fills: Vec<(usize, Fill)>,
    pub data_labels: Option<bool>,
    pub smooth: bool,
    pub marker: Option<String>,
    /// Combo charts: this series' own type.
    pub kind: Option<ChartType>,
    pub secondary_axis: bool,
    /// The number format of the series' data labels (Excel code): the labels' own, else the
    /// source data's.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub number_format: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Chart {
    pub kind: ChartType,
    pub title: Option<String>,
    pub categories: Vec<String>,
    pub series: Vec<Series>,
    pub legend: Option<String>,
    /// `c:legend/c:overlay`: the legend is drawn over the plot instead of shrinking it.
    pub legend_overlay: bool,
    /// Text size in points of the axis labels and legend (from their `c:txPr`, else the chart's);
    /// `None`: the file gives none.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_size: Option<f64>,
    /// Title text size in points (its rich text or `c:txPr`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title_size: Option<f64>,
    pub data_labels: bool,
    pub gridlines: bool,
    pub axis_titles: (Option<String>, Option<String>),
    /// Chart style number (1–48) and colour palette (`colorful1`, `monochrome3`…).
    pub style: u32,
    pub palette: String,
    pub plot_fill: Option<Fill>,
    pub chart_fill: Option<Fill>,
    pub text_color: Option<ColorRef>,
    pub hole_size: f64,
    pub gap_width: f64,
    pub overlap: f64,
    pub vary_colors: bool,
    pub y_min: Option<f64>,
    pub y_max: Option<f64>,
    /// The value axis' major unit, when the chart fixes it.
    pub y_major: Option<f64>,
    /// The value axis' number format (an Excel format code such as `#,##0` or `0.0%`).
    pub y_format: Option<String>,
    /// The value axis is deleted (`c:delete`): no labels, though gridlines may stay.
    pub y_hidden: bool,
    /// A combo chart's secondary value axis: bounds, major unit and number format.
    pub y2_min: Option<f64>,
    pub y2_max: Option<f64>,
    pub y2_major: Option<f64>,
    pub y2_format: Option<String>,
    pub y2_hidden: bool,
    /// The chart's language (`c:lang`, e.g. `ru-RU`): digit grouping and decimal mark.
    pub lang: Option<String>,
    /// Original chart XML part kept for round-trip of unmodelled options.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw: Option<String>,
}

impl Default for Chart {
    fn default() -> Self {
        Chart {
            kind: ChartType::Column,
            title: Some("Chart Title".into()),
            categories: vec![],
            series: vec![],
            legend: Some("b".into()),
            legend_overlay: false,
            text_size: None,
            title_size: None,
            data_labels: false,
            gridlines: true,
            axis_titles: (None, None),
            style: 2,
            palette: "colorful1".into(),
            plot_fill: None,
            chart_fill: None,
            text_color: None,
            hole_size: 0.5,
            gap_width: 2.19,
            overlap: -0.27,
            vary_colors: false,
            y_min: None,
            y_max: None,
            y_major: None,
            y_format: None,
            y_hidden: false,
            y2_min: None,
            y2_max: None,
            y2_major: None,
            y2_format: None,
            y2_hidden: false,
            lang: None,
            raw: None,
        }
    }
}

impl Chart {
    /// The sample data a new chart starts with (our own numbers).
    pub fn sample(kind: ChartType) -> Self {
        let cats = ["Q1", "Q2", "Q3", "Q4"];
        let data = [[4.2, 2.6, 2.1], [2.8, 4.1, 2.4], [3.6, 1.9, 3.3], [4.7, 3.0, 5.1]];
        let mut c = Chart { kind, categories: cats.iter().map(|s| s.to_string()).collect(), ..Default::default() };
        let n_series =
            if matches!(kind, ChartType::Pie | ChartType::Doughnut | ChartType::Funnel | ChartType::Waterfall | ChartType::Treemap) { 1 } else { 3 };
        for s in 0..n_series {
            c.series.push(Series {
                name: format!("Series {}", s + 1),
                values: data.iter().map(|row| row.get(s).copied()).collect(),
                x_values: if kind == ChartType::Scatter { (1..=4).map(|x| Some(x as f64)).collect() } else { vec![] },
                ..Default::default()
            });
        }
        c.vary_colors = n_series == 1;
        c
    }
    /// A series' own chart type: its own in a combo chart (columns when it names none), else the
    /// chart's.
    pub fn series_kind(&self, s: &Series) -> ChartType {
        match (self.kind, s.kind) {
            (ChartType::Combo, Some(k)) if k != ChartType::Combo => k,
            (ChartType::Combo, _) => ChartType::Column,
            (k, _) => k,
        }
    }

    /// Whether a series is plotted against the secondary value axis (combo charts only).
    pub fn on_secondary(&self, s: &Series) -> bool {
        self.kind == ChartType::Combo && s.secondary_axis
    }

    /// The value range of one value axis: the primary axis (`y_min`/`y_max` apply), or a combo
    /// chart's secondary axis. A combo chart stacks only its stacked series. `None` when no series
    /// uses that axis.
    pub fn axis_range(&self, secondary: bool) -> Option<(f64, f64)> {
        if self.kind != ChartType::Combo {
            return (!secondary).then(|| self.value_range());
        }
        let (lo, hi) = self.plotted_extent(secondary)?;
        let (lo, hi) = (lo.min(0.0), hi.max(0.0));
        Some(if secondary { (lo, hi) } else { (self.y_min.unwrap_or(lo), self.y_max.unwrap_or(hi)) })
    }

    /// The smallest and largest value plotted against one value axis: plain values as they are,
    /// stacked series as running sums from zero, 100 % series as 0–1. `(0, 0)` without finite
    /// values; `None` when no series uses the axis.
    fn plotted_extent(&self, secondary: bool) -> Option<(f64, f64)> {
        let ser: Vec<&Series> = self.series.iter().filter(|s| self.on_secondary(s) == secondary).collect();
        if ser.is_empty() {
            return (!secondary && self.kind != ChartType::Combo).then_some((0.0, 0.0));
        }
        let (mut lo, mut hi) = (f64::INFINITY, f64::NEG_INFINITY);
        let n = ser.iter().map(|s| s.values.len()).max().unwrap_or(0);
        let mut percent = false;
        for i in 0..n {
            let (mut p, mut m, mut stacked) = (0.0, 0.0, false);
            for s in &ser {
                let k = self.series_kind(s);
                let Some(v) = s.values.get(i).copied().flatten().filter(|v| v.is_finite()) else { continue };
                if k.percent() {
                    percent = true;
                } else if k.stacked() {
                    stacked = true;
                    if v >= 0.0 { p += v } else { m += v }
                } else {
                    hi = hi.max(v);
                    lo = lo.min(v);
                }
            }
            if stacked {
                hi = hi.max(p);
                lo = lo.min(m);
            }
        }
        if percent {
            lo = lo.min(0.0);
            hi = hi.max(1.0);
        }
        Some(if lo <= hi { (lo, hi) } else { (0.0, 0.0) })
    }

    /// A value axis' scale — (min, max, major unit) — with at most `max_intervals` intervals:
    /// the bounds and unit the chart fixes, else Office's automatic ones ([`axis_scale`](fn@axis_scale)).
    /// `None` when no series uses the axis.
    pub fn axis_scale(&self, secondary: bool, max_intervals: usize) -> Option<(f64, f64, f64)> {
        let (lo, hi) = self.plotted_extent(secondary)?;
        let (fmin, fmax, major) = if secondary { (self.y2_min, self.y2_max, self.y2_major) } else { (self.y_min, self.y_max, self.y_major) };
        let full_percent = !secondary && self.kind.percent();
        Some(axis_scale(lo, hi, fmin, fmax, major, max_intervals, !full_percent))
    }

    /// A value axis label in the axis' number format and the chart's language.
    pub fn axis_label(&self, secondary: bool, v: f64) -> String {
        let code = if secondary { &self.y2_format } else { &self.y_format };
        crate::numfmt::format(code.as_deref().unwrap_or("General"), v, self.lang.as_deref())
    }

    /// A data label of series `s`: its number format, else its axis', in the chart's language.
    pub fn data_label(&self, s: &Series, v: f64) -> String {
        let axis = if self.on_secondary(s) { &self.y2_format } else { &self.y_format };
        let code = s.number_format.as_ref().or(axis.as_ref());
        crate::numfmt::format(code.map_or("General", String::as_str), v, self.lang.as_deref())
    }

    /// Value range across series (stacked sums for stacked types).
    pub fn value_range(&self) -> (f64, f64) {
        let mut lo: f64 = 0.0;
        let mut hi: f64 = 0.0;
        let n = self.categories.len().max(self.series.iter().map(|s| s.values.len()).max().unwrap_or(0));
        if self.kind.stacked() {
            for i in 0..n {
                let (mut p, mut m) = (0.0, 0.0);
                for s in &self.series {
                    let v = s.values.get(i).copied().flatten().unwrap_or(0.0);
                    if v >= 0.0 { p += v } else { m += v }
                }
                hi = hi.max(p);
                lo = lo.min(m);
            }
        } else {
            for s in &self.series {
                for v in s.values.iter().flatten() {
                    if v.is_finite() {
                        hi = hi.max(*v);
                        lo = lo.min(*v);
                    }
                }
            }
        }
        if self.kind.percent() {
            return (0.0, 1.0);
        }
        (self.y_min.unwrap_or(lo), self.y_max.unwrap_or(hi))
    }
}

/// The (min, max, major unit) of an axis over the data range [lo, hi]: `fmin`/`fmax`/`major`
/// where the chart fixes them; otherwise Office's automatic limits, rounded out to the smallest
/// 1-2-5 unit giving at most `max_intervals` intervals:
///
/// - **Maximum:** `hi + (hi − lo)/20` (with `headroom`), rounded up to the unit.
/// - **Minimum:** zero when every value is ≥ 0 and `lo` is below 5/6 of `hi`; otherwise
///   `lo − (hi − lo)/20`, rounded down to the unit, so values bunched high get an axis that starts
///   near them.
/// - All-negative values mirror this: the axis ends at zero when `hi` is above 5/6 of `lo`.
///   Mixed signs get headroom at both ends.
pub fn axis_scale(
    lo: f64,
    hi: f64,
    fmin: Option<f64>,
    fmax: Option<f64>,
    major: Option<f64>,
    max_intervals: usize,
    headroom: bool,
) -> (f64, f64, f64) {
    // Values past ±1e100 aren't axis values; clamping keeps the arithmetic finite.
    let fin = |v: Option<f64>| v.filter(|v| v.is_finite()).map(|v| v.clamp(-1e100, 1e100));
    let (fmin, fmax) = (fin(fmin), fin(fmax));
    let (lo, hi) = if lo.is_finite() && hi.is_finite() { (lo.min(hi).clamp(-1e100, 1e100), lo.max(hi).clamp(-1e100, 1e100)) } else { (0.0, 1.0) };
    // Office's automatic limits ("How chart axis limits are determined", as Jon Peltier
    // summarises it): 5 % of the data range past the data, and zero when the values don't
    // sit within the top sixth of their largest magnitude. A single value is measured from zero.
    let flat = hi - lo < 1e-12;
    let pad = if !headroom {
        0.0
    } else if flat {
        hi.abs() / 20.0
    } else {
        (hi - lo) / 20.0
    };
    let auto_lo = if lo >= 0.0 && (flat || lo < hi * 5.0 / 6.0) { 0.0 } else { lo - pad };
    let auto_hi = if hi <= 0.0 && (flat || hi > lo * 5.0 / 6.0) { 0.0 } else { hi + pad };
    let mut lo = fmin.unwrap_or(auto_lo);
    let mut hi = fmax.unwrap_or(auto_hi);
    if hi - lo < 1e-12 {
        // A fixed limit on the far side of the data: the automatic end gives way.
        match (fmin, fmax) {
            (None, Some(_)) => lo = if hi > 0.0 { 0.0 } else { hi - hi.abs().max(1.0) },
            (Some(_), None) if lo < 0.0 => hi = 0.0,
            _ => hi = lo + lo.abs().max(1.0),
        }
    }
    let n = max_intervals.clamp(1, 50) as f64;
    let step = match fin(major).filter(|m| *m > 0.0 && (hi - lo) / m <= 1000.0) {
        Some(m) => m,
        None => {
            let mut step = 10f64.powf(((hi - lo) / n).log10().floor());
            let mut k = 0;
            // 1, 2, 5, 10, 20… × the magnitude until the intervals fit.
            while ((hi / step).ceil() - (lo / step).floor()) > n && k < 30 {
                step *= if matches!(k % 3, 1) { 2.5 } else { 2.0 };
                k += 1;
            }
            step
        }
    };
    let min = fmin.unwrap_or((lo / step + 1e-9).floor() * step);
    let max = fmax.unwrap_or((hi / step - 1e-9).ceil() * step).max(min + step);
    (min, max, step)
}

/// "Nice" axis scale: (min, max, major step) covering [lo, hi].
pub fn nice_scale(lo: f64, hi: f64) -> (f64, f64, f64) {
    let (lo, hi) = if lo.is_finite() && hi.is_finite() { (lo.min(hi), lo.max(hi)) } else { (0.0, 1.0) };
    let span = if hi - lo < 1e-12 { hi.abs().max(1.0) } else { hi - lo };
    let raw = span / 5.0;
    let mag = 10f64.powf(raw.log10().floor());
    let norm = raw / mag;
    let step = if norm <= 1.0 {
        1.0
    } else if norm <= 2.0 {
        2.0
    } else if norm <= 2.5 {
        2.5
    } else if norm <= 5.0 {
        5.0
    } else {
        10.0
    } * mag;
    let min = (lo / step).floor() * step;
    let max = (hi / step).ceil() * step;
    let max = if (max - min).abs() < 1e-12 { min + step } else { max };
    (min, max, step)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nice_scale_covers() {
        let (a, b, s) = nice_scale(0.0, 5.1);
        assert!(a <= 0.0 && b >= 5.1 && s > 0.0);
        let (a, b, _) = nice_scale(f64::NAN, 3.0);
        assert!(a.is_finite() && b.is_finite());
        let (a, b, _) = nice_scale(7.0, 7.0);
        assert!(b > a);
    }

    #[test]
    fn automatic_axis_scales_like_powerpoint() {
        // Weekly spend up to ~715 000 on a plot with room for 9 intervals: 0–800 000 by 100 000.
        assert_eq!(axis_scale(0.0, 715_000.0, None, None, None, 9, true), (0.0, 800_000.0, 100_000.0));
        // Clicks up to 6 600: 0–7 000 by 1 000.
        assert_eq!(axis_scale(0.0, 6_600.0, None, None, None, 9, true), (0.0, 7_000.0, 1_000.0));
        // Fewer intervals fit on a short axis.
        assert_eq!(axis_scale(0.0, 6_600.0, None, None, None, 4, true), (0.0, 8_000.0, 2_000.0));
        // Negative data gets room below; fixed bounds and unit are kept.
        let (min, max, _) = axis_scale(-30.0, 100.0, None, None, None, 8, true);
        assert!(min <= -30.0 && max >= 105.0);
        assert_eq!(axis_scale(0.0, 715_000.0, Some(100_000.0), Some(750_000.0), Some(50_000.0), 9, true), (100_000.0, 750_000.0, 50_000.0));
        // 100 % stacked stays at 0–1 without headroom.
        assert_eq!(axis_scale(0.0, 1.0, None, None, None, 5, false), (0.0, 1.0, 0.2));
        // Nonsense never loops or divides by zero.
        for (lo, hi) in [(f64::NAN, 1.0), (5.0, 5.0), (0.0, 0.0), (-1e300, 1e300)] {
            let (a, b, s) = axis_scale(lo, hi, None, None, Some(0.0), 0, true);
            assert!(a.is_finite() && b.is_finite() && s > 0.0 && b > a, "{lo} {hi}: {a} {b} {s}");
        }
    }

    fn column(values: &[f64]) -> Chart {
        let series = Series { name: "Sales".into(), values: values.iter().map(|v| Some(*v)).collect(), ..Default::default() };
        Chart { categories: (0..values.len()).map(|i| format!("C{i}")).collect(), series: vec![series], ..Default::default() }
    }

    /// The primary axis of a chart with room for ten intervals.
    fn scale(c: &Chart) -> (f64, f64, f64) {
        c.axis_scale(false, 10).unwrap_or_default()
    }

    /// Office's documented automatic limits (#144), with room for ten intervals.
    #[test]
    fn automatic_axis_matches_office() {
        // #144: 42.1 / 18.7 / 6.3 → 0–45 in steps of 5, not 0–50 in steps of 10.
        assert_eq!(scale(&column(&[42.1, 18.7, 6.3])), (0.0, 45.0, 5.0));
        // PowerPoint's default chart data (max 5): 0–6 in steps of 1.
        let mut c = column(&[4.3, 2.5, 3.5, 4.5]);
        c.series.push(Series { values: [2.4, 4.4, 1.8, 2.8].map(Some).to_vec(), ..Default::default() });
        c.series.push(Series { values: [2.0, 2.0, 3.0, 5.0].map(Some).to_vec(), ..Default::default() });
        assert_eq!(scale(&c), (0.0, 6.0, 1.0));
        // A maximum of 100 (or 10) gets 5 % headroom, which rounds up to the next major unit.
        assert_eq!(scale(&column(&[100.0, 40.0])), (0.0, 120.0, 20.0));
        assert_eq!(scale(&column(&[10.0, 3.0])), (0.0, 12.0, 2.0));
        // Peltier's example: 2.95 + 2.95/20 ≈ 3.1 rounds up to 3.5.
        assert_eq!(scale(&column(&[2.95, 0.0])), (0.0, 3.5, 0.5));
    }

    #[test]
    fn automatic_axis_minimum() {
        // Below 5/6 of the maximum the axis starts at zero; at or above it, a unit below
        // min − (max − min)/20.
        assert_eq!(scale(&column(&[59.0, 49.0])).0, 0.0);
        assert_eq!(scale(&column(&[59.0, 50.0])), (48.0, 60.0, 2.0));
        assert_eq!(scale(&column(&[85.0, 92.0, 97.0])), (84.0, 98.0, 2.0));
        // All-negative data mirrors it: the axis ends at zero, or near the data.
        assert_eq!(scale(&column(&[-42.1, -18.7, -6.3])), (-45.0, 0.0, 5.0));
        assert_eq!(scale(&column(&[-59.0, -49.0])).1, 0.0);
        assert_eq!(scale(&column(&[-59.0, -50.0])), (-60.0, -48.0, 2.0));
        // Mixed signs get headroom at both ends.
        assert_eq!(scale(&column(&[42.1, -18.7])), (-30.0, 50.0, 10.0));
        // Stacked bars build up from zero, so their axis keeps it.
        let mut s = column(&[85.0, 92.0]);
        s.kind = ChartType::StackedColumn;
        assert_eq!(scale(&s).0, 0.0);
        // A combo chart's secondary axis follows the same rule.
        let mut c = Chart::sample(ChartType::Combo);
        c.series.truncate(2);
        c.series[0].values = vec![Some(200_000.0), Some(310_000.0)];
        c.series[1].kind = Some(ChartType::Line);
        c.series[1].values = vec![Some(7_000.0), Some(7_400.0)];
        c.series[1].secondary_axis = true;
        assert_eq!(c.axis_scale(false, 10), Some((0.0, 350_000.0, 50_000.0)));
        assert_eq!(c.axis_scale(true, 10), Some((6_950.0, 7_450.0, 50.0)));
    }

    #[test]
    fn fixed_axis_limits_are_kept() {
        let mut c = column(&[42.1, 18.7, 6.3]);
        c.y_max = Some(50.0);
        assert_eq!(scale(&c), (0.0, 50.0, 5.0));
        c.y_min = Some(10.0);
        c.y_max = None;
        assert_eq!(scale(&c), (10.0, 45.0, 5.0));
        // A fixed limit beyond the data: the automatic end gives way to it.
        let mut c = column(&[85.0, 92.0, 97.0]);
        c.y_max = Some(50.0);
        assert_eq!(scale(&c), (0.0, 50.0, 5.0));
        let mut c = column(&[-50.0, -48.0]);
        c.y_min = Some(-10.0);
        assert_eq!(scale(&c), (-10.0, 0.0, 1.0));
    }

    #[test]
    fn percent_axis_is_zero_to_one() {
        let mut p = column(&[42.1, 18.7, 6.3]);
        p.kind = ChartType::PercentColumn;
        assert_eq!(scale(&p), (0.0, 1.0, 0.1));
        assert_eq!(p.axis_scale(false, 5), Some((0.0, 1.0, 0.2)));
    }

    #[test]
    fn degenerate_axes_are_finite() {
        for (a, b) in [(0.0, 0.0), (7.0, 7.0), (-7.0, -7.0), (f64::NAN, 3.0), (f64::INFINITY, 1.0), (-f64::MAX, f64::MAX), (0.0, 1e-320)] {
            let (min, max, step) = scale(&column(&[a, b]));
            assert!(min.is_finite() && max > min && step > 0.0, "{a} {b}: {min} {max} {step}");
        }
        let (min, max, _) = scale(&column(&[7.0, 7.0]));
        assert!(min == 0.0 && max > 7.0);
        assert_eq!(scale(&column(&[])), (0.0, 1.0, 0.1));
        for (fmin, fmax) in [(Some(5.0), Some(5.0)), (Some(5.0), Some(1.0)), (Some(f64::NAN), Some(f64::INFINITY)), (Some(-1e300), None)] {
            let (a, b, s) = axis_scale(0.0, 1.0, fmin, fmax, None, 10, true);
            assert!(a.is_finite() && b.is_finite() && b > a && s > 0.0, "{fmin:?} {fmax:?}: {a} {b} {s}");
        }
    }

    #[test]
    fn axis_labels_use_the_format_and_language() {
        let mut c = Chart::sample(ChartType::Column);
        c.y_format = Some(r#"_-* #\ ##0\ "₽"_-;\-* #\ ##0\ "₽"_-;_-* "-"??\ "₽"_-;_-@_-"#.into());
        c.lang = Some("ru-RU".into());
        assert_eq!(c.axis_label(false, 800_000.0), "800\u{a0}000 ₽");
        assert_eq!(c.axis_label(false, 0.0), "- ₽");
        // No format: General.
        assert_eq!(c.axis_label(true, 2.5), "2,5");
        c.lang = None;
        assert_eq!(c.axis_label(true, 2.5), "2.5");
        // Data labels: the series' format, else the axis'.
        c.lang = Some("ru-RU".into());
        c.series[0].number_format = Some(r#"#,##0\ "₽""#.into());
        assert_eq!(c.data_label(&c.series[0], 516_600_000.0), "516\u{a0}600\u{a0}000 ₽");
        assert_eq!(c.data_label(&c.series[1], 0.0), "- ₽");
    }

    #[test]
    fn combo_axis_ranges() {
        // Spend in columns on the primary axis, clicks as a line on the secondary one.
        let mut c = Chart::sample(ChartType::Combo);
        c.series.truncate(2);
        c.series[0].kind = Some(ChartType::Column);
        c.series[0].values = vec![Some(200_000.0), Some(310_000.0)];
        c.series[1].kind = Some(ChartType::Line);
        c.series[1].values = vec![Some(6_000.0), Some(7_400.0)];
        c.series[1].secondary_axis = true;
        assert_eq!(c.axis_range(false), Some((0.0, 310_000.0)));
        assert_eq!(c.axis_range(true), Some((0.0, 7_400.0)));
        assert_eq!(c.series_kind(&c.series[1]), ChartType::Line);
        // Without a secondary series there is no secondary axis; a non-combo chart never has one.
        c.series[1].secondary_axis = false;
        assert_eq!(c.axis_range(true), None);
        let col = Chart::sample(ChartType::Column);
        assert_eq!(col.axis_range(false), Some(col.value_range()));
        assert_eq!(col.axis_range(true), None);
        assert!(!col.on_secondary(&col.series[0]));
    }

    #[test]
    fn sample_ranges() {
        let c = Chart::sample(ChartType::StackedColumn);
        let (_, hi) = c.value_range();
        assert!(hi > 10.0);
        assert_eq!(Chart::sample(ChartType::Pie).series.len(), 1);
    }
}
