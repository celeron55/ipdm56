#![no_std]

use common::*;

pub mod can_simulator;
pub mod parameters;
use parameters::*;

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
use bitvec::prelude::*;

const ObcDcdc12VSupply: DigitalOutput = DigitalOutput::HOUT1;
const DcdcEnable: DigitalOutput = DigitalOutput::HOUT6;
const BatteryPump: DigitalOutput = DigitalOutput::HOUT4;
const BrakeBooster: DigitalOutput = DigitalOutput::HOUT10;

const BatteryNeutralSolenoid: DigitalOutput = DigitalOutput::LOUT3;
const BatteryHeatSolenoid: DigitalOutput = DigitalOutput::LOUT2;
const CoolingFan: DigitalOutput = DigitalOutput::LOUT4;
const HeatLoopPump: DigitalOutput = DigitalOutput::LOUT5;

const CpPwmToObc: PwmOutput = PwmOutput::SPWM1;

pub struct MainState {
    update_counter: u32,
    log_can: bool,
    last_millis: u64,
    dt_ms: u64,
    last_test_print_ms: u64,
    last_solenoid_update_ms: u64,
    last_can_30ms: u64,
    last_can_200ms: u64,
    last_can_500ms: u64,
    last_heater_update_ms: u64,
    request_wakeup_and_main_contactor: bool,
    request_heater_power_percent: f32,
    ignition_last_on_ms: u64,
    last_aux_low_ms: u64,
}

impl MainState {
    pub fn new() -> Self {
        init_parameters();

        Self {
            update_counter: 0,
            log_can: false,
            last_millis: 0,
            dt_ms: 0,
            last_test_print_ms: 0,
            last_solenoid_update_ms: 0,
            last_can_30ms: 0,
            last_can_200ms: 0,
            last_can_500ms: 0,
            last_heater_update_ms: 0,
            request_wakeup_and_main_contactor: false,
            request_heater_power_percent: 0.0,
            ignition_last_on_ms: 0,
            last_aux_low_ms: 0,
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

        self.manage_power(hw);

        self.update_outputs(hw);

        self.update_charging(hw);

        if hw.millis() - self.last_heater_update_ms >= 2000 {
            self.last_heater_update_ms = hw.millis();
            self.update_heater(hw);
        }

        if hw.millis() - self.last_can_500ms >= 500 {
            self.last_can_500ms = hw.millis();
            self.send_can_500ms(hw);
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
        get_parameter(ParameterId::PcbT).set_value(hw.get_analog_input(AnalogInput::PcbT), hw.millis());

        if !get_parameter(ParameterId::Soc).value.is_nan() &&
                get_parameter(ParameterId::Soc).value >= 0.5 &&
                get_parameter(ParameterId::Soc).value <= 100.5 {
            get_parameter(ParameterId::LastSeenSoc).set_value(
                    get_parameter(ParameterId::Soc).value, hw.millis());
        }

        self.timeout_parameters(hw);
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

        if hw.get_digital_input(DigitalInput::Ignition) {
            self.ignition_last_on_ms = hw.millis();
        }
    }

    fn manage_power(&mut self, hw: &mut dyn HardwareInterface) {
        let ignition_input = hw.get_digital_input(DigitalInput::Ignition);

        let enough_soc_for_remote_operations =
                get_parameter(ParameterId::LastSeenSoc).value.is_nan() ||
                        get_parameter(ParameterId::LastSeenSoc).value >= 10.0;

        if get_parameter(ParameterId::AuxVoltage).value < 11.8 {
            self.last_aux_low_ms = hw.millis();
        }

        // This is to charge the 12V battery
        let daily_wakeup = (
            hw.millis() > (1000 * 3600 * 1)
            &&
            (
                // Always every 24h for 30min
                hw.millis() % (1000 * 3600 * 24) < (1000 * 60 * 30)
                ||
                (
                    // Every 4h for 30min if 12V battery is low
                    hw.millis() % (1000 * 3600 * 4) < (1000 * 60 * 30)
                    &&
                    hw.millis() - self.last_aux_low_ms < 1000 * 3600
                )
            )
        );

        self.request_wakeup_and_main_contactor =
                ignition_input ||
                get_parameter(ParameterId::ActivateEvse).value > 0.5 ||
                (enough_soc_for_remote_operations && (
                    get_parameter(ParameterId::HvacRequested).value > 0.5 ||
                    daily_wakeup
                ));
    }

    fn update_charging(&mut self, hw: &mut dyn HardwareInterface) {
        let mut charge_current = 0.0;
        if !get_parameter(ParameterId::CcsCurrent).value.is_nan() {
            charge_current += get_parameter(ParameterId::CcsCurrent).value;
        }
        if !get_parameter(ParameterId::ObcDcc).value.is_nan() {
            charge_current += get_parameter(ParameterId::ObcDcc).value;
        }

        if get_parameter(ParameterId::BatteryVMax).value >= 4.10 &&
                charge_current < 2.0 {
            get_parameter(ParameterId::ChargeComplete).set_value(1.0, hw.millis());
        } else if get_parameter(ParameterId::BatteryVMax).value < 4.04 {
            get_parameter(ParameterId::ChargeComplete).set_value(0.0, hw.millis());
        }

        // ActivateEvse applies to both DC and AC charging
        let activate_evse =
                get_parameter(ParameterId::FoccciCPPWM).value >= 1.0 &&
                get_parameter(ParameterId::FoccciCPPWM).value <= 96.0 &&
                get_parameter(ParameterId::ChargeComplete).value < 0.5;

        get_parameter(ParameterId::ActivateEvse).set_value(
                if activate_evse { 1.0 } else { 0.0 }, hw.millis());

        // ActivateObc applies only to AC charging and ends up instructing
        // Foccci into AC charging mode
        let activate_obc =
                get_parameter(ParameterId::FoccciCPPWM).value >= 8.0 &&
                get_parameter(ParameterId::FoccciCPPWM).value <= 96.0 &&
                get_parameter(ParameterId::ChargeComplete).value < 0.5;

        get_parameter(ParameterId::ActivateObc).set_value(
                if activate_evse { 1.0 } else { 0.0 }, hw.millis());
    }

    fn update_heater(&mut self, hw: &mut dyn HardwareInterface) {
        let heating_needed =
                (
                    hw.get_digital_input(DigitalInput::Ignition)
                    ||
                    get_parameter(ParameterId::HvacRequested).value > 0.5
                )
                &&
                (
                    get_parameter(ParameterId::CabinT).value.is_nan()
                    ||
                    get_parameter(ParameterId::CabinT).value < 28.0
                );

        let target_temperature = {
            if get_parameter(ParameterId::CabinT).value.is_nan() {
                60.0
            } else if get_parameter(ParameterId::CabinT).value < 10.0 {
                60.0
            } else if get_parameter(ParameterId::CabinT).value < 28.0 &&
                    hw.get_digital_input(DigitalInput::Ignition) {
                60.0
            } else {
                60.0 - (get_parameter(ParameterId::CabinT).value - 10.0) * 1.8
            }
        };

        self.request_heater_power_percent = if
            !heating_needed ||
            get_parameter(ParameterId::HeaterT).value.is_nan() ||
            get_parameter(ParameterId::MainContactor).value < 0.5 ||
            get_parameter(ParameterId::BmsMaxDischargeCurrent).value < 50.0
        {
            0.0
        } else if get_parameter(ParameterId::HeaterT).value < target_temperature - 5.0 {
            100.0
        } else if get_parameter(ParameterId::HeaterT).value < target_temperature {
            50.0
        } else {
            0.0
        };
        info!("request_heater_power_percent = {:?}", self.request_heater_power_percent);
    }

    fn update_outputs(&mut self, hw: &mut dyn HardwareInterface) {
        let ignition_input = hw.get_digital_input(DigitalInput::Ignition);

        // Require main contactor so that DC/DC can be operating
        let allow_solenoids = get_parameter(ParameterId::MainContactor).value > 0.5;

        if hw.millis() - self.last_solenoid_update_ms > 10000 {
            self.last_solenoid_update_ms = hw.millis();

            let heat_battery_to_t = {
                if get_parameter(ParameterId::HvacRequested).value > 0.5 {
                    22.0
                } else {
                    3.0
                }
            };

            // Update battery solenoids
            let heat_battery = (
                    get_parameter(ParameterId::BatteryTMin).value < heat_battery_to_t &&
                    get_parameter(ParameterId::BatteryTMax).value < 30.0 &&
                    (
                        // Only allow 100% duty cycle if battery < 3°C or
                        // cabin > 15°C
                        get_parameter(ParameterId::BatteryTMin).value < 3.0
                        ||
                        get_parameter(ParameterId::CabinT).value > 15.0
                        ||
                        hw.millis() % 120000 < 60000 // 50% duty cycle
                    )
            );
            let cool_battery = get_parameter(ParameterId::BatteryTMin).value > 23.0 &&
                    get_parameter(ParameterId::BatteryTMax).value > 30.0;
            hw.set_digital_output(BatteryNeutralSolenoid,
                    allow_solenoids && !cool_battery && !heat_battery);
            hw.set_digital_output(BatteryHeatSolenoid,
                    allow_solenoids && heat_battery);

            // Update cooling fan
            // TODO: Trigger on inverter, motor and OBC temperature also
            hw.set_digital_output(CoolingFan, allow_solenoids &&
                    get_parameter(ParameterId::BatteryTMax).value > 35.0);

            // Update heating loop pump
            hw.set_digital_output(HeatLoopPump,
                allow_solenoids &&
                (
                    get_parameter(ParameterId::OutlanderHeaterHeating).value > 0.5 ||
                    get_parameter(ParameterId::OutlanderHeaterPowerPercent).value > 0.5 ||
                    get_parameter(ParameterId::OutlanderHeaterT).value > 30.0
                )
            );

            info!("MainContactor: {:?}", get_parameter(ParameterId::MainContactor).value);
        }

        // Wakeup line
        // This powers inverter_controller and BMS
        hw.set_digital_output(DigitalOutput::Wakeup,
                ignition_input ||
                self.request_wakeup_and_main_contactor ||
                get_parameter(ParameterId::Precharging).value > 0.5 ||
                get_parameter(ParameterId::MainContactor).value > 0.5 ||
                get_parameter(ParameterId::ActivateEvse).value > 0.5 ||
                get_parameter(ParameterId::HvacRequested).value > 0.5);

        // Update OBC/DCDC 12V supply
        // * Enable this when:
        //   * The BMS is precharging (it needs the DC link voltage measurement)
        //   * When main contactor is closed (for DC/DC)
        //   * Based on the CP value from Foccci
        // * Toggle this off for 5 seconds when ignition key is turned off. That
        //   ensures charging will start afterwards if the OBC is in a weird
        //   state, into which it often likes to go
        // * Also toggle this for 5 seconds every 30 minutes if the DC/DC is not
        //   running while the main contactor is closed
        hw.set_digital_output(ObcDcdc12VSupply, {
            if self.last_millis > 60000 &&
                    self.last_millis - self.ignition_last_on_ms >= 1000 &&
                    self.last_millis - self.ignition_last_on_ms <= 6000 &&
                    get_parameter(ParameterId::ObcDcc).value <= 0.1 {
                false
            } else if self.last_millis > 120000 &&
                    // De-synced by 30s from wakeups happening on millis() % N,
                    // so that this doesn't mess up the precharge
                    (self.last_millis - 30000) % (1000 * 60 * 30) < (1000 * 5) &&
                    get_parameter(ParameterId::AuxVoltage).value <= 12.5 &&
                    get_parameter(ParameterId::DcdcStatus).value != 0x22 as f32 &&
                    get_parameter(ParameterId::Precharging).value < 0.5 &&
                    get_parameter(ParameterId::MainContactor).value > 0.5 {
                false
            } else {
                ignition_input ||
                self.request_wakeup_and_main_contactor ||
                get_parameter(ParameterId::Precharging).value > 0.5 ||
                get_parameter(ParameterId::MainContactor).value > 0.5 ||
                get_parameter(ParameterId::ActivateEvse).value > 0.5 ||
                get_parameter(ParameterId::HvacRequested).value > 0.5
            }
        });

        // Update DC/DC enable
        hw.set_digital_output(DcdcEnable,
                get_parameter(ParameterId::MainContactor).value > 0.5);

        // Update battery pump
        hw.set_digital_output(BatteryPump,
                get_parameter(ParameterId::MainContactor).value > 0.5);

        // Update brake booster
        hw.set_digital_output(BrakeBooster, ignition_input);

        // Update CP PWM to OBC
        // (PWM value is received from Foccci)
        hw.set_pwm_output(CpPwmToObc,
                if get_parameter(ParameterId::FoccciCPPWM).value.is_nan() { 0.00 }
                else { get_parameter(ParameterId::FoccciCPPWM).value * 0.01 });
    }

    fn send_can_500ms(&mut self, hw: &mut dyn HardwareInterface) {
        {
            // Send charge completion voltage setting to BMS
            let old_value: u16 = get_parameter(
                    ParameterId::BmsChargeCompleteVoltageSetting).value as u16;
            let new_value: u16 = 4120;
            if old_value != new_value {
                self.send_setting_frame(hw, 0x120, 0, old_value, new_value);
            }
        }

        {
            // Publish generic inputs for external monitoring

            let ignition = hw.get_digital_input(DigitalInput::Ignition);
            let m7 = hw.get_digital_input(DigitalInput::M7);
            let m8 = hw.get_digital_input(DigitalInput::M8);
            let m9 = hw.get_digital_input(DigitalInput::M9);
            let m10 = hw.get_digital_input(DigitalInput::M10);
            let m11 = hw.get_digital_input(DigitalInput::M11);
            let m12 = hw.get_digital_input(DigitalInput::M12);
            let m13 = hw.get_digital_input(DigitalInput::M13);

            let m1 = (hw.get_analog_input(AnalogInput::M1) * 128.0) as u16;
            let m2 = (hw.get_analog_input(AnalogInput::M2) * 128.0) as u16;
            let m3 = (hw.get_analog_input(AnalogInput::M3) * 128.0) as u16;
            let m4 = (hw.get_analog_input(AnalogInput::M4) * 128.0) as u16;
            let m5 = (hw.get_analog_input(AnalogInput::M5) * 128.0) as u16;
            let m6 = (hw.get_analog_input(AnalogInput::M6) * 128.0) as u16;

            let mut data = [0u8; 8];
            let mut bits = data.view_bits_mut::<Msb0>();
            bits[0..8].store_be(
                if ignition { (1<<0) } else { 0 } |
                if m7 { (1<<1) } else { 0 } |
                if m8 { (1<<2) } else { 0 } |
                if m9 { (1<<3) } else { 0 } |
                if m10 { (1<<4) } else { 0 } |
                if m11 { (1<<5) } else { 0 } |
                if m12 { (1<<6) } else { 0 } |
                if m13 { (1<<7) } else { 0 }
            );
            bits[8..16].store_be(0);
            // 12 bits for each analog value (big endian)
            bits[16..28].store_be(m1);
            bits[28..40].store_be(m2);
            bits[40..52].store_be(m3);
            bits[52..64].store_be(m4);

            self.send_normal_frame(hw, 0x204, &data);

            let mut data = [0u8; 8];
            let mut bits = data.view_bits_mut::<Msb0>();
            bits[0..12].store_be(m5);
            bits[12..24].store_be(m6);

            self.send_normal_frame(hw, 0x205, &data);
        }

        {
            // Publish current measurements for external monitoring

            let current1 = (hw.get_analog_input(AnalogInput::Current1) * 256.0) as u16;
            let current2 = (hw.get_analog_input(AnalogInput::Current2) * 256.0) as u16;
            let current3 = (hw.get_analog_input(AnalogInput::Current3) * 256.0) as u16;
            let current4 = (hw.get_analog_input(AnalogInput::Current4) * 256.0) as u16;
            let currentL = (hw.get_analog_input(AnalogInput::CurrentL) * 256.0) as u16;

            let mut data = [0u8; 8];
            let mut bits = data.view_bits_mut::<Msb0>();
            // 12 bits for each value (big endian)
            bits[0..12].store_be(current1);
            bits[12..24].store_be(current2);
            bits[24..36].store_be(current3);
            bits[36..48].store_be(current4);
            bits[48..60].store_be(currentL);

            self.send_normal_frame(hw, 0x206, &data);
        }
    }

    fn send_can_200ms(&mut self, hw: &mut dyn HardwareInterface) {
        let ignition_input = hw.get_digital_input(DigitalInput::Ignition);

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
                        get_parameter(ParameterId::ActivateEvse).value > 0.5 {
                    let ac_request_DCA = ac_v / dc_v * user_current_request_ACA;
                    let obc_limit_DCA = 12.0;
                    // TODO: If the heater is operating, allow that much extra
                    //       charging current so that it's possible to heat the
                    //       battery using AC power
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
            // This is an old PDM message, which we have inherited
            // We use this to:
            // * Request main contactor from the BMS for charging
            //   and heating
            // * Request the inverter to be disabled while charging
            // * Provide a DC bus voltage reading to Foccci
            // * Provide an OBC DC current reading to old SIM900 unit
            // * Send AcObcState and enable parameters to Foccci so that it can
            //   enable EVSE state C for AC charging

            let request_main_contactor: bool = self.request_wakeup_and_main_contactor;

            let request_inverter_disable: bool =
                    get_parameter(ParameterId::FoccciPlugPresent).value >= 0.5;

            let dc_link_voltage_Vx10: u16 =
                    (get_parameter(ParameterId::ObcDcv).value * 10.0) as u16;

            let obc_Ax10: u16 =
                    (get_parameter(ParameterId::ObcDcc).value * 10.0) as u16;

            let ac_obc_state = if get_parameter(ParameterId::ActivateObc).value > 0.5 {
                2
            } else {
                0
            };

            let group1oc = hw.get_digital_input(DigitalInput::Group1OC);
            let group2oc = hw.get_digital_input(DigitalInput::Group2OC);
            let group3oc = hw.get_digital_input(DigitalInput::Group3OC);
            let group4oc = hw.get_digital_input(DigitalInput::Group4OC);

            self.send_normal_frame(hw, 0x200, &[
                0x00 |
                    if request_main_contactor { (1<<0) } else { 0 } |
                    if request_inverter_disable { (1<<3) } else { 0 } |
                    if ignition_input { (1<<6) } else { 0 } |
                    (1<<7) /* Foccci.enable (new) */,
                (dc_link_voltage_Vx10 >> 8) as u8,
                (dc_link_voltage_Vx10 & 0xff) as u8,
                (obc_Ax10 >> 8) as u8,
                (obc_Ax10 & 0xff) as u8,
                get_parameter(ParameterId::PcbT).value as u8,
                ac_obc_state /* Foccci.AcObcState (new) */,
                0x00 |
                    if group1oc { (1<<0) } else { 0 } |
                    if group2oc { (1<<1) } else { 0 } |
                    if group3oc { (1<<2) } else { 0 } |
                    if group4oc { (1<<3) } else { 0 },
            ]);

            if request_inverter_disable {
                // For some reason inverter_controller isn't following the value
                // in 0x200, so we send this also which it does follow
                self.send_setting_frame(hw, 0x320, 1, 0, 1);
            }
        }
    }

    fn send_can_30ms(&mut self, hw: &mut dyn HardwareInterface) {
        if get_parameter(ParameterId::MainContactor).value > 0.5 {
            // Outlander HV status message (for heater and OBC)
            // 10...30ms is fine for this (EV-Omega uses 30ms)
            let activate_evse = get_parameter(ParameterId::ActivateObc).value > 0.5;
            self.send_normal_frame(hw, 0x285, &[
                0x00, 0x00,
                0x14 | if activate_evse { 0xb6 } else { 0 }, // 0xb6 = Activate EVSE (OBC)
                0x21, 0x90, 0xfe, 0x0c, 0x10
            ]);
        }
    }

    fn send_normal_frame(&mut self, hw: &mut dyn HardwareInterface,
            frame_id: u16, data: &[u8]) {
        if let Some(frame_data) = bxcan::Data::new(data) {
            hw.send_can(bxcan::Frame::new_data(
                bxcan::StandardId::new(frame_id).unwrap(),
                frame_data
            ));
        } else {
            warn!("-!- send_normal_frame(): Invalid data for frame {:?}: {:?}",
                    frame_id, data);
        }
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

        update_parameters_on_can(frame, self.last_millis);
    }
}
