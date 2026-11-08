use freya::prelude::*;
use freya::query::QueryStateData;
use freya::router::*;

use super::terminal_backdrop::TerminalBackdrop;
use crate::Route;
use crate::components::Button;
use crate::components::{
    AppNavbar, CodeBlock, CrashDialog, FileDropOverlay, Icon, IconType, OverlayPopup, accept_drop,
    copy_button,
};
use crate::i18n::{t, tf};
use crate::layout::AnimatedAppOutlet;
use oneclient_db::models::ClusterId;

use crate::hooks::{
    ActiveClusterState, BROWSER_COMPAT_DEFAULT, BrowserCompatState, BrowserStateStore,
    BrowserTypeState, DataPackWorldState, use_clusters, use_dispatch, use_game_snapshot,
    use_launcher, use_provide_active_cluster, use_provide_browser_compat,
    use_provide_browser_state, use_provide_browser_type, use_provide_datapack_world, use_splash,
};
use crate::theme::{colors, rounded, text_size};
use oneclient_events::LaunchStage;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(PartialEq)]
pub struct AppShell;

impl Component for AppShell {
    fn render(&self) -> impl IntoElement {
        let active_cluster = use_state(|| None::<ClusterId>);
        use_provide_active_cluster(ActiveClusterState(active_cluster));

        let browser_compat = use_state(|| BROWSER_COMPAT_DEFAULT);
        use_provide_browser_compat(BrowserCompatState(browser_compat));

        let browser_state = use_state(HashMap::new);
        use_provide_browser_state(BrowserStateStore(browser_state));

        let browser_type = use_state(|| "mod".to_string());
        use_provide_browser_type(BrowserTypeState(browser_type));

        let datapack_world = use_state(HashMap::new);
        use_provide_datapack_world(DataPackWorldState(datapack_world));

        // `FileDrop` bubbles so anything a drop zone doesn't `stop_propagation()` lands here
        let mut drop_hovering = use_state(|| false);
        let drop_pending = use_state(Vec::<PathBuf>::new);
        let dropped_modpacks = use_state(Vec::<PathBuf>::new);

        let game = use_game_snapshot();

        let running_cluster = game
            .stages
            .iter()
            .find(|(_, s)| **s == LaunchStage::Running)
            .map(|(id, _)| *id);

        // Peeked so turning the setting back on mid-game doesn't yank the view
        let station = use_dispatch().station();
        use_side_effect_with_deps(&running_cluster, move |running| {
            let open_log = station.peek().settings.settings.open_game_log_on_launch;
            if let Some(cluster_id) = *running
                && open_log
            {
                spawn(async move {
                    let _ = RouterContext::get().push(Route::ProcessLogs { cluster_id });
                });
            }
        });

        rect()
            .vertical()
            .width(Size::fill())
            .height(Size::fill())
            .color(colors::fg_primary())
            .overflow(Overflow::Clip)
            .on_global_file_hover(move |_| drop_hovering.set(true))
            .on_global_file_hover_cancelled(move |_| drop_hovering.set(false))
            .on_file_drop(move |e: Event<FileEventData>| {
                drop_hovering.set(false);
                accept_drop(&e.file_paths, drop_pending, dropped_modpacks);
            })
            .child(AppNavbar)
            .child(AppHomeBackground)
            .child(
                rect()
                    .width(Size::fill())
                    .height(Size::fill())
                    .child(AnimatedRouter::<Route>::new(AnimatedAppOutlet)),
            )
            .child(FileDropOverlay {
                hovering: drop_hovering,
                pending: drop_pending,
                modpacks: dropped_modpacks,
            })
            .maybe_child(
                game.error
                    .clone()
                    .map(|message| LaunchErrorDialog { message }.into_element()),
            )
            .maybe_child(
                game.current_crash()
                    .filter(|_| game.error.is_none())
                    .map(|crash| CrashDialog { crash }.into_element()),
            )
    }
}

pub fn hides_overlay(route: &Route) -> bool {
    matches!(route, Route::Home {})
}

#[derive(PartialEq)]
struct LaunchErrorDialog {
    message: String,
}

impl Component for LaunchErrorDialog {
    fn render(&self) -> impl IntoElement {
        let dispatch = crate::hooks::use_dispatch();
        let close = dispatch.clone();
        let outside_close = dispatch.clone();
        let copy_dispatch = dispatch.clone();
        OverlayPopup::new()
            .on_close(move |_| close.dismiss_game_error())
            .child(
                rect()
                    .width(Size::window_percent(100.))
                    .height(Size::window_percent(100.))
                    .center()
                    .on_press(move |_| outside_close.dismiss_game_error())
                    .child(
                        rect()
                            .vertical()
                            .width(Size::px(560.))
                            .background(crate::theme::glass::glass_over(colors::page_elevated()))
                            .corner_radius(rounded(16.))
                            .padding(Gaps::new_all(24.))
                            .spacing(16.)
                            .border(crate::ui::border_all_color(1., colors::component_border()))
                            .on_press(|e: Event<PressEventData>| e.stop_propagation())
                            .child(
                                rect()
                                    .horizontal()
                                    .cross_align(Alignment::Center)
                                    .spacing(10.)
                                    .child(
                                        Icon::new(IconType::AlertTriangle)
                                            .size(20.)
                                            .color(colors::danger()),
                                    )
                                    .child(
                                        label()
                                            .text(t("oneclient.layout.game_error.title"))
                                            .font_size(text_size(18.))
                                            .font_weight(FontWeight::SEMI_BOLD)
                                            .color(colors::fg_primary()),
                                    ),
                            )
                            .child(
                                label()
                                    .text(t("oneclient.layout.game_error.description"))
                                    .font_size(text_size(13.))
                                    .color(colors::fg_secondary()),
                            )
                            .child(
                                CodeBlock::new(self.message.clone())
                                    .background(colors::page())
                                    .height(260.),
                            )
                            .child(
                                rect()
                                    .horizontal()
                                    .width(Size::fill())
                                    .main_align(Alignment::End)
                                    .spacing(10.)
                                    .child(copy_button(
                                        self.message.clone(),
                                        t("oneclient.layout.game_error.copied"),
                                        copy_dispatch,
                                    ))
                                    .child(
                                        Button::new()
                                            .primary()
                                            .on_press(move |_| dispatch.dismiss_game_error())
                                            .text(t("oneclient.common.close")),
                                    ),
                            ),
                    ),
            )
    }
}

pub(crate) fn appshell_overlay(alpha: f32) -> Rect {
    let alpha = alpha.clamp(0., 1.);
    let scale = |a: u8| (f32::from(a) * alpha).round() as u8;
    let top = if colors::is_light() { 215 } else { 100 };

    rect()
        .width(Size::fill())
        .height(Size::fill())
        .position(Position::new_absolute())
        .interactive(false)
        .layer(Layer::Relative(2))
        .child(
            rect()
                .height(Size::px(200.))
                .width(Size::fill())
                .background(
                    LinearGradient::new()
                        .stop((colors::shade().with_a(scale(top)), 0.))
                        .stop((colors::page().with_a(scale(255)), 95.0)),
                ),
        )
        .child(
            rect()
                .height(Size::fill())
                .width(Size::fill())
                .background(colors::page().with_a(scale(255))),
        )
}

pub const HOME_BACKGROUND_ASSET: &str = "backgrounds/VeloxIsland.webp";

#[derive(PartialEq, Clone, Copy)]
pub struct AppHomeBackground;

impl Component for AppHomeBackground {
    fn render(&self) -> impl IntoElement {
        let clusters_query = use_clusters();
        let launcher = use_launcher();
        let splash = use_splash();

        // The home view is populated once these settle which releases the splash curtain
        let clusters_settled = matches!(
            &*clusters_query.read().state(),
            QueryStateData::Settled { .. }
        );
        let home_ready = clusters_settled && !launcher.fetching;
        use_side_effect_with_deps(&home_ready, move |&ready| {
            if ready {
                let mut flag = splash.home_ready;
                if !*flag.peek() {
                    flag.set(true);
                }
            }
        });

        // VeloxClient: home sits on black with a terminal grid, no version art
        TerminalBackdrop
    }
}

pub(crate) fn navigate_back(at_home: bool) {
    let router = RouterContext::get();
    if router.can_go_back() {
        router.go_back();
    } else if !at_home {
        let _ = router.push(Route::Home {});
    }
}

pub(crate) fn back_button(destination: &str) -> impl IntoElement {
    Button::new()
        .ghost()
        .small()
        .on_press(|_| {
            // Only rendered off home so the history-empty fallback is reachable
            navigate_back(false);
        })
        .margin(Gaps::new(0., 0., 0., 32.))
        .child(Icon::new(IconType::ArrowLeft).size(12.))
        .text(tf("oneclient.layout.back_to", &[&destination]))
}
