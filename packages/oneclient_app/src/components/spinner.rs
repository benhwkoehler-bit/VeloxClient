use freya::animation::{AnimNum, Function, OnCreation, OnFinish, use_animation};
use freya::prelude::*;

use super::{Icon, IconType};
use crate::theme::{colors, text_size};

const SPIN_TIME: u64 = 900;

#[derive(PartialEq)]
pub struct Spinner {
    size: f32,
    color: Option<Color>,
}

impl Spinner {
    pub fn new(size: f32) -> Self {
        Self { size, color: None }
    }

    /// Spinner in a specific colour (defaults to the secondary text colour).
    #[must_use]
    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }
}

impl Component for Spinner {
    fn render(&self) -> impl IntoElement {
        let spin = use_animation(|conf| {
            conf.on_creation(OnCreation::Run);
            conf.on_finish(OnFinish::restart());
            AnimNum::new(0., 360.)
                .time(SPIN_TIME)
                .function(Function::Linear)
        });

        rect().rotate(spin.get().value()).child(
            Icon::new(IconType::Loading02)
                .size(self.size)
                .color(self.color.unwrap_or_else(colors::fg_secondary)),
        )
    }
}

pub fn centered_spinner(message: &str) -> Element {
    rect()
        .vertical()
        .width(Size::fill())
        .height(Size::fill())
        .center()
        .spacing(12.)
        .child(Spinner::new(22.))
        .child(
            label()
                .text(message.to_string())
                .font_size(text_size(13.))
                .color(colors::fg_secondary()),
        )
        .into_element()
}
