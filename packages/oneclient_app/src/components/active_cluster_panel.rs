use freya::animation::*;
use freya::prelude::*;
use freya::router::RouterContext;

use crate::components::{Button, ClusterFacts, Icon, IconType, cluster_path};
use oneclient_core::clusters::Cluster;

use crate::hooks::{
    settled_or_loading, use_active_cluster_id, use_clusters, use_dispatch, use_game_snapshot,
    use_launcher,
};
use crate::i18n::t;
use crate::routes::Route;
use crate::theme::{self, colors, rounded, text_size};
use crate::utils::sort_clusters_for_home;
use crate::view::app::{launch_button_state, launch_syncing, shared_dir_taken};

const LAUNCH_WIDTH_PX: f32 = 300.;
const LAUNCH_HEIGHT_PX: f32 = 62.;
const PICKER_TOGGLE_WIDTH_PX: f32 = 50.;
const PICKER_GAP_PX: f32 = 8.;
const PICKER_MAX_ITEMS: usize = 8;
const PICKER_ROW_HEIGHT_PX: f32 = 34.;
const PICKER_HEAD_HEIGHT_PX: f32 = 38.;
const INFO_MAX_WIDTH_PX: f32 = 320.;
const LAUNCH_ENTER_MS: u64 = 480;
const LAUNCH_ENTER_RISE_PX: f32 = 28.;
const PICKER_OPEN_MS: u64 = 200;
const PICKER_OPEN_RISE_PX: f32 = 10.;

/// Clusters in home order plus the one the panel shows; seeds the active id on first use
pub fn use_home_clusters() -> (Vec<Cluster>, Option<Cluster>) {
    let clusters_query = use_clusters();
    let mut active_id = use_active_cluster_id();
    let sorted = sort_clusters_for_home(settled_or_loading(&clusters_query).unwrap_or_default());

    if active_id.read().is_none()
        && let Some(first) = sorted.first()
    {
        *active_id.write() = Some(first.id);
    }

    let active = active_id
        .read()
        .and_then(|id| sorted.iter().find(|c| c.id == id).cloned())
        .or_else(|| sorted.first().cloned());
    (sorted, active)
}

pub fn cluster_title(cluster: &Cluster) -> String {
    if cluster.user_created {
        cluster.name.clone()
    } else {
        format!("{} {}", cluster.mc_version, cluster.mc_loader)
    }
}

/// VeloxClient: path, title and facts of the active version in the stage's top-left corner
#[derive(PartialEq)]
pub struct ActiveClusterPanel;

impl Component for ActiveClusterPanel {
    fn render(&self) -> impl IntoElement {
        let (_, active) = use_home_clusters();

        let dep = active.as_ref().map(|c| c.id);
        let intro = use_animation_with_dependencies(&dep, |conf, _| {
            conf.on_creation(OnCreation::Run);
            conf.on_change(OnChange::Rerun);
            AnimNum::new(0., 1.)
                .time(440)
                .ease(Ease::Out)
                .function(Function::Cubic)
        });
        let p = intro.get().value();
        let slide_x = (p - 1.0) * 48.0;

        let Some(cluster) = active else {
            return rect().vertical().child(
                label()
                    .text(t("oneclient.components.active_cluster_panel.empty"))
                    .font_size(text_size(24.))
                    .color(colors::on_art_secondary()),
            );
        };

        rect()
            .vertical()
            .max_width(Size::px(INFO_MAX_WIDTH_PX))
            .cross_align(Alignment::Start)
            .offset_x(slide_x)
            .opacity(p)
            .spacing(6.)
            .child(cluster_path(&cluster))
            .child(ClusterFacts { cluster })
    }
}

/// VeloxClient: wide play button with a version picker toggle, centred at the stage's foot
#[derive(PartialEq)]
pub struct LaunchBar;

impl Component for LaunchBar {
    fn render(&self) -> impl IntoElement {
        let (sorted, active) = use_home_clusters();
        let game = use_game_snapshot();
        let launcher = use_launcher();
        let picker_open = use_state(|| false);
        let enter = use_animation(|conf| {
            conf.on_creation(OnCreation::Run);
            AnimNum::new(0., 1.)
                .time(LAUNCH_ENTER_MS)
                .ease(Ease::Out)
                .function(Function::Cubic)
        });
        let entered = enter.get().value();

        let Some(cluster) = active else {
            return rect().into_element();
        };

        let cluster_id = cluster.id;
        let syncing = launch_syncing(&launcher, cluster.uses_bundles());
        let taken = shared_dir_taken(&game, &sorted, cluster_id);
        let (state_label, enabled) = launch_button_state(&game, cluster_id, syncing, taken);
        let version_line = format!("{} · {}", cluster.mc_version, cluster.mc_loader).to_lowercase();
        let picker = (*picker_open.read()).then(|| {
            VersionPicker {
                clusters: sorted.iter().take(PICKER_MAX_ITEMS).cloned().collect(),
                active_id: cluster_id,
                open: picker_open,
            }
            .into_element()
        });

        rect()
            .horizontal()
            .cross_align(Alignment::Center)
            .spacing(8.)
            .offset_y((1. - entered) * LAUNCH_ENTER_RISE_PX)
            .opacity(entered)
            .child(
                rect()
                    .horizontal()
                    .maybe_child(picker)
                    .child(PlayButton {
                        title: state_label,
                        subtitle: version_line,
                        enabled,
                        cluster_id,
                    })
                    .child(PickerToggle { open: picker_open }),
            )
            .child(cluster_settings_button(cluster_id))
            .into_element()
    }
}

#[derive(PartialEq)]
struct PlayButton {
    title: &'static str,
    subtitle: String,
    enabled: bool,
    cluster_id: i64,
}

impl Component for PlayButton {
    fn render(&self) -> impl IntoElement {
        let mut hovering = use_state(|| false);
        let enabled = self.enabled;
        let cluster_id = self.cluster_id;
        let dispatch = use_dispatch();

        let background = if !enabled {
            colors::brand_disabled()
        } else if hovering() {
            colors::brand_hover()
        } else {
            colors::brand()
        };

        rect()
            .vertical()
            .width(Size::px(LAUNCH_WIDTH_PX))
            .height(Size::px(LAUNCH_HEIGHT_PX))
            .center()
            .spacing(4.)
            .background(background)
            .corner_radius(CornerRadius::new(4., 0., 0., 4.))
            .shadow(
                Shadow::new()
                    .blur(30.)
                    .y(10.)
                    .color(colors::brand().with_a(90)),
            )
            .cursor(if enabled {
                CursorIcon::Pointer
            } else {
                CursorIcon::Default
            })
            .on_pointer_over(move |_| hovering.set(true))
            .on_pointer_out(move |_| hovering.set(false))
            .on_press(move |_| {
                if enabled {
                    dispatch.launch_cluster(cluster_id);
                }
            })
            .child(
                label()
                    .text(format!("▶ {}", self.title))
                    .font_family(theme::PIXEL_FONT)
                    .font_size(text_size(18.))
                    .color(colors::on_brand()),
            )
            .child(
                label()
                    .text(self.subtitle.clone())
                    .font_family(theme::PIXEL_FONT)
                    .font_size(text_size(11.))
                    .color(colors::on_brand().with_a(210)),
            )
    }
}

#[derive(PartialEq)]
struct PickerToggle {
    open: State<bool>,
}

impl Component for PickerToggle {
    fn render(&self) -> impl IntoElement {
        let mut hovering = use_state(|| false);
        let mut open = self.open;

        rect()
            .width(Size::px(PICKER_TOGGLE_WIDTH_PX))
            .height(Size::px(LAUNCH_HEIGHT_PX))
            .center()
            .background(if hovering() {
                colors::brand_hover()
            } else {
                colors::brand()
            })
            .border(
                Border::new()
                    .fill(colors::on_brand().with_a(50))
                    .width(BorderWidth {
                        top: 0.,
                        right: 0.,
                        bottom: 0.,
                        left: 1.,
                    })
                    .alignment(BorderAlignment::Inner),
            )
            .corner_radius(CornerRadius::new(0., 4., 4., 0.))
            .cursor(CursorIcon::Pointer)
            .on_pointer_over(move |_| hovering.set(true))
            .on_pointer_out(move |_| hovering.set(false))
            .on_press(move |_| {
                let next = !*open.peek();
                open.set(next);
            })
            .child(
                Icon::new(IconType::ChevronDown)
                    .size(18.)
                    .color(colors::on_brand()),
            )
    }
}

#[derive(PartialEq)]
struct VersionPicker {
    clusters: Vec<Cluster>,
    active_id: i64,
    open: State<bool>,
}

impl Component for VersionPicker {
    fn render(&self) -> impl IntoElement {
        let height = PICKER_HEAD_HEIGHT_PX + self.clusters.len() as f32 * PICKER_ROW_HEIGHT_PX;
        let open = use_animation(|conf| {
            conf.on_creation(OnCreation::Run);
            AnimNum::new(0., 1.)
                .time(PICKER_OPEN_MS)
                .ease(Ease::Out)
                .function(Function::Cubic)
        });
        let opened = open.get().value();

        rect()
            .vertical()
            .width(Size::px(LAUNCH_WIDTH_PX + PICKER_TOGGLE_WIDTH_PX))
            .position(
                Position::new_absolute()
                    .top(-(height + PICKER_GAP_PX))
                    .left(0.),
            )
            .offset_y((1. - opened) * PICKER_OPEN_RISE_PX)
            .opacity(opened)
            .layer(Layer::OverlayLevel(3))
            .background(colors::page_elevated())
            .corner_radius(rounded(4.))
            .overflow(Overflow::Clip)
            .border(
                Border::new()
                    .fill(colors::component_border())
                    .width(1.)
                    .alignment(BorderAlignment::Inner),
            )
            .child(
                rect()
                    .width(Size::fill())
                    .height(Size::px(PICKER_HEAD_HEIGHT_PX))
                    .main_align(Alignment::Center)
                    .padding(Gaps::new_symmetric(0., 14.))
                    .child(
                        paragraph()
                            .font_family(theme::PIXEL_FONT)
                            .font_size(text_size(12.))
                            .span(Span::new("❯ ").color(colors::brand()))
                            .span(
                                Span::new(t("oneclient.home.pick_version"))
                                    .color(colors::fg_secondary()),
                            ),
                    ),
            )
            .children(self.clusters.iter().map(|cluster| {
                PickerRow {
                    cluster: cluster.clone(),
                    active: cluster.id == self.active_id,
                    open: self.open,
                }
                .into_element()
            }))
    }
}

#[derive(PartialEq)]
struct PickerRow {
    cluster: Cluster,
    active: bool,
    open: State<bool>,
}

impl Component for PickerRow {
    fn render(&self) -> impl IntoElement {
        let mut hovering = use_state(|| false);
        let mut active_id = use_active_cluster_id();
        let mut open = self.open;
        let cluster_id = self.cluster.id;
        let highlighted = self.active || hovering();

        rect()
            .horizontal()
            .width(Size::fill())
            .height(Size::px(PICKER_ROW_HEIGHT_PX))
            .content(Content::Flex)
            .cross_align(Alignment::Center)
            .padding(Gaps::new_symmetric(0., 14.))
            .background(if hovering() {
                colors::component_bg_hover()
            } else {
                Color::TRANSPARENT
            })
            .cursor(CursorIcon::Pointer)
            .on_pointer_over(move |_| hovering.set(true))
            .on_pointer_out(move |_| hovering.set(false))
            .on_press(move |_| {
                *active_id.write() = Some(cluster_id);
                open.set(false);
            })
            .child(
                paragraph()
                    .width(Size::flex(1.0))
                    .max_lines(1)
                    .font_family(theme::PIXEL_FONT)
                    .font_size(text_size(12.))
                    .maybe(self.active, |p| p.span(Span::new("# ").color(colors::brand())))
                    .span(
                        Span::new(cluster_title(&self.cluster)).color(if highlighted {
                            colors::fg_primary()
                        } else {
                            colors::fg_secondary()
                        }),
                    ),
            )
            .child(
                label()
                    .text(self.cluster.mc_loader.to_string().to_lowercase())
                    .font_family(theme::PIXEL_FONT)
                    .font_size(text_size(11.))
                    .color(colors::fg_secondary()),
            )
    }
}

fn cluster_settings_button(cluster_id: i64) -> impl IntoElement {
    Button::new()
        .ghost()
        .icon()
        .foreground(colors::on_art())
        .tooltip(t("oneclient.components.active_cluster_panel.settings"))
        .on_press(move |_| {
            let _ = RouterContext::get().push(Route::ClusterOverview { cluster_id });
        })
        .child(Icon::new(IconType::Settings04).size(20.))
}
