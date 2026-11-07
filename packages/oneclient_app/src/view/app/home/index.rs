use freya::prelude::*;

use crate::components::{ActiveClusterPanel, HomeStatusLine, RecentsRow};
use crate::hooks::{settled_or_loading, use_active_cluster_id, use_clusters};
use crate::theme;
use crate::utils::sort_clusters_for_home;

#[derive(PartialEq)]
pub struct Home;

impl Component for Home {
    fn render(&self) -> impl IntoElement {
        rect()
            .vertical()
            .width(Size::fill())
            .height(Size::fill())
            .content(Content::Flex)
            .overflow(Overflow::Clip)
            .padding(theme::HOME_PADDING_PX)
            .main_align(Alignment::Center)
            .child(
                rect()
                    .vertical()
                    .width(Size::fill())
                    .height(Size::flex(1.0))
                    .main_align(Alignment::Center)
                    .child(ActiveClusterPanel),
            )
            .child(RecentsRow)
            .child(HomeStatusFooter)
    }
}

/// VeloxClient: terminal-style status line for the cluster the panel shows
#[derive(PartialEq)]
struct HomeStatusFooter;

impl Component for HomeStatusFooter {
    fn render(&self) -> impl IntoElement {
        let clusters_query = use_clusters();
        let active_id = *use_active_cluster_id().read();
        let sorted = sort_clusters_for_home(settled_or_loading(&clusters_query).unwrap_or_default());
        let active = active_id
            .and_then(|id| sorted.iter().find(|c| c.id == id).cloned())
            .or_else(|| sorted.first().cloned());

        rect()
            .width(Size::fill())
            .maybe_child(active.map(|cluster| HomeStatusLine { cluster }))
    }
}
