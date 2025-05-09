#![no_std]

pub mod command_accumulator;

pub extern crate bxcan;
pub extern crate log;

use arrayvec::ArrayString;
use bxcan::StandardId;
use fixedstr::str_format;
use int_enum::IntEnum;
#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};
use ringbuffer::RingBuffer;

#[derive(Debug, Clone, Copy)]
pub enum AnalogInput {
    AuxVoltage,
    PcbT,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DigitalOutput {
    Wakeup,
    // TODO: Add digital outputs
}

pub trait HardwareInterface {
    fn millis(&mut self) -> u64;

    fn reboot(&mut self);
    fn activate_dfu(&mut self);

    fn send_can(&mut self, frame: bxcan::Frame);

    fn get_analog_input(&mut self, input: AnalogInput) -> f32;

    fn set_digital_output(&mut self, output: DigitalOutput, value: bool);
}

// Parameter definitions

pub enum CanBitSelection {
    Bit(u8),
    Uint8(u8),
    Int8(u8),
    Function(fn(&[u8]) -> f32),
}

pub struct CanMap {
    pub id: bxcan::Id,
    pub bits: CanBitSelection,
    pub scale: f32,
}

pub struct ReportMap<'a> {
    pub name: &'a str,
    pub decimals: u8,
    pub scale: f32,
}

pub struct Parameter<'a, ID> {
    pub id: ID,
    pub display_name: &'a str,
    pub value: f32,
    pub decimals: u8,
    pub unit: &'a str,
    pub can_map: Option<CanMap>,
    pub report_map: Option<ReportMap<'a>>,
    pub update_timestamp: u64,
}

impl<'a, ID> Parameter<'a, ID> {
    pub const fn new(
        id: ID,
        display_name: &'a str,
        value: f32,
        decimals: u8,
        unit: &'a str,
        can_map: Option<CanMap>,
        report_map: Option<ReportMap<'a>>,
    ) -> Self {
        Self {
            id: id,
            display_name: display_name,
            value: value,
            decimals: decimals,
            unit: unit,
            can_map: can_map,
            report_map: report_map,
            update_timestamp: 0,
        }
    }
    pub fn set_value(&mut self, value: f32, millis: u64) {
        self.value = value;
        self.update_timestamp = millis;
    }
}

