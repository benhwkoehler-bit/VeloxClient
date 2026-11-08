//! VeloxClient: the frosted material of the macOS Dock, shared by the shell, the
//! launch button and every card. A calm neutral grey, a blur of whatever sits
//! behind the big panels and one thin light hairline around the edge. Boxes stay
//! rectangular with slightly rounded corners.
//!
//! Clear elements such as the launch button are liquid glass: they bend the real
//! picture behind them. The refraction, Fresnel and glare maths are ported from
//! GLSL to SkSL from https://github.com/iyinchao/liquid-glass-studio
//! Copyright (c) 2024 Charles Yin, MIT licensed, see `glass.LICENSE`.

use std::rc::Rc;

use freya::engine::prelude::{Data, RuntimeEffect};
use freya::prelude::*;
use skia_safe::canvas::SaveLayerRec;
use skia_safe::runtime_effect::RuntimeShaderBuilder;
use skia_safe::{Rect as SkRect, image_filters};

use super::rounded;

const GLASS_RADIUS: f32 = 6.;
pub const GLASS_RADIUS_PX: f32 = GLASS_RADIUS;
const LARGEST_CARD_RADIUS: f32 = 24.;
const PANEL_BLUR: f32 = 16.;

const PANE_TINT: u8 = 16;
const PANE_HOVER_TINT: u8 = 40;
/// How far the refraction can reach for backdrop pixels, in logical pixels
const REFRACTION_REACH: f32 = 26.;
const LIQUID_BLUR: f32 = 3.;
const EDGE_ALPHA: u8 = 10;
const EDGE_HOVER_ALPHA: u8 = 44;

/// Dark palette colours whose channels differ less than this count as neutral
const NEUTRAL_SPREAD: u8 = 32;
const FROST_LIFT: f32 = 0.09;

const GLASS_SHADER: &str = r#"
uniform float4 iRect;
uniform float iRadius;
uniform float iScale;
uniform float4 iTint;

const float HAIRLINE = 1.0;
const float HAIRLINE_ALPHA = 0.16;
const float TOP_ALPHA = 0.08;

float boxDistance(float2 p, float2 halfSize, float radius) {
    float2 q = abs(p) - halfSize + radius;
    return length(max(q, float2(0.0))) + min(max(q.x, q.y), 0.0) - radius;
}

half4 main(float2 fragCoord) {
    float2 halfSize = iRect.zw * 0.5;
    float2 p = fragCoord - iRect.xy - halfSize;
    float inset = -boxDistance(p, halfSize, iRadius) / iScale;

    float4 c = float4(iTint.rgb * iTint.a, iTint.a);
    float line = smoothstep(HAIRLINE + 0.5, HAIRLINE - 0.5, inset);
    float upper = smoothstep(halfSize.y, -halfSize.y, p.y);
    return half4(mix(c, float4(1.0), line * (HAIRLINE_ALPHA + TOP_ALPHA * upper)));
}
"#;

const LIQUID_SHADER: &str = r#"
uniform shader content;
uniform float2 iSize;
uniform float iRadius;
uniform float4 iTint;

const float PI = 3.14159265;
const float THICKNESS = 16.0;
const float REFRACTIVE_INDEX = 1.5;
const float REFRACTION_DISTANCE = 18.0;
const float FRESNEL_RANGE = 30.0;
const float FRESNEL_HARDNESS = 0.02;
const float FRESNEL_FACTOR = 0.3;
const float GLARE_RANGE = 24.0;
const float GLARE_HARDNESS = 0.08;
const float GLARE_FACTOR = 0.9;
const float GLARE_CONVERGENCE = 0.5;
const float GLARE_ANGLE = -0.785398;

float falloff(float edge, float range, float hardness) {
    float ratio = 500.0 / range;
    float base = 1.0 + edge / 1500.0 * ratio * ratio + hardness;
    return clamp(pow(max(base, 0.0), 5.0), 0.0, 1.0);
}

float boxDistance(float2 p, float2 halfSize, float radius) {
    float2 q = abs(p) - halfSize + radius;
    return length(max(q, float2(0.0))) + min(max(q.x, q.y), 0.0) - radius;
}

float2 boxNormal(float2 p, float2 halfSize, float radius) {
    float2 q = abs(p) - halfSize + radius;
    float2 side = q.x > 0.0 && q.y > 0.0
        ? normalize(q)
        : (q.x > q.y ? float2(1.0, 0.0) : float2(0.0, 1.0));
    return side * sign(p + float2(0.0001));
}

float refraction(float inset) {
    if (inset <= 0.0 || inset >= THICKNESS) { return 0.0; }
    float ratio = 1.0 - inset / THICKNESS;
    float incident = asin(clamp(ratio * ratio, 0.0, 1.0));
    float transmitted = asin(sin(incident) / REFRACTIVE_INDEX);
    return -tan(transmitted - incident);
}

float glareAt(float2 normal) {
    float angle = (atan(normal.y, normal.x) - PI / 4.0 + GLARE_ANGLE) * 2.0;
    bool farSide = (angle > PI * 1.5 && angle < PI * 3.5) || angle < PI * -0.5;
    float strength = (0.5 + sin(angle) * 0.5) * (farSide ? 0.8 : 1.2) * GLARE_FACTOR;
    return clamp(pow(strength, 0.3 + GLARE_CONVERGENCE * 1.5), 0.0, 1.0);
}

half4 main(float2 coord) {
    float2 halfSize = iSize * 0.5;
    float2 p = coord - halfSize;
    float inset = -boxDistance(p, halfSize, iRadius);
    float2 normal = boxNormal(p, halfSize, iRadius);

    float3 color = content.eval(coord - normal * refraction(inset) * REFRACTION_DISTANCE).rgb;
    color = mix(color, iTint.rgb, iTint.a);

    float fresnel = falloff(-inset, FRESNEL_RANGE, FRESNEL_HARDNESS) * FRESNEL_FACTOR;
    float glare = falloff(-inset, GLARE_RANGE, GLARE_HARDNESS) * glareAt(normal) * 0.7;
    color = mix(color, float3(1.0), clamp(fresnel + glare, 0.0, 1.0));
    return half4(half3(color), 1.0);
}
"#;

thread_local! {
    static LIQUID_EFFECT: Option<RuntimeEffect> = compile_liquid();
}

fn compile_liquid() -> Option<RuntimeEffect> {
    RuntimeEffect::make_for_shader(LIQUID_SHADER, None)
        .inspect_err(|err| tracing::warn!("[theme] liquid glass shader failed: {err}"))
        .ok()
}

/// Glass that refracts whatever was drawn behind it
fn liquid_fill(tint: Color, blur: f32) -> Fill {
    let Some(effect) = LIQUID_EFFECT.with(Clone::clone) else {
        return tint.into();
    };
    let radius = super::rounded_px(GLASS_RADIUS);
    let channels = [tint.r(), tint.g(), tint.b(), tint.a()].map(|c| f32::from(c) / 255.);
    RenderCallback::new(move |ctx: &mut FillRenderContext| {
        let RenderTarget::Background { canvas, .. } = &ctx.target else {
            return;
        };
        let (width, height) = (ctx.size.width, ctx.size.height);
        if width <= 0. || height <= 0. {
            return;
        }
        let mut builder = RuntimeShaderBuilder::new(effect.clone());
        let uniforms = builder
            .set_uniform_float("iSize", &[width, height])
            .and_then(|_| builder.set_uniform_float("iRadius", &[radius]))
            .and_then(|_| builder.set_uniform_float("iTint", &channels));
        if let Err(err) = uniforms {
            tracing::warn!("[theme] liquid glass uniforms failed: {err:?}");
            return;
        }
        let blurred = image_filters::blur((blur, blur), None, None, None);
        let Some(filter) = image_filters::runtime_shader_with_options(
            &builder,
            REFRACTION_REACH,
            "content",
            blurred,
            false,
        ) else {
            return;
        };
        let bounds = SkRect::from_wh(width, height);
        canvas.save_layer(&SaveLayerRec::default().bounds(&bounds).backdrop(&filter));
        canvas.restore();
    })
    .into()
}

thread_local! {
    static GLASS_EFFECT: Option<RuntimeEffect> = compile_glass();
    static GLASS_SOURCE: Rc<str> = Rc::from(GLASS_SHADER);
}

fn compile_glass() -> Option<RuntimeEffect> {
    RuntimeEffect::make_for_shader(GLASS_SHADER, None)
        .inspect_err(|err| tracing::warn!("[theme] glass shader failed: {err}"))
        .ok()
}

/// `tint` with the hairline on top, or plain `tint` if the shader did not compile
fn glass_fill(tint: Color) -> Fill {
    let Some(effect) = GLASS_EFFECT.with(Clone::clone) else {
        return tint.into();
    };
    let scale = super::scale_factor();
    let radius = super::rounded_px(GLASS_RADIUS) * scale;
    let channels = [tint.r(), tint.g(), tint.b(), tint.a()].map(|c| f32::from(c) / 255.);
    ShaderFill::new(
        GLASS_SOURCE.with(Clone::clone),
        effect,
        move |effect, bounds| {
            let uniforms = [
                bounds.origin.x,
                bounds.origin.y,
                bounds.width(),
                bounds.height(),
                radius,
                scale,
                channels[0],
                channels[1],
                channels[2],
                channels[3],
            ];
            let bytes: Vec<u8> = uniforms.iter().flat_map(|v| v.to_ne_bytes()).collect();
            effect.make_shader(Data::new_copy(&bytes), &[], None)
        },
    )
    .into()
}

/// Boxes stay rectangular: every card radius settles on the same small corner.
/// Tiny radii and the huge ones meaning "fully round" pass through untouched.
pub fn capped_radius(radius: f32) -> f32 {
    if radius > GLASS_RADIUS && radius <= LARGEST_CARD_RADIUS {
        GLASS_RADIUS
    } else {
        radius
    }
}

pub fn glass_corners() -> CornerRadius {
    rounded(GLASS_RADIUS)
}

/// A clear pane of liquid glass for the launch button, chips and highlights
pub fn glass_pane(hovering: bool) -> Fill {
    liquid_fill(pane_tint(hovering), LIQUID_BLUR)
}

fn pane_tint(hovering: bool) -> Color {
    Color::WHITE.with_a(if hovering { PANE_HOVER_TINT } else { PANE_TINT })
}

pub fn glass_edge(hovering: bool) -> Color {
    Color::WHITE.with_a(if hovering {
        EDGE_HOVER_ALPHA
    } else {
        EDGE_ALPHA
    })
}

pub fn glass_border(hovering: bool) -> Border {
    Border::new()
        .fill(glass_edge(hovering))
        .width(1.)
        .alignment(BorderAlignment::Inner)
}

/// The rail and top bar: a frosted pane that blurs what passes behind it
pub fn glass_panel(panel: Rect) -> Rect {
    panel.background(liquid_fill(pane_tint(false), PANEL_BLUR))
}

/// Moves `base` a share of the way to white, keeping its alpha
fn lifted(base: Color, share: f32) -> Color {
    let channel = |from: u8| (f32::from(from) + (255. - f32::from(from)) * share).round() as u8;
    Color::from_argb(
        base.a(),
        channel(base.r()),
        channel(base.g()),
        channel(base.b()),
    )
}

/// The Dock's grey for near-neutral surfaces; real colours like the brand stay
fn frosted(base: Color) -> Color {
    let (r, g, b) = (base.r(), base.g(), base.b());
    if r.max(g).max(b) - r.min(g).min(b) > NEUTRAL_SPREAD {
        return base;
    }
    let grey = ((u16::from(r) + u16::from(g) + u16::from(b)) / 3) as u8;
    lifted(Color::from_argb(base.a(), grey, grey, grey), FROST_LIFT)
}

/// Turns a solid surface colour into the frosted material
pub fn glass_over(base: Color) -> Fill {
    glass_fill(frosted(base))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn card_corners_are_capped_but_circles_and_small_corners_are_not() {
        assert_eq!(capped_radius(16.), GLASS_RADIUS);
        assert_eq!(capped_radius(24.), GLASS_RADIUS);
        assert_eq!(capped_radius(4.), 4.);
        assert_eq!(capped_radius(999.), 999.);
    }

    #[test]
    fn dark_neutrals_become_a_lifted_grey_and_keep_alpha() {
        let frost = frosted(Color::from_argb(200, 18, 17, 26));
        assert_eq!(frost.a(), 200);
        assert_eq!(frost.r(), frost.g());
        assert_eq!(frost.g(), frost.b());
        assert!(frost.r() > 20);
    }

    #[test]
    fn brand_colours_stay_untouched() {
        let brand = Color::from_argb(255, 124, 58, 237);
        assert_eq!(frosted(brand), brand);
    }

    #[test]
    fn the_glass_shader_compiles() {
        assert!(compile_glass().is_some());
    }

    #[test]
    fn the_liquid_shader_compiles() {
        assert!(compile_liquid().is_some());
    }

    #[test]
    fn hover_rims_are_brighter() {
        assert!(glass_edge(true).a() > glass_edge(false).a());
    }
}
