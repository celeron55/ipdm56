// Local modules
mod cli;
use cli::Cli;

// Internal crates
use common::*;
use app::can_simulator::CanSimulator;

// Platform-specific dependencies
use ::image as im;
use ::image::ImageBuffer;
use ::image::Pixel;
use ::image::Rgba;
use clap::Parser;
use embedded_graphics_simulator::{
    BinaryColorTheme, OutputSettingsBuilder, SimulatorDisplay, Window,
};
use piston_window::*;

// Embedded-compatible libraries
#[allow(unused_imports)]
use log::{info, warn};
use ringbuffer::{ConstGenericRingBuffer, RingBuffer};

// General purpose libraries
use std::f64::consts::PI;
//use nalgebra::{Vector2, Point2, UnitComplex, Rotation2};
use arrayvec::ArrayString;
use std::collections::HashMap;

const FPS: u64 = 50;
const UPS: u64 = 50;

const DISPLAY_W: u32 = 320;
const DISPLAY_H: u32 = 240;
const DISPLAY_BORDER: u32 = 10;
const DISPLAY_SCALE: u32 = 1;

struct HardwareImplementation {
    ms_counter: u64,
    can_sim: CanSimulator,
    digital_output_states: HashMap<DigitalOutput, bool>,
}

impl HardwareImplementation {
    fn new() -> Self {
        Self {
            ms_counter: 0,
            can_sim: CanSimulator::new(),
            digital_output_states: HashMap::new(),
        }
    }
}

impl HardwareImplementation {
}

impl HardwareInterface for HardwareImplementation {
    fn millis(&mut self) -> u64 {
        self.ms_counter
    }

    fn reboot(&mut self) {
        warn!("reboot() does nothing in desktop mode");
    }

    fn activate_dfu(&mut self) {
        warn!("activate_dfu() does nothing in desktop mode");
    }

    fn send_can(&mut self, frame: bxcan::Frame) {
        info!("send_can(): {:?}", frame);
    }

    fn get_analog_input(&mut self, input: AnalogInput) -> f32 {
        // TODO: ???
        14.0
    }

    fn set_digital_output(&mut self, output: DigitalOutput, value: bool) {
        if let Some(old_value) = self.digital_output_states.get(&output) {
            if value != *old_value {
                info!("set_digital_output(): {:?}: {:?}", output, value);
            }
        } else {
            info!("set_digital_output(): {:?}: {:?}", output, value);
        }
        self.digital_output_states.insert(output, value);
    }
}

fn main() {
    let cli = Cli::parse();

    stderrlog::new()
        .verbosity(log::LevelFilter::Info)
        .show_module_names(true)
        .module(module_path!())
        .module("common")
        .module("app")
        .init()
        .unwrap();
    log::set_max_level(log::LevelFilter::Info);

    let mut window: PistonWindow = WindowSettings::new(
        "ipdmrust",
        [
            (DISPLAY_W + DISPLAY_BORDER * 2) * DISPLAY_SCALE,
            (DISPLAY_H + DISPLAY_BORDER * 2) * DISPLAY_SCALE,
        ],
    )
    .exit_on_esc(true)
    .build()
    .unwrap();

    let event_settings = EventSettings::new().max_fps(FPS).ups(UPS);
    let mut events = Events::new(event_settings);
    window.events = events;

    let mut state = app::MainState::new();

    let mut hw = HardwareImplementation::new();

    let mut counter: u64 = 0;

    while let Some(e) = window.next() {
        if e.render_args().is_some() {
            // TODO: Draw something to window using e.g. window.draw_2d()
        }

        if e.update_args().is_some() {
            hw.can_sim.update(hw.ms_counter);
            while let Some(frame) = hw.can_sim.txbuf.dequeue() {
                state.on_can(frame);
            }

            state.update(&mut hw);

            counter += 1;
            hw.ms_counter += 1000 / UPS;
        }

        if let Some(piston_window::Button::Keyboard(key)) = e.press_args() {
            match key {
                Key::Q => {
                    break;
                }
                _ => {}
            }
        }
    }
}
