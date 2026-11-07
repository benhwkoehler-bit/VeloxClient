//! VeloxClient "hybrid" home: the normal home screen with a few terminal
//! touches. A path line above the title, key/value facts under it and a slim
//! status line at the bottom, all set in the pixel font.

use freya::prelude::*;
use oneclient_content::bundles::BundleWithUpdateStatus;
use oneclient_content::packages::ContentType;
use oneclient_core::clusters::Cluster;

use oneclient_events::LaunchStage;

use crate::hooks::{
    bundles_with_status_items, cluster_content_items, use_bundles_with_status,
    use_cluster_content, use_game_snapshot, use_settings_snapshot,
};
use crate::i18n::t;
use crate::theme::{self, colors, text_size};
use crate::view::app::cluster::renderer_switch::ships_pure_metal;

const SODIUM_PROJECT: &str = "AANobbMI";
const FACT_KEY_WIDTH: f32 = 92.;
const FACT_SIZE: f32 = 13.;
const STATUS_SIZE: f32 = 11.;
const STATUS_HEIGHT: f32 = 26.;

/// Lines the Velox Companion mod writes into the game log
const FPS_MARKER: &str = "velox:fps=";
const STARTUP_MARKER: &str = "velox:startup_ms=";

/// The newest number after `marker` in the game log, read from the end
fn latest_metric(lines: &[std::sync::Arc<str>], marker: &str) -> Option<u64> {
    lines.iter().rev().find_map(|line| {
        let (_, rest) = line.split_once(marker)?;
        let digits: String = rest.chars().take_while(char::is_ascii_digit).collect();
        digits.parse().ok()
    })
}

/// `~/versions/26.3` above the title
pub fn cluster_path(cluster: &Cluster) -> impl IntoElement {
    paragraph()
        .font_family(theme::PIXEL_FONT)
        .font_size(text_size(12.))
        .span(Span::new("~/versions/").color(colors::on_art_secondary()))
        .span(Span::new(cluster.mc_version.clone()).color(colors::brand()))
}

#[derive(Clone, Copy, PartialEq)]
enum Renderer {
    PureMetal,
    Sodium,
    Standard,
}

impl Renderer {
    fn label(self) -> &'static str {
        match self {
            Self::PureMetal => "pure metal ●",
            Self::Sodium => "sodium",
            Self::Standard => "standard",
        }
    }
}

fn renderer_for(bundles: &[BundleWithUpdateStatus], metal_setting: bool) -> Renderer {
    if metal_setting && ships_pure_metal(bundles) {
        return Renderer::PureMetal;
    }
    let ships_sodium = bundles.iter().any(|bundle| {
        bundle
            .archive
            .manifest
            .files
            .iter()
            .any(|file| file.kind.package_id() == SODIUM_PROJECT)
    });
    if ships_sodium { Renderer::Sodium } else { Renderer::Standard }
}

/// What the home panel and the status line both show about a cluster
fn use_cluster_summary(cluster_id: i64) -> (Renderer, usize) {
    let bundles_query = use_bundles_with_status(cluster_id);
    let mods_query = use_cluster_content(cluster_id, ContentType::Mod);
    let metal_setting = use_settings_snapshot().settings.metal_renderer;

    let renderer = renderer_for(&bundles_with_status_items(&bundles_query), metal_setting);
    let mods = cluster_content_items(&mods_query)
        .iter()
        .filter(|item| item.enabled)
        .count();
    (renderer, mods)
}

fn fact_row(key: &str, value: String, value_color: Color) -> impl IntoElement {
    rect()
        .horizontal()
        .child(
            label()
                .text(key.to_string())
                .width(Size::px(FACT_KEY_WIDTH))
                .font_family(theme::PIXEL_FONT)
                .font_size(text_size(FACT_SIZE))
                .color(colors::on_art_secondary()),
        )
        .child(
            label()
                .text(value)
                .font_family(theme::PIXEL_FONT)
                .font_size(text_size(FACT_SIZE))
                .color(value_color),
        )
}

/// renderer / mods / loader under the title
#[derive(PartialEq)]
pub struct ClusterFacts {
    pub cluster: Cluster,
}

impl Component for ClusterFacts {
    fn render(&self) -> impl IntoElement {
        let (renderer, mods) = use_cluster_summary(self.cluster.id);
        let loader = match &self.cluster.mc_loader_version {
            Some(version) => format!("{} {version}", self.cluster.mc_loader),
            None => self.cluster.mc_loader.to_string(),
        };
        let renderer_color = if renderer == Renderer::PureMetal {
            colors::brand()
        } else {
            colors::on_art()
        };

        rect()
            .vertical()
            .spacing(6.)
            .margin(Gaps::new(14., 0., 6., 0.))
            .child(fact_row(
                t("oneclient.home.facts.renderer"),
                renderer.label().to_string(),
                renderer_color,
            ))
            .child(fact_row(
                t("oneclient.home.facts.mods"),
                mods.to_string(),
                colors::on_art(),
            ))
            .child(fact_row(
                t("oneclient.home.facts.loader"),
                loader.to_lowercase(),
                colors::on_art(),
            ))
    }
}

/// `● bereit │ 26.3 fabric │ pure metal ●` along the bottom of home
#[derive(PartialEq)]
pub struct HomeStatusLine {
    pub cluster: Cluster,
}

impl Component for HomeStatusLine {
    fn render(&self) -> impl IntoElement {
        let (renderer, mods) = use_cluster_summary(self.cluster.id);
        let game = use_game_snapshot();
        let running = matches!(game.stage(self.cluster.id), Some(LaunchStage::Running));
        let logs = game.logs_for(self.cluster.id);
        let fps = running.then(|| latest_metric(&logs, FPS_MARKER)).flatten();
        let startup_ms = latest_metric(&logs, STARTUP_MARKER);

        let (state_text, state_color) = if running {
            (t("oneclient.home.status.running"), colors::brand())
        } else {
            (t("oneclient.home.status.ready"), colors::on_art())
        };
        let separator = || Span::new("  │  ").color(colors::on_art_secondary().with_a(120));
        let version = format!("{} {}", self.cluster.mc_version, self.cluster.mc_loader).to_lowercase();
        let renderer_color = if renderer == Renderer::PureMetal {
            colors::brand()
        } else {
            colors::on_art_secondary()
        };

        rect()
            .horizontal()
            .width(Size::fill())
            .height(Size::px(STATUS_HEIGHT))
            .cross_align(Alignment::Center)
            .child(
                paragraph()
                    .font_family(theme::PIXEL_FONT)
                    .font_size(text_size(STATUS_SIZE))
                    .span(Span::new(format!("● {state_text}")).color(state_color))
                    .span(separator())
                    .span(Span::new(version).color(colors::on_art_secondary()))
                    .span(separator())
                    .span(Span::new(renderer.label()).color(renderer_color))
                    .span(separator())
                    .span(
                        Span::new(format!("{mods} {}", t("oneclient.home.facts.mods")))
                            .color(colors::on_art_secondary()),
                    )
                    .maybe(fps.is_some(), |p| {
                        p.span(separator()).span(
                            Span::new(format!("{} fps", fps.unwrap_or_default()))
                                .color(colors::brand()),
                        )
                    })
                    .maybe(startup_ms.is_some(), |p| {
                        p.span(separator()).span(
                            Span::new(format!(
                                "start {:.1} s",
                                startup_ms.unwrap_or_default() as f64 / 1000.
                            ))
                            .color(colors::on_art_secondary()),
                        )
                    }),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renderer_labels_read_like_terminal_values() {
        assert_eq!(Renderer::PureMetal.label(), "pure metal ●");
        assert_eq!(Renderer::Sodium.label(), "sodium");
        assert_eq!(Renderer::Standard.label(), "standard");
    }

    #[test]
    fn the_newest_companion_metric_wins() {
        let lines: Vec<std::sync::Arc<str>> = vec![
            "[15:37:45] [Render thread/INFO]: velox:startup_ms=15873".into(),
            "[15:37:49] [Render thread/INFO]: velox:fps=59".into(),
            "[15:37:54] [Render thread/INFO]: something else".into(),
            "[15:37:54] [Render thread/INFO]: velox:fps=214".into(),
        ];
        assert_eq!(latest_metric(&lines, FPS_MARKER), Some(214));
        assert_eq!(latest_metric(&lines, STARTUP_MARKER), Some(15873));
        assert_eq!(latest_metric(&[], FPS_MARKER), None);
    }

    #[test]
    fn no_bundles_means_the_standard_renderer() {
        assert!(renderer_for(&[], true) == Renderer::Standard);
        assert!(renderer_for(&[], false) == Renderer::Standard);
    }
}
