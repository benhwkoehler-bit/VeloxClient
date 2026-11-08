//! VeloxClient: the black home background with a grid of faint terminal
//! characters (`#`, `+`, `:`, `.`) drawn as square pixels. It is one shader,
//! so a window full of glyphs costs no more than a plain fill. The grid is
//! quiet in the middle behind the player and a little stronger at the edges.

use freya::engine::prelude::{Data, RuntimeEffect};
use freya::prelude::*;

const TERMINAL_GRID_SHADER: &str = r#"
uniform float2 iResolution;
uniform float iScale;

const float CELL = 34.0;
const float DOT = 2.0;

float hash(float2 p) {
    return fract(sin(dot(p, float2(127.1, 311.7))) * 43758.5453);
}

bool at(float value, float target) {
    return abs(value - target) < 0.5;
}

bool glyph(float pick, float2 g) {
    if (pick < 0.08) { return false; }
    if (pick < 0.3) { return at(g.x, 2.0) && at(g.y, 2.0); }
    if (pick < 0.55) { return at(g.x, 2.0) || at(g.y, 2.0); }
    if (pick < 0.68) { return at(g.x, 2.0) && (at(g.y, 1.0) || at(g.y, 3.0)); }
    return at(g.x, 1.0) || at(g.x, 3.0) || at(g.y, 1.0) || at(g.y, 3.0);
}

half4 main(float2 fragCoord) {
    float2 point = fragCoord / iScale;
    float2 cell = floor(point / CELL);
    float2 local = point - cell * CELL;
    float2 g = floor((local - (CELL - 5.0 * DOT) * 0.5) / DOT);

    if (g.x < 0.0 || g.y < 0.0 || g.x > 4.0 || g.y > 4.0 || !glyph(hash(cell), g)) {
        return half4(0.0, 0.0, 0.0, 1.0);
    }

    float2 uv = fragCoord / iResolution;
    float fromCenter = length((uv - float2(0.5, 0.52)) * float2(1.0, 1.2));
    float strength = mix(0.11, 0.34, smoothstep(0.1, 0.75, fromCenter));
    strength *= 0.55 + 0.45 * hash(cell + 7.0);

    float3 color = hash(cell + 13.0) > 0.9
        ? float3(1.0, 0.8, 1.6)
        : float3(0.62, 0.6, 0.72);
    return half4(half3(color * strength), 1.0);
}
"#;

fn compile_grid() -> Option<RuntimeEffect> {
    RuntimeEffect::make_for_shader(TERMINAL_GRID_SHADER, None)
        .inspect_err(|err| tracing::warn!("[home] terminal grid shader failed: {err}"))
        .ok()
}

#[derive(PartialEq, Clone, Copy)]
pub struct TerminalBackdrop;

impl Component for TerminalBackdrop {
    fn render(&self) -> impl IntoElement {
        let effect = use_hook(compile_grid);
        let scale = *Platform::get().current_window().scale_factor.read() as f32;

        rect()
            .width(Size::fill())
            .height(Size::fill())
            .position(Position::new_absolute())
            .background(Color::BLACK)
            .interactive(false)
            .maybe_child(effect.map(|effect| {
                rect()
                    .width(Size::fill())
                    .height(Size::fill())
                    .background(ShaderFill::new(
                        TERMINAL_GRID_SHADER,
                        effect,
                        move |effect, bounds| {
                            let uniforms = [bounds.width(), bounds.height(), scale];
                            let bytes: Vec<u8> =
                                uniforms.iter().flat_map(|v| v.to_ne_bytes()).collect();
                            effect.make_shader(Data::new_copy(&bytes), &[], None)
                        },
                    ))
            }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_grid_shader_compiles() {
        assert!(compile_grid().is_some());
    }
}
