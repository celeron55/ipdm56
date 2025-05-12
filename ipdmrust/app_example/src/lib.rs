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

const ObcDcdc12VSupply: DigitalOutput = DigitalOutput::HOUT1;
const DcdcEnable: DigitalOutput = DigitalOutput::HOUT6;
const BatteryPump: DigitalOutput = DigitalOutput::HOUT4;
const BrakeBooster: DigitalOutput = DigitalOutput::HOUT10;

const BatteryNeutralSolenoid: DigitalOutput = DigitalOutput::LOUT3;
const BatteryHeatSolenoid: DigitalOutput = DigitalOutput::LOUT2;
const CoolingFan: DigitalOutput = DigitalOutput::LOUT4;
const HeatLoopPump: DigitalOutput = DigitalOutput::LOUT5;

const CpPwmToObc: PwmOutput = PwmOutput::SPWM1;

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
    // TODO: Delete these and replace with max discharge current and max charge
    //       current. These are redundant.
    PermitDischarge = 30,
    PermitCharge = 31,
    HvacRequested = 32,
    FocciCPPWM = 33,
    ActivateEvse = 34,
    BmsMaxChargeCurrent = 35,
    BmsMaxDischargeCurrent = 36,
}

static mut PARAMETERS: [Parameter<ParameterId>; 37] = [
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
        display_name: "OBC DC V",
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
        display_name: "OBC DC A",
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
        display_name: "OBC AC V",
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
    Parameter {
        id: ParameterId::PermitDischarge,
        display_name: "Permit dischg.",
        value: f32::NAN,
        decimals: 0,
        unit: "",
        can_map: Some(CanMap {
            id: bxcan::Id::Standard(StandardId::new(0x100).unwrap()),
            bits: CanBitSelection::Bit(1),
            scale: 1.0,
        }),
        update_timestamp: 0,
    },
    Parameter {
        id: ParameterId::PermitCharge,
        display_name: "Permit charge",
        value: f32::NAN,
        decimals: 0,
        unit: "",
        can_map: Some(CanMap {
            id: bxcan::Id::Standard(StandardId::new(0x100).unwrap()),
            bits: CanBitSelection::Bit(0),
            scale: 1.0,
        }),
        update_timestamp: 0,
    },
    Parameter {
        id: ParameterId::HvacRequested,
        display_name: "HVAC requested",
        value: f32::NAN,
        decimals: 0,
        unit: "",
        can_map: Some(CanMap {
            id: bxcan::Id::Standard(StandardId::new(0x570).unwrap()),
            bits: CanBitSelection::Function(|data: &[u8]| -> f32 {
                if data[0] == 2 && data[5] == 1 {
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
        id: ParameterId::FocciCPPWM,
        display_name: "CP duty",
        value: f32::NAN,
        decimals: 0,
        unit: "%",
        can_map: Some(CanMap {
            id: bxcan::Id::Standard(StandardId::new(0x506).unwrap()),
            bits: CanBitSelection::Uint8(1),
            scale: 1.0,
        }),
        update_timestamp: 0,
    },
    Parameter {
        id: ParameterId::ActivateEvse,
        display_name: "Activate EVSE",
        value: 0.0,
        decimals: 0,
        unit: "",
        can_map: None,
        update_timestamp: 0,
    },
    Parameter {
        id: ParameterId::BmsMaxChargeCurrent,
        display_name: "Max charge",
        value: f32::NAN,
        decimals: 1,
        unit: "A",
        can_map: Some(CanMap {
            id: bxcan::Id::Standard(StandardId::new(0x102).unwrap()),
            bits: CanBitSelection::Function(|data: &[u8]| -> f32 {
                (((data[2] as u16) << 8) | data[3] as u16) as f32
            }),
            scale: 0.1,
        }),
        update_timestamp: 0,
    },
    Parameter {
        id: ParameterId::BmsMaxDischargeCurrent,
        display_name: "Max discharge",
        value: f32::NAN,
        decimals: 1,
        unit: "A",
        can_map: Some(CanMap {
            id: bxcan::Id::Standard(StandardId::new(0x102).unwrap()),
            bits: CanBitSelection::Function(|data: &[u8]| -> f32 {
                (((data[4] as u16) << 8) | data[5] as u16) as f32
            }),
            scale: 0.1,
        }),
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
    last_solenoid_update_ms: u64,
    last_can_30ms: u64,
    last_can_200ms: u64,
    last_heater_update_ms: u64,
    request_heater_power_percent: f32,
}

impl MainState {
    pub fn new() -> Self {
        Self {
            update_counter: 0,
            log_can: false,
            last_millis: 0,
            dt_ms: 0,
            last_test_print_ms: 0,
            last_solenoid_update_ms: 0,
            last_can_30ms: 0,
            last_can_200ms: 0,
            last_heater_update_ms: 0,
            request_heater_power_percent: 0.0,
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

        self.read_inputs(hw);

        self.update_outputs(hw);

        self.update_charging(hw);

        if hw.millis() - self.last_heater_update_ms >= 2000 {
            self.last_heater_update_ms = hw.millis();
            self.update_heater(hw);
        }

        if hw.millis() - self.last_can_200ms >= 200 {
            self.last_can_200ms = hw.millis();
            self.send_can_200ms(hw);
        }

        if hw.millis() - self.last_can_30ms >= 30 {
            self.last_can_30ms = hw.millis();
            self.send_can_30ms(hw);
        }

        if hw.millis() - self.last_test_print_ms >= 15000 {
            self.last_test_print_ms = hw.millis();

            info!("-!- ipdmrust running");
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

    fn update_charging(&mut self, hw: &mut dyn HardwareInterface) {
        let activate_evse =
                get_parameter(ParameterId::FocciCPPWM).value >= 8.0 &&
                get_parameter(ParameterId::FocciCPPWM).value <= 96.0;

        get_parameter(ParameterId::ActivateEvse).set_value(
                if activate_evse { 1.0 } else { 0.0 }, hw.millis());
    }

    fn update_heater(&mut self, hw: &mut dyn HardwareInterface) {
        self.request_heater_power_percent = if
            get_parameter(ParameterId::HeaterT).value.is_nan() ||
            get_parameter(ParameterId::MainContactor).value < 0.5 ||
            get_parameter(ParameterId::PermitDischarge).value < 0.5
        {
            0.0
        } else if get_parameter(ParameterId::HeaterT).value < 55.0 {
            100.0
        } else if get_parameter(ParameterId::HeaterT).value < 60.0 {
            50.0
        } else {
            0.0
        };
        info!("request_heater_power_percent = {:?}", self.request_heater_power_percent);
    }

    fn read_inputs(&mut self, hw: &mut dyn HardwareInterface) {
        if hw.get_digital_input(DigitalInput::Group1OC) {
            info!("-!- DigitalInput::Group1OC");
        }
        if hw.get_digital_input(DigitalInput::Group2OC) {
            info!("-!- DigitalInput::Group2OC");
        }
        if hw.get_digital_input(DigitalInput::Group3OC) {
            info!("-!- DigitalInput::Group3OC");
        }
        if hw.get_digital_input(DigitalInput::Group4OC) {
            info!("-!- DigitalInput::Group4OC");
        }
    }

    fn update_outputs(&mut self, hw: &mut dyn HardwareInterface) {
        // TODO: Get ignition key state and control things based on it
        // TODO: Also follow ParameterId::HvacRequested

        if hw.millis() - self.last_solenoid_update_ms > 10000 {
            self.last_solenoid_update_ms = hw.millis();

            // Update battery solenoids
            let heat_battery = get_parameter(ParameterId::BatteryTMin).value < 3.0 &&
                    get_parameter(ParameterId::BatteryTMax).value < 25.0;
            let cool_battery = get_parameter(ParameterId::BatteryTMin).value > 23.0 &&
                    get_parameter(ParameterId::BatteryTMax).value > 30.0;
            hw.set_digital_output(BatteryNeutralSolenoid, !cool_battery && !heat_battery);
            hw.set_digital_output(BatteryHeatSolenoid, heat_battery);

            // Update cooling fan
            // TODO: Trigger on inverter, motor and OBC temperature also
            hw.set_digital_output(CoolingFan,
                    get_parameter(ParameterId::BatteryTMax).value > 35.0 &&
                    get_parameter(ParameterId::MainContactor).value > 0.5);

            // Update heating loop pump
            hw.set_digital_output(HeatLoopPump,
                    get_parameter(ParameterId::OutlanderHeaterHeating).value > 0.5 ||
                    get_parameter(ParameterId::OutlanderHeaterPowerPercent).value > 0.5 ||
                    get_parameter(ParameterId::OutlanderHeaterT).value > 30.0);

            info!("MainContactor: {:?}", get_parameter(ParameterId::MainContactor).value);
        }

        // Update OBC/DCDC 12V supply
        // TODO: When main contactor is closed, enable this (for DC/DC)
        // TODO: When ignition is on, enable this (for DC/DC and precharge)
        // TODO: Read CP value from Foccci and enable this based on that when
        //       ignition is off
        hw.set_digital_output(ObcDcdc12VSupply,
                get_parameter(ParameterId::MainContactor).value > 0.5 ||
                get_parameter(ParameterId::ActivateEvse).value > 0.5);

        // Update DC/DC enable
        hw.set_digital_output(DcdcEnable,
                get_parameter(ParameterId::MainContactor).value > 0.5);

        // Update battery pump
        hw.set_digital_output(BatteryPump,
                get_parameter(ParameterId::MainContactor).value > 0.5);

        // Update brake booster
        // TODO: Control based on ignition key state
        hw.set_digital_output(BrakeBooster, true);

        // Update CP PWM to OBC
        // (PWM value is received from Foccci)
        hw.set_pwm_output(CpPwmToObc,
                if get_parameter(ParameterId::FocciCPPWM).value.is_nan() { 0.00 }
                else { get_parameter(ParameterId::FocciCPPWM).value * 0.01 });

        // TODO: Send outlander OBC control CAN messages
    }

    fn send_can_200ms(&mut self, hw: &mut dyn HardwareInterface) {
        {
            // Outlander heater control
            let requested_power_command = if self.request_heater_power_percent > 70.0 {
                0xa2
            } else if self.request_heater_power_percent > 30.0 {
                0x32
            } else {
                0
            };
            self.send_normal_frame(hw, 0x188, &[
                0x03, 0x50,
                requested_power_command,
                0x4D, 0x00, 0x00, 0x00, 0x00
            ]);
        }

        {
            let charge_voltage_setpoint_Vx10: u16 = 3020;

            // TODO: Make maximum AC charge current configurable (ui8d already
            //       is capable of sending requests to change this)
            let user_current_request_ACA: f32 = 10.0;

            let ac_v = get_parameter(ParameterId::AcVoltage).value;
            let dc_v = get_parameter(ParameterId::ObcDcv).value;
            let dc_current_request_Ax10: u8 =
                if get_parameter(ParameterId::MainContactor).value > 0.5 &&
                        get_parameter(ParameterId::BmsMaxChargeCurrent).value > 10.0 &&
                        get_parameter(ParameterId::ActivateEvse).value > 0.5 {
                    let ac_request_DCA = ac_v / dc_v * user_current_request_ACA;
                    let obc_limit_DCA = 12.0;
                    let bms_limit_DCA = get_parameter(ParameterId::BmsMaxChargeCurrent).value;
                    (ac_request_DCA.min(obc_limit_DCA).min(bms_limit_DCA).max(0.0) * 10.0) as u8
                } else {
                    0
                };

            // Outlander OBC control
            self.send_normal_frame(hw, 0x286, &[
                (charge_voltage_setpoint_Vx10 >> 8) as u8,
                (charge_voltage_setpoint_Vx10 & 0xff) as u8,
                dc_current_request_Ax10, // DC current, 0.1A / bit
                0, 0, 0, 0, 0
            ]);
        }

        {
            // Send AcObcState to Foccci so that it can enable EVSE state C
            let ac_obc_state = if get_parameter(ParameterId::ActivateEvse).value > 0.5 {
                2
            } else {
                0
            };
            self.send_normal_frame(hw, 0x404, &[
                0x00 |
                    (1<<0) /* Foccci.enable */,
                ac_obc_state /* Foccci.AcObcState */,
                0, 0,
                0, 0, 0, 0,
            ]);
        }

        // TODO: Request main contactor based on FocciCPPWM
        // TODO: Request main contactor based on HvacRequested
    }

    fn send_can_30ms(&mut self, hw: &mut dyn HardwareInterface) {
        if get_parameter(ParameterId::MainContactor).value > 0.5 {
            // Outlander HV status message (for heater and OBC)
            // 10...30ms is fine for this (EV-Omega uses 30ms)
            let activate_evse = get_parameter(ParameterId::ActivateEvse).value > 0.5;
            self.send_normal_frame(hw, 0x285, &[
                0x00, 0x00,
                0x14 | if activate_evse { 0xb6 } else { 0 }, // 0xb6 = Activate EVSE (OBC)
                0x21, 0x90, 0xfe, 0x0c, 0x10
            ]);
        }
    }

    fn send_normal_frame(&mut self, hw: &mut dyn HardwareInterface,
            frame_id: u16, data: &[u8]) {
        hw.send_can(bxcan::Frame::new_data(
            bxcan::StandardId::new(frame_id).unwrap(),
            bxcan::Data::new(data).unwrap()
        ));
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
                                param.set_value(
                                    if (data[(bit_i as usize) / 8] & (1 << (bit_i % 8)) != 0) { 1.0 } else { 0.0 }
                                        * can_map.scale,
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
