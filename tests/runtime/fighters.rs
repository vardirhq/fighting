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
    fn new() -> Self {
        let scene = SceneDocument::from_json(include_str!("../../main.scene.json")).unwrap();
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
        self.check_poses_and_shadows();
    }

    fn frames(&mut self, count: usize) {
        for _ in 0..count {
            self.step(DT);
        }
    }

    fn active_pose(&self, fighter: &str) -> EntityId {
        let parent = self.entity(fighter);
        let poses: Vec<_> = self
            .world
            .entities()
            .filter(|(id, data)| {
                data.parent == Some(parent)
                    && self.world.is_active(*id)
                    && data.components.contains_key("sindri.animation.sprite")
            })
            .map(|(id, _)| id)
            .collect();
        assert_eq!(
            poses.len(),
            1,
            "{fighter} must have exactly one visible pose"
        );
        poses[0]
    }

    fn clip(&self, fighter: &str) -> &str {
        self.world
            .get(self.active_pose(fighter))
            .unwrap()
            .components["sindri.animation.sprite"]["playing"]
            .as_str()
            .unwrap()
    }

    fn check_poses_and_shadows(&self) {
        for name in ["Agnes", "Dad"] {
            let root = self.world.world_transform(self.entity(name)).unwrap();
            let expected_scale = if name == "Dad" {
                [4.21875, 6.0, 1.0]
            } else {
                [3.0, 3.0, 1.0]
            };
            assert_eq!(root.scale, expected_scale);
            let pose = self.active_pose(name);
            let placed = self.world.world_transform(pose).unwrap();
            assert_eq!(placed.position, root.position, "pose inherits its fighter");
            assert_eq!(placed.scale, root.scale, "switching poses preserves scale");
            let shadows: Vec<_> = self
                .world
                .entities()
                .filter(|(_, data)| data.parent == Some(pose))
                .map(|(id, _)| id)
                .collect();
            assert_eq!(shadows.len(), 9);
            for shadow in shadows {
                assert_eq!(
                    self.world.get(shadow).unwrap().components["sindri.sprite"]["layer"],
                    5,
                    "shadows stay below both fighters"
                );
                assert!(self.world.is_active(shadow));
                assert_eq!(self.animations.sprite(shadow), self.animations.sprite(pose));
            }
        }
    }
}

#[test]
fn both_fighters_retreat_facing_the_opponent_and_reverse_the_actual_frames() {
    let mut run = Run::new();
    assert_eq!(run.clip("Agnes"), "idle_right");
    assert_eq!(run.clip("Dad"), "idle_left");
    run.key(Key::A, true);
    run.key(Key::L, true);
    run.frames(12);
    assert!(run.x("Agnes") < -1.5);
    assert!(run.x("Dad") > 1.1);
    assert_eq!(run.clip("Agnes"), "back_right");
    assert_eq!(run.clip("Dad"), "back_left");
    for name in ["Agnes", "Dad"] {
        let mut seen = Vec::new();
        for _ in 0..5 {
            seen.push(
                run.animations
                    .sprite(run.active_pose(name))
                    .unwrap()
                    .to_owned(),
            );
            run.step(0.07);
        }
        for pair in seen.windows(2) {
            let last = |s: &str| s.rsplit('_').next().unwrap().parse::<usize>().unwrap();
            assert_eq!(last(&pair[1]), (last(&pair[0]) + 8) % 9);
        }
    }
    run.key(Key::A, false);
    run.key(Key::L, false);
    run.key(Key::D, true);
    run.key(Key::J, true);
    run.step(DT);
    assert_eq!(run.clip("Agnes"), "move_right");
    assert_eq!(run.clip("Dad"), "move_left");
    run.key(Key::D, false);
    run.key(Key::J, false);
    run.frames(8);
    assert_eq!(run.clip("Agnes"), "idle_right");
    assert_eq!(run.clip("Dad"), "idle_left");
}

#[test]
fn crossing_sides_turns_both_fighters_in_place_even_without_input() {
    let mut run = Run::new();
    run.set_x("Agnes", 3.0);
    run.set_x("Dad", -3.0);
    run.step(DT);
    assert_eq!(run.clip("Agnes"), "right_to_left");
    assert_eq!(run.clip("Dad"), "left_to_right");
    run.key(Key::A, true);
    run.key(Key::L, true);
    run.frames(4);
    assert_eq!(run.x("Agnes"), 3.0, "no translation during the turn");
    assert_eq!(run.x("Dad"), -3.0);
    run.key(Key::A, false);
    run.key(Key::L, false);
    run.frames(3);
    assert_eq!(run.clip("Agnes"), "idle_left");
    assert_eq!(run.clip("Dad"), "idle_right");
    run.set_x("Agnes", 0.0);
    run.set_x("Dad", 0.0);
    run.frames(8);
    assert_eq!(
        run.clip("Agnes"),
        "idle_left",
        "ties keep the previous facing"
    );
    assert_eq!(run.clip("Dad"), "idle_right");
}

#[test]
fn attacks_face_the_opponent_and_side_changes_take_priority() {
    let mut run = Run::new();
    run.key(Key::Space, true);
    run.key(Key::I, true);
    run.step(DT);
    assert_eq!(run.clip("Agnes"), "attack_right");
    assert_eq!(run.clip("Dad"), "attack_left");
    run.frames(14);
    assert!(run.effects.live() > 0, "attack effects fired");
    run.set_x("Agnes", 3.0);
    run.set_x("Dad", -3.0);
    run.step(DT);
    assert_eq!(run.clip("Agnes"), "right_to_left");
    assert_eq!(run.clip("Dad"), "left_to_right");
    run.frames(8);
    assert_eq!(run.clip("Agnes"), "idle_left");
    assert_eq!(run.clip("Dad"), "idle_right");
}

#[test]
fn touch_moves_only_agnes_horizontally_and_uses_backward_playback() {
    let mut run = Run::new();
    let dad_x = run.x("Dad");
    let agnes_x = run.x("Agnes");
    run.input.apply(InputEvent::TouchStarted {
        id: 1,
        x: 120.0,
        y: 510.0,
    });
    run.step(DT);
    run.input.apply(InputEvent::TouchMoved {
        id: 1,
        x: 55.0,
        y: 510.0,
    });
    run.frames(12);
    assert!(run.x("Agnes") < agnes_x);
    assert_eq!(run.x("Dad"), dad_x, "Dad must not consume Agnes's joystick");
    for fighter in ["Agnes", "Dad"] {
        assert_eq!(
            run.world.world_transform(run.entity(fighter)).unwrap().position[1],
            -2.6
        );
    }
    assert_eq!(run.clip("Agnes"), "back_right");
    run.input.apply(InputEvent::TouchEnded { id: 1 });
    run.frames(8);
    assert_eq!(run.clip("Agnes"), "idle_right");
}

#[test]
fn ground_position_orders_every_pose_with_shorter_fighter_winning_ties() {
    let mut run = Run::new();
    let layers = |run: &Run, fighter: &str, expected: i64| {
        let parent = run.entity(fighter);
        for (_, data) in run
            .world
            .entities()
            .filter(|(_, data)| data.parent == Some(parent))
        {
            assert_eq!(data.components["sindri.sprite"]["layer"], expected);
        }
    };
    layers(&run, "Agnes", 11);
    layers(&run, "Dad", 10);
    // X crossings and attack/turn pose switches must preserve the ground tie.
    run.set_x("Agnes", 1.1);
    run.key(Key::Space, true);
    run.frames(14);
    layers(&run, "Agnes", 11);
    run.set_x("Agnes", 3.0);
    run.frames(8);
    layers(&run, "Agnes", 11);
    layers(&run, "Dad", 10);
    let dad = run.entity("Dad");
    // Dad is physically closer: ground position overrides relative height.
    run.world
        .get_mut(dad)
        .unwrap()
        .transform_3d
        .as_mut()
        .unwrap()
        .position[1] = -2.8;
    run.step(DT);
    layers(&run, "Agnes", 10);
    layers(&run, "Dad", 11);
    // A tiny ground difference counts as the same line, avoiding noisy ties.
    run.world
        .get_mut(dad)
        .unwrap()
        .transform_3d
        .as_mut()
        .unwrap()
        .position[1] = -2.61;
    run.step(DT);
    layers(&run, "Agnes", 11);
    layers(&run, "Dad", 10);
    run.world
        .get_mut(dad)
        .unwrap()
        .transform_3d
        .as_mut()
        .unwrap()
        .position[1] = -2.4;
    run.step(DT);
    layers(&run, "Agnes", 11);
    layers(&run, "Dad", 10);
}
