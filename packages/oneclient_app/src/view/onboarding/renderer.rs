//! VeloxClient: choose between Pure Metal (Apple's Metal API, runs with Sodium)
//! and the standard OpenGL path. Only shown on Apple Silicon Macs; Pure Metal
//! ships in the 26.3 pack, every other version keeps the standard renderer.

use freya::prelude::*;

use crate::components::{IconType, toggle};
use crate::hooks::{use_dispatch, use_settings_snapshot};
use crate::i18n::t;
use crate::routes::Route;
use crate::view::app::settings::settings_row;
use crate::view::onboarding::{
    onboarding_illustration, onboarding_nav, onboarding_page, step_heading,
};

#[derive(PartialEq)]
pub struct OnboardingRenderer;

impl Component for OnboardingRenderer {
    fn render(&self) -> impl IntoElement {
        let settings = use_settings_snapshot().settings;
        let dispatch = use_dispatch();

        let metal_on = use_state({
            let v = settings.metal_renderer;
            move || v
        });

        let mut first = use_state(|| true);
        use_side_effect(move || {
            let metal = *metal_on.read();
            if *first.peek() {
                first.set(false);
                return;
            }
            dispatch.edit_settings(|settings| settings.metal_renderer = metal);
        });

        let content = rect()
            .vertical()
            .width(Size::fill())
            .spacing(20.)
            .child(step_heading(
                t("oneclient.onboarding.renderer.title"),
                t("oneclient.onboarding.renderer.subtitle"),
            ))
            .child(settings_row(
                IconType::Rocket02,
                t("oneclient.onboarding.renderer.metal.title"),
                t("oneclient.onboarding.renderer.metal.description"),
                toggle(metal_on),
            ))
            .child(
                label()
                    .text(t("oneclient.onboarding.renderer.standard_note"))
                    .color(crate::theme::colors::fg_secondary()),
            )
            .into_element();

        onboarding_page(
            onboarding_illustration(IconType::Rocket02),
            content,
            onboarding_nav(
                Some(Route::OnboardingAccount {}),
                Route::OnboardingBundles {},
                true,
            ),
        )
    }
}
