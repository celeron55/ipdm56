use bxcan::{Id, StandardId};
use common::*;

define_parameters! {
    TicksMs {
        display_name: "Ticks",
        unit: "ms",
        log_threshold: f32::NAN,
    },
    AuxVoltage {
        display_name: "Aux battery",
        decimals: 2,
        unit: "V",
        log_threshold: 0.1,
    },
    BatteryTMin {
        display_name: "Bat T min",
        unit: "degC",
        can_map: CanMap {
            id: bxcan::Id::Standard(StandardId::new(0x101).unwrap()),
            bits: CanBitSelection::Int8(3),
            scale: 1.0,
        },
    },
    BatteryTMax {
        display_name: "Bat T max",
        unit: "degC",
        can_map: CanMap {
            id: bxcan::Id::Standard(StandardId::new(0x101).unwrap()),
            bits: CanBitSelection::Int8(4),
            scale: 1.0,
        },
    },
    BatteryVMin {
        display_name: "Bat V min",
        decimals: 2,
        unit: "V",
        can_map: CanMap {
            id: bxcan::Id::Standard(StandardId::new(0x101).unwrap()),
            bits: CanBitSelection::Function(|data: &[u8]| -> Option<f32> {
                Some((((data[0] as u16) << 4) | ((data[1] as u16) >> 4)) as f32)
            }),
            scale: 0.01,
        },
        log_threshold: 0.1,
    },
    BatteryVMax {
        display_name: "Bat V max",
        decimals: 2,
        unit: "V",
        can_map: CanMap {
            id: bxcan::Id::Standard(StandardId::new(0x101).unwrap()),
            bits: CanBitSelection::Function(|data: &[u8]| -> Option<f32> {
                Some(((((data[1] & 0x0f) as u16) << 8) | data[2] as u16) as f32)
            }),
            scale: 0.01,
        },
        log_threshold: 0.1,
    },
    Soc {
        display_name: "SoC",
        unit: "%",
        can_map: CanMap {
            id: bxcan::Id::Standard(StandardId::new(0x102).unwrap()),
            bits: CanBitSelection::Uint8(6),
            scale: 100.0 / 255.0,
        },
    },
    HeaterT {
        display_name: "Heater T",
        unit: "degC",
        can_map: CanMap {
            id: bxcan::Id::Standard(StandardId::new(0x398).unwrap()),
            bits: CanBitSelection::Function(|data: &[u8]| -> Option<f32> {
                let t1 = data[3] as i8 - 40;
                let t2 = data[4] as i8 - 40;
                Some((if t1 > t2 { t1 } else { t2 }) as f32)
            }),
            scale: 1.0,
        },
    },
    HeaterHeating {
        display_name: "Heater heating",
        unit: "",
        can_map: CanMap {
            id: bxcan::Id::Standard(StandardId::new(0x398).unwrap()),
            bits: CanBitSelection::Function(|data: &[u8]| -> Option<f32> {
                if data[5] > 0 {
                    Some(1.0)
                } else {
                    Some(0.0)
                }
            }),
            scale: 1.0,
        },
    },
    HeaterPowerPercent {
        display_name: "Heater power",
        unit: "%",
        can_map: CanMap {
            id: bxcan::Id::Standard(StandardId::new(0x398).unwrap()),
            bits: CanBitSelection::Function(|data: &[u8]| -> Option<f32> {
                // TODO: This accurate. The heater can be requested different
                //       power levels in 0x188
                if data[5] > 0 {
                    Some(100.0)
                } else {
                    Some(0.0)
                }
            }),
            scale: 1.0,
        },
    },
    CabinT {
        display_name: "Cabin T",
        unit: "degC",
        can_map: CanMap {
            id: bxcan::Id::Standard(StandardId::new(0x404).unwrap()),
            bits: CanBitSelection::Int8(1),
            scale: 1.0,
        },
    },
    PcbT {
        display_name: "PcbT",
        decimals: 1,
        unit: "degC",
    },
    ReqWakeupAndContactor {
        display_name: "ReqWakeupAndContactor",
        unit: "",
    },
    ReqHeaterPowerPercent {
        display_name: "ReqHeaterPowerPercent",
        unit: "%",
    },
    MainContactor {
        display_name: "Main contactor",
        unit: "",
        can_map: CanMap {
            id: bxcan::Id::Standard(StandardId::new(0x100).unwrap()),
            bits: CanBitSelection::Bit(2),
            scale: 1.0,
        },
    },
    PrechargeFailed {
        display_name: "Precharge failed",
        unit: "",
        can_map: CanMap {
            id: bxcan::Id::Standard(StandardId::new(0x100).unwrap()),
            bits: CanBitSelection::Bit(6),
            scale: 1.0,
        },
    },
    Balancing {
        display_name: "Balancing",
        unit: "",
        can_map: CanMap {
            id: bxcan::Id::Standard(StandardId::new(0x101).unwrap()),
            bits: CanBitSelection::Bit(5 * 8 + 0),
            scale: 1.0,
        },
    },
    ObcDcv {
        display_name: "OBC DC V",
        unit: "V",
        can_map: CanMap {
            id: bxcan::Id::Standard(StandardId::new(0x389).unwrap()),
            bits: CanBitSelection::Uint8(0),
            scale: 2.0,
        },
    },
    ObcDcc {
        display_name: "OBC DC A",
        decimals: 1,
        unit: "A",
        can_map: CanMap {
            id: bxcan::Id::Standard(StandardId::new(0x389).unwrap()),
            bits: CanBitSelection::Uint8(2),
            scale: 0.1,
        },
    },
    AcVoltage {
        display_name: "OBC AC V",
        unit: "V",
        can_map: CanMap {
            id: bxcan::Id::Standard(StandardId::new(0x389).unwrap()),
            bits: CanBitSelection::Uint8(1),
            scale: 1.0,
        },
    },
    PdmState {
        display_name: "PdmState",
        unit: "",
        can_map: CanMap {
            id: bxcan::Id::Standard(StandardId::new(0x203).unwrap()),
            bits: CanBitSelection::Function(|data: &[u8]| -> Option<f32> {
                Some((data[0] >> 4) as f32)
            }),
            scale: 1.0,
        },
    },
    OutlanderHeaterT {
        display_name: "OutlH T",
        unit: "degC",
        can_map: CanMap {
            id: bxcan::Id::Standard(StandardId::new(0x398).unwrap()),
            bits: CanBitSelection::Function(|data: &[u8]| -> Option<f32> {
                let t1 = data[3] as i8 - 40;
                let t2 = data[4] as i8 - 40;
                Some((if t1 > t2 { t1 } else { t2 }) as f32)
            }),
            scale: 1.0,
        },
    },
    OutlanderHeaterHeating {
        display_name: "OutlH heating",
        unit: "",
        can_map: CanMap {
            id: bxcan::Id::Standard(StandardId::new(0x398).unwrap()),
            bits: CanBitSelection::Function(|data: &[u8]| -> Option<f32> {
                if data[5] > 0 {
                    Some(1.0)
                } else {
                    Some(0.0)
                }
            }),
            scale: 1.0,
        },
    },
    OutlanderHeaterPowerPercent {
        display_name: "OutlH power",
        unit: "%",
        can_map: CanMap {
            id: bxcan::Id::Standard(StandardId::new(0x398).unwrap()),
            bits: CanBitSelection::Function(|data: &[u8]| -> Option<f32> {
                // TODO: This accurate. The heater can be requested different
                //       power levels in 0x188
                if data[5] > 0 {
                    Some(100.0)
                } else {
                    Some(0.0)
                }
            }),
            scale: 1.0,
        },
    },
    CruiseActive {
        display_name: "Cruise active",
        unit: "",
        can_map: CanMap {
            id: bxcan::Id::Standard(StandardId::new(0x300).unwrap()),
            bits: CanBitSelection::Bit(2),
            scale: 1.0,
        },
    },
    CruiseRequested {
        display_name: "Cruise requested",
        unit: "",
    },
    HvacRequested {
        display_name: "HVAC requested",
        unit: "",
        can_map: CanMap {
            id: bxcan::Id::Standard(StandardId::new(0x570).unwrap()),
            bits: CanBitSelection::Function(|data: &[u8]| -> Option<f32> {
                if data[0] == 2 {
                    if data[4] == 1 {
                        Some(1.0)
                    } else {
                        Some(0.0)
                    }
                } else {
                    None
                }
            }),
            scale: 1.0,
        },
    },
    FoccciCPPWM {
        display_name: "Foccci CP PWM",
        unit: "%",
        can_map: CanMap {
            id: bxcan::Id::Standard(StandardId::new(0x506).unwrap()),
            bits: CanBitSelection::Uint8(1),
            scale: 1.0,
        },
    },
    ActivateEvse {
        display_name: "Activate EVSE",
        unit: "",
    },
    ActivateObc {
        display_name: "Activate OBC",
        unit: "",
    },
    BmsMaxChargeCurrent {
        display_name: "Max charge",
        decimals: 1,
        unit: "A",
        can_map: CanMap {
            id: bxcan::Id::Standard(StandardId::new(0x102).unwrap()),
            bits: CanBitSelection::Function(|data: &[u8]| -> Option<f32> {
                Some((((data[2] as u16) << 8) | data[3] as u16) as f32)
            }),
            scale: 0.1,
        },
    },
    BmsMaxDischargeCurrent {
        display_name: "Max discharge",
        decimals: 1,
        unit: "A",
        can_map: CanMap {
            id: bxcan::Id::Standard(StandardId::new(0x102).unwrap()),
            bits: CanBitSelection::Function(|data: &[u8]| -> Option<f32> {
                Some((((data[4] as u16) << 8) | data[5] as u16) as f32)
            }),
            scale: 0.1,
        },
    },
    CcsCurrent {
        display_name: "CCS",
        unit: "A",
        can_map: CanMap {
            id: bxcan::Id::Standard(StandardId::new(0x506).unwrap()),
            bits: CanBitSelection::Uint8(5),
            scale: 2.0,
        },
    },
    ChargeComplete {
        // This is internally generated, not the one provided by the BMS
        display_name: "ChargeComplete",
        unit: "",
    },
    LastSeenSoc {
        display_name: "SoC (last seen)",
        unit: "%",
    },
    Precharging {
        display_name: "Precharging",
        unit: "",
        can_map: CanMap {
            id: bxcan::Id::Standard(StandardId::new(0x100).unwrap()),
            bits: CanBitSelection::Bit(5),
            scale: 1.0,
        },
    },
    DcdcStatus {
        display_name: "DCDC status",
        unit: "",
        can_map: CanMap {
            id: bxcan::Id::Standard(StandardId::new(0x377).unwrap()),
            bits: CanBitSelection::Uint8(7),
            scale: 1.0,
        },
    },
    BmsChargeCompleteVoltageSetting {
        display_name: "BmsChgCompV",
        unit: "mV",
        can_map: CanMap {
            id: bxcan::Id::Standard(StandardId::new(0x104).unwrap()),
            bits: CanBitSelection::Function(|data: &[u8]| -> Option<f32> {
                Some((((data[0] as u16) << 8) | data[1] as u16) as f32)
            }),
            scale: 1.0,
        },
    },
    FoccciPlugPresent {
        display_name: "FoccciPlugPresent",
        unit: "",
        can_map: CanMap {
            id: bxcan::Id::Standard(StandardId::new(0x506).unwrap()),
            bits: CanBitSelection::Bit(2),
            scale: 1.0,
        },
    },
    EvaporatorT {
        display_name: "A/C evaporator T",
        decimals: 0,
        unit: "degC",
        log_threshold: 5.0,
    },
    AcCompressorPercent {
        display_name: "A/C compressor power %",
        decimals: 0,
        unit: "%",
        log_threshold: 5.0,
    },
}
