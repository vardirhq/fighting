//! Run the authored scene and real Decay controller through Sindri's public host.
use std::time::Duration;

use sindri_core::{ComponentSchemaRegistry, EntityId, SceneDocument, World};
use sindri_decay::{ScriptComponent, ScriptFrame, ScriptSources, Scripts};
use sindri_platform::{InputEvent, InputState, Key};
use sindri_scene::{Effects2d, SceneExtractor, ScreenExtent, ScreenUi, SpriteAnimations};

const DT: f32 = 1.0 / 60.0;

struct Run {
    world: World,
    components: ComponentSchemaRegistry,
    sources: ScriptSources,
    scripts: Scripts,
    animations: SpriteAnimations,
    effects: Effects2d,
    screen: ScreenUi,
    input: InputState,
}

impl Run {
    fn combat(ai: bool) -> Self {
        let mut run = Self::configured(4.2, ai);
        run.frames(80);
        run
    }

    fn configured(speed: f32, ai: bool) -> Self {
        let mut authored: serde_json::Value =
            serde_json::from_str(include_str!("../../main.scene.json")).unwrap();
        for entity in authored["entities"].as_array_mut().unwrap() {
            if entity["name"] == "Agnes" || entity["name"] == "Dad" {
                let is_dad = entity["name"] == "Dad";
                let properties = &mut entity["components"]["sindri.script"]["properties"];
                properties["speed"] = speed.into();

                properties["ai_enabled"] = (ai && is_dad).into();
                properties["sound_enabled"] = false.into();
            }
        }
        let scene = SceneDocument::from_json(&authored.to_string()).unwrap();
        let mut components = SceneExtractor::new().unwrap().components().clone();
        components.register::<ScriptComponent>("Script").unwrap();
        let mut sources = ScriptSources::new();
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts");
        for entry in std::fs::read_dir(root).unwrap() {
            let path = entry.unwrap().path();
            if path.extension().is_some_and(|ext| ext == "decay") {
                let name = path.file_name().unwrap().to_string_lossy();
                sources.insert(
                    format!("scripts/{name}"),
                    std::fs::read_to_string(&path).unwrap(),
                );
            }
        }
        let mut run = Self {
            world: World::from_scene(&scene).unwrap().world,
            components,
            sources,
            scripts: Scripts::new(),
            animations: SpriteAnimations::new(),
            effects: Effects2d::default(),
            screen: ScreenUi::new(),
            input: InputState::default(),
        };
        run.step(DT);
        run
    }

    fn text(&self, name: &str) -> &str {
        self.world.get(self.entity(name)).unwrap().components["sindri.ui.text"]["text"]
            .as_str()
            .unwrap()
    }

    fn balance(&self, fighter: &str) -> f32 {
        self.text(&format!("{fighter} Balance"))
            .split_whitespace()
            .last()
            .unwrap()
            .trim_end_matches('%')
            .parse()
            .unwrap()
    }

    fn entity(&self, name: &str) -> EntityId {
        self.world
            .entities()
            .find(|(_, data)| data.name.as_deref() == Some(name))
            .map(|(id, _)| id)
            .unwrap_or_else(|| panic!("missing entity {name}"))
    }

    fn x(&self, name: &str) -> f32 {
        self.world
            .get(self.entity(name))
            .unwrap()
            .transform_3d
            .unwrap()
            .position[0]
    }

    fn set_x(&mut self, name: &str, x: f32) {
        let id = self.entity(name);
        self.world
            .get_mut(id)
            .unwrap()
            .transform_3d
            .as_mut()
            .unwrap()
            .position[0] = x;
    }

    fn key(&mut self, key: Key, down: bool) {
        self.input.apply(if down {
            InputEvent::KeyPressed(key)
        } else {
            InputEvent::KeyReleased(key)
        });
    }

    fn step(&mut self, dt: f32) {
        self.screen
            .update(
                &mut self.world,
                &self.components,
                ScreenExtent::new(360.0, 640.0),
                self.input.presses(),
            )
            .unwrap();
        let report = self.scripts.advance(
            &mut self.world,
            &self.components,
            ScriptFrame::new(&self.sources, &self.input, dt)
                .with_screen_ui(&self.screen)
                .with_effects(&mut self.effects)
                .with_animations(&mut self.animations),
        );
        assert!(report.failures.is_empty(), "{:?}", report.failures);
        self.animations
            .advance(&self.world, &self.components, dt)
            .unwrap();
        self.effects.advance(Duration::from_secs_f32(dt));
        self.input.begin_frame(Duration::from_secs_f32(dt));
    }

    fn frames(&mut self, count: usize) {
        for _ in 0..count {
            self.step(DT);
        }
    }

    fn tap(&mut self, key: Key) {
        self.key(key, true);
        self.step(DT);
        self.key(key, false);
    }
    fn close(&mut self) {
        self.set_x("Agnes", 0.0);
        self.set_x("Dad", 1.0);
        self.frames(1);
    }
    fn energy(&self, name: &str) -> f32 {
        self.text(&format!("{name} Stamina"))
            .split_whitespace()
            .last()
            .unwrap()
            .parse()
            .unwrap()
    }
    fn height(&self, name: &str) -> f32 {
        self.world
            .get(self.entity(name))
            .unwrap()
            .transform_3d
            .unwrap()
            .position[1]
    }
    fn touch(&mut self, name: &str, down: bool) {
        let rect = self.screen.rect(self.entity(name)).unwrap();
        if down {
            self.input.apply(InputEvent::TouchStarted {
                id: 2,
                x: 180.0 + rect.center[0] * 320.0,
                y: 320.0 - rect.center[1] * 320.0,
            });
        } else {
            self.input.apply(InputEvent::TouchEnded { id: 2 });
        }
        self.step(DT);
    }
}

#[test]
fn scene_uses_shapes_and_keeps_only_the_background_sprite() {
    let mut run = Run::combat(false);
    assert_eq!(
        run.world
            .entities()
            .filter(|(_, e)| e.components.contains_key("sindri.sprite"))
            .count(),
        1
    );
    for name in ["Agnes", "Dad"] {
        assert!(
            run.world
                .get(run.entity(&format!("{name} Body")))
                .unwrap()
                .components
                .contains_key("sindri.shape")
        );
    }
    run.key(Key::D, true);
    run.frames(120);
    assert!(
        run.x("Dad") - run.x("Agnes") >= 0.67,
        "grounded bodies cannot pass through each other"
    );
}

#[test]
fn jab_has_startup_range_and_hits_once_per_press() {
    let mut run = Run::combat(false);
    run.close();
    run.tap(Key::Space);
    run.frames(4);
    assert_eq!(run.balance("Dad"), 100.0);
    run.frames(10);
    assert_eq!(run.balance("Dad"), 88.0);
    run.frames(60);
    assert_eq!(run.balance("Dad"), 88.0);
    let mut miss = Run::combat(false);
    miss.tap(Key::Space);
    miss.frames(35);
    assert_eq!(miss.balance("Dad"), 100.0);
}

#[test]
fn jumping_evades_sweep_but_aerial_attack_can_connect() {
    let mut run = Run::combat(false);
    run.close();
    run.tap(Key::H);
    run.tap(Key::W);
    run.frames(24);
    assert_eq!(run.balance("Agnes"), 100.0, "jump must clear low sweep");
    assert!(run.height("Agnes Head") > 1.55);
    run.frames(50);
    assert!(run.height("Agnes Head") < 1.4, "gravity returns to floor");
    run.close();
    run.tap(Key::W);
    run.frames(8);
    run.tap(Key::Space);
    run.frames(12);
    assert_eq!(run.balance("Dad"), 82.0, "airborne jab becomes a kick");
}

#[test]
fn high_guard_blocks_jab_but_sweep_requires_low_guard() {
    let mut run = Run::combat(false);
    run.close();
    run.key(Key::N, true);
    run.frames(8);
    run.tap(Key::Space);
    run.frames(20);
    assert_eq!(run.balance("Dad"), 100.0);
    assert!(run.energy("Dad") < 90.0);
    run.frames(30);
    run.close();
    run.tap(Key::E);
    run.frames(30);
    assert_eq!(run.balance("Dad"), 74.0);
    let mut low = Run::combat(false);
    low.close();
    low.key(Key::N, true);
    low.key(Key::M, true);
    low.frames(8);
    low.tap(Key::E);
    low.frames(30);
    assert_eq!(low.balance("Dad"), 100.0);
    assert!(low.energy("Dad") < 70.0);
}

#[test]
fn timed_guard_parries_instead_of_dealing_damage() {
    let mut run = Run::combat(false);
    run.close();
    run.tap(Key::Space);
    run.frames(4);
    run.key(Key::N, true);
    run.frames(10);
    assert_eq!(run.balance("Dad"), 100.0);
    assert_eq!(run.text("Fight Feedback"), "PARRY!");
    assert!(
        !run.world.is_active(run.entity("Agnes Zone")),
        "parry interrupts attacker"
    );
}

#[test]
fn dodge_crosses_body_and_has_a_punishable_end() {
    let mut run = Run::combat(false);
    run.close();
    run.key(Key::D, true);
    run.tap(Key::K);
    run.frames(12);
    assert!(run.x("Agnes") > run.x("Dad"));
    assert!(run.energy("Agnes") < 85.0);
    run.key(Key::D, false);
    run.frames(25);
    assert!(run.energy("Agnes") > 78.0);
}

#[test]
fn whiff_recovery_rewards_a_counter_hit() {
    let mut run = Run::combat(false);
    run.set_x("Agnes", 0.0);
    run.set_x("Dad", 2.0);
    run.tap(Key::E);
    run.frames(26);
    run.set_x("Dad", 1.0);
    run.tap(Key::I);
    run.frames(15);
    assert_eq!(
        run.balance("Agnes"),
        85.0,
        "12 damage plus 25 percent punish bonus"
    );
    assert_eq!(run.text("Fight Feedback"), "PUNISH!");
}

#[test]
fn stamina_regenerates_and_held_attack_does_not_repeat() {
    let mut run = Run::combat(false);
    run.close();
    run.key(Key::E, true);
    run.frames(55);
    assert_eq!(run.balance("Dad"), 74.0);
    assert!(run.energy("Agnes") < 100.0);
    run.key(Key::E, false);
    run.frames(150);
    assert_eq!(run.energy("Agnes"), 100.0);
}

#[test]
fn ai_approaches_and_uses_combat_without_player_input() {
    let mut run = Run::combat(true);
    let start = run.x("Dad");
    run.frames(300);
    assert!(run.x("Dad") < start);
    assert!(run.balance("Agnes") < 100.0);
}

#[test]
fn round_timeout_and_rematch_reset_health_energy_and_positions() {
    let mut run = Run::combat(false);
    run.close();
    run.tap(Key::Space);
    run.frames(25);
    run.frames(2700);
    assert!(run.text("Bout Score").starts_with("1"));
    run.tap(Key::R);
    run.frames(3);
    assert_eq!(run.balance("Dad"), 100.0);
    assert_eq!(run.energy("Agnes"), 100.0);
    assert!(run.text("Bout Score").starts_with("0  :  0"));
}

#[test]
fn portrait_buttons_fit_and_touch_jump_jab_guard_dodge_work() {
    let mut run = Run::combat(false);
    for name in [
        "Dodge Button",
        "Attack Button",
        "Heavy Button",
        "Jump Button",
        "Guard Button",
    ] {
        let r = run.screen.rect(run.entity(name)).unwrap();
        assert!(r.center[0].abs() + r.size[0] * 0.5 <= 360.0 / 640.0);
        assert!(r.center[1].abs() + r.size[1] * 0.5 <= 1.0);
    }
    run.touch("Jump Button", true);
    run.touch("Jump Button", false);
    run.frames(8);
    assert!(run.height("Agnes Head") > 1.55);
    run.frames(60);
    run.close();
    run.touch("Attack Button", true);
    run.touch("Attack Button", false);
    run.frames(20);
    assert_eq!(run.balance("Dad"), 88.0);
    run.frames(25);
    run.touch("Guard Button", true);
    run.frames(10);
    assert!(run.world.is_active(run.entity("Agnes Guard")));
    run.touch("Guard Button", false);
    run.frames(2);
    assert!(!run.world.is_active(run.entity("Agnes Guard")));
    let before = run.x("Agnes");
    run.touch("Dodge Button", true);
    run.touch("Dodge Button", false);
    run.frames(8);
    assert!(run.x("Agnes") < before);
}

#[test]
fn training_disables_ai_and_touch_stick_moves_only_player() {
    let mut run = Run::combat(true);
    run.tap(Key::P);
    run.frames(5);
    let dad = run.x("Dad");
    let agnes = run.x("Agnes");
    run.input.apply(InputEvent::TouchStarted {
        id: 1,
        x: 125.0,
        y: 535.0,
    });
    run.step(DT);
    run.input.apply(InputEvent::TouchMoved {
        id: 1,
        x: 75.0,
        y: 535.0,
    });
    run.frames(15);
    assert!(run.x("Agnes") < agnes);
    assert_eq!(run.x("Dad"), dad);
}

#[test]
fn guard_break_takes_damage_and_confirmed_jab_can_chain_into_sweep() {
    let mut guard = Run::combat(false);
    guard.close();
    guard.key(Key::N, true);
    guard.key(Key::M, true);
    guard.frames(8);
    for _ in 0..3 {
        guard.close();
        guard.tap(Key::E);
        guard.frames(60);
    }
    assert_eq!(guard.balance("Dad"), 74.0, "third sweep exhausts low guard");
    let mut combo = Run::combat(false);
    combo.close();
    combo.tap(Key::Space);
    combo.frames(11);
    combo.tap(Key::E);
    combo.frames(45);
    assert_eq!(
        combo.balance("Dad"),
        62.0,
        "confirmed jab earns a sweep cancel"
    );
}

#[test]
fn match_finishes_at_three_and_rematch_clears_the_winner() {
    let mut run = Run::combat(false);
    for _ in 0..3 {
        for _ in 0..9 {
            run.close();
            run.tap(Key::Space);
            run.frames(32);
        }
        run.frames(120);
    }
    assert!(run.text("Bout Score").starts_with("3"));
    assert!(run.text("Bout Notice").contains("Agnes wins"));
    let before = run.x("Agnes");
    run.key(Key::D, true);
    run.frames(30);
    assert_eq!(run.x("Agnes"), before, "winner freezes combat");
    run.key(Key::D, false);
    run.tap(Key::R);
    run.frames(3);
    assert!(run.text("Bout Score").starts_with("0  :  0"));
    assert_eq!(run.balance("Dad"), 100.0);
}

#[test]
fn dodge_invulnerability_expires_before_the_dash_finishes() {
    let mut early = Run::combat(false);
    early.close();
    early.tap(Key::I);
    early.frames(3);
    early.tap(Key::K);
    for _ in 0..8 {
        early.set_x("Agnes", 0.0);
        early.set_x("Dad", 1.0);
        early.step(DT);
    }
    assert_eq!(early.balance("Agnes"), 100.0);
    let mut late = Run::combat(false);
    late.close();
    late.tap(Key::K);
    late.frames(8);
    late.set_x("Agnes", 0.0);
    late.set_x("Dad", 1.0);
    late.tap(Key::I);
    for _ in 0..12 {
        late.set_x("Agnes", 0.0);
        late.set_x("Dad", 1.0);
        late.step(DT);
    }
    assert_eq!(late.balance("Agnes"), 88.0, "dash end can be hit");
}

#[test]
fn multitouch_down_stick_plus_guard_selects_low_guard() {
    let mut run = Run::combat(false);
    run.input.apply(InputEvent::TouchStarted {
        id: 1,
        x: 125.0,
        y: 535.0,
    });
    run.step(DT);
    run.input.apply(InputEvent::TouchMoved {
        id: 1,
        x: 125.0,
        y: 595.0,
    });
    run.step(DT);
    run.touch("Guard Button", true);
    run.frames(4);
    assert!(run.world.is_active(run.entity("Agnes Guard")));
    assert!(
        run.height("Agnes Head") < 0.9,
        "downward touch crouches while second finger guards"
    );
}
