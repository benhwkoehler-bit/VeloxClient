//! VeloxClient: bring profiles over from other launchers during the first start.
//!
//! Lists every install found (Prism, PolyMC, MultiMC, Modrinth App, NoRisk,
//! Lunar) with the same picker as "New instance → Import". Picked profiles are
//! copied into VeloxClient in the background; the originals stay untouched.

use freya::prelude::*;
use freya::router::RouterContext;
use oneclient_core::{ExternalDetection, ExternalInstance};

use crate::hooks::{ExternalImportJob, use_dispatch};
use crate::i18n::t;
use crate::routes::Route;
use crate::view::app::clusters::ImportStep;
use crate::view::onboarding::{onboarding_nav_action, onboarding_slide, step_heading};

const PAGE_PADDING: (f32, f32, f32, f32) = (40., 80., 12., 80.);

#[derive(PartialEq)]
pub struct OnboardingImport;

impl Component for OnboardingImport {
    fn render(&self) -> impl IntoElement {
        let dispatch = use_dispatch();
        let chosen = use_state(Vec::<ExternalInstance>::new);
        let extra = use_state(Vec::<ExternalDetection>::new);

        let picked = chosen.read().len();
        let next_label = if picked == 0 {
            t("oneclient.onboarding.import.skip")
        } else {
            t("oneclient.onboarding.import.import")
        };

        let on_next = move |()| {
            let jobs: Vec<ExternalImportJob> = chosen
                .read()
                .iter()
                .cloned()
                .map(ExternalImportJob::as_is)
                .collect();
            if !jobs.is_empty() {
                dispatch.import_external_instances(jobs);
            }
            let _ = RouterContext::get().replace(Route::OnboardingAccount {});
        };

        rect()
            .vertical()
            .width(Size::fill())
            .height(Size::fill())
            .content(Content::Flex)
            .child(onboarding_slide(
                rect()
                    .vertical()
                    .width(Size::fill())
                    .height(Size::fill())
                    .content(Content::Flex)
                    .padding(PAGE_PADDING)
                    .spacing(16.)
                    .child(step_heading(
                        t("oneclient.onboarding.import.title"),
                        t("oneclient.onboarding.import.subtitle"),
                    ))
                    .child(
                        rect()
                            .width(Size::fill())
                            .height(Size::flex(1.0))
                            .child(ImportStep { chosen, extra }),
                    ),
            ))
            .child(onboarding_nav_action(
                Some(Route::OnboardingLanguage {}),
                next_label,
                true,
                on_next,
            ))
    }
}
