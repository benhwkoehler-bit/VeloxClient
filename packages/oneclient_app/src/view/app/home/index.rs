use std::time::Duration;

use freya::animation::{
    AnimNum, Ease, Function, OnChange, OnCreation, use_animation_with_dependencies,
};
use freya::prelude::*;

use super::player_scene::{
    animation_enabled, current_scene, scene_animation, scene_props, set_animation_enabled,
};
use crate::components::{
    ActiveClusterPanel, HomeStatusLine, LaunchBar, PlayerModel, toggle_controlled,
    use_home_clusters,
};
use crate::hooks::{try_default_account, use_current_account};
use crate::i18n::t;
use crate::theme::{self, colors, rounded, text_size};

const STAGE_INSET_PX: f32 = 22.;
const SCENE_CHECK_INTERVAL: Duration = Duration::from_secs(5);
const SCENE_FADE_MS: u64 = 700;
const NAMETAG_TOP_PX: f32 = 26.;
const NAMETAG_TEXT_SIZE: f32 = 24.;
const TOGGLE_LABEL_SIZE: f32 = 12.;
// Square-on and large, with the feet tucked behind the launch button
const PLAYER_YAW: f32 = 0.;
const PLAYER_ZOOM: f32 = 0.9;
const PLAYER_FOCUS_Y: f32 = 2.5;

/// VeloxClient: black stage with a large player and the launch button
#[derive(PartialEq)]
pub struct Home;

impl Component for Home {
    fn render(&self) -> impl IntoElement {
        let mut animate = use_state(animation_enabled);
        let animating = *animate.read();

        rect()
            .horizontal()
            .width(Size::fill())
            .height(Size::fill())
            .content(Content::Flex)
            .overflow(Overflow::Clip)
            .child(
                rect()
                    .width(Size::flex(1.0))
                    .height(Size::fill())
                    .overflow(Overflow::Clip)
                    .child(HomePlayer { animate: animating })
                    .child(
                        rect()
                            .position(
                                Position::new_absolute()
                                    .top(STAGE_INSET_PX)
                                    .left(STAGE_INSET_PX),
                            )
                            .child(ActiveClusterPanel),
                    )
                    .child(
                        rect()
                            .horizontal()
                            .position(
                                Position::new_absolute()
                                    .top(STAGE_INSET_PX)
                                    .right(STAGE_INSET_PX),
                            )
                            .cross_align(Alignment::Center)
                            .spacing(10.)
                            .child(
                                label()
                                    .text(t("oneclient.home.skin_animation"))
                                    .font_family(theme::PIXEL_FONT)
                                    .font_size(text_size(TOGGLE_LABEL_SIZE))
                                    .color(colors::fg_secondary()),
                            )
                            .child(toggle_controlled(
                                animating,
                                (move |()| {
                                    let next = !animating;
                                    set_animation_enabled(next);
                                    animate.set(next);
                                })
                                .into(),
                            )),
                    )
                    .child(
                        rect()
                            .vertical()
                            .position(Position::new_absolute().bottom(0.).left(0.))
                            .width(Size::fill())
                            .cross_align(Alignment::Center)
                            .spacing(12.)
                            .child(LaunchBar)
                            .child(
                                rect()
                                    .width(Size::fill())
                                    .padding(Gaps::new(0., STAGE_INSET_PX, 4., STAGE_INSET_PX))
                                    .child(HomeStatusFooter),
                            ),
                    ),
            )
    }
}

/// The signed-in player under a name tag, settled into one calm scene at a time
#[derive(PartialEq)]
struct HomePlayer {
    animate: bool,
}

impl Component for HomePlayer {
    fn render(&self) -> impl IntoElement {
        let current_account = use_current_account();
        let account = try_default_account(&current_account);
        let mut scene = use_state(current_scene);

        use_hook(move || {
            spawn(async move {
                loop {
                    tokio::time::sleep(SCENE_CHECK_INTERVAL).await;
                    let next = current_scene();
                    if *scene.peek() != next {
                        scene.set(next);
                    }
                }
            });
        });

        let index = *scene.read();
        let fade = use_animation_with_dependencies(&index, |conf, _| {
            conf.on_creation(OnCreation::Run);
            conf.on_change(OnChange::Rerun);
            AnimNum::new(0., 1.)
                .time(SCENE_FADE_MS)
                .ease(Ease::Out)
                .function(Function::Cubic)
        });
        let faded_in = fade.get().value();

        let stage = rect()
            .vertical()
            .width(Size::fill())
            .height(Size::fill())
            .content(Content::Flex)
            .cross_align(Alignment::Center)
            .padding(Gaps::new(NAMETAG_TOP_PX, 0., 0., 0.))
            .spacing(4.)
            .opacity(faded_in);

        let Some(account) = account else {
            return stage.into_element();
        };

        let (emote, props) = if self.animate {
            (scene_animation(index), scene_props(index))
        } else {
            (None, Vec::new())
        };

        stage
            .child(name_tag(account.username.clone()))
            .child(
                PlayerModel::new(account.id.to_string())
                    .emote(emote)
                    .cosmetics(props)
                    .yaw(PLAYER_YAW)
                    .zoom(PLAYER_ZOOM)
                    .focus_y(PLAYER_FOCUS_Y)
                    .width(Size::fill())
                    .height(Size::flex(1.0)),
            )
            .into_element()
    }
}

fn name_tag(name: String) -> impl IntoElement {
    rect()
        .padding(Gaps::new_symmetric(6., 14.))
        .corner_radius(rounded(3.))
        .background(Color::BLACK.with_a(150))
        .child(
            paragraph()
                .font_family(theme::PIXEL_FONT)
                .font_size(text_size(NAMETAG_TEXT_SIZE))
                .span(Span::new("> ").color(colors::brand()))
                .span(Span::new(name).color(colors::on_art())),
        )
}

/// VeloxClient: terminal-style status line for the cluster the panel shows
#[derive(PartialEq)]
struct HomeStatusFooter;

impl Component for HomeStatusFooter {
    fn render(&self) -> impl IntoElement {
        let (_, active) = use_home_clusters();

        rect()
            .width(Size::fill())
            .maybe_child(active.map(|cluster| HomeStatusLine { cluster }))
    }
}
