mod account;
mod bundles;
mod import;
mod language;
mod location;
mod migration;
mod preferences;
mod renderer;
mod selection;
mod summary;
mod terms;
#[cfg(test)]
pub(crate) mod test_support;
mod welcome;

pub(crate) use selection::{
    archive_selected, default_selection, is_default_bundle, pkg_key, set_archive_selected,
};

pub use account::OnboardingAccount;
pub use bundles::OnboardingBundles;
pub use import::OnboardingImport;
pub use language::OnboardingLanguage;
pub use location::OnboardingLocation;
pub use migration::OnboardingMigration;
pub(crate) use migration::matching_new_cluster_id;
pub use preferences::OnboardingPreferences;
pub use renderer::OnboardingRenderer;
pub use summary::OnboardingSummary;
pub use terms::OnboardingTerms;
pub use welcome::OnboardingWelcome;

use std::sync::atomic::{AtomicUsize, Ordering};

use freya::animation::*;
use freya::prelude::*;
use freya::router::{RouterContext, use_route};

use crate::Route;
use crate::components::{Button, Icon, IconType};
use crate::hooks::{has_migration_data, use_migration, use_onboarding_selection};
use crate::i18n::t;
use crate::theme::{colors, rounded, text_size};
use crate::ui::{border_all_color, entrance_motion_layer};

/// VeloxClient: the renderer step (Pure Metal vs. standard) only exists on Apple Silicon Macs.
pub fn shows_renderer_step() -> bool {
    cfg!(all(target_os = "macos", target_arch = "aarch64"))
}

/// First step after welcome/location. VeloxClient has no terms step, so this is the
/// import step when another install was found, otherwise language.
pub fn after_location(has_migration: bool) -> Route {
    if has_migration {
        Route::OnboardingMigration {}
    } else {
        Route::OnboardingLanguage {}
    }
}

/// Step that comes right before the import/language step.
pub fn before_migration(picks_location: bool) -> Route {
    if picks_location {
        Route::OnboardingLocation {}
    } else {
        Route::OnboardingWelcome {}
    }
}

/// Step after the account step (renderer on Apple Silicon, else bundles).
pub fn after_account() -> Route {
    if shows_renderer_step() {
        Route::OnboardingRenderer {}
    } else {
        Route::OnboardingBundles {}
    }
}

pub fn onboarding_total(has_migration: bool, picks_location: bool) -> usize {
    7 + usize::from(has_migration)
        + usize::from(picks_location)
        + usize::from(shows_renderer_step())
}

pub fn onboarding_step_index(route: &Route, has_migration: bool, picks_location: bool) -> usize {
    let location = usize::from(picks_location);
    let shift = location + usize::from(has_migration);
    let renderer = usize::from(shows_renderer_step());

    match route {
        Route::OnboardingWelcome {} => 0,
        Route::OnboardingLocation {} => 1,
        Route::OnboardingMigration {} => 1 + location,
        Route::OnboardingLanguage {} => 1 + shift,
        Route::OnboardingImport {} => 2 + shift,
        Route::OnboardingAccount {} => 3 + shift,
        Route::OnboardingRenderer {} => 4 + shift,
        Route::OnboardingBundles {} => 4 + shift + renderer,
        Route::OnboardingPreferences {} => 5 + shift + renderer,
        Route::OnboardingSummary {} => 6 + shift + renderer,
        _ => 0,
    }
}

static LAST_STEP: AtomicUsize = AtomicUsize::new(0);

const SLIDE_DISTANCE: f32 = 44.;

pub(crate) fn onboarding_slide(content: impl IntoElement) -> impl IntoElement {
    let route = use_route::<Route>();
    let migration_query = use_migration();
    let selection = use_onboarding_selection();
    let step = onboarding_step_index(
        &route,
        has_migration_data(&migration_query),
        *selection.picks_location.read(),
    );
    let reduce_motion = selection.reduce_motion;

    let direction = use_hook(|| {
        let prev = LAST_STEP.swap(step, Ordering::Relaxed);
        if step >= prev { 1.0_f32 } else { -1.0 }
    });

    let anim = use_animation(|conf| {
        conf.on_creation(OnCreation::Run);
        AnimNum::new(0., 1.)
            .time(320)
            .ease(Ease::Out)
            .function(Function::Cubic)
    });

    let anim_finished = *anim.has_run_yet().read() && !*anim.is_running().read();
    let motion = !*reduce_motion.read();

    use_side_effect_with_deps(&anim_finished, move |&finished| {
        if finished {
            Platform::get().current_window().request_redraw();
        }
    });

    let p = if anim_finished {
        1.0
    } else {
        anim.get().value()
    };
    let slide_x = if motion {
        direction * (1. - p) * SLIDE_DISTANCE
    } else {
        0.
    };
    let opacity = if motion && !anim_finished { p } else { 1. };

    rect()
        .width(Size::fill())
        .height(Size::flex(1.0))
        .child(entrance_motion_layer(slide_x, 0., opacity, content))
        .into_element()
}

pub(crate) fn onboarding_page(
    illustration: impl IntoElement,
    content: impl IntoElement,
    nav: impl IntoElement,
) -> impl IntoElement {
    rect()
        .vertical()
        .width(Size::fill())
        .height(Size::fill())
        .content(Content::Flex)
        .child(onboarding_slide(
            rect()
                .horizontal()
                .width(Size::fill())
                .height(Size::fill())
                .content(Content::Flex)
                .child(
                    rect()
                        .width(Size::flex(1.0))
                        .height(Size::fill())
                        .center()
                        .padding(Gaps::new_all(48.))
                        .child(illustration),
                )
                .child(
                    rect()
                        .vertical()
                        .width(Size::flex(1.0))
                        .height(Size::fill())
                        .main_align(Alignment::Center)
                        .padding(Gaps::new(48., 80., 24., 24.))
                        .child(terminal_panel(content)),
                ),
        ))
        .child(nav)
        .into_element()
}

pub(crate) fn onboarding_illustration(icon: IconType) -> impl IntoElement {
    Icon::new(icon).size(240.).color(colors::fg_strong())
}

pub(crate) fn onboarding_nav(
    back: Option<Route>,
    next: Route,
    next_enabled: bool,
) -> impl IntoElement {
    onboarding_nav_action(back, t("oneclient.common.next"), next_enabled, move |()| {
        let _ = RouterContext::get().replace(next.clone());
    })
}

pub(crate) fn onboarding_nav_action(
    back: Option<Route>,
    next_label: &str,
    next_enabled: bool,
    on_next: impl FnMut(()) + 'static,
) -> impl IntoElement {
    let mut on_next = on_next;

    rect()
        .horizontal()
        .width(Size::fill())
        .main_align(Alignment::End)
        .cross_align(Alignment::Center)
        .spacing(12.)
        .padding(Gaps::new(0., 40., 32., 40.))
        .maybe_child(back.map(|route| {
            Button::new()
                .secondary()
                .width(Size::px(128.))
                .on_press(move |_| {
                    let _ = RouterContext::get().replace(route.clone());
                })
                .text(t("oneclient.common.back"))
                .into_element()
        }))
        .child(
            Button::new()
                .primary()
                .width(Size::px(140.))
                .enabled(next_enabled)
                .on_press(move |_| on_next(()))
                .text(next_label.to_string())
                .child(Icon::new(IconType::ArrowRight).size(16.)),
        )
        .into_element()
}

pub(crate) fn choice_row(
    title: &str,
    subtitle: &str,
    active: bool,
    on_press: impl FnMut(()) + 'static,
) -> Element {
    choice_row_sized(title, subtitle, active, Size::auto(), on_press)
}

pub(crate) fn choice_row_sized(
    title: &str,
    subtitle: &str,
    active: bool,
    height: Size,
    on_press: impl FnMut(()) + 'static,
) -> Element {
    let mut on_press = on_press;
    let border_color = if active {
        colors::fg_primary()
    } else {
        colors::component_border()
    };

    rect()
        .horizontal()
        .width(Size::fill())
        .height(height)
        .content(Content::Flex)
        .cross_align(Alignment::Center)
        .spacing(10.)
        .padding(Gaps::new_symmetric(8., 12.))
        .corner_radius(rounded(8.))
        .border(border_all_color(1., border_color))
        .a11y_role(AccessibilityRole::Button)
        .cursor(CursorIcon::Pointer)
        .on_press(move |_| on_press(()))
        .child(
            rect()
                .vertical()
                .width(Size::flex(1.0))
                .spacing(2.)
                .child(
                    label()
                        .text(title.to_string())
                        .font_size(text_size(13.))
                        .font_weight(FontWeight::MEDIUM)
                        .color(colors::fg_primary()),
                )
                .maybe_child((!subtitle.is_empty()).then(|| {
                    label()
                        .text(subtitle.to_string())
                        .font_size(text_size(11.))
                        .color(colors::fg_primary())
                        .into_element()
                })),
        )
        .maybe_child(active.then(|| Icon::new(IconType::Check).size(15.)))
        .into_element()
}

pub(crate) fn version_chip(
    text: &str,
    active: bool,
    on_press: impl FnMut(()) + 'static,
) -> Element {
    let mut on_press = on_press;
    let (bg, border, fg) = if active {
        (
            colors::brand().with_a(38),
            colors::brand(),
            colors::fg_primary(),
        )
    } else {
        (
            Color::TRANSPARENT,
            colors::component_border(),
            colors::fg_secondary(),
        )
    };

    rect()
        .horizontal()
        .height(Size::px(32.))
        .center()
        .padding(Gaps::new_symmetric(0., 14.))
        .corner_radius(rounded(999.))
        .background(bg)
        .border(border_all_color(1.5, border))
        .a11y_role(AccessibilityRole::Button)
        .cursor(CursorIcon::Pointer)
        .on_press(move |_| on_press(()))
        .child(
            label()
                .text(text.to_string())
                .font_size(text_size(13.))
                .font_weight(FontWeight::MEDIUM)
                .color(fg),
        )
        .into_element()
}

/// VeloxClient: every step's content sits in a terminal window
pub(crate) fn terminal_panel(content: impl IntoElement) -> impl IntoElement {
    rect()
        .vertical()
        .width(Size::fill())
        .corner_radius(rounded(8.))
        .background(colors::page_elevated())
        .border(border_all_color(1., colors::component_border()))
        .overflow(Overflow::Clip)
        .child(
            rect()
                .width(Size::fill())
                .padding(Gaps::new_symmetric(9., 16.))
                .border(border_all_color(1., colors::component_border()))
                .child(
                    label()
                        .text("velox@client: ~/setup")
                        .font_family(crate::theme::PIXEL_FONT)
                        .font_size(text_size(12.))
                        .color(colors::fg_secondary()),
                ),
        )
        .child(
            rect()
                .vertical()
                .width(Size::fill())
                .padding(Gaps::new_all(24.))
                .spacing(24.)
                .child(content),
        )
}

/// `setup --choose-your-renderer` from a step title
fn setup_command(title: &str) -> String {
    let slug: Vec<String> = title
        .split(|c: char| !c.is_alphanumeric())
        .filter(|word| !word.is_empty())
        .map(str::to_lowercase)
        .collect();
    format!("setup --{}", slug.join("-"))
}

pub(crate) fn step_heading(title: &str, subtitle: &str) -> impl IntoElement {
    rect()
        .vertical()
        .width(Size::fill())
        .spacing(8.)
        .child(
            paragraph()
                .font_family(crate::theme::PIXEL_FONT)
                .font_size(text_size(13.))
                .span(Span::new("velox ❯ ").color(colors::brand()))
                .span(Span::new(setup_command(title)).color(colors::fg_secondary())),
        )
        .child(
            paragraph()
                .font_family(crate::theme::PIXEL_FONT)
                .font_size(text_size(32.))
                .span(Span::new("# ").color(colors::brand()))
                .span(Span::new(title.to_string()).color(colors::fg_primary())),
        )
        .child(
            label()
                .text(subtitle.to_string())
                .font_size(text_size(16.))
                .color(colors::fg_secondary()),
        )
        .into_element()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flow(has_migration: bool, picks_location: bool) -> Vec<Route> {
        let mut routes = vec![Route::OnboardingWelcome {}];

        if picks_location {
            routes.push(Route::OnboardingLocation {});
        }

        if has_migration {
            routes.push(Route::OnboardingMigration {});
        }

        routes.extend([
            Route::OnboardingLanguage {},
            Route::OnboardingImport {},
            Route::OnboardingAccount {},
        ]);
        if shows_renderer_step() {
            routes.push(Route::OnboardingRenderer {});
        }
        routes.extend([
            Route::OnboardingBundles {},
            Route::OnboardingPreferences {},
            Route::OnboardingSummary {},
        ]);

        routes
    }

    #[test]
    fn every_run_numbers_its_steps_one_after_another() {
        for has_migration in [false, true] {
            for picks_location in [false, true] {
                let routes = flow(has_migration, picks_location);
                let total = onboarding_total(has_migration, picks_location);

                assert_eq!(
                    routes.len(),
                    total,
                    "the bar is drawn out of {total} but the run has {} steps \
                     (migration: {has_migration}, folder: {picks_location})",
                    routes.len()
                );

                for (position, route) in routes.iter().enumerate() {
                    assert_eq!(
                        onboarding_step_index(route, has_migration, picks_location),
                        position,
                        "{route:?} is numbered off its place in the run \
                         (migration: {has_migration}, folder: {picks_location})"
                    );
                }
            }
        }
    }
}
