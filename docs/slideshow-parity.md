# Slide show and presenter view: parity with PowerPoint

> **Last reviewed:** 2026-10-10 · **Last updated:** 2026-10-13 · **Change:** minor (interactive-deck clicks in the slide show; previous: major (created; measured from `crates/ui-egui/src/show.rs` and `crates/anim/src/show.rs` against PowerPoint 16.113's Slide Show tab and menu)) · **Target:** Microsoft PowerPoint for Mac 16.113 (Microsoft 365)
> **Last reviewed:** 2026-10-10 · **Last updated:** 2026-10-13 · **Change:** minor (video seek frame-loss race fixed, upstream #416; previous: major, created; measured from `crates/ui-egui/src/show.rs` and `crates/anim/src/show.rs` against PowerPoint 16.113's Slide Show tab and menu) · **Target:** Microsoft PowerPoint for Mac 16.113 (Microsoft 365)
> **Last reviewed:** 2026-10-10 · **Last updated:** 2026-10-14 · **Change:** minor (keyboard navigation from Presenter View) · **Target:** Microsoft PowerPoint for Mac 16.113 (Microsoft 365)

Presenting is the job PowerPoint is hired for. This doc covers the show window, presenter view,
rehearsal and recording. Transition and animation playback are in
[animation-parity.md](animation-parity.md); display hardware in
[hardware-parity.md](hardware-parity.md). Area score: checklist breadth **66%** (measured), **~60%
ready** (estimated), **10–16 h** to parity.

## Running the show

| Feature | PowerPoint | DeckCraft | Notes |
|---|---|---|---|
| Play from Start / from Current Slide (F5, Shift+F5, Cmd+Enter) | yes | yes | |
| Next/previous by click, arrows, Space, Page Up/Down, Enter, Backspace | yes | yes | #17 reported arrows not working in v0.3.0 AppImage; recheck |
| Go to slide (number + Enter), first/last | yes | yes | |
| Black / white screen (B / W) | yes | yes | |
| End screen, Esc to leave | yes | yes | |
| Step clock: a click after a long pause plays its step (#11) | yes | yes | |
| Hidden slides skipped | yes | yes | |
| Hyperlinks and action settings followed (slides, web, mail, End Show, Last Viewed) | yes | yes | Only `http(s)` and `mailto` open outside the show, by design (`safe_url`) |
| A shape with a triggered animation plays it when clicked; shapes an animation has hidden cannot be clicked; a shape without an action on top of one with an action blocks it; the pointer is a hand over what can be clicked | yes | yes | Click boxes are bounding boxes; "play sound" actions (`ppaction://media`) do nothing yet |
| A slide the show returns to (link, arrow keys) looks as it was left; a triggered animation goes on where it stopped | yes | yes | A main-sequence step still playing when the show leaves is finished |
| "Advance on click" off: a click never leaves the slide (animation steps still play, keys still move on) | yes | yes | |
| Media playback with controls, play across slides, full-screen video | yes | yes | WMA/WMV not decodable |
| Media playback with controls, play across slides, full-screen video | yes | yes | Seeks and loops preserve new-generation video frames and end-of-stream; queue regression and repeated seeks across H.264/HEVC/VP9/AV1 in `crates/media`. WMA/WMV not decodable |
| Animated GIFs play; pause from their corner button | yes | yes (#39) | |
| See All Slides grid during the show | yes | no | |
| Zoom into part of a slide during the show | yes | no | |
| Pen and highlighter ink | yes | pen only (Cmd+P pen, Cmd+A arrow, E erases all ink) | No highlighter, per-stroke eraser, ink colours, or keep/discard annotations prompt |
| Laser pointer | yes | **no** | 1–2 h |
| Subtitles and live captions | yes (cloud speech) | no | AI dimension |
| Custom shows | yes | yes | |
| Set Up Show: loop until Esc, kiosk (browsed at a kiosk), show without animation/narration, pen colour, slides range | yes | yes, most options | |
| Use rehearsed timings | yes | yes | |

## Presenter view

| Feature | PowerPoint | DeckCraft | Notes |
|---|---|---|---|
| Separate presenter window with current slide, next slide, notes | yes | yes (an egui viewport, 1100×700) | |
| Elapsed timer | yes | yes | |
| Pause / reset timer, clock | yes | no | |
| Slide counter, previous/next/end buttons | yes | yes | |
| Keyboard navigation while Presenter View has focus | yes | yes | Shares the audience window's key handling; handled keys are consumed before buttons, and the presenter stays available at the end screen |
| Notes font size, scrolling | yes | scrolling only | |
| Placed on the presenter's display automatically, show on the projector | yes | **no**: opens wherever the OS puts it | Beta blocker, 3–5 h |
| Swap displays | yes | no | |
| Pen, laser, black screen, zoom from presenter view | yes | keys only | |
| See all slides from presenter view | yes | no | |
| Rehearse with Coach (Speaker Coach) | yes (cloud) | no | AI dimension |

## Rehearse and record

| Feature | PowerPoint | DeckCraft |
|---|---|---|
| Rehearse Timings, keep timings | yes | yes |
| Record Slide Show (narration, ink, laser, camera; Cameo) | yes | no |
| Clear timing / narration | yes | timings only |
| Export the recorded show to video | yes | no (see [file-format-parity.md](file-format-parity.md)) |

## Reading View

Reading View in a window with navigation: present (checklist row done).

## Estimates

| Work | Hours |
|---|---|
| Presenter view on the right display, swap displays, pause/reset timer, clock, notes size | 3–5 |
| Laser pointer, highlighter, eraser, keep-annotations prompt | 2–4 |
| See All Slides and zoom during the show | 2–3 |
| Record slide show with narration (no camera) | 5–8 |
| **Total** | **~12–20** (the slide-show row of the feature table carries 10–16 h; recording overlaps the Media row) |

## Revision history

| Date | Change | Summary |
|---|---|---|
| 2026-10-13 | minor | Click handling for interactive decks: triggers, per-slide state kept across visits, visible-shape hit testing with hand cursor, advance on click |
| 2026-10-13 | minor | Video seek handoff no longer discards freshly decoded frames or end-of-stream (upstream #416); deterministic queue and four-codec seek regression tests |
| 2026-10-14 | minor | Presenter View handles slideshow keys in its own viewport, including returning from the end screen and Escape; regression tests cover navigation and avoiding duplicate button activation |
| 2026-10-10 | major | Created: show, presenter view, rehearse and record, compared with PowerPoint 16.113's Slide Show tab and menu |
