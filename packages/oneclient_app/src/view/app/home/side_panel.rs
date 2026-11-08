//! VeloxClient: right-hand column on home, the most recent versions as large picture cards

use freya::animation::{
    AnimColor, AnimNum, Ease, Function, OnCreation, use_animation, use_animation_transition,
};
use freya::prelude::*;
use freya::router::RouterContext;

use crate::components::{ART_PREVIEW_EDGE, DynamicArt, cluster_title, use_home_clusters};
use crate::hooks::use_active_cluster_id;
use crate::i18n::t;
use crate::routes::Route;
use crate::theme::{self, colors, rounded, text_size};
use oneclient_core::clusters::Cluster;

const SIDE_PANEL_WIDTH_PX: f32 = 360.;
const CARD_COUNT: usize = 3;
const CARD_GAP_PX: f32 = 14.;
const HEADING_SIZE: f32 = 12.;
const CARD_TITLE_SIZE: f32 = 12.;
const HOVER_FADE_MS: u64 = 140;
const HOVER_TINT_ALPHA: u8 = 28;
const CARD_BORDER_PX: f32 = 2.;
const PANEL_ENTER_MS: u64 = 480;
const PANEL_ENTER_SHIFT_PX: f32 = 40.;

#[derive(PartialEq)]
pub struct HomeSidePanel;

impl Component for HomeSidePanel {
    fn render(&self) -> impl IntoElement {
        let (sorted, active) = use_home_clusters();
        let active_id = active.as_ref().map(|c| c.id);
        let enter = use_animation(|conf| {
            conf.on_creation(OnCreation::Run);
            AnimNum::new(0., 1.)
                .time(PANEL_ENTER_MS)
                .ease(Ease::Out)
                .function(Function::Cubic)
        });
        let entered = enter.get().value();

        rect()
            .vertical()
            .width(Size::px(SIDE_PANEL_WIDTH_PX))
            .height(Size::fill())
            .content(Content::Flex)
            .padding(Gaps::new_all(14.))
            .spacing(CARD_GAP_PX)
            .background(colors::page_elevated())
            .border(
                Border::new()
                    .fill(colors::component_border())
                    .width(BorderWidth {
                        top: 0.,
                        right: 0.,
                        bottom: 0.,
                        left: 1.,
                    })
                    .alignment(BorderAlignment::Inner),
            )
            .offset_x((1. - entered) * PANEL_ENTER_SHIFT_PX)
            .opacity(entered)
            .child(panel_heading())
            .children(sorted.into_iter().take(CARD_COUNT).map(|cluster| {
                let selected = Some(cluster.id) == active_id;
                VersionTile { cluster, selected }.into_element()
            }))
    }
}

fn panel_heading() -> impl IntoElement {
    rect()
        .horizontal()
        .width(Size::fill())
        .content(Content::Flex)
        .child(
            paragraph()
                .width(Size::flex(1.0))
                .font_family(theme::PIXEL_FONT)
                .font_size(text_size(HEADING_SIZE))
                .span(Span::new("# ").color(colors::brand()))
                .span(Span::new(t("oneclient.home.recent")).color(colors::fg_secondary())),
        )
        .child(
            rect()
                .cursor(CursorIcon::Pointer)
                .on_press(|_| {
                    let _ = RouterContext::get().push(Route::Clusters {});
                })
                .child(
                    label()
                        .text(format!(
                            "{} →",
                            t("oneclient.components.navbar.versions").to_lowercase()
                        ))
                        .font_family(theme::PIXEL_FONT)
                        .font_size(text_size(HEADING_SIZE))
                        .color(colors::fg_secondary()),
                ),
        )
}

/// Eases a colour towards `target` whenever it changes
fn fade_color(target: Color) -> Color {
    use_animation_transition(target, |from, to| {
        AnimColor::new(from, to)
            .time(HOVER_FADE_MS)
            .ease(Ease::Out)
            .function(Function::Quad)
    })
    .read()
    .value()
}

#[derive(PartialEq)]
struct VersionTile {
    cluster: Cluster,
    selected: bool,
}

impl Component for VersionTile {
    fn render(&self) -> impl IntoElement {
        let mut hovering = use_state(|| false);
        let mut active_id = use_active_cluster_id();
        let cluster_id = self.cluster.id;
        let selected = self.selected;

        let border = fade_color(if selected {
            colors::brand()
        } else if hovering() {
            colors::component_border_hover()
        } else {
            colors::component_border()
        });
        let tint = fade_color(if hovering() {
            Color::WHITE.with_a(HOVER_TINT_ALPHA)
        } else {
            Color::TRANSPARENT
        });
        let title_color = fade_color(if selected || hovering() {
            colors::fg_primary()
        } else {
            colors::fg_secondary()
        });

        rect()
            .vertical()
            .width(Size::fill())
            .height(Size::flex(1.0))
            .content(Content::Flex)
            .spacing(6.)
            .cursor(CursorIcon::Pointer)
            .on_pointer_over(move |_| hovering.set(true))
            .on_pointer_out(move |_| hovering.set(false))
            .on_press(move |_| *active_id.write() = Some(cluster_id))
            .child(
                paragraph()
                    .max_lines(1)
                    .font_family(theme::PIXEL_FONT)
                    .font_size(text_size(CARD_TITLE_SIZE))
                    .maybe(selected, |p| p.span(Span::new("> ").color(colors::brand())))
                    .span(Span::new(cluster_title(&self.cluster).to_lowercase()).color(title_color)),
            )
            .child(
                rect()
                    .width(Size::fill())
                    .height(Size::flex(1.0))
                    .corner_radius(rounded(4.))
                    .overflow(Overflow::Clip)
                    .child(
                        rect()
                            .width(Size::fill())
                            .height(Size::fill())
                            .position(Position::new_absolute())
                            .child(DynamicArt::for_cluster(&self.cluster).max_edge(ART_PREVIEW_EDGE)),
                    )
                    .child(
                        rect()
                            .width(Size::fill())
                            .height(Size::fill())
                            .position(Position::new_absolute())
                            .corner_radius(rounded(4.))
                            .background(tint)
                            .border(
                                Border::new()
                                    .fill(border)
                                    .width(CARD_BORDER_PX)
                                    .alignment(BorderAlignment::Inner),
                            ),
                    ),
            )
    }
}
