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
        Self::with_speed(5.0)
    }

    fn with_speed(speed: f32) -> Self {
        let mut authored: serde_json::Value =
            serde_json::from_str(include_str!("../../main.scene.json")).unwrap();
        for entity in authored["entities"].as_array_mut().unwrap() {
            if entity["name"] == "Agnes" || entity["name"] == "Dad" {
                entity["components"]["sindri.script"]["properties"]["speed"] = speed.into();
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

    fn playback_speed(&self, name: &str) -> f64 {
        self.world
            .get(self.active_pose(name))
            .unwrap()
            .components["sindri.animation.sprite"]["speed"]
            .as_f64()
            .unwrap()
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
            let parent = self.entity(name);
            let poses: Vec<_> = self
                .world
                .entities()
                .filter(|(_, data)| data.parent == Some(parent))
                .collect();
            assert_eq!(poses.len(), 7, "no start/stop entities remain");
            let pose = self.active_pose(name);
            let placed = self.world.world_transform(pose).unwrap();
            assert_eq!(placed.position, root.position, "pose inherits its fighter");
            let mut expected_pose_scale = root.scale;
            if name == "Dad" && self.clip(name) == "attack_left" {
                expected_pose_scale[0] = -expected_pose_scale[0];
            }
            assert_eq!(
                placed.scale, expected_pose_scale,
                "pose sizing or attack mirror"
            );
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
                assert_eq!(
                    self.world.get(shadow).unwrap().components["sindri.animation.sprite"]["speed"],
                    self.playback_speed(name),
                    "shadow tempo matches the fighter"
                );
                let projection = self.world.world_transform(shadow).unwrap();
                assert_eq!(projection.scale[0], expected_pose_scale[0]);
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
    run.step(DT);
    assert_eq!(run.clip("Agnes"), "back_right", "no start transition");
    assert_eq!(run.clip("Dad"), "back_left", "no start transition");
    run.frames(11);
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
    run.step(DT);
    assert_eq!(run.clip("Agnes"), "idle_right", "no stop transition");
    assert_eq!(run.clip("Dad"), "idle_left", "no stop transition");
}

#[test]
fn crossing_sides_turns_both_fighters_without_blocking_movement() {
    let mut run = Run::new();
    run.set_x("Agnes", 3.0);
    run.set_x("Dad", -3.0);
    run.key(Key::A, true);
    run.key(Key::L, true);
    run.step(DT);
    assert!(run.x("Agnes") < 3.0, "movement on the first turn frame");
    assert!(run.x("Dad") > -3.0);
    assert_eq!(run.clip("Agnes"), "right_to_left");
    assert_eq!(run.clip("Dad"), "left_to_right");
    run.frames(4);
    assert!(run.x("Agnes") < 2.6, "movement throughout the turn");
    assert!(run.x("Dad") > -2.6);
    let agnes_x = run.x("Agnes");
    let dad_x = run.x("Dad");
    run.key(Key::A, false);
    run.key(Key::L, false);
    run.frames(3);
    assert_eq!(
        run.x("Agnes"),
        agnes_x,
        "release stops translation immediately"
    );
    assert_eq!(run.x("Dad"), dad_x);
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
            run.world
                .world_transform(run.entity(fighter))
                .unwrap()
                .position[1],
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

#[test]
fn three_pose_turns_match_their_shadows_and_can_be_interrupted_by_attacks() {
    let mut run = Run::new();
    for name in ["Agnes", "Dad"] {
        let turn = run.entity(&format!("{name} Turn"));
        let pivot = if name == "Agnes" { "turn_5" } else { "turn_4" };
        let end = if name == "Agnes" { "turn_8" } else { "turn_7" };
        for (id, data) in run.world.entities() {
            if id != turn && data.parent != Some(turn) {
                continue;
            }
            let clips = &data.components["sindri.animation.sprite"]["clips"];
            for (direction, expected) in [
                ("right_to_left", ["turn_0", pivot, end]),
                ("left_to_right", [end, pivot, "turn_0"]),
            ] {
                let frames: Vec<_> = clips[direction]["frames"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|frame| frame.as_str().unwrap())
                    .collect();
                assert_eq!(frames, expected);
                assert_eq!(clips[direction]["seconds_per_frame"], 0.03);
            }
        }
    }
    run.set_x("Agnes", 3.0);
    run.set_x("Dad", -3.0);
    run.step(DT);
    assert_eq!(run.clip("Agnes"), "right_to_left");
    assert_eq!(run.clip("Dad"), "left_to_right");
    run.key(Key::Space, true);
    run.key(Key::I, true);
    run.step(DT);
    assert_eq!(run.clip("Agnes"), "attack_left", "attack interrupts pivot");
    assert_eq!(run.clip("Dad"), "attack_right");
}

#[test]
fn analog_walk_tempo_tracks_travel_and_changes_without_restarting() {
    let mut run = Run::new();
    run.set_x("Dad", 8.0);
    run.input.apply(InputEvent::TouchStarted {
        id: 1,
        x: 120.0,
        y: 510.0,
    });
    run.step(DT);
    run.input.apply(InputEvent::TouchMoved {
        id: 1,
        x: 240.0,
        y: 510.0,
    });
    run.frames(6);
    let pose = run.active_pose("Agnes");
    let frame = run.animations.frame(pose).unwrap();
    assert!(frame > 0, "full speed advances the walk cycle");
    assert!((run.playback_speed("Agnes") - 1.0).abs() < 0.0001);
    run.input.apply(InputEvent::TouchMoved {
        id: 1,
        x: 180.0,
        y: 510.0,
    });
    let before = run.x("Agnes");
    run.step(DT);
    let actual_rate = f64::from((run.x("Agnes") - before).abs() / (DT * 5.0));
    assert!(
        actual_rate > 0.0 && actual_rate < 0.75,
        "gentle input slows steps"
    );
    assert!((run.playback_speed("Agnes") - actual_rate).abs() < 0.0001);
    assert_eq!(run.active_pose("Agnes"), pose);
    assert!(
        run.animations.frame(pose).unwrap() >= frame,
        "tempo change keeps phase"
    );
    // Reverse retreat uses the same positive multiplier and reversed frames.
    run.input.apply(InputEvent::TouchMoved {
        id: 1,
        x: 60.0,
        y: 510.0,
    });
    run.step(DT);
    assert_eq!(run.clip("Agnes"), "back_right");
    assert!((run.playback_speed("Agnes") - actual_rate).abs() < 0.0001);
    run.key(Key::Space, true);
    run.step(DT);
    assert_eq!(run.clip("Agnes"), "attack_right");
    assert_eq!(
        run.playback_speed("Agnes"),
        1.0,
        "attack retains its timing"
    );
    run.key(Key::Space, false);
    run.input.apply(InputEvent::TouchEnded { id: 1 });
    run.frames(40);
    assert_eq!(run.clip("Agnes"), "idle_right");
    assert_eq!(run.playback_speed("Agnes"), 1.0);
}

#[test]
fn configured_velocity_scales_cadence_for_both_fighters() {
    let mut run = Run::with_speed(10.0);
    run.key(Key::A, true);
    run.key(Key::L, true);
    run.frames(6);
    for name in ["Agnes", "Dad"] {
        assert!((run.playback_speed(name) - 2.0).abs() < 0.0001);
    }
    let mut stopped = Run::with_speed(0.0);
    stopped.key(Key::D, true);
    stopped.key(Key::J, true);
    stopped.step(DT);
    assert_eq!(stopped.clip("Agnes"), "idle_right");
    assert_eq!(stopped.clip("Dad"), "idle_left");
}

#[test]
fn clipped_steps_slow_the_cycle_and_blocked_input_returns_to_idle() {
    let mut run = Run::new();
    run.set_x("Agnes", 8.58);
    run.set_x("Dad", -8.6);
    run.frames(8);
    run.key(Key::D, true);
    run.key(Key::J, true);
    let before = run.x("Agnes");
    run.step(DT);
    let actual_rate = f64::from((run.x("Agnes") - before) / (DT * 5.0));
    assert!(actual_rate > 0.0 && actual_rate < 0.3);
    assert!((run.playback_speed("Agnes") - actual_rate).abs() < 0.0001);
    assert_eq!(
        run.clip("Dad"),
        "idle_right",
        "blocked from the first frame"
    );
    run.step(DT);
    assert_eq!(
        run.clip("Agnes"),
        "idle_left",
        "no walking against boundary"
    );
    assert_eq!(run.playback_speed("Agnes"), 1.0);
    run.key(Key::D, false);
    run.key(Key::A, true);
    run.step(DT);
    assert_eq!(run.clip("Agnes"), "move_left");
    assert!((run.playback_speed("Agnes") - 1.0).abs() < 0.0001);
}

#[test]
fn dad_uses_256_sheets_and_mirrors_only_the_left_attack_pose() {
    let run = Run::new();
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    for (_, data) in run.world.entities() {
        if !data
            .name
            .as_deref()
            .is_some_and(|name| name.starts_with("Dad "))
        {
            continue;
        }
        let sprite = &data.components["sindri.sprite"]["texture"];
        let reference = sprite.as_str().unwrap();
        let texture = reference.split('#').next().unwrap();
        assert!(texture.ends_with("_256.png"), "{reference}");
        let bytes = std::fs::read(root.join(texture)).unwrap();
        let width = u32::from_be_bytes(bytes[16..20].try_into().unwrap());
        let height = u32::from_be_bytes(bytes[20..24].try_into().unwrap());
        assert_eq!([width, height], [540, 768], "180x256 frames in a 3x3 grid");
        let sheet: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(root.join(texture.replace(".png", ".sheet"))).unwrap(),
        )
        .unwrap();
        assert_eq!(sheet["anchor"], "bottom");
        assert_eq!(sheet["grid"]["names"].as_array().unwrap().len(), 9);
        let clips = data.components["sindri.animation.sprite"]["clips"]
            .as_object()
            .unwrap();
        for clip in clips.values() {
            for frame in clip["frames"].as_array().unwrap() {
                assert!(sheet["grid"]["names"].as_array().unwrap().contains(frame));
            }
        }
    }
    let left = run.world.get(run.entity("Dad Attack Left")).unwrap();
    let right = run.world.get(run.entity("Dad Attack Right")).unwrap();
    assert_eq!(
        left.components["sindri.sprite"]["texture"],
        right.components["sindri.sprite"]["texture"]
    );
    assert_eq!(left.transform_3d.unwrap().scale, [-1.0, 1.0, 1.0]);
    assert_eq!(right.transform_3d.unwrap().scale, [1.0, 1.0, 1.0]);
    assert_eq!(
        left.components["sindri.animation.sprite"]["clips"]["attack_left"]["frames"],
        right.components["sindri.animation.sprite"]["clips"]["attack_right"]["frames"]
    );
}
