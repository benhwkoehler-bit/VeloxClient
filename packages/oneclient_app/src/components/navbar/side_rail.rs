//! VeloxClient: icon rail along the left edge of the app shell

use freya::{
    animation::{AnimColor, *},
    prelude::*,
    router::{RouterContext, use_route},
};

use crate::{
    Route,
    components::{Icon, IconType},
    hooks::{
        settled_or_loading, use_active_cluster_id, use_browser_type, use_clusters, use_dispatch,
    },
    i18n::t,
    theme::{self, colors, rounded, text_size},
    utils::default_cluster,
};

const RAIL_INTRO_MS: u64 = 420;
const LOGO_PX: f32 = 30.;
const ITEM_WIDTH_PX: f32 = 68.;
const ITEM_ICON_PX: f32 = 26.;
const ITEM_LABEL_PX: f32 = 10.;
const MARKER_WIDTH_PX: f32 = 2.;
const MARKER_TOP_PX: f32 = 13.;
const MARKER_HEIGHT_PX: f32 = 36.;
const ITEM_HEIGHT_PX: f32 = 62.;
const PROMPT_TOP_PX: f32 = 12.;
const PROMPT_LEFT_PX: f32 = 10.;
const INDICATOR_GLIDE_MS: u64 = 320;
const INDICATOR_FADE_MS: u64 = 180;
const HOVER_FADE_MS: u64 = 140;
const RAIL_ITEM_COUNT: usize = 6;
const RAIL_TOP_PADDING_PX: f32 = 14.;
const RAIL_BOTTOM_PADDING_PX: f32 = 12.;

const HOME_INDEX: usize = 0;
const VERSIONS_INDEX: usize = 1;
const BROWSE_INDEX: usize = 2;
const WARDROBE_INDEX: usize = 3;
const STATS_INDEX: usize = 4;
const SETTINGS_INDEX: usize = 5;

#[derive(PartialEq, Clone)]
enum RailAction {
    Route(Route),
    ControlCenter,
}

#[derive(PartialEq)]
pub struct SideRail;

impl Component for SideRail {
    fn render(&self) -> impl IntoElement {
        let route = use_route::<Route>();
        let browse = browse_target();
        let item_tops = use_state(|| [0f32; RAIL_ITEM_COUNT]);
        let active_index = active_rail_index(&route);
        let intro = use_animation(|conf| {
            conf.on_creation(OnCreation::Run);
            AnimNum::new(0., 1.)
                .time(RAIL_INTRO_MS)
                .ease(Ease::Out)
                .function(Function::Cubic)
        });
        let eased = intro.get().value();

        let item = |index, icon, label, action| RailItem {
            index,
            icon,
            label,
            active: active_index == Some(index),
            action,
            item_tops,
        };

        rect()
            .vertical()
            .width(Size::px(theme::RAIL_WIDTH_PX))
            .height(Size::window_percent(100.))
            .content(Content::Flex)
            .position(Position::new_absolute().top(0.).left(0.))
            .layer(Layer::OverlayLevel(2))
            .background(colors::page_elevated())
            .border(
                Border::new()
                    .fill(colors::component_border())
                    .width(BorderWidth {
                        top: 0.,
                        right: 1.,
                        bottom: 0.,
                        left: 0.,
                    })
                    .alignment(BorderAlignment::Inner),
            )
            .cross_align(Alignment::Center)
            .padding(Gaps::new(RAIL_TOP_PADDING_PX, 0., RAIL_BOTTOM_PADDING_PX, 0.))
            .spacing(4.)
            .offset_x((eased - 1.0) * theme::RAIL_WIDTH_PX)
            .child(RailIndicator {
                active_index,
                item_tops,
            })
            .child(
                rect()
                    .window_drag()
                    .width(Size::fill())
                    .center()
                    .padding(Gaps::new(4., 0., 18., 0.))
                    .child(
                        Icon::new(IconType::IconLogo)
                            .size(LOGO_PX)
                            .color(colors::brand()),
                    ),
            )
            .child(item(
                HOME_INDEX,
                IconType::Play,
                t("oneclient.components.navbar.home"),
                RailAction::Route(Route::Home {}),
            ))
            .child(item(
                VERSIONS_INDEX,
                IconType::DotsGrid,
                t("oneclient.components.navbar.versions"),
                RailAction::Route(Route::Clusters {}),
            ))
            .child(item(
                BROWSE_INDEX,
                IconType::SearchMd,
                t("oneclient.components.navbar.browse"),
                RailAction::Route(browse),
            ))
            .child(item(
                WARDROBE_INDEX,
                IconType::Brush01,
                t("oneclient.components.navbar.wardrobe"),
                RailAction::Route(Route::Wardrobe {}),
            ))
            .child(item(
                STATS_INDEX,
                IconType::LineChartUp01,
                t("oneclient.components.navbar.stats"),
                RailAction::Route(Route::Stats {}),
            ))
            .child(rect().width(Size::fill()).height(Size::flex(1.0)).window_drag())
            .child(item(
                SETTINGS_INDEX,
                IconType::Settings02,
                t("oneclient.components.navbar.rail.settings"),
                RailAction::ControlCenter,
            ))
    }
}

fn active_rail_index(route: &Route) -> Option<usize> {
    match route {
        Route::Home {} => Some(HOME_INDEX),
        Route::Clusters {} => Some(VERSIONS_INDEX),
        Route::Browser {
            pick_cluster: true, ..
        } => Some(BROWSE_INDEX),
        Route::Wardrobe {} => Some(WARDROBE_INDEX),
        Route::Stats {} => Some(STATS_INDEX),
        _ => None,
    }
}

/// Highlight, edge bar and `>` prompt that glide together to the active entry
#[derive(PartialEq)]
struct RailIndicator {
    active_index: Option<usize>,
    item_tops: State<[f32; RAIL_ITEM_COUNT]>,
}

impl Component for RailIndicator {
    fn render(&self) -> impl IntoElement {
        let target_top = self
            .active_index
            .map(|index| self.item_tops.read()[index])
            .filter(|top| *top > 0.);
        let mut last_top = use_state(|| 0f32);
        if let Some(top) = target_top
            && *last_top.peek() != top
        {
            last_top.set(top);
        }

        let top = use_animation_transition(*last_top.read(), |from, to| {
            AnimNum::new(from, to)
                .time(INDICATOR_GLIDE_MS)
                .ease(Ease::Out)
                .function(Function::Expo)
        })
        .read()
        .value();

        let visible = target_top.is_some();
        let opacity = use_animation_transition(visible, |_, shown| {
            let (start, end) = if shown { (0., 1.) } else { (1., 0.) };
            AnimNum::new(start, end)
                .time(INDICATOR_FADE_MS)
                .ease(Ease::Out)
                .function(Function::Quad)
        })
        .read()
        .value();

        let inset = (theme::RAIL_WIDTH_PX - ITEM_WIDTH_PX) / 2.;
        rect()
            // Absolute children are placed inside the rail's padding
            .position(
                Position::new_absolute()
                    .top(top - RAIL_TOP_PADDING_PX)
                    .left(inset),
            )
            .width(Size::px(ITEM_WIDTH_PX))
            .height(Size::px(ITEM_HEIGHT_PX))
            .corner_radius(rounded(4.))
            .background(colors::component_bg())
            .border(
                Border::new()
                    .fill(colors::component_border_hover())
                    .width(1.)
                    .alignment(BorderAlignment::Inner),
            )
            .shadow(Shadow::new().blur(10.).y(3.).color(Color::BLACK.with_a(120)))
            .opacity(opacity)
            .interactive(false)
            .child(
                rect()
                    .position(Position::new_absolute().top(MARKER_TOP_PX).left(-inset))
                    .width(Size::px(MARKER_WIDTH_PX))
                    .height(Size::px(MARKER_HEIGHT_PX))
                    .background(colors::brand()),
            )
            .child(
                rect()
                    .position(
                        Position::new_absolute()
                            .top(PROMPT_TOP_PX)
                            .left(PROMPT_LEFT_PX),
                    )
                    .child(
                        label()
                            .text(">")
                            .font_family(theme::PIXEL_FONT)
                            .font_size(text_size(ITEM_LABEL_PX))
                            .color(colors::brand()),
                    ),
            )
    }
}

/// Falls back active cluster then most recently played then Versions when none exist
fn browse_target() -> Route {
    let clusters = settled_or_loading(&use_clusters()).unwrap_or_default();
    let active = *use_active_cluster_id().read();
    let package_type = use_browser_type().read().clone();

    match default_cluster(clusters, active) {
        Some(cluster) => Route::Browser {
            cluster_id: cluster.id,
            package_type: crate::view::app::browser::browsable_type(&package_type, &cluster),
            pick_cluster: true,
        },
        None => Route::Clusters {},
    }
}

#[derive(PartialEq)]
struct RailItem {
    index: usize,
    icon: IconType,
    label: &'static str,
    active: bool,
    action: RailAction,
    item_tops: State<[f32; RAIL_ITEM_COUNT]>,
}

impl Component for RailItem {
    fn render(&self) -> impl IntoElement {
        let mut hovering = use_state(|| false);
        let a11y_id = use_a11y();
        let focused = use_focus(a11y_id);
        let dispatch = use_dispatch();

        let active = self.active;
        let action = self.action.clone();
        let highlighted = active || hovering() || focused().is_focused();
        let color = if highlighted {
            colors::fg_primary()
        } else {
            colors::fg_secondary()
        };
        let background = if !active && (hovering() || focused().is_focused()) {
            colors::ghost_overlay_hover()
        } else {
            Color::TRANSPARENT
        };
        let background = use_animation_transition(background, |from, to| {
            AnimColor::new(from, to)
                .time(HOVER_FADE_MS)
                .ease(Ease::Out)
                .function(Function::Quad)
        })
        .read()
        .value();
        let color = use_animation_transition(color, |from, to| {
            AnimColor::new(from, to)
                .time(HOVER_FADE_MS)
                .ease(Ease::Out)
                .function(Function::Quad)
        })
        .read()
        .value();

        let index = self.index;
        let mut item_tops = self.item_tops;

        rect()
            .vertical()
            .width(Size::px(ITEM_WIDTH_PX))
            .height(Size::px(ITEM_HEIGHT_PX))
            .main_align(Alignment::Center)
            .spacing(6.)
            .cross_align(Alignment::Center)
            .corner_radius(rounded(4.))
            .background(background)
            .a11y_id(a11y_id)
            .a11y_focusable(true)
            .a11y_role(AccessibilityRole::Button)
            .cursor(CursorIcon::Pointer)
            .on_sized(move |event: Event<SizedEventData>| {
                let top = event.data().area.origin.y;
                if item_tops.peek()[index] != top {
                    item_tops.write()[index] = top;
                }
            })
            .on_pointer_over(move |_| hovering.set(true))
            .on_pointer_out(move |_| hovering.set(false))
            .on_press(move |e: Event<PressEventData>| {
                e.prevent_default();
                match &action {
                    RailAction::Route(_) if active => {}
                    RailAction::Route(route) => {
                        let _ = RouterContext::get().push(route.clone());
                    }
                    RailAction::ControlCenter => dispatch.toggle_control_center(),
                }
            })
            .child(Icon::new(self.icon).size(ITEM_ICON_PX).color(color))
            .child(
                paragraph()
                    .font_family(theme::PIXEL_FONT)
                    .font_size(text_size(ITEM_LABEL_PX))
                    .span(Span::new(self.label.to_lowercase()).color(color)),
            )
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rail_pages_map_to_their_entries() {
        assert_eq!(active_rail_index(&Route::Home {}), Some(HOME_INDEX));
        assert_eq!(active_rail_index(&Route::Stats {}), Some(STATS_INDEX));
    }

    #[test]
    fn pages_outside_the_rail_have_no_entry() {
        assert_eq!(active_rail_index(&Route::SettingsJava {}), None);
    }
}
