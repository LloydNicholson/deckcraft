use super::*;
use deckcraft_model::Fill;
use deckcraft_model::defaults;
use deckcraft_model::text::{Paragraph, Run};
use deckcraft_model::{LayoutType, Presentation, Xfrm};

fn setup(text: &str, w: f64, h: f64) -> (Presentation, Shape) {
    let mut p = Presentation::default();
    let lay = defaults::layout_of_kind(&p, LayoutType::TitleAndContent).unwrap();
    let mut s = defaults::new_slide(&mut p, lay);
    let mut body = s.shapes.remove(1);
    body.xfrm = Some(Xfrm::new(0.0, 0.0, w, h));
    body.text = Some(TextBody::from_text(text));
    s.shapes = vec![body.clone()];
    p.slides = vec![std::sync::Arc::new(s)];
    (p, body)
}

fn lay(p: &Presentation, sh: &Shape, w: f64, h: f64) -> TextLayout {
    let ctx = Ctx::for_slide(p, &p.slides[0]).unwrap();
    layout(&ctx, sh, sh.text.as_ref().unwrap(), &Opts { rect: Rect::new(0.0, 0.0, w, h), fields: &NoFields, prompt_color: None, no_shrink: false })
}

#[test]
fn wraps_long_text_into_lines() {
    let text = "The quick brown fox jumps over the lazy dog and keeps on running far away";
    let (p, sh) = setup(text, 200.0, 400.0);
    let l = lay(&p, &sh, 200.0, 400.0);
    assert!(l.lines.len() >= 3, "{} lines", l.lines.len());
    for li in &l.lines {
        assert!(li.caret_x.last().unwrap() <= &(200.0 + 0.5), "line too wide: {:?}", li.caret_x.last());
        assert_eq!(li.caret_x.len(), li.end - li.start + 1);
    }
    // Lines cover the text without gaps (spaces at wraps are skipped).
    assert_eq!(l.lines[0].start, 0);
    assert!(!l.runs.is_empty());
}

#[test]
fn bullets_are_drawn_for_body_text() {
    let (p, sh) = setup("One\nTwo", 600.0, 300.0);
    let l = lay(&p, &sh, 600.0, 300.0);
    // Two text runs + two bullet runs.
    assert!(l.runs.len() >= 4, "{:?}", l.runs);
    assert_eq!(l.para_lines.len(), 2);
}

/// Glyph count and width of `text` laid out in one line of `family` at 40 pt, kern threshold `kern`.
fn glyphs_and_width(family: &str, text: &str, kern: Option<f64>) -> (usize, f64) {
    let (p, mut sh) = setup(text, 2000.0, 300.0);
    let para = &mut sh.text.as_mut().unwrap().paragraphs[0];
    para.props.bullet = Some(Bullet::None);
    for r in &mut para.runs {
        r.props.font = Some(family.into());
        r.props.size = Some(40.0);
        r.props.kern = kern;
    }
    let l = lay(&p, &sh, 2000.0, 300.0);
    let n = l.runs.iter().map(|r| r.glyphs.len()).sum();
    (n, l.lines[0].extent.1 - l.lines[0].extent.0)
}

/// The first installed or bundled family of `families` for which `pick` holds.
fn first_family(families: &[&str], pick: impl Fn(&FontFace) -> bool) -> Option<String> {
    let db = FontDb::global();
    families.iter().filter(|f| db.has_family(f)).map(|f| db.face(f, "Regular")).find(|f| pick(f)).map(|f| f.family.clone())
}

const LATIN_FAMILIES: &[&str] = &["Carlito", "Calibri", "DejaVu Sans", "Liberation Sans", "Inter", "Arial", "Cambria", "Times New Roman"];

#[test]
fn latin_text_draws_no_ligatures() {
    // PowerPoint keeps "fi", "ffi" and Calibri's "ti" apart, although the fonts have them as
    // standard ligatures.
    let ligates = |f: &FontFace| deckcraft_fonts::shape(f, "office", &[], |c| c).len() < 6;
    let Some(family) = first_family(LATIN_FAMILIES, ligates) else {
        eprintln!("skipped: no font with an ffi ligature");
        return;
    };
    let (n, _) = glyphs_and_width(&family, "office fifth", Some(0.0));
    assert_eq!(n, 12, "{family}: one glyph per character");
}

#[test]
fn pairs_are_kerned_only_from_the_kern_size() {
    let kerns = |f: &FontFace| {
        let w = |feat: &[deckcraft_fonts::Feature]| deckcraft_fonts::shape(f, "AVATAR", feat, |c| c).iter().map(|g| g.x_advance).sum::<i32>();
        w(&[]) != w(&deckcraft_fonts::feature("-kern").into_iter().collect::<Vec<_>>())
    };
    let Some(family) = first_family(LATIN_FAMILIES, kerns) else {
        eprintln!("skipped: no font that kerns AVATAR");
        return;
    };
    // `kern` is the smallest size kerned; 0 is no kerning.
    let (_, off) = glyphs_and_width(&family, "AVATAR WAVE", Some(0.0));
    let (_, on) = glyphs_and_width(&family, "AVATAR WAVE", Some(12.0));
    let (_, at) = glyphs_and_width(&family, "AVATAR WAVE", Some(40.0));
    let (_, above) = glyphs_and_width(&family, "AVATAR WAVE", Some(48.0));
    assert!(on < off - 2.0, "{family}: kerned {on} vs {off}");
    assert!((at - on).abs() < 0.01, "kerned at exactly the threshold size");
    assert!((above - off).abs() < 0.01, "40 pt is below a 48 pt threshold");
}

#[test]
fn ligatures_stay_for_scripts_that_need_them() {
    let has = |f: &[deckcraft_fonts::Feature], tag: &str| f.iter().any(|x| Some(*x) == deckcraft_fonts::feature(tag));
    let cells =
        |s: &str| s.chars().map(|ch| Cell { ch, style: 0, face: 0, level: 0, glyphs: vec![], adv: 0.0, head: 0, boundary: true }).collect::<Vec<_>>();
    let latin = shaping_features(&cells("office"), &[], Some(*b"Latn"));
    assert!(has(&latin, "-liga") && has(&latin, "-clig") && has(&latin, "-kern"));
    // Arabic needs its ligatures; emoji sequences are joined by them.
    assert!(!has(&shaping_features(&cells("\u{644}\u{627}"), &[], Some(*b"Arab")), "-liga"));
    assert!(!has(&shaping_features(&cells("a\u{1F469}\u{200D}\u{1F4BB}"), &[], Some(*b"Latn")), "-liga"));
    assert!(!has(&shaping_features(&cells("12"), &[], None), "-liga"));
}

#[test]
fn only_the_first_character_of_a_bullet_is_drawn() {
    // "••" (seen in the wild) draws one dot in PowerPoint.
    for (char, glyphs) in [("\u{2022}\u{2022}", 1), ("\u{2022}", 1), ("-->", 1)] {
        let (p, mut sh) = setup("One", 600.0, 300.0);
        sh.text.as_mut().unwrap().paragraphs[0].props.bullet = Some(Bullet::Char { char: char.into() });
        let l = lay(&p, &sh, 600.0, 300.0);
        let bullet = l.runs.iter().find(|r| r.chars == (0, 0)).expect("bullet run");
        assert_eq!(bullet.glyphs.len(), glyphs, "{char:?}");
    }
    // The first grapheme cluster is kept whole.
    assert_eq!(map_symbol_bullet("e\u{301}x", None), "e\u{301}");
}

#[test]
fn shrink_on_overflow() {
    let text = (0..30).map(|i| format!("Line number {i}")).collect::<Vec<_>>().join("\n");
    let (p, sh) = setup(&text, 600.0, 200.0);
    let l = lay(&p, &sh, 600.0, 200.0);
    assert!(l.font_scale < 1.0, "scale {}", l.font_scale);
    assert!(!l.overflow || l.font_scale <= 0.11);
}

#[test]
fn caret_and_hit_roundtrip() {
    let (p, sh) = setup("Hello world", 600.0, 100.0);
    let l = lay(&p, &sh, 600.0, 100.0);
    for ch in 0..=11 {
        let pos = Pos { para: 0, ch };
        let (x, top, bot) = l.caret(pos).unwrap();
        let back = l.hit(Point::new(x + 0.1, (top + bot) / 2.0));
        assert_eq!(back, pos, "at {ch}");
    }
    let r = l.selection_rects(Pos { para: 0, ch: 0 }, Pos { para: 0, ch: 5 });
    assert_eq!(r.len(), 1);
}

/// #101: the text an input method is composing is underlined in place, under the baseline of each
/// line it occupies, and only over the composed characters.
#[test]
fn composition_is_underlined_per_line() {
    let (p, sh) = setup("Hello 한국어 world", 600.0, 100.0);
    let l = lay(&p, &sh, 600.0, 100.0);
    let (a, b) = (Pos { para: 0, ch: 6 }, Pos { para: 0, ch: 9 });
    let u = l.composition_underlines(a, b);
    assert_eq!(u.len(), 1, "{u:?}");
    let (x0, x1, y) = u[0];
    let (cx0, ..) = l.caret(a).unwrap();
    let (cx1, ..) = l.caret(b).unwrap();
    assert!((x0 - cx0).abs() < 0.01 && (x1 - cx1).abs() < 0.01, "{u:?} vs carets {cx0} {cx1}");
    let line = &l.lines[0];
    assert!(y > line.baseline && y <= line.bottom, "under the baseline: {y} ({} – {})", line.baseline, line.bottom);
    // Reversed ends give the same; an empty composition, nothing.
    assert_eq!(l.composition_underlines(b, a), u);
    assert!(l.composition_underlines(a, a).is_empty());

    // Wrapped: one underline per line.
    let (p, sh) = setup("가나다라마바사아자차카타파하 가나다라마바사아자차카타파하 가나다라마바사", 120.0, 400.0);
    let l = lay(&p, &sh, 120.0, 400.0);
    assert!(l.lines.len() >= 2);
    let u = l.composition_underlines(Pos { para: 0, ch: 0 }, Pos { para: 0, ch: 40 });
    assert!(u.len() >= 2, "{u:?}");
    assert!(u.windows(2).all(|w| w[1].2 > w[0].2), "each lower than the last: {u:?}");
    // Positions past the text never panic.
    let _ = l.composition_underlines(Pos { para: 3, ch: 0 }, Pos { para: 9, ch: 99 });
}

#[test]
fn alignment_center_and_right() {
    let (p, mut sh) = setup("Hi", 400.0, 100.0);
    sh.text.as_mut().unwrap().paragraphs[0].props.align = Some(Align::Center);
    sh.text.as_mut().unwrap().paragraphs[0].props.bullet = Some(Bullet::None);
    let c = lay(&p, &sh, 400.0, 100.0);
    sh.text.as_mut().unwrap().paragraphs[0].props.align = Some(Align::Right);
    let r = lay(&p, &sh, 400.0, 100.0);
    let cx = c.lines[0].caret_x[0];
    let rx = r.lines[0].caret_x[0];
    assert!(rx > cx && cx > 100.0, "center {cx} right {rx}");
}

#[test]
fn empty_and_hostile_bodies_never_panic() {
    let (p, mut sh) = setup("", 10.0, 10.0);
    let _ = lay(&p, &sh, 10.0, 10.0);
    let _ = lay(&p, &sh, 0.0, 0.0);
    let t = sh.text.as_mut().unwrap();
    t.paragraphs = vec![Paragraph {
        runs: vec![
            Run::new("\t\u{b}\u{b}🙂 ﷽ 日本語"),
            Run { text: String::new(), props: Default::default(), kind: deckcraft_model::text::RunKind::Break },
        ],
        ..Default::default()
    }];
    t.paragraphs[0].props.margin_left = Some(-50.0);
    t.paragraphs[0].props.indent = Some(f64::NAN);
    t.paragraphs[0].props.line_spacing = Some(Spacing::Pct(-3.0));
    t.body.columns = Some(1000);
    let l = lay(&p, &sh, 50.0, 50.0);
    let _ = l.hit(Point::new(-100.0, 1e9));
    let _ = l.caret(Pos { para: 99, ch: 99 });
    let _ = l.selection_rects(Pos { para: 5, ch: 0 }, Pos { para: 0, ch: 3 });
}

#[test]
fn autonum_formats() {
    assert_eq!(autonum("arabicPeriod", 3), "3.");
    assert_eq!(autonum("romanUcPeriod", 4), "IV.");
    assert_eq!(autonum("alphaLcParenR", 2), "b)");
    assert_eq!(autonum("arabicParenBoth", 1), "(1)");
    assert_eq!(autonum("alphaUcPeriod", 27), "AA.");
}

#[test]
fn numbered_list_counts() {
    let (p, mut sh) = setup("a\nb\nc", 400.0, 300.0);
    for para in &mut sh.text.as_mut().unwrap().paragraphs {
        para.props.bullet = Some(Bullet::AutoNum { scheme: "arabicPeriod".into(), start_at: 1 });
    }
    let l = lay(&p, &sh, 400.0, 300.0);
    assert!(l.runs.len() >= 6);
}

// ---------- Bidirectional text and Arabic typography ----------

const MIXED: &str = "سنة 2024 (Retrait) فقط";

/// A one-paragraph body with no bullet, `rtl` and `align` set.
fn bidi_setup(runs: Vec<Run>, w: f64, rtl: bool, align: Align) -> (Presentation, Shape) {
    let (p, mut sh) = setup("x", w, 300.0);
    let t = sh.text.as_mut().unwrap();
    t.paragraphs = vec![Paragraph { runs, ..Default::default() }];
    let pp = &mut t.paragraphs[0].props;
    pp.rtl = Some(rtl);
    pp.align = Some(align);
    pp.bullet = Some(Bullet::None);
    pp.margin_left = Some(0.0);
    pp.indent = Some(0.0);
    (p, sh)
}

/// Is any face able to draw Arabic here (installed or bundled)? Shaping assertions that need
/// real Arabic glyphs are skipped without one.
fn arabic_face() -> Option<Arc<FontFace>> {
    let db = FontDb::global();
    let f = db.cascade(&['\u{0633}'], &[db.face("Inter", "Regular")], "Regular");
    f.covers('\u{0633}').then_some(f)
}

#[test]
fn rtl_mixed_string_is_reordered_per_line() {
    let (p, sh) = bidi_setup(vec![Run::new(MIXED)], 2000.0, true, Align::Right);
    let l = lay(&p, &sh, 2000.0, 300.0);
    assert_eq!(l.lines.len(), 1);
    let line = &l.lines[0];
    assert!(line.rtl);
    let n = MIXED.chars().count();
    assert_eq!(line.edges.len(), n);
    assert_eq!(line.caret_x.len(), n + 1);
    let mid = |k: usize| (line.edges[k].0 + line.edges[k].1) / 2.0;
    // Visual order right to left: سنة | 2024 | (Retrait) | فقط.
    assert!(mid(0) > mid(4), "first Arabic word right of the number");
    assert!(mid(4) > mid(10), "number right of the Latin word");
    assert!(mid(10) > mid(19), "Latin right of the last Arabic word");
    // Inside the Arabic words, logical order runs right to left; digits and Latin left to right.
    assert!(mid(0) > mid(1) && mid(1) > mid(2));
    assert!(mid(4) < mid(5) && mid(5) < mid(7));
    assert!(mid(10) < mid(16));
    // The brackets sit outside "Retrait": "(" (logically first) on its right.
    assert!(mid(9) > mid(16) && mid(17) < mid(10));
    // Right-aligned at the start edge S = W - rIns; the downstream caret at 0 is that edge.
    let right_edge = l.inner.x1;
    assert!((line.extent.1 - right_edge).abs() < 0.5, "{} vs {right_edge}", line.extent.1);
    assert!((line.caret_x[0] - line.extent.1).abs() < 0.01);
    // Levels after L1: Arabic odd, digits and Latin even.
    assert_eq!(line.levels[0], 1);
    assert_eq!(line.levels[4], 2);
    assert_eq!(line.levels[10], 2);
}

#[test]
fn rtl_paragraph_starts_at_the_right_with_the_bullet_outside() {
    let (p, mut sh) = bidi_setup(vec![Run::new("مرحبا بكم")], 400.0, true, Align::Right);
    let pp = &mut sh.text.as_mut().unwrap().paragraphs[0].props;
    pp.bullet = Some(Bullet::Char { char: "•".into() });
    pp.margin_left = Some(36.0);
    pp.indent = Some(-36.0);
    let l = lay(&p, &sh, 400.0, 300.0);
    let line = &l.lines[0];
    // Text starts marL from the right edge; the bullet hangs at S - (marL + indent) = S.
    let s_edge = l.inner.x1;
    assert!((line.extent.1 - (s_edge - 36.0)).abs() < 0.5, "text right {} expected {}", line.extent.1, s_edge - 36.0);
    let bullet = l.runs.iter().find(|r| r.chars == (0, 0)).expect("bullet run");
    let bx = bullet.glyphs[0].1;
    assert!(bx > line.extent.1 - 0.5, "bullet {bx} right of the text {}", line.extent.1);
    assert!(bx < s_edge, "bullet inside the box");
    // An LTR paragraph keeps the bullet on the left.
    sh.text.as_mut().unwrap().paragraphs[0].props.rtl = Some(false);
    sh.text.as_mut().unwrap().paragraphs[0].props.align = Some(Align::Left);
    let l = lay(&p, &sh, 400.0, 300.0);
    let bullet = l.runs.iter().find(|r| r.chars == (0, 0)).expect("bullet run");
    assert!(bullet.glyphs[0].1 < l.lines[0].extent.0);
}

#[test]
fn rtl_left_and_center_alignment() {
    let (p, sh) = bidi_setup(vec![Run::new("سلام")], 400.0, true, Align::Left);
    let l = lay(&p, &sh, 400.0, 300.0);
    assert!((l.lines[0].extent.0 - l.inner.x0).abs() < 0.5, "algn=l is flush left in RTL too");
    let (p, sh) = bidi_setup(vec![Run::new("سلام")], 400.0, true, Align::Center);
    let l = lay(&p, &sh, 400.0, 300.0);
    let c = (l.lines[0].extent.0 + l.lines[0].extent.1) / 2.0;
    assert!((c - (l.inner.x0 + l.inner.x1) / 2.0).abs() < 0.5);
}

#[test]
fn bidi_caret_stops_round_trip_through_hit_testing() {
    for rtl in [true, false] {
        let (p, sh) = bidi_setup(vec![Run::new(MIXED)], 2000.0, rtl, if rtl { Align::Right } else { Align::Left });
        let l = lay(&p, &sh, 2000.0, 300.0);
        let line = &l.lines[0];
        for (x, ch, aff) in line.visual_stops() {
            let (cx, ..) = l.caret_at(Pos { para: 0, ch }, aff).unwrap();
            assert!((cx - x).abs() < 0.01, "stop {ch} {aff:?}: caret {cx} vs stop {x}");
        }
        // Clicking just inside either side of each grapheme puts the caret on that side.
        for (a, b) in line.graphemes() {
            let Some((x0, x1)) = line.grapheme_box(a, b) else { continue };
            if x1 - x0 < 1.0 {
                continue;
            }
            let y = (line.top + line.bottom) / 2.0;
            for x in [x0 + 0.2, x1 - 0.2] {
                let (pos, aff) = l.hit_affinity(Point::new(x, y));
                let (cx, ..) = l.caret_at(pos, aff).unwrap();
                let side = if x < (x0 + x1) / 2.0 { x0 } else { x1 };
                assert!((cx - side).abs() < 0.01, "rtl={rtl} grapheme {a}: hit {x} -> {pos:?} {aff:?} caret {cx}, expected {side}");
            }
        }
    }
}

#[test]
fn visual_movement_walks_the_line_left_to_right() {
    let (p, sh) = bidi_setup(vec![Run::new(MIXED)], 2000.0, true, Align::Right);
    let l = lay(&p, &sh, 2000.0, 300.0);
    let stops = l.lines[0].visual_stops();
    let (x0, ch0, aff0) = stops[0];
    let (mut pos, mut aff) = (Pos { para: 0, ch: ch0 }, aff0);
    let mut xs = vec![x0];
    for _ in 0..200 {
        let (np, na) = l.move_caret(pos, aff, true, MoveMode::Visual);
        if (np, na) == (pos, aff) {
            break;
        }
        (pos, aff) = (np, na);
        xs.push(l.caret_at(pos, aff).unwrap().0);
    }
    assert!(xs.windows(2).all(|w| w[1] > w[0]), "moving right always moves right: {xs:?}");
    let right = l.lines[0].extent.1;
    assert!((xs.last().unwrap() - right).abs() < 0.01, "reaches the right edge");
    // Right arrow at the right edge of an RTL paragraph's only line goes nowhere.
    assert_eq!(l.move_caret(pos, aff, true, MoveMode::Visual).0, pos);
    // Moving left from the right edge in an RTL paragraph is logically forward.
    let (p1, _) = l.move_caret(Pos { para: 0, ch: 0 }, Affinity::Downstream, false, MoveMode::Visual);
    assert_eq!(p1.ch, 1);
}

#[test]
fn logical_movement_steps_over_harakat() {
    // beh + fatha + teh: the fatha belongs to the beh's grapheme.
    let (p, sh) = bidi_setup(vec![Run::new("\u{0628}\u{064E}\u{062A}")], 400.0, true, Align::Right);
    let l = lay(&p, &sh, 400.0, 300.0);
    let (n1, _) = l.move_caret(Pos { para: 0, ch: 0 }, Affinity::Downstream, true, MoveMode::Logical);
    assert_eq!(n1.ch, 2);
    let (n2, _) = l.move_caret(n1, Affinity::Downstream, false, MoveMode::Logical);
    assert_eq!(n2.ch, 0);
    // No caret stop inside a grapheme.
    assert!(l.lines[0].visual_stops().iter().all(|s| s.1 != 1));
}

#[test]
fn selection_is_the_union_of_visual_boxes() {
    // LTR paragraph with an Arabic word: visually "ab ةنس cd".
    let (p, sh) = bidi_setup(vec![Run::new("ab سنة cd")], 2000.0, false, Align::Left);
    let l = lay(&p, &sh, 2000.0, 300.0);
    // Logical [1, 5) = "b", " ", "س", "ن": two separate boxes on screen.
    let r = l.selection_rects(Pos { para: 0, ch: 1 }, Pos { para: 0, ch: 5 });
    assert_eq!(r.len(), 2, "{r:?}");
    // The whole line is one box.
    assert_eq!(l.selection_rects(Pos { para: 0, ch: 0 }, Pos { para: 0, ch: 9 }).len(), 1);
}

#[test]
fn paint_only_style_changes_keep_arabic_joining() {
    let Some(_) = arabic_face() else {
        eprintln!("skipped: no Arabic face available");
        return;
    };
    let word = "سنة";
    let glyphs = |runs: Vec<Run>| {
        let (p, sh) = bidi_setup(runs, 400.0, true, Align::Right);
        let l = lay(&p, &sh, 400.0, 300.0);
        let mut g: Vec<(u32, f64)> = l.runs.iter().flat_map(|r| r.glyphs.iter().map(|g| (g.0, g.1))).collect();
        g.sort_by(|a, b| a.1.total_cmp(&b.1));
        g.into_iter().map(|g| g.0).collect::<Vec<_>>()
    };
    let whole = glyphs(vec![Run::new(word)]);
    let mut red = Run::new("سن");
    red.props.fill = Some(Fill::solid(deckcraft_model::style::ColorRef::rgb(Rgba::from_hex("C00000").unwrap())));
    let split = glyphs(vec![red, Run::new("ة")]);
    assert_eq!(whole, split, "a colour change inside a word keeps the joined forms");
    // And the joined forms differ from the isolated letters.
    let isolated: Vec<u32> = word.chars().map(|c| arabic_face().unwrap().glyph_for(c)).collect();
    assert_ne!(whole.iter().rev().copied().collect::<Vec<_>>(), isolated);
}

#[test]
fn just_low_stretches_arabic_with_kashida() {
    if arabic_face().is_none() {
        eprintln!("skipped: no Arabic face available");
        return;
    }
    // Tatweel glyphs drawn, counted in each run's own face (the one the layout picked).
    let tatweels = |l: &TextLayout| {
        l.runs
            .iter()
            .map(|r| {
                let tg = r.face.glyph_for('\u{0640}');
                if tg == 0 { 0 } else { r.glyphs.iter().filter(|g| g.0 == tg).count() }
            })
            .sum::<usize>()
    };
    let text = "بسم الله الرحمن الرحيم نستعين به ونستغفره ونتوب إليه";
    let (p, sh) = bidi_setup(vec![Run::new(text)], 220.0, true, Align::JustLow);
    let l = lay(&p, &sh, 220.0, 600.0);
    assert!(l.lines.len() >= 2);
    let first = &l.lines[0];
    let w = l.inner.width();
    assert!((first.extent.1 - first.extent.0 - w).abs() < 0.5, "first line fills the box: {:?} vs {w}", first.extent);
    if l.runs.iter().all(|r| r.face.glyph_for('\u{0640}') == 0) {
        eprintln!("skipped: the Arabic face has no tatweel");
        return;
    }
    assert!(tatweels(&l) > 0, "kashida drawn");
    // The last line isn't stretched and sits at the start (right) edge.
    let last = l.lines.last().unwrap();
    assert!((last.extent.1 - l.inner.x1).abs() < 0.5);
    // Plain justify uses spaces: no tatweel.
    let (p, sh) = bidi_setup(vec![Run::new(text)], 220.0, true, Align::Justify);
    let l = lay(&p, &sh, 220.0, 600.0);
    assert_eq!(tatweels(&l), 0);
}

#[test]
fn wrapped_rtl_lines_break_in_logical_order() {
    let text = "واحد اثنان ثلاثة أربعة خمسة ستة سبعة ثمانية تسعة عشرة";
    let (p, sh) = bidi_setup(vec![Run::new(text)], 160.0, true, Align::Right);
    let l = lay(&p, &sh, 160.0, 600.0);
    assert!(l.lines.len() >= 2);
    // Lines cover the text in logical order: the first line holds the first words.
    assert_eq!(l.lines[0].start, 0);
    for w in l.lines.windows(2) {
        assert!(w[1].start >= w[0].end);
    }
    for line in &l.lines {
        assert!(line.extent.0 >= l.inner.x0 - 0.5, "line fits: {:?}", line.extent);
        assert!((line.extent.1 - l.inner.x1).abs() < 0.5, "flush right");
    }
    // Upstream affinity at a wrap keeps the caret on the earlier line.
    let end0 = l.lines[0].end;
    assert_eq!(l.line_of_affine(Pos { para: 0, ch: end0 }, Affinity::Upstream), Some(0));
}

#[test]
fn hostile_bidi_input_never_panics() {
    let nasty = format!("{}\t\u{b}\u{202E}abc\u{2069}{}(]))[ ١٢٣ \u{0640}\u{064E}\u{064E}\u{200D}", "\u{202B}".repeat(200), "\u{2067}".repeat(130));
    for rtl in [true, false] {
        for align in [Align::Left, Align::Right, Align::Center, Align::Justify, Align::JustLow, Align::Distributed] {
            let (p, sh) = bidi_setup(vec![Run::new(nasty.clone()), Run::new(MIXED)], 60.0, rtl, align);
            let l = lay(&p, &sh, 60.0, 60.0);
            for line in &l.lines {
                assert_eq!(line.caret_x.len(), line.end - line.start + 1);
                assert_eq!(line.edges.len(), line.end - line.start);
            }
            let _ = l.hit_affinity(Point::new(-1e9, 1e9));
            let _ = l.selection_rects(Pos { para: 0, ch: 0 }, Pos { para: 0, ch: 9999 });
            let mut pos = (Pos::default(), Affinity::Downstream);
            for i in 0..50 {
                pos = l.move_caret(pos.0, pos.1, i % 3 != 0, if i % 2 == 0 { MoveMode::Visual } else { MoveMode::Logical });
            }
            let _ = l.caret_at(Pos { para: 7, ch: 7 }, Affinity::Upstream);
        }
    }
}

/// Width of `word` laid out in a run with `baseline` (0 = normal, 0.3 = superscript …).
fn run_width(word: &str, baseline: Option<f64>) -> f64 {
    let mut run = Run::new(word);
    run.props.size = Some(24.0);
    run.props.baseline = baseline;
    let (p, sh) = bidi_setup(vec![run], 2000.0, false, Align::Left);
    let l = lay(&p, &sh, 2000.0, 300.0);
    let line = &l.lines[0];
    line.caret_x.last().unwrap() - line.caret_x.first().unwrap()
}

#[test]
fn superscript_and_subscript_advance_at_their_drawn_size() {
    // The glyphs are drawn at two thirds of the size; advancing by the full size spread the
    // letters apart ("s u p e r").
    let normal = run_width("superscript", None);
    for b in [Some(0.3), Some(-0.25)] {
        let w = run_width("superscript", b);
        assert!((w / normal - SUB_SUPER).abs() < 0.02, "{b:?}: {w} vs {normal}");
    }
}

#[test]
fn superscript_glyphs_sit_inside_their_cells() {
    let mut run = Run::new("super");
    run.props.size = Some(24.0);
    run.props.baseline = Some(0.3);
    let (p, sh) = bidi_setup(vec![run], 2000.0, false, Align::Left);
    let l = lay(&p, &sh, 2000.0, 300.0);
    let line = &l.lines[0];
    let r = l.runs.iter().find(|r| !r.cells.is_empty()).unwrap();
    assert!((r.size - 24.0 * SUB_SUPER).abs() < 1e-6);
    for (g, (ci, _)) in r.glyphs.iter().zip(&r.cells) {
        let (x0, x1) = line.edges[*ci - line.start];
        assert!(g.1 >= x0 - 0.01 && g.1 < x1, "glyph at {} outside its cell {x0}..{x1}", g.1);
    }
}

/// A wrapping numbered paragraph with `marL` and `indent`; returns (bullet x, first line's text
/// x, second line's text x).
fn hanging(margin: f64, indent: f64, bullet: Bullet) -> (Option<f64>, f64, f64) {
    let text = "Wrapping words go on and on so the paragraph needs at least two or three lines here";
    let (p, mut sh) = setup(text, 300.0, 300.0);
    let para = &mut sh.text.as_mut().unwrap().paragraphs[0];
    para.props.bullet = Some(bullet);
    para.props.margin_left = Some(margin);
    para.props.indent = Some(indent);
    let l = lay(&p, &sh, 300.0, 300.0);
    assert!(l.lines.len() >= 2, "{} lines", l.lines.len());
    let bx = l.runs.iter().find(|r| r.cells.is_empty() && !r.glyphs.is_empty()).map(|r| r.glyphs[0].1);
    (bx, l.lines[0].caret_x[0], l.lines[1].caret_x[0])
}

#[test]
fn a_hang_past_the_box_edge_keeps_its_width_on_the_first_line() {
    // PowerPoint, marL 0 / indent -36pt: the number sits at the box's edge, the first line's text
    // 36pt in, and the wrapped lines at the margin (observed with PowerPoint's slide export).
    let num = || Bullet::AutoNum { scheme: "arabicPeriod".into(), start_at: 1 };
    let (bx, first, rest) = hanging(0.0, -36.0, num());
    assert!((first - rest - 36.0).abs() < 0.5, "first {first} rest {rest}");
    assert!((bx.unwrap() - rest).abs() < 0.5, "bullet {bx:?} rest {rest}");
    // marL 18 / indent -54: number at the edge, first line at 54, the rest at 18.
    let (bx, first, rest) = hanging(18.0, -54.0, num());
    assert!((first - rest - 36.0).abs() < 0.5, "first {first} rest {rest}");
    assert!((rest - bx.unwrap() - 18.0).abs() < 0.5, "bullet {bx:?} rest {rest}");
    // A character bullet behaves the same.
    let (_, first, rest) = hanging(0.0, -36.0, Bullet::Char { char: "•".into() });
    assert!((first - rest - 36.0).abs() < 0.5, "first {first} rest {rest}");
    // Without a bullet the first line just starts at the edge.
    let (_, first, rest) = hanging(0.0, -36.0, Bullet::None);
    assert!((first - rest).abs() < 0.5, "first {first} rest {rest}");
}

#[test]
fn an_ordinary_hang_still_puts_the_text_at_the_margin() {
    let (bx, first, rest) = hanging(36.0, -36.0, Bullet::AutoNum { scheme: "arabicPeriod".into(), start_at: 1 });
    assert!((first - rest).abs() < 0.5, "first {first} rest {rest}");
    assert!((rest - bx.unwrap() - 36.0).abs() < 0.5);
}

#[test]
fn single_spaced_lines_are_1_2_times_the_size_apart_whatever_the_font() {
    // PowerPoint spaces single-spaced lines 1.2 × the font size apart in every font, with the
    // baseline where the font's Windows ascent/descent split puts it (observed on twenty fonts).
    for size in [18.0, 40.0] {
        let mut run = Run::new("HHH");
        run.props.size = Some(size);
        let (p, mut sh) = bidi_setup(vec![run], 2000.0, false, Align::Left);
        let t = sh.text.as_mut().unwrap();
        t.body.autofit = Some(AutoFit::None);
        let second = t.paragraphs[0].clone();
        t.paragraphs.push(second.clone());
        t.paragraphs.push(second);
        for para in &mut t.paragraphs {
            para.props.line_spacing = Some(Spacing::Pct(1.0));
            para.props.space_before = Some(Spacing::Pts(0.0));
            para.props.space_after = Some(Spacing::Pts(0.0));
        }
        let l = lay(&p, &sh, 2000.0, 600.0);
        assert_eq!(l.lines.len(), 3);
        // Each line is 1.2 × the size tall (the face is not compared across lines: another
        // test's thread can load the family between them, see #252).
        for line in &l.lines {
            assert!((line.bottom - line.top - size * LINE_PITCH).abs() < 1e-6, "{size}: {}", line.bottom - line.top);
        }
        let face = &l.runs[0].face;
        let first = &l.lines[0];
        assert!((first.baseline - first.top - size * LINE_PITCH * face.line_split).abs() < 1e-6);
    }
}

/// The lines of `text` in an Arial-free (whatever face resolves) box `w` wide, as strings.
fn wrapped(text: &str, w: f64, set: impl Fn(&mut ParaProps)) -> Vec<String> {
    let (p, mut sh) = bidi_setup(vec![Run::new(text)], w, false, Align::Left);
    let t = sh.text.as_mut().unwrap();
    t.body.inset_l = Some(0.0);
    t.body.inset_r = Some(0.0);
    set(&mut t.paragraphs[0].props);
    let l = lay(&p, &sh, w, 600.0);
    let chars: Vec<char> = text.chars().collect();
    l.lines.iter().map(|li| chars[li.start..li.end].iter().collect()).collect()
}

/// Width of `text` on one line.
fn one_line(text: &str) -> f64 {
    let (p, sh) = bidi_setup(vec![Run::new(text)], 5000.0, false, Align::Left);
    let l = lay(&p, &sh, 5000.0, 300.0);
    l.lines[0].caret_x.last().unwrap() - l.lines[0].caret_x[0]
}

#[test]
fn a_word_does_not_break_after_a_slash_or_backslash() {
    // PowerPoint (latinLnBrk off, its default) keeps "alpha/beta/gamma" whole and only breaks it
    // where it no longer fits; Unicode line breaking would allow a break after each slash.
    for sep in ["/", "\\"] {
        let text = ["alpha", "beta", "gamma", "delta", "epsilon"].join(sep);
        let w = one_line(&text) * 0.6;
        let lines = wrapped(&text, w, |_| {});
        assert!(lines.len() >= 2, "{lines:?}");
        assert!(lines[0].chars().count() > "alpha/beta".len(), "filled to the edge, not cut at a separator: {lines:?}");
    }
    // latinLnBrk on: Unicode's break after a slash is allowed again.
    let text = "alpha/beta/gamma/delta/epsilon";
    let lines = wrapped(text, one_line(text) * 0.6, |p| p.latin_line_break = Some(true));
    assert!(lines[0].ends_with('/'), "{lines:?}");
    // A slash followed by a space still breaks, and hyphens still break.
    let lines = wrapped("alpha-beta-gamma-delta-epsilon", one_line("alpha-beta-gamma-") + 1.0, |_| {});
    assert_eq!(lines[0], "alpha-beta-gamma-");
}

#[test]
fn korean_wraps_between_words() {
    let text = "한국어 안녕하세요 반갑습니다 감사합니다";
    let w = one_line("한국어 안녕하세요 반") + 0.5;
    let lines = wrapped(text, w, |_| {});
    assert_eq!(lines.iter().map(|l| l.trim_end()).collect::<Vec<_>>(), ["한국어 안녕하세요", "반갑습니다", "감사합니다"], "{lines:?}");
    // Japanese and Chinese still break between characters.
    let lines = wrapped("日本語の文章は空白なしで続きます", one_line("日本語の文章") + 0.5, |_| {});
    assert_eq!(lines[0], "日本語の文章");
}

#[test]
fn closing_punctuation_hangs_past_the_end_with_hanging_punct() {
    // PowerPoint (hangingPunct on, as its default text styles set it) lets "。" hang past the
    // edge instead of pulling the character before it onto the next line.
    let text = "白なしで続きます。句読点も含めて。";
    let w = one_line("白なしで続きます") + 0.5;
    let lines = wrapped(text, w, |p| p.hanging_punct = Some(true));
    assert_eq!(lines[0], "白なしで続きます。", "{lines:?}");
    // Without it, "。" can't start a line, so "す。" moves down together.
    let lines = wrapped(text, w, |p| p.hanging_punct = Some(false));
    assert_eq!(lines[0], "白なしで続きま", "{lines:?}");
}

#[test]
fn emoji_take_a_colour_face_when_one_is_installed() {
    // PowerPoint draws emoji (and ZWJ sequences, skin tones, flags) from the colour emoji font
    // even when the run's font has a plain glyph for some of them. Skipped without an installed
    // colour emoji font.
    let db = FontDb::global();
    let probe = db.cascade(&['😀'], &[], "Regular");
    if !probe.is_color() {
        return;
    }
    for s in ["😀", "👍🏽", "❤\u{FE0F}", "👨\u{200D}👩\u{200D}👧"] {
        let (p, sh) = bidi_setup(vec![Run::new(format!("a{s}b"))], 2000.0, false, Align::Left);
        let l = lay(&p, &sh, 2000.0, 300.0);
        let emoji = l.runs.iter().find(|r| r.cells.iter().any(|(_, c)| !c.is_ascii())).unwrap();
        assert!(emoji.face.is_color(), "{s:?} drawn with {}", emoji.face.family);
        assert!(l.runs.iter().filter(|r| r.cells.iter().all(|(_, c)| c.is_ascii())).all(|r| !r.face.is_color()), "the letters keep a text face");
    }
}

#[test]
fn a_bullet_the_text_font_lacks_falls_back_to_a_face_that_has_it() {
    // circleNumDbPlain numbers ①②…: the text font usually has no circled digits; PowerPoint draws
    // them from a fallback font instead of a missing-glyph box. Skipped when no installed or
    // bundled face has ①.
    let db = FontDb::global();
    if !db.cascade(&['①'], &[], "Regular").covers('①') {
        return;
    }
    let (p, mut sh) = setup("One\nTwo", 600.0, 300.0);
    for para in &mut sh.text.as_mut().unwrap().paragraphs {
        para.props.bullet = Some(Bullet::AutoNum { scheme: "circleNumDbPlain".into(), start_at: 1 });
    }
    let l = lay(&p, &sh, 600.0, 300.0);
    let bullets: Vec<_> = l.runs.iter().filter(|r| r.cells.is_empty()).collect();
    assert_eq!(bullets.len(), 2);
    for b in bullets {
        assert!(b.face.covers('①'), "drawn with {}, which lacks the circled digit", b.face.family);
        assert!(b.glyphs.iter().all(|g| g.0 != 0), "no missing-glyph box");
    }
}

#[test]
fn symbol_font_bullets_map_by_font() {
    // Wingdings 2 code 0xEA is a star, Wingdings 3 0x7D a right-pointing triangle; plain
    // Wingdings keeps its own table (0xA7 is a small square).
    let star = String::from_utf8(vec![0xEF, 0x83, 0xAA]).unwrap(); // U+F0EA
    assert_eq!(map_symbol_bullet(&star, Some("Wingdings 2")), "★");
    assert_eq!(map_symbol_bullet("\u{7d}", Some("Wingdings 3")), "►");
    assert_eq!(map_symbol_bullet("\u{f0a7}", Some("Wingdings")), "▪");
    assert_eq!(map_symbol_bullet("•", Some("Arial")), "•");
    assert!(is_symbol_font("Wingdings 2") && is_symbol_font("Symbol") && !is_symbol_font("Arial"));
}

#[test]
fn an_installed_symbol_font_draws_its_own_bullet() {
    // Where the deck's symbol font is installed, its glyph is drawn (as PowerPoint does), not a
    // look-alike. Skipped without Wingdings 2.
    let db = FontDb::global();
    if !db.has_family("Wingdings 2") {
        return;
    }
    let face = db.face("Wingdings 2", "Regular");
    assert_eq!(symbol_font_char(&face, "\u{ea}", Some("Wingdings 2")), Some('\u{f0ea}'));
    assert_eq!(symbol_font_char(&face, "\u{ea}", Some("Arial")), None);
}
