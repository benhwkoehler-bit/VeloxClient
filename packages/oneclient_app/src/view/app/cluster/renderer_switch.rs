//! VeloxClient: switch Pure Metal on or off right in the mod list.
//! Only on Apple Silicon and only for versions whose pack ships Pure Metal (26.3).

use freya::prelude::*;
use oneclient_content::bundles::{BundleWithUpdateStatus, METAL_ONLY_PROJECTS};

use crate::components::{IconType, toggle_controlled};
use crate::hooks::{
    ClusterAction, mutation_is_running, use_cluster_mutation, use_dispatch, use_settings_snapshot,
};
use crate::i18n::t;
use crate::view::app::settings::settings_row;

const SWITCH_PADDING: (f32, f32, f32, f32) = (12., 24., 0., 24.);

/// True on Apple Silicon when one of the cluster's bundles carries Pure Metal.
pub fn ships_pure_metal(bundles: &[BundleWithUpdateStatus]) -> bool {
    cfg!(all(target_os = "macos", target_arch = "aarch64"))
        && bundles.iter().any(|bundle| {
            bundle
                .archive
                .manifest
                .files
                .iter()
                .any(|file| METAL_ONLY_PROJECTS.contains(&file.kind.package_id().as_str()))
        })
}

#[derive(PartialEq)]
pub struct RendererSwitch {
    pub cluster_id: i64,
}

impl Component for RendererSwitch {
    fn render(&self) -> impl IntoElement {
        let cluster_id = self.cluster_id;
        let metal_on = use_settings_snapshot().settings.metal_renderer;
        let dispatch = use_dispatch();
        let mutation = use_cluster_mutation();
        let busy = mutation_is_running(&mutation);

        let on_toggle = move |()| {
            let metal = !metal_on;
            dispatch.edit_settings(move |settings| settings.metal_renderer = metal);
            mutation.mutate(ClusterAction::SetRenderer { cluster_id, metal });
        };

        let description = if metal_on {
            t("oneclient.cluster.mods.renderer.metal_on")
        } else {
            t("oneclient.cluster.mods.renderer.metal_off")
        };

        rect()
            .width(Size::fill())
            .padding(SWITCH_PADDING)
            .child(settings_row(
                IconType::Rocket02,
                t("oneclient.cluster.mods.renderer.title"),
                description,
                toggle_controlled(metal_on, on_toggle.into()).disabled(busy),
            ))
            .into_element()
    }
}
