//! VeloxClient: the first start opens like a terminal. A window fades in, the
//! VELOX logo is drawn pixel by pixel, a few setup lines type themselves, then
//! "Welcome to VeloxClient" is typed out in large letters with a blinking
//! cursor before the rest of the wizard appears.

use freya::animation::*;
use freya::prelude::*;

use crate::hooks::{has_migration_data, use_migration, use_onboarding_selection};
use crate::i18n::t;
use crate::routes::Route;
use crate::theme::{self, colors, rounded, text_size};
use crate::view::onboarding::onboarding_nav;

/// Whole intro, from the window opening to the welcome line being typed out
const INTRO_MS: u64 = 5400;
const CURSOR_BLINK_MS: u64 = 530;

/// Share of the intro each phase ends at
const WINDOW_OPEN_END: f32 = 0.08;
const LOGO_END: f32 = 0.3;
const BOOT_LINES_END: f32 = 0.56;
const WELCOME_END: f32 = 0.84;

/// VELOX in a 5 x 5 pixel font, drawn with squares so it never depends on
/// which block glyphs a font has
const LOGO: [&str; 5] = [
    "X...X XXXXX X.... .XXX. X...X",
    "X...X X.... X.... X...X .X.X.",
    "X...X XXXX. X.... X...X ..X..",
    ".X.X. X.... X.... X...X .X.X.",
    "..X.. XXXXX XXXXX .XXX. X...X",
];
const LOGO_PIXEL: f32 = 14.;
const LOGO_GAP: f32 = 2.;

const TERMINAL_WIDTH: f32 = 860.;
const LINE_SIZE: f32 = 14.;
const WELCOME_SIZE: f32 = 50.;

#[derive(PartialEq)]
pub struct OnboardingWelcome;

impl Component for OnboardingWelcome {
    fn render(&self) -> impl IntoElement {
        let selection = use_onboarding_selection();
        let migration_query = use_migration();
        let next = if *selection.picks_location.read() {
            Route::OnboardingLocation {}
        } else {
            crate::view::onboarding::after_location(has_migration_data(&migration_query))
        };

        let intro = use_animation(|conf| {
            conf.on_creation(OnCreation::Run);
            AnimNum::new(0., 1.).time(INTRO_MS).function(Function::Linear)
        });
        let cursor = use_animation(|conf| {
            conf.on_creation(OnCreation::Run);
            conf.on_finish(OnFinish::reverse());
            AnimNum::new(0., 1.).time(CURSOR_BLINK_MS).function(Function::Linear)
        });

        let progress = if *selection.reduce_motion.read() {
            1.
        } else {
            intro.get().value()
        };
        let cursor_on = cursor.get().value() > 0.5;
        let rest = phase(progress, WELCOME_END, 1.);

        let mut chrome = selection.chrome_opacity;
        use_side_effect_with_deps(&rest, move |&rest| chrome.set_if_modified(rest));
        use_drop(move || chrome.set_if_modified(1.));

        let opened = phase(progress, 0., WINDOW_OPEN_END);

        rect()
            .vertical()
            .width(Size::fill())
            .height(Size::fill())
            .content(Content::Flex)
            .child(
                rect()
                    .vertical()
                    .width(Size::fill())
                    .height(Size::flex(1.0))
                    .center()
                    .child(
                        rect()
                            .opacity(opened)
                            .scale((0.96 + 0.04 * opened, 0.9 + 0.1 * opened))
                            .child(terminal_window(progress, rest, cursor_on)),
                    ),
            )
            .child(
                rect()
                    .width(Size::fill())
                    .opacity(rest)
                    .child(onboarding_nav(None, next, true)),
            )
    }
}

fn terminal_window(progress: f32, rest: f32, cursor_on: bool) -> impl IntoElement {
    rect()
        .vertical()
        .width(Size::px(TERMINAL_WIDTH))
        .corner_radius(rounded(8.))
        .background(colors::page_elevated())
        .border(
            Border::new()
                .fill(colors::component_border())
                .width(1.)
                .alignment(BorderAlignment::Inner),
        )
        .overflow(Overflow::Clip)
        .child(title_bar())
        .child(
            rect()
                .vertical()
                .width(Size::fill())
                .padding(Gaps::new(20., 24., 24., 24.))
                .spacing(6.)
                .child(pixel_logo(phase(progress, WINDOW_OPEN_END, LOGO_END)))
                .child(rect().height(Size::px(14.)))
                .children(boot_lines(progress))
                .child(rect().height(Size::px(18.)))
                .child(welcome_line(progress, cursor_on))
                .child(
                    label()
                        .text(format!("# {}", t("oneclient.onboarding.welcome.subtitle")))
                        .font_family(theme::PIXEL_FONT)
                        .font_size(text_size(LINE_SIZE))
                        .color(colors::fg_secondary())
                        .opacity(rest),
                ),
        )
}

fn title_bar() -> impl IntoElement {
    rect()
        .horizontal()
        .width(Size::fill())
        .padding(Gaps::new_symmetric(10., 16.))
        .border(
            Border::new()
                .fill(colors::component_border())
                .width(1.)
                .alignment(BorderAlignment::Inner),
        )
        .child(
            label()
                .text("velox@client: ~/setup")
                .font_family(theme::PIXEL_FONT)
                .font_size(text_size(12.))
                .color(colors::fg_secondary()),
        )
}

/// `velox ❯ ./setup` and the `[ ok ]` checks, typed one after another
fn boot_lines(progress: f32) -> Vec<Element> {
    let lines: [(&str, String); 4] = [
        ("velox ❯ ", "./setup".to_string()),
        ("[ ok ] ", format!("system ........... {}", system_name())),
        ("[ ok ] ", "renderer ......... ready".to_string()),
        ("[ ok ] ", "velox pack ....... ready".to_string()),
    ];

    let share = (BOOT_LINES_END - LOGO_END) / lines.len() as f32;
    lines
        .iter()
        .enumerate()
        .filter(|(i, _)| progress >= LOGO_END + share * *i as f32)
        .map(|(i, (prefix, text))| {
            let start = LOGO_END + share * i as f32;
            let prefix_color = if i == 0 {
                colors::brand()
            } else {
                colors::fg_secondary()
            };
            paragraph()
                .font_family(theme::PIXEL_FONT)
                .font_size(text_size(LINE_SIZE))
                .span(Span::new(prefix.to_string()).color(prefix_color))
                .span(Span::new(typed(text, phase(progress, start, start + share))).color(colors::fg_primary()))
                .into_element()
        })
        .collect()
}

/// The logo sweeps in column by column, like a scanline drawing it
fn pixel_logo(drawn: f32) -> impl IntoElement {
    let columns = LOGO[0].chars().count();
    let visible_columns = (drawn * columns as f32).ceil() as usize;

    rect().vertical().spacing(LOGO_GAP).children(LOGO.iter().map(|row| {
        rect()
            .horizontal()
            .spacing(LOGO_GAP)
            .children(row.chars().enumerate().map(|(column, cell)| {
                let lit = cell == 'X' && column < visible_columns;
                // The newest column flashes white before settling to purple
                let fill = match (lit, column + 1 == visible_columns && drawn < 1.) {
                    (true, true) => colors::fg_primary(),
                    (true, false) => colors::brand(),
                    (false, _) => Color::TRANSPARENT,
                };
                rect()
                    .width(Size::px(LOGO_PIXEL))
                    .height(Size::px(LOGO_PIXEL))
                    .background(fill)
                    .into_element()
            }))
            .into_element()
    }))
}

fn welcome_line(progress: f32, cursor_on: bool) -> impl IntoElement {
    let welcome = t("oneclient.onboarding.welcome.title");
    let shown = typed(welcome, phase(progress, BOOT_LINES_END, WELCOME_END));
    // Hidden until the line starts, solid while typing, blinking once complete
    let typing = progress >= BOOT_LINES_END && progress < WELCOME_END;
    let done = progress >= WELCOME_END;
    let cursor = if typing || (done && cursor_on) {
        colors::brand()
    } else {
        Color::TRANSPARENT
    };

    paragraph()
        .font_family(theme::PIXEL_FONT)
        .font_size(text_size(WELCOME_SIZE))
        .span(Span::new(shown).color(colors::fg_primary()))
        .span(Span::new("█").color(cursor))
}

/// 0 before `start`, 1 after `end`, linear in between
fn phase(progress: f32, start: f32, end: f32) -> f32 {
    ((progress - start) / (end - start)).clamp(0., 1.)
}

/// The first `share` of `text`, by characters so umlauts never split
fn typed(text: &str, share: f32) -> String {
    let count = text.chars().count();
    let shown = (share * count as f32).round() as usize;
    text.chars().take(shown.min(count)).collect()
}

fn system_name() -> &'static str {
    if cfg!(all(target_os = "macos", target_arch = "aarch64")) {
        "macos · apple silicon"
    } else if cfg!(target_os = "macos") {
        "macos · intel"
    } else if cfg!(target_os = "windows") {
        "windows"
    } else {
        "linux"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn typing_shows_a_growing_prefix() {
        assert_eq!(typed("Welcome", 0.), "");
        assert_eq!(typed("Welcome", 0.5), "Welc");
        assert_eq!(typed("Welcome", 1.), "Welcome");
    }

    #[test]
    fn typing_never_splits_an_umlaut() {
        assert_eq!(typed("Grüß", 0.5), "Gr");
        assert_eq!(typed("Grüß", 0.75), "Grü");
    }

    #[test]
    fn logo_rows_are_equally_wide() {
        let width = LOGO[0].chars().count();
        assert!(LOGO.iter().all(|row| row.chars().count() == width));
    }

    #[test]
    fn phases_are_clamped() {
        assert_eq!(phase(0.0, 0.2, 0.4), 0.);
        assert!((phase(0.3, 0.2, 0.4) - 0.5).abs() < 1e-5);
        assert_eq!(phase(0.9, 0.2, 0.4), 1.);
    }
}
