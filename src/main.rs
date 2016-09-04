/*
 * Project Rust-Bucket
 *
 * Design (wip):
 *  - world: state of objects
 *    - object: position, inertia
 *    - control_state: how an object is being externally controlled
 *  - tick_actions: all actions of all players in a single tick
 *    - world = world.clone()
 *    - world.update_control_states(tick_actions)
 *      - world.object[*].control_state.update_using(tick_actions)
 *    - world.update_objects()
 *  - gamestate: local copy of world, plus player information
 *    - sends actions to tick_actions, directly or via network
 */

extern crate gfx_graphics;
extern crate piston_window;
extern crate piston;
extern crate vecmath;

/*
extern crate rand;
use std::io::stdin;
use std::cmp::Ordering;
use rand::Rng;
use std::cmp::Ord;
 */


use piston_window::{WindowSettings, PistonWindow,
                    PressEvent, ReleaseEvent, RenderEvent, MouseCursorEvent};
use piston_window::{Context, G2d, clear, rectangle};
use piston_window::Transformed;
use piston::input::{Button, Key, MouseButton};
use piston_window::{Window, Size};
//use vecmath::vec2_scale;

use std::time::Instant;
use std::collections::HashMap;

mod world;
use world::{World, ObjectName};

pub mod color {
    pub static RED: [f32; 4] = [1.0, 0.0, 0.0, 1.0];
    pub static GREEN: [f32; 4] = [0.0, 1.0, 0.0, 1.0];
    pub static BLUE: [f32; 4] = [0.0, 0.0, 1.0, 1.0];

    pub static L_RED: [f32; 4] = [1.0, 0.5, 0.5, 1.0];
    pub static L_GREEN: [f32; 4] = [0.5, 1.0, 0.5, 1.0];
    pub static L_BLUE: [f32; 4] = [0.5, 0.5, 1.0, 1.0];

    pub static D_RED: [f32; 4] = [0.5, 0.0, 0.0, 1.0];
    pub static D_GREEN: [f32; 4] = [0.0, 0.5, 0.0, 1.0];
    pub static D_BLUE: [f32; 4] = [0.0, 0.0, 0.5, 1.0];
}


struct GameState {
    now: Instant,
    rotation: f64,

    window_half_width: f64,
    window_half_height: f64,

    fade_from_white: f32,
    mouse_position: [f64; 2],

    moving: MovingState,

    world: World,
    player_object_name: ObjectName,
}

#[derive(Default)]
struct MovingState {
    left: bool, right: bool,
    up: bool, down: bool,
}

impl MovingState {
    fn horizontal(&self) -> i32 {
        return if self.left {-1} else {0} + if self.right {1} else {0};
    }
    fn vertical(&self) -> i32 {
        return if self.down {-1} else {0} + if self.up {1} else {0};
    }
}

trait FloatDelta {
     fn get_delta(&self) -> (f64, Instant);
}

impl FloatDelta for Instant {
    fn get_delta(self: &Instant) -> (f64, Instant) {
        let now = Instant::now();
        let duration = now - *self;
        let delta_seconds = duration.as_secs() as f64;
        let delta_nano = duration.subsec_nanos() as f64;
        let delta = delta_seconds + delta_nano / 1_000_000_000.0;

        return (delta, now);
    }
}

fn update(gamestate: &mut GameState) {

    let (delta, now) = gamestate.now.get_delta();
    gamestate.now = now;

    gamestate.rotation += delta;

    if gamestate.fade_from_white >= 0.0 {
        gamestate.fade_from_white -= delta as f32 * 1.5;
    }

    gamestate.world.update();
}

trait ActionHandler {
    fn handle_action(&mut self, action: &Action);
    fn handle_motion(&mut self, motion: [f64; 2]);
}

impl ActionHandler for GameState {
    fn handle_action(&mut self, action: &Action) {
        fn update_movement(gs: &mut GameState) {
            gs.world.set_object_inertia(&gs.player_object_name,
                [gs.moving.horizontal() * 10, gs.moving.vertical() * 10]);
        };

        match *action {
            Action::StartMovingLeft => {
                println!("Started moving up.");
                self.moving.left = true;
                update_movement(self);
            },
            Action::StopMovingLeft => {
                println!("Stopped moving left.");
                self.moving.left = false;
                update_movement(self);
            },
            Action::StartMovingUp => {
                println!("Started moving up.");
                self.moving.up = true;
                update_movement(self);
            },
            Action::StopMovingUp => {
                println!("Stopped moving up.");
                self.moving.up = false;
                update_movement(self);
            },
            _ => (),
        }
    }

    fn handle_motion(&mut self, motion: [f64; 2]) {
        self.mouse_position = motion;
    }
}

fn draw(gamestate: &GameState, c: Context, g: &mut G2d) {
    clear(color::L_RED, g);

    {
        // Make 0,0 be the center of the window
        let c = c.trans(gamestate.window_half_width,
                        gamestate.window_half_height);
        // Flip so that positive numbers becomes 'up'.
        let c = c.flip_v();

        // let c = c.rot_rad(gamestate.rotation);
        //
        // rectangle(color::RED,
        //           [-50.0, -50.0, 100.0, 100.0], // rectangle
        //           c.transform, g);

        for (_, object) in gamestate.world.objects.iter() {
            let position = object.get_position();
            let c = c.trans(position[0] as f64 / 100.0, position[1] as f64 / 100.0);
            let c = c.rot_rad(gamestate.rotation);
            rectangle(color::RED, [-50.0, -50.0, 100.0, 100.0], c.transform, g);
        }
    }

    {
        let c = c.trans(gamestate.mouse_position[0],
                        gamestate.mouse_position[1]);
        let c = c.rot_rad(gamestate.rotation * 2.0);

        rectangle(color::BLUE, [-2.0, -2.0, 4.0, 4.0],
                  c.transform, g);
    }

    // Intro: Fade from white.
    let mut f = gamestate.fade_from_white;
    if gamestate.fade_from_white <= 0.0 {
        f = 0.0;
    }

    let r = c.viewport.unwrap().rect;
    let window_rect = [r[0] as f64, r[1] as f64, r[2] as f64, r[3] as f64];
    rectangle([1.0, 1.0, 1.0, f],
              window_rect,
              c.transform, g);
}

enum Action {
    StartMovingUp,
    StartMovingDown,
    StartMovingLeft,
    StartMovingRight,

    StopMovingUp,
    StopMovingDown,
    StopMovingLeft,
    StopMovingRight,
}

fn main() {
    let mut window: PistonWindow = WindowSettings::new("Hello", (640, 480))
        .exit_on_esc(true)
        .build()
        .unwrap_or_else(|e| { panic!("Failed to build PistonWindow: {}", e) });

    let Size{width, height} = window.draw_size();

    let mut key_press_map: HashMap<Button, Action> = HashMap::new();
    key_press_map.insert(Button::Keyboard(Key::W), Action::StartMovingUp);
    key_press_map.insert(Button::Keyboard(Key::S), Action::StartMovingDown);
    key_press_map.insert(Button::Keyboard(Key::A), Action::StartMovingLeft);
    key_press_map.insert(Button::Keyboard(Key::D), Action::StartMovingRight);
    key_press_map.insert(Button::Mouse(MouseButton::Left), Action::StartMovingLeft);

    let mut key_release_map = HashMap::new();
    key_release_map.insert(Button::Keyboard(Key::W), Action::StopMovingUp);
    key_release_map.insert(Button::Keyboard(Key::S), Action::StopMovingDown);
    key_release_map.insert(Button::Keyboard(Key::A), Action::StopMovingLeft);
    key_release_map.insert(Button::Keyboard(Key::D), Action::StopMovingRight);
    key_release_map.insert(Button::Mouse(MouseButton::Left), Action::StopMovingLeft);

    let mut world = World::new();
    let player_object_name = world.spawn_object();

    let gamestate: &mut GameState = &mut GameState {
        now: Instant::now(),
        rotation: 0.0,
        window_half_width: (width / 2) as f64,
        window_half_height: (height / 2) as f64,
        fade_from_white: 1.0,
        mouse_position: [0.0, 0.0],
        world: world,
        player_object_name: player_object_name,
        moving: Default::default(),
    };

    while let Some(e) = window.next() {
        if let Some(button) = e.press_args() {
            if let Some(action) = key_press_map.get(&button) {
                gamestate.handle_action(action);
            }
        }

        if let Some(button) = e.release_args() {
            if let Some(action) = key_release_map.get(&button) {
                gamestate.handle_action(action)
            }
        }

        if let Some(motion) = e.mouse_cursor_args() {
            gamestate.handle_motion(motion);
        }

        update(gamestate);

        if let Some(_) = e.render_args() {
            window.draw_2d(&e, |c, g| draw(gamestate, c, g));
        }
    }
}
