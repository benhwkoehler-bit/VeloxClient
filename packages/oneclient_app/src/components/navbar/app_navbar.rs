use freya::{
    animation::*,
    prelude::*,
    router::{RouterContext, use_route},
};

use super::side_rail::SideRail;
use crate::theme::{rounded, text_size};
use crate::{
    Route,
    components::{Avatar, Button, Icon, IconType, cluster_title, use_home_clusters},
    hooks::{try_default_account, use_current_account, use_dispatch, use_notifications_snapshot},
    i18n::t,
    theme::{self, colors},
};

const TOPBAR_INTRO_MS: u64 = 460;
const TOPBAR_SIDE_PADDING_PX: f32 = 16.;
const CHIP_HEIGHT_PX: f32 = 40.;
const CHIP_PADDING_PX: f32 = 12.;
const CHIP_AVATAR_INSET_PX: f32 = 4.;
const STATUS_DOT_PX: f32 = 7.;
const TITLE_SIZE: f32 = 15.;
const PATH_SIZE: f32 = 11.;
const CHIP_TEXT_SIZE: f32 = 12.;
const NAV_BUTTON_PX: f32 = 32.;
const HOME_SEGMENT: &str = "home";

/// VeloxClient: icon rail on the left plus a full top bar with title, version and account
#[derive(PartialEq)]
pub struct Navbar;

impl Component for Navbar {
    fn render(&self) -> impl IntoElement {
        rect()
            .width(Size::px(0.))
            .height(Size::px(0.))
            .position(Position::new_absolute().top(0.).left(0.))
            .layer(Layer::OverlayLevel(2))
            .child(SideRail)
            .child(TopBar)
    }
}

#[derive(PartialEq)]
struct TopBar;

impl Component for TopBar {
    fn render(&self) -> impl IntoElement {
        let route = use_route::<Route>();
        let intro = use_animation(|conf| {
            conf.on_creation(OnCreation::Run);
            AnimNum::new(0., 1.)
                .time(TOPBAR_INTRO_MS)
                .ease(Ease::Out)
                .function(Function::Cubic)
        });
        let eased = intro.get().value();
        let (parent, leaf) = route_path(&route.to_string());

        rect()
            .width(Size::window_percent(100.))
            .height(Size::px(theme::TOPBAR_HEIGHT_PX))
            .position(Position::new_absolute().top(0.).left(0.))
            .padding(Gaps::new(0., 0., 0., theme::RAIL_WIDTH_PX))
            .child(
                rect()
                    .width(Size::fill())
                    .height(Size::fill())
                    .background(colors::page_elevated())
                    .border(
                        Border::new()
                            .fill(colors::component_border())
                            .width(BorderWidth {
                                top: 0.,
                                right: 0.,
                                bottom: 1.,
                                left: 0.,
                            })
                            .alignment(BorderAlignment::Inner),
                    )
                    .child(
                        rect()
                            .window_drag()
                            .width(Size::fill())
                            .height(Size::fill())
                            .position(Position::new_absolute().top(0.).left(0.)),
                    )
                    .child(
                        rect()
                            .horizontal()
                            .width(Size::fill())
                            .height(Size::fill())
                            .content(Content::Flex)
                            .cross_align(Alignment::Center)
                            .spacing(10.)
                            .padding(Gaps::new_symmetric(0., TOPBAR_SIDE_PADDING_PX))
                            .opacity(eased)
                            .child(history_buttons())
                            .child(title_block(parent, leaf))
                            .child(notification_button())
                            .child(VersionChip)
                            .child(AccountChip)
                            .child(super::window_controls()),
                    ),
            )
    }
}

fn history_buttons() -> impl IntoElement {
    let router = RouterContext::get();
    let (can_back, can_forward) = (router.can_go_back(), router.can_go_forward());
    let nav_button = |icon, enabled, on_press: fn()| {
        Button::new()
            .ghost()
            .icon()
            .width(Size::px(NAV_BUTTON_PX))
            .height(Size::px(NAV_BUTTON_PX))
            .enabled(enabled)
            .on_press(move |_| on_press())
            .child(Icon::new(icon).size(16.))
    };

    rect()
        .horizontal()
        .spacing(2.)
        .child(nav_button(IconType::ArrowLeft, can_back, || {
            RouterContext::get().go_back();
        }))
        .child(nav_button(IconType::ArrowRight, can_forward, || {
            RouterContext::get().go_forward();
        }))
}

fn title_block(parent: String, leaf: String) -> impl IntoElement {
    let version = format!("  v{}", env!("CARGO_PKG_VERSION"));
    rect()
        .vertical()
        .width(Size::flex(1.0))
        .spacing(3.)
        .child(
            label()
                .text(oneclient_common::brand::APP_NAME.to_lowercase())
                .font_family(theme::PIXEL_FONT)
                .font_size(text_size(TITLE_SIZE))
                .color(colors::fg_primary()),
        )
        .child(
            paragraph()
                .font_family(theme::PIXEL_FONT)
                .font_size(text_size(PATH_SIZE))
                .max_lines(1)
                .span(Span::new(format!("~/{parent}")).color(colors::fg_secondary()))
                .span(Span::new(leaf).color(colors::brand()))
                .span(Span::new(version).color(colors::fg_secondary())),
        )
}

fn notification_button() -> impl IntoElement {
    let unread = use_notifications_snapshot().unread_count();
    let dispatch = use_dispatch();
    super::navbar_button(None)
        .tooltip(t("oneclient.components.navbar.notifications"))
        .child(notification_bell(unread))
        .on_press(move |_| dispatch.toggle_notification_center())
}

fn notification_bell(unread: usize) -> impl IntoElement {
    rect()
        .width(Size::px(20.))
        .height(Size::px(20.))
        .child(Icon::new(IconType::Bell01).size(20.))
        .maybe_child((unread > 0).then(|| {
            rect()
                .position(Position::new_absolute().top(-4.).right(-4.))
                .width(Size::px(16.))
                .height(Size::px(16.))
                .corner_radius(rounded(8.))
                .background(colors::danger())
                .layer(Layer::Relative(3))
                .center()
                .child(
                    label()
                        .text(if unread > 9 {
                            "9+".to_string()
                        } else {
                            unread.to_string()
                        })
                        .font_size(text_size(10.))
                        .font_weight(FontWeight::SEMI_BOLD)
                        .color(colors::fg_primary()),
                )
                .into_element()
        }))
}

/// `/app/clusters/3/mods?x` → (`clusters/3/`, `mods`), and the bare `/app` reads as `home`
fn route_path(url: &str) -> (String, String) {
    let path = url.split('?').next().unwrap_or_default();
    let trimmed = path.strip_prefix("/app").unwrap_or(path).trim_matches('/');
    if trimmed.is_empty() {
        return (String::new(), HOME_SEGMENT.to_string());
    }
    match trimmed.rsplit_once('/') {
        Some((parent, leaf)) => (format!("{parent}/"), leaf.to_string()),
        None => (String::new(), trimmed.to_string()),
    }
}

fn chip(hovering: bool) -> Rect {
    rect()
        .horizontal()
        .height(Size::px(CHIP_HEIGHT_PX))
        .cross_align(Alignment::Center)
        .spacing(8.)
        .corner_radius(rounded(4.))
        .background(colors::component_bg())
        .border(
            Border::new()
                .fill(if hovering {
                    colors::component_border_hover()
                } else {
                    colors::component_border()
                })
                .width(1.)
                .alignment(BorderAlignment::Inner),
        )
        .cursor(CursorIcon::Pointer)
}

fn chip_text_color(hovering: bool) -> Color {
    if hovering {
        colors::fg_primary()
    } else {
        colors::fg_secondary()
    }
}

/// The version Launch would start; opens the full list
#[derive(PartialEq)]
struct VersionChip;

impl Component for VersionChip {
    fn render(&self) -> impl IntoElement {
        let mut hovering = use_state(|| false);
        let (_, active) = use_home_clusters();
        let Some(cluster) = active else {
            return rect().into_element();
        };

        chip(hovering())
            .padding(Gaps::new_symmetric(0., CHIP_PADDING_PX))
            .on_pointer_over(move |_| hovering.set(true))
            .on_pointer_out(move |_| hovering.set(false))
            .on_press(|_| {
                let _ = RouterContext::get().push(Route::Clusters {});
            })
            .child(
                paragraph()
                    .font_family(theme::PIXEL_FONT)
                    .font_size(text_size(CHIP_TEXT_SIZE))
                    .span(Span::new("# ").color(colors::brand()))
                    .span(
                        Span::new(cluster_title(&cluster).to_lowercase())
                            .color(chip_text_color(hovering())),
                    ),
            )
            .into_element()
    }
}

#[derive(PartialEq)]
struct AccountChip;

impl Component for AccountChip {
    fn render(&self) -> impl IntoElement {
        let current_account = use_current_account();
        let dispatch = use_dispatch();
        let mut hovering = use_state(|| false);

        let account = try_default_account(&current_account);
        let uuid = account
            .as_ref()
            .map(|a| a.id.to_string())
            .unwrap_or_else(|| uuid::Uuid::nil().to_string());
        let name = account.map(|a| a.username).unwrap_or_default();
        let text_color = chip_text_color(hovering());

        chip(hovering())
            .padding(Gaps::new(
                0.,
                CHIP_PADDING_PX,
                0.,
                CHIP_AVATAR_INSET_PX,
            ))
            .on_pointer_over(move |_| hovering.set(true))
            .on_pointer_out(move |_| hovering.set(false))
            .on_press(move |_| dispatch.toggle_control_center())
            .child(Avatar::new(uuid))
            .maybe_child((!name.is_empty()).then(|| {
                label()
                    .text(name)
                    .font_family(theme::PIXEL_FONT)
                    .font_size(text_size(CHIP_TEXT_SIZE))
                    .color(text_color)
                    .into_element()
            }))
            .child(
                rect()
                    .width(Size::px(STATUS_DOT_PX))
                    .height(Size::px(STATUS_DOT_PX))
                    .corner_radius(rounded(STATUS_DOT_PX / 2.))
                    .background(colors::success()),
            )
            .child(Icon::new(IconType::ChevronDown).size(14.).color(text_color))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn home_route_reads_as_home() {
        assert_eq!(route_path("/app"), (String::new(), "home".to_string()));
    }

    #[test]
    fn nested_route_splits_parent_and_leaf() {
        assert_eq!(
            route_path("/app/clusters/3/mods"),
            ("clusters/3/".to_string(), "mods".to_string())
        );
    }

    #[test]
    fn query_string_is_dropped() {
        assert_eq!(
            route_path("/app/browser/2/mod?pick_cluster=true"),
            ("browser/2/".to_string(), "mod".to_string())
        );
    }
}
