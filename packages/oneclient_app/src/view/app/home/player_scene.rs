//! VeloxClient: the calm scene the player settles into on home, held for 10 to 20 minutes.
//! The clock outlives the page, so leaving home and coming back keeps the same scene.

use std::cell::Cell;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use bytes::Bytes;
use oneclient_polyplus::api;

use crate::AppAssets;
use crate::hooks::wardrobe::CosmeticBundle;

const HOLD_MIN_SECS: u64 = 10 * 60;
const HOLD_SPREAD_SECS: u64 = 10 * 60;
const SOFA_GEOMETRY: &str = "animations/home/sofa.geo.json";
const SOFA_TEXTURE: &str = "animations/home/sofa.png";
/// Keeps the two random choices of one change from following each other.
const SCENE_SEED_SHIFT: u32 = 17;

struct Scene {
    animation: &'static str,
    has_sofa: bool,
}

const fn scene(animation: &'static str) -> Scene {
    Scene {
        animation,
        has_sofa: false,
    }
}

const SCENES: [Scene; 6] = [
    Scene {
        animation: "animations/home/sofa.json",
        has_sofa: true,
    },
    scene("animations/home/ground_sit.json"),
    scene("animations/home/sleeping.json"),
    scene("animations/home/standing.json"),
    scene("animations/home/mining.json"),
    scene("animations/home/dancing.json"),
];

#[derive(Clone, Copy, Debug, PartialEq)]
struct SceneClock {
    scene: usize,
    until: Instant,
}

impl SceneClock {
    /// Always opens on the sofa.
    fn first(now: Instant, seed: u64) -> Self {
        Self {
            scene: 0,
            until: now + hold_time(seed),
        }
    }

    fn at(self, now: Instant, seed: u64) -> Self {
        if now < self.until {
            return self;
        }
        Self {
            scene: next_scene(self.scene, seed >> SCENE_SEED_SHIFT),
            until: now + hold_time(seed),
        }
    }
}

fn hold_time(seed: u64) -> Duration {
    Duration::from_secs(HOLD_MIN_SECS + seed % (HOLD_SPREAD_SECS + 1))
}

/// Picks any scene except the current one.
fn next_scene(current: usize, seed: u64) -> usize {
    let others = SCENES.len() - 1;
    (current + 1 + (seed % others as u64) as usize) % SCENES.len()
}

fn random_seed() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.subsec_nanos().into())
}

thread_local! {
    static CLOCK: Cell<Option<SceneClock>> = const { Cell::new(None) };
    static ANIMATION_ENABLED: Cell<bool> = const { Cell::new(true) };
}

pub fn animation_enabled() -> bool {
    ANIMATION_ENABLED.get()
}

pub fn set_animation_enabled(enabled: bool) {
    ANIMATION_ENABLED.set(enabled);
}

/// The scene to show right now, moving on once its time is up.
pub fn current_scene() -> usize {
    let now = Instant::now();
    let seed = random_seed();
    CLOCK.with(|clock| {
        let next = clock
            .get()
            .map_or_else(|| SceneClock::first(now, seed), |c| c.at(now, seed));
        clock.set(Some(next));
        next.scene
    })
}

pub fn scene_animation(index: usize) -> Option<Bytes> {
    AppAssets::get_bytes(SCENES[index].animation)
}

/// The sofa travels as a cosmetic, which draws it with the player without a second model.
pub fn scene_props(index: usize) -> Vec<(api::BodySlot, CosmeticBundle)> {
    if !SCENES[index].has_sofa {
        return Vec::new();
    }
    vec![(
        api::BodySlot::Pet,
        CosmeticBundle {
            geometry: AppAssets::get_bytes(SOFA_GEOMETRY),
            texture: AppAssets::get_bytes(SOFA_TEXTURE),
            ..Default::default()
        },
    )]
}

#[cfg(test)]
mod tests {
    use super::*;
    use plusrenderer::animation::Loop;

    #[test]
    fn opens_on_the_sofa() {
        assert_eq!(SceneClock::first(Instant::now(), 123).scene, 0);
    }

    #[test]
    fn keeps_the_scene_until_its_time_is_up() {
        let now = Instant::now();
        let clock = SceneClock::first(now, 0);

        let later = clock.at(now + Duration::from_secs(HOLD_MIN_SECS - 1), 99);

        assert_eq!(later, clock);
    }

    #[test]
    fn moves_to_a_different_scene_when_time_is_up() {
        let now = Instant::now();
        let clock = SceneClock::first(now, 0);

        for seed in 0..64 {
            let next = clock.at(clock.until, seed << SCENE_SEED_SHIFT);
            assert_ne!(next.scene, clock.scene);
            assert!(next.scene < SCENES.len());
            assert!(next.until > clock.until);
        }
    }

    #[test]
    fn holds_for_ten_to_twenty_minutes() {
        for seed in [0, 1, HOLD_SPREAD_SECS, HOLD_SPREAD_SECS + 1, u64::MAX] {
            let hold = hold_time(seed).as_secs();
            assert!((600..=1200).contains(&hold), "{hold}");
        }
    }

    #[test]
    fn every_scene_is_a_loop_that_drives_the_player() {
        for index in 0..SCENES.len() {
            let bytes = scene_animation(index).expect("asset");
            let json = std::str::from_utf8(&bytes).expect("utf8");

            let animations =
                plusrenderer::animation::parse(json, &mut Default::default()).expect("parses");

            let animation = animations.first().expect("one animation");
            assert_eq!(animation.loop_mode, Loop::Repeat, "{index}");
            assert!(animation.bones.iter().any(|b| b.name == "rightArm"));
        }
    }

    #[test]
    fn sofa_scene_brings_its_geometry_and_texture() {
        let props = scene_props(0);
        let (_, sofa) = props.first().expect("sofa");
        let geometry = sofa.geometry.as_deref().expect("geometry");

        plusrenderer::bedrock::parse_one(std::str::from_utf8(geometry).expect("utf8"))
            .expect("parses");
        assert!(sofa.texture.is_some());
    }

    #[test]
    fn other_scenes_have_no_props() {
        assert!(scene_props(1).is_empty());
    }
}
