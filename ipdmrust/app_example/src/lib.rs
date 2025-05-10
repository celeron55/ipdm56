#![no_std]

use common::*;

pub mod can_simulator;

pub extern crate bxcan;
pub extern crate log;
pub extern crate profont;

use arrayvec::ArrayString;
use bxcan::StandardId;
use fixedstr::str_format;
use int_enum::IntEnum;
#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};
use ringbuffer::RingBuffer;

#[repr(usize)]
#[derive(IntEnum, Debug, Clone, Copy)]
enum ParameterId {
    TicksMs = 0,
    AuxVoltage = 1,
    BatteryTMin = 2,
    BatteryTMax = 3,
    BatteryVMin = 4,
    BatteryVMax = 5,
    Soc = 6,
    RangeKm = 7,
    AllowedChargePower = 8,
    TripKm = 9,
    TripConsumption = 10,
    RecentKm = 11,
    RecentConsumption = 12,
    HvacCountdown = 13,
    HeaterT = 14,
    HeaterHeating = 15,
    HeaterPowerPercent = 16,
    CabinT = 17,
    MainContactor = 18,
    PrechargeFailed = 19,
    Balancing = 20,
    ObcDcv = 21,
    ObcDcc = 22,
    AcVoltage = 23,
    PdmState = 24, // TODO: Remove
    OutlanderHeaterT = 25,
    OutlanderHeaterHeating = 26,
    OutlanderHeaterPowerPercent = 27,
    CruiseActive = 28,
    CruiseRequested = 29,
}

static mut PARAMETERS: [Parameter<ParameterId>; 30] = [
    Parameter {
        id: ParameterId::TicksMs,
        display_name: "Ticks",
        value: 0.0,
        decimals: 0,
        unit: "ms",
        can_map: None,
        update_timestamp: 0,
    },
    Parameter {
        id: ParameterId::AuxVoltage,
        display_name: "Aux battery",
        value: f32::NAN,
        decimals: 2,
        unit: "V",
        can_map: None,
        update_timestamp: 0,
    },
    Parameter {
        id: ParameterId::BatteryTMin,
        display_name: "Bat T min",
        value: f32::NAN,
        decimals: 0,
        unit: "degC",
        can_map: Some(CanMap {
            id: bxcan::Id::Standard(StandardId::new(0x101).unwrap()),
            bits: CanBitSelection::Int8(3),
            scale: 1.0,
        }),
        update_timestamp: 0,
    },
    Parameter {
        id: ParameterId::BatteryTMax,
        display_name: "Bat T max",
        value: f32::NAN,
        decimals: 0,
        unit: "degC",
        can_map: Some(CanMap {
            id: bxcan::Id::Standard(StandardId::new(0x101).unwrap()),
            bits: CanBitSelection::Int8(4),
            scale: 1.0,
        }),
        update_timestamp: 0,
    },
    Parameter {
        id: ParameterId::BatteryVMin,
        display_name: "Bat V min",
        value: f32::NAN,
        decimals: 2,
        unit: "V",
        can_map: Some(CanMap {
            id: bxcan::Id::Standard(StandardId::new(0x101).unwrap()),
            bits: CanBitSelection::Function(|data: &[u8]| -> f32 {
                (((data[0] as u16) << 4) | ((data[1] as u16) >> 4)) as f32
            }),
            scale: 0.01,
        }),
        update_timestamp: 0,
    },
    Parameter {
        id: ParameterId::BatteryVMax,
        display_name: "Bat V max",
        value: f32::NAN,
        decimals: 2,
        unit: "V",
        can_map: Some(CanMap {
            id: bxcan::Id::Standard(StandardId::new(0x101).unwrap()),
            bits: CanBitSelection::Function(|data: &[u8]| -> f32 {
                ((((data[1] & 0x0f) as u16) << 8) | data[2] as u16) as f32
            }),
            scale: 0.01,
        }),
        update_timestamp: 0,
    },
    Parameter {
        id: ParameterId::Soc,
        display_name: "SoC",
        value: f32::NAN,
        decimals: 0,
        unit: "%",
        can_map: Some(CanMap {
            id: bxcan::Id::Standard(StandardId::new(0x102).unwrap()),
            bits: CanBitSelection::Uint8(6),
            scale: 100.0 / 255.0,
        }),
        update_timestamp: 0,
    },
    Parameter {
        id: ParameterId::RangeKm,
        display_name: "Range",
        value: f32::NAN,
        decimals: 0,
        unit: "km",
        can_map: None,
        update_timestamp: 0,
    },
    Parameter {
        id: ParameterId::AllowedChargePower,
        display_name: "Chg allow",
        value: f32::NAN,
        decimals: 0,
        unit: "kW",
        can_map: None,
        update_timestamp: 0,
    },
    Parameter {
        id: ParameterId::TripKm,
        display_name: "Trip",
        value: f32::NAN,
        decimals: 0,
        unit: "km",
        can_map: None,
        update_timestamp: 0,
    },
    Parameter {
        id: ParameterId::TripConsumption,
        display_name: "Trip",
        value: f32::NAN,
        decimals: 0,
        unit: "Wh/km",
        can_map: None,
        update_timestamp: 0,
    },
    Parameter {
        id: ParameterId::RecentKm,
        display_name: "Recent",
        value: f32::NAN,
        decimals: 0,
        unit: "km",
        can_map: None,
        update_timestamp: 0,
    },
    Parameter {
        id: ParameterId::RecentConsumption,
        display_name: "Recent",
        value: f32::NAN,
        decimals: 0,
        unit: "Wh/km",
        can_map: None,
        update_timestamp: 0,
    },
    Parameter {
        id: ParameterId::HvacCountdown,
        display_name: "HvacCountdown",
        value: 0.0,
        decimals: 1,
        unit: "s",
        can_map: None,
        update_timestamp: 0,
    },
    Parameter {
        id: ParameterId::HeaterT,
        display_name: "Heater T",
        value: f32::NAN,
        decimals: 0,
        unit: "degC",
        can_map: Some(CanMap {
            id: bxcan::Id::Standard(StandardId::new(0x398).unwrap()),
            bits: CanBitSelection::Function(|data: &[u8]| -> f32 {
                let t1 = data[3] as i8 - 40;
                let t2 = data[4] as i8 - 40;
                (if t1 > t2 { t1 } else { t2 }) as f32
            }),
            scale: 1.0,
        }),
        update_timestamp: 0,
    },
    Parameter {
        id: ParameterId::HeaterHeating,
        display_name: "Heater heating",
        value: f32::NAN,
        decimals: 0,
        unit: "",
        can_map: Some(CanMap {
            id: bxcan::Id::Standard(StandardId::new(0x398).unwrap()),
            bits: CanBitSelection::Function(|data: &[u8]| -> f32 {
                if data[5] > 0 {
                    1.0
                } else {
                    0.0
                }
            }),
            scale: 1.0,
        }),
        update_timestamp: 0,
    },
    Parameter {
        id: ParameterId::HeaterPowerPercent,
        display_name: "Heater power",
        value: f32::NAN,
        decimals: 0,
        unit: "%",
        can_map: Some(CanMap {
            id: bxcan::Id::Standard(StandardId::new(0x398).unwrap()),
            bits: CanBitSelection::Function(|data: &[u8]| -> f32 {
                // TODO: This accurate. The heater can be requested different
                //       power levels in 0x188
                if data[5] > 0 {
                    100.0
                } else {
                    0.0
                }
            }),
            scale: 1.0,
        }),
        update_timestamp: 0,
    },
    Parameter {
        id: ParameterId::CabinT,
        display_name: "CabinT",
        value: f32::NAN,
        decimals: 1,
        unit: "degC",
        can_map: None, // TODO: Get from UI8D
        update_timestamp: 0,
    },
    Parameter {
        id: ParameterId::MainContactor,
        display_name: "Main contactor",
        value: f32::NAN,
        decimals: 0,
        unit: "",
        can_map: Some(CanMap {
            id: bxcan::Id::Standard(StandardId::new(0x100).unwrap()),
            bits: CanBitSelection::Bit(2),
            scale: 1.0,
        }),
        update_timestamp: 0,
    },
    Parameter {
        id: ParameterId::PrechargeFailed,
        display_name: "Precharge failed",
        value: f32::NAN,
        decimals: 0,
        unit: "",
        can_map: Some(CanMap {
            id: bxcan::Id::Standard(StandardId::new(0x100).unwrap()),
            bits: CanBitSelection::Bit(6),
            scale: 1.0,
        }),
        update_timestamp: 0,
    },
    Parameter {
        id: ParameterId::Balancing,
        display_name: "Balancing",
        value: f32::NAN,
        decimals: 0,
        unit: "",
        can_map: Some(CanMap {
            id: bxcan::Id::Standard(StandardId::new(0x101).unwrap()),
            bits: CanBitSelection::Bit(5 * 8 + 0),
            scale: 1.0,
        }),
        update_timestamp: 0,
    },
    Parameter {
        id: ParameterId::ObcDcv,
        display_name: "OBC V DC",
        value: f32::NAN,
        decimals: 0,
        unit: "V",
        can_map: Some(CanMap {
            id: bxcan::Id::Standard(StandardId::new(0x389).unwrap()),
            bits: CanBitSelection::Uint8(0),
            scale: 2.0,
        }),
        update_timestamp: 0,
    },
    Parameter {
        id: ParameterId::ObcDcc,
        display_name: "OBC A DC",
        value: f32::NAN,
        decimals: 1,
        unit: "A",
        can_map: Some(CanMap {
            id: bxcan::Id::Standard(StandardId::new(0x389).unwrap()),
            bits: CanBitSelection::Uint8(2),
            scale: 0.1,
        }),
        update_timestamp: 0,
    },
    Parameter {
        id: ParameterId::AcVoltage,
        display_name: "AC voltage",
        value: f32::NAN,
        decimals: 0,
        unit: "V",
        can_map: Some(CanMap {
            id: bxcan::Id::Standard(StandardId::new(0x389).unwrap()),
            bits: CanBitSelection::Uint8(1),
            scale: 1.0,
        }),
        update_timestamp: 0,
    },
    Parameter {
        id: ParameterId::PdmState,
        display_name: "PdmState",
        value: f32::NAN,
        decimals: 0,
        unit: "",
        can_map: Some(CanMap {
            id: bxcan::Id::Standard(StandardId::new(0x203).unwrap()),
            bits: CanBitSelection::Function(|data: &[u8]| -> f32 { (data[0] >> 4) as f32 }),
            scale: 1.0,
        }),
        update_timestamp: 0,
    },
    Parameter {
        id: ParameterId::OutlanderHeaterT,
        display_name: "OutlH T",
        value: f32::NAN,
        decimals: 0,
        unit: "degC",
        can_map: Some(CanMap {
            id: bxcan::Id::Standard(StandardId::new(0x398).unwrap()),
            bits: CanBitSelection::Function(|data: &[u8]| -> f32 {
                let t1 = data[3] as i8 - 40;
                let t2 = data[4] as i8 - 40;
                (if t1 > t2 { t1 } else { t2 }) as f32
            }),
            scale: 1.0,
        }),
        update_timestamp: 0,
    },
    Parameter {
        id: ParameterId::OutlanderHeaterHeating,
        display_name: "OutlH heating",
        value: f32::NAN,
        decimals: 0,
        unit: "",
        can_map: Some(CanMap {
            id: bxcan::Id::Standard(StandardId::new(0x398).unwrap()),
            bits: CanBitSelection::Function(|data: &[u8]| -> f32 {
                if data[5] > 0 {
                    1.0
                } else {
                    0.0
                }
            }),
            scale: 1.0,
        }),
        update_timestamp: 0,
    },
    Parameter {
        id: ParameterId::OutlanderHeaterPowerPercent,
        display_name: "OutlH power",
        value: f32::NAN,
        decimals: 0,
        unit: "%",
        can_map: Some(CanMap {
            id: bxcan::Id::Standard(StandardId::new(0x398).unwrap()),
            bits: CanBitSelection::Function(|data: &[u8]| -> f32 {
                // TODO: This accurate. The heater can be requested different
                //       power levels in 0x188
                if data[5] > 0 {
                    100.0
                } else {
                    0.0
                }
            }),
            scale: 1.0,
        }),
        update_timestamp: 0,
    },
    Parameter {
        id: ParameterId::CruiseActive,
        display_name: "Cruise active",
        value: f32::NAN,
        decimals: 0,
        unit: "",
        can_map: Some(CanMap {
            id: bxcan::Id::Standard(StandardId::new(0x300).unwrap()),
            bits: CanBitSelection::Bit(2),
            scale: 1.0,
        }),
        update_timestamp: 0,
    },
    Parameter {
        id: ParameterId::CruiseRequested,
        display_name: "Cruise requested",
        value: 0.0,
        decimals: 0,
        unit: "",
        can_map: None,
        update_timestamp: 0,
    },
];

fn get_parameters() -> &'static mut [Parameter<'static, ParameterId>] {
    unsafe {
        return &mut PARAMETERS;
    }
}
fn get_parameter(id: ParameterId) -> &'static mut Parameter<'static, ParameterId> {
    unsafe {
        return &mut PARAMETERS[usize::from(id)];
    }
}

fn check_parameter_id_consistency() -> bool {
    for (i, param) in get_parameters().iter().enumerate() {
        if usize::from(param.id) != i {
            error!(
                "Parameter [{}].id == {}: ID mismatch",
                i,
                usize::from(param.id)
            );
            return false;
        }
    }
    return true;
}

pub struct MainState {
    update_counter: u32,
    log_can: bool,
    last_millis: u64,
    dt_ms: u64,
    last_test_print_ms: u64,
}

impl MainState {
    pub fn new() -> Self {
        Self {
            update_counter: 0,
            log_can: false,
            last_millis: 0,
            dt_ms: 0,
            last_test_print_ms: 0,
        }
    }

    // This should be called at 20ms interval
    pub fn update(&mut self, hw: &mut dyn HardwareInterface) {
        // Timekeeping
        let millis = hw.millis();
        self.dt_ms = if millis > self.last_millis {
            millis - self.last_millis
        } else {
            0
        };

        self.update_parameters(hw);

        if hw.millis() - self.last_test_print_ms > 1000 {
            self.last_test_print_ms = hw.millis();

            info!("Test print");
        }

        self.last_millis = millis;
        self.update_counter += 1;
    }

    fn timeout_parameters(&mut self, hw: &mut dyn HardwareInterface) {
        for (i, param) in get_parameters().iter_mut().enumerate() {
            if param.can_map.is_some() && !param.value.is_nan() {
                let age_ms = hw.millis() - param.update_timestamp;
                if age_ms >= 5000 {
                    param.value = f32::NAN;
                }
            }
        }
    }

    fn update_parameters(&mut self, hw: &mut dyn HardwareInterface) {
        get_parameter(ParameterId::TicksMs).set_value(hw.millis() as f32, hw.millis());
        get_parameter(ParameterId::AuxVoltage).set_value(hw.get_analog_input(AnalogInput::AuxVoltage), hw.millis());

        self.timeout_parameters(hw);
    }

    fn send_setting_frame(&mut self, hw: &mut dyn HardwareInterface,
            frame_id: u16, setting_id: u8, old_value: u16, new_value: u16) {
        let mut data: [u8; 8] = [0; 8];
        data[0] = setting_id;
        data[1..3].copy_from_slice(&old_value.to_be_bytes());
        data[3..5].copy_from_slice(&new_value.to_be_bytes());
        hw.send_can(bxcan::Frame::new_data(
            bxcan::StandardId::new(frame_id).unwrap(),
            bxcan::Data::new(&data).unwrap()
        ));
    }

    pub fn on_console_command(&mut self, command: &str, hw: &mut dyn HardwareInterface) -> bool {
        if command == "reboot" {
            hw.reboot();
            true
        } else if command == "dfu" {
            hw.activate_dfu();
            true
        } else if command == "panic" {
            panic!();
            true
        } else if command == "log can" {
            self.log_can = !self.log_can;
            info!(
                "Can logging {}",
                if self.log_can { "enabled" } else { "disabled" }
            );
            true
        } else {
            false
        }
    }

    pub fn list_console_commands(&self) {
        info!("  dfu  - Activate DFU mode");
        info!("  panic  - Call panic!()");
        info!("  log can  - Enable logging of CAN messages on console");
    }

    pub fn on_can(&mut self, frame: bxcan::Frame) {
        if self.log_can {
            if let bxcan::Id::Standard(id) = frame.id() {
                if let Some(data) = frame.data() {
                    info!("on_can: {:?}: {:?}", id, data);
                }
            }
        }

        for (i, param) in get_parameters().iter_mut().enumerate() {
            if let Some(can_map) = &param.can_map {
                if let Some(data) = frame.data() {
                    if can_map.id == frame.id() {
                        match can_map.bits {
                            CanBitSelection::Bit(bit_i) => {
                                param.set_value((data[(bit_i as usize) / 8] & (1 << (bit_i % 8)))
                                        as f32 * can_map.scale,
                                    self.last_millis);
                            }
                            CanBitSelection::Uint8(byte_i) => {
                                param.set_value((data[byte_i as usize] as u8) as
                                        f32 * can_map.scale,
                                    self.last_millis);
                            }
                            CanBitSelection::Int8(byte_i) => {
                                param.set_value((data[byte_i as usize] as i8) as
                                        f32 * can_map.scale,
                                    self.last_millis);
                            }
                            CanBitSelection::Function(function) => {
                                param.set_value(function(data) * can_map.scale,
                                    self.last_millis);
                            }
                        }
                    }
                }
            }
        }
    }
}
