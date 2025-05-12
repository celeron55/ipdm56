#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

// Local modules

// Internal crates
use command_accumulator::CommandAccumulator;
use common::*;

// Platform-specific dependencies
use adc::{config::AdcConfig, Adc};
use critical_section::Mutex;
use hal::{
    adc::{self, config::SampleTime},
    gpio,
    gpio::PinExt,
    otg_fs, pac,
    prelude::*,
    serial,
};
use rtic_monotonics::{systick::*, Monotonic};
use stm32f4xx_hal as hal;
use usb_device::prelude::*;
use usbd_serial::{self, USB_CLASS_CDC};
// use eeprom24x::Eeprom24x;
use embedded_hal_bus;

// Standard library utilities
use core::{cell::RefCell, fmt::Write, ops::DerefMut};

// General purpose libraries
use arrayvec::{ArrayString, ArrayVec};
use fixedstr::str_format;
use log::{debug, error, info, trace, warn, Log, Metadata, Record};
use micromath::F32Ext;
use ringbuffer::{ConstGenericRingBuffer, RingBuffer};

// Constants

const LOG_BUFFER_SIZE: usize = 1024;
const CONSOLE_RX_BUF_SIZE: usize = 100;
const MAINBOARD_RX_BUF_SIZE: usize = 200;
const MAINBOARD_TX_BUF_SIZE: usize = 200;
const CAN_ENABLE_LOOPBACK_MODE: bool = false;

// Log buffering system

struct MultiLogger {
    uart_buffer: Mutex<RefCell<Option<ArrayString<LOG_BUFFER_SIZE>>>>,
    usb_buffer: Mutex<RefCell<Option<ArrayString<LOG_BUFFER_SIZE>>>>,
    display_buffer: Mutex<RefCell<Option<ArrayString<LOG_BUFFER_SIZE>>>>,
}

impl MultiLogger {
    fn get_uart_buffer(&self) -> Option<ArrayString<LOG_BUFFER_SIZE>> {
        let mut buf2: Option<ArrayString<LOG_BUFFER_SIZE>> = Some(ArrayString::new());
        critical_section::with(|cs| {
            // This replaces the logger buffer with an empty one, and we get the
            // possibly filled in one
            buf2 = self.uart_buffer.borrow(cs).replace(buf2);
        });
        buf2
    }
    fn get_usb_buffer(&self) -> Option<ArrayString<LOG_BUFFER_SIZE>> {
        let mut buf2: Option<ArrayString<LOG_BUFFER_SIZE>> = Some(ArrayString::new());
        critical_section::with(|cs| {
            // This replaces the logger buffer with an empty one, and we get the
            // possibly filled in one
            buf2 = self.usb_buffer.borrow(cs).replace(buf2);
        });
        buf2
    }
    fn get_display_buffer(&self) -> Option<ArrayString<LOG_BUFFER_SIZE>> {
        let mut buf2: Option<ArrayString<LOG_BUFFER_SIZE>> = Some(ArrayString::new());
        critical_section::with(|cs| {
            // This replaces the logger buffer with an empty one, and we get the
            // possibly filled in one
            buf2 = self.display_buffer.borrow(cs).replace(buf2);
        });
        buf2
    }
}

impl Log for MultiLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= log::Level::Info // TODO: Adjust as needed
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            critical_section::with(|cs| {
                if let Some(ref mut buffer) = self.uart_buffer.borrow(cs).borrow_mut().deref_mut() {
                    let _ = buffer.write_fmt(format_args!(
                        "[{}] {}\r\n",
                        record.level(),
                        record.args()
                    ));
                    if buffer.is_full() {
                        let warning = " | LOG BUFFER FULL\r\n";
                        buffer.truncate(buffer.capacity() - warning.len());
                        let _ = buffer.try_push_str(warning);
                    }
                }
                if let Some(ref mut buffer) = self.usb_buffer.borrow(cs).borrow_mut().deref_mut() {
                    let _ = buffer.write_fmt(format_args!(
                        "[{}] {}\r\n",
                        record.level(),
                        record.args()
                    ));
                    if buffer.is_full() {
                        let warning = " | LOG BUFFER FULL\r\n";
                        buffer.truncate(buffer.capacity() - warning.len());
                        let _ = buffer.try_push_str(warning);
                    }
                }
                if let Some(ref mut buffer) =
                    self.display_buffer.borrow(cs).borrow_mut().deref_mut()
                {
                    let _ = buffer.write_fmt(format_args!("{}\r\n", record.args()));
                    if buffer.is_full() {
                        let warning = " | LOG BUFFER FULL\r\n";
                        buffer.truncate(buffer.capacity() - warning.len());
                        let _ = buffer.try_push_str(warning);
                    }
                }
            });
            // Trigger write to hardware by triggering USART1 interrupt
            pac::NVIC::pend(pac::Interrupt::USART1);
            // Trigger write to hardware by triggering OTG_FS interrupt
            pac::NVIC::pend(pac::Interrupt::OTG_FS);
        }
    }

    fn flush(&self) {
        // Flushing is handled elsewhere
    }
}

static MULTI_LOGGER: MultiLogger = MultiLogger {
    uart_buffer: Mutex::new(RefCell::new(None)),
    usb_buffer: Mutex::new(RefCell::new(None)),
    display_buffer: Mutex::new(RefCell::new(None)),
};

// Function to initialize the logger
fn init_logger() {
    critical_section::with(|cs| {
        MULTI_LOGGER
            .uart_buffer
            .borrow(cs)
            .replace(Some(ArrayString::new()));
        MULTI_LOGGER
            .usb_buffer
            .borrow(cs)
            .replace(Some(ArrayString::new()));
        MULTI_LOGGER
            .display_buffer
            .borrow(cs)
            .replace(Some(ArrayString::new()));
    });
    log::set_logger(&MULTI_LOGGER).unwrap();
    log::set_max_level(log::LevelFilter::Info); // TODO: Adjust as needed
}

// CAN driver

pub struct CAN1 {
    _private: (),
}
unsafe impl bxcan::Instance for CAN1 {
    const REGISTERS: *mut bxcan::RegisterBlock = 0x4000_6400 as *mut _;
}
unsafe impl bxcan::FilterOwner for CAN1 {
    const NUM_FILTER_BANKS: u8 = 28;
}

// TIM3 PWM

type Tim3Pwm = hal::timer::PwmHz<
    hal::pac::TIM3,
    (
        hal::timer::ChannelBuilder<hal::pac::TIM3, 0, false>, // LPWM2
        hal::timer::ChannelBuilder<hal::pac::TIM3, 1, false>, // LPWM3
    ),
>;

fn set_lpwm2(pwm: f32, pwm_timer: &mut Tim3Pwm) {
    pwm_timer.set_duty(
        hal::timer::Channel::C1,
        (pwm_timer.get_max_duty() as f32 * pwm) as u16,
    );
}

fn set_lpwm3(pwm: f32, pwm_timer: &mut Tim3Pwm) {
    pwm_timer.set_duty(
        hal::timer::Channel::C2,
        (pwm_timer.get_max_duty() as f32 * pwm) as u16,
    );
}

// TIM4 PWM

type Tim4Pwm = hal::timer::PwmHz<
    hal::pac::TIM4,
    (
        hal::timer::ChannelBuilder<hal::pac::TIM4, 0, false>, // LCUR1
        hal::timer::ChannelBuilder<hal::pac::TIM4, 1, false>, // SPWM1
        hal::timer::ChannelBuilder<hal::pac::TIM4, 2, false>, // SPWM2
        //hal::timer::ChannelBuilder<hal::pac::TIM4, 3, false>, // Will be LWPM1 in next version
    ),
>;

fn set_lcur1(pwm: f32, pwm_timer: &mut Tim4Pwm) {
    pwm_timer.set_duty(
        hal::timer::Channel::C1,
        (pwm_timer.get_max_duty() as f32 * pwm) as u16,
    );
}

fn set_spwm1(pwm: f32, pwm_timer: &mut Tim4Pwm) {
    pwm_timer.set_duty(
        hal::timer::Channel::C2,
        (pwm_timer.get_max_duty() as f32 * pwm) as u16,
    );
}

fn set_spwm2(pwm: f32, pwm_timer: &mut Tim4Pwm) {
    pwm_timer.set_duty(
        hal::timer::Channel::C3,
        (pwm_timer.get_max_duty() as f32 * pwm) as u16,
    );
}

type WkupPin = gpio::Pin<'A', 0, gpio::Input>;

// HardwareInterface implementation

type Group1OCPin = gpio::Pin<'D', 8, gpio::Input>;
type Group2OCPin = gpio::Pin<'D', 9, gpio::Input>;
type Group3OCPin = gpio::Pin<'C', 8, gpio::Input>;
type Group4OCPin = gpio::Pin<'C', 9, gpio::Input>;

type IgnInputPin = gpio::Pin<'D', 15, gpio::Input>;

type Boot0ControlPin = gpio::Pin<'B', 8, gpio::Output<gpio::PushPull>>;
type WakeupOutputPin = gpio::Pin<'A', 15, gpio::Output<gpio::PushPull>>;
type HOUT1Pin = gpio::Pin<'E', 0, gpio::Output<gpio::PushPull>>;
type HOUT2Pin = gpio::Pin<'E', 1, gpio::Output<gpio::PushPull>>;
type HOUT3Pin = gpio::Pin<'E', 2, gpio::Output<gpio::PushPull>>;
type HOUT4Pin = gpio::Pin<'E', 3, gpio::Output<gpio::PushPull>>;
type HOUT5Pin = gpio::Pin<'E', 4, gpio::Output<gpio::PushPull>>;
type HOUT6Pin = gpio::Pin<'E', 5, gpio::Output<gpio::PushPull>>;
type HOUT7Pin = gpio::Pin<'E', 6, gpio::Output<gpio::PushPull>>;
type HOUT8Pin = gpio::Pin<'D', 2, gpio::Output<gpio::PushPull>>;
type HOUT9Pin = gpio::Pin<'D', 3, gpio::Output<gpio::PushPull>>;
type HOUT10Pin = gpio::Pin<'D', 4, gpio::Output<gpio::PushPull>>;
type HOUT11Pin = gpio::Pin<'D', 7, gpio::Output<gpio::PushPull>>;
type HOUT12Pin = gpio::Pin<'C', 12, gpio::Output<gpio::PushPull>>;
type LOUT1Pin = gpio::Pin<'E', 10, gpio::Output<gpio::PushPull>>;
type LOUT2Pin = gpio::Pin<'E', 11, gpio::Output<gpio::PushPull>>;
type LOUT3Pin = gpio::Pin<'E', 12, gpio::Output<gpio::PushPull>>;
type LOUT4Pin = gpio::Pin<'E', 13, gpio::Output<gpio::PushPull>>;
type LOUT5Pin = gpio::Pin<'E', 14, gpio::Output<gpio::PushPull>>;
type LOUT6Pin = gpio::Pin<'E', 15, gpio::Output<gpio::PushPull>>;

struct HardwareImplementation {
    boot0_control_pin: &'static mut Boot0ControlPin,
    wakeup_output_pin: WakeupOutputPin,
    can_tx_buf: ConstGenericRingBuffer<bxcan::Frame, 10>,
    adc_result_vbat: f32,
    adc_result_tpcb: f32,
    tim3_pwm: Tim3Pwm,
    tim4_pwm: Tim4Pwm,
    group1oc_pin: Group1OCPin,
    group2oc_pin: Group2OCPin,
    group3oc_pin: Group3OCPin,
    group4oc_pin: Group4OCPin,
    ign_input_pin: IgnInputPin,
    hout1_pin: HOUT1Pin,
    hout2_pin: HOUT2Pin,
    hout3_pin: HOUT3Pin,
    hout4_pin: HOUT4Pin,
    hout5_pin: HOUT5Pin,
    hout6_pin: HOUT6Pin,
    hout7_pin: HOUT7Pin,
    hout8_pin: HOUT8Pin,
    hout9_pin: HOUT9Pin,
    hout10_pin: HOUT10Pin,
    hout11_pin: HOUT11Pin,
    hout12_pin: HOUT12Pin,
    lout1_pin: LOUT1Pin,
    lout2_pin: LOUT2Pin,
    lout3_pin: LOUT3Pin,
    lout4_pin: LOUT4Pin,
    lout5_pin: LOUT5Pin,
    lout6_pin: LOUT6Pin,
}

impl HardwareInterface for HardwareImplementation {
    fn millis(&mut self) -> u64 {
        // NOTE: This rolls over at 49.71 days
        Systick::now().duration_since_epoch().to_millis() as u64
    }

    fn reboot(&mut self) {
        cortex_m::peripheral::SCB::sys_reset();
    }

    fn activate_dfu(&mut self) {
        self.boot0_control_pin.set_high();
        long_busywait();
        cortex_m::peripheral::SCB::sys_reset();
    }

    fn send_can(&mut self, frame: bxcan::Frame) {
        //info!("send_can(): {:?}", frame);
        self.can_tx_buf.push(frame);
    }

    fn get_analog_input(&mut self, input: AnalogInput) -> f32 {
        match input {
            AnalogInput::AuxVoltage => self.adc_result_vbat,
            AnalogInput::PcbT => self.adc_result_tpcb,
            _ => f32::NAN,
        }
    }

    fn get_digital_input(&mut self, input: DigitalInput) -> bool {
        match input {
            DigitalInput::Group1OC => self.group1oc_pin.is_low(),
            DigitalInput::Group2OC => self.group2oc_pin.is_low(),
            DigitalInput::Group3OC => self.group3oc_pin.is_low(),
            DigitalInput::Group4OC => self.group4oc_pin.is_low(),
            DigitalInput::Ignition => self.ign_input_pin.is_low(),
        }
    }

    fn set_digital_output(&mut self, output: DigitalOutput, value: bool) {
        let old_value = match output {
            DigitalOutput::Wakeup => { self.wakeup_output_pin.is_set_high() }
            DigitalOutput::HOUT1 => { self.hout1_pin.is_set_high() }
            DigitalOutput::HOUT2 => { self.hout2_pin.is_set_high() }
            DigitalOutput::HOUT3 => { self.hout3_pin.is_set_high() }
            DigitalOutput::HOUT4 => { self.hout4_pin.is_set_high() }
            DigitalOutput::HOUT5 => { self.hout5_pin.is_set_high() }
            DigitalOutput::HOUT6 => { self.hout6_pin.is_set_high() }
            DigitalOutput::HOUT7 => { self.hout7_pin.is_set_high() }
            DigitalOutput::HOUT8 => { self.hout8_pin.is_set_high() }
            DigitalOutput::HOUT9 => { self.hout9_pin.is_set_high() }
            DigitalOutput::HOUT10 => { self.hout10_pin.is_set_high() }
            DigitalOutput::HOUT11 => { self.hout11_pin.is_set_high() }
            DigitalOutput::HOUT12 => { self.hout12_pin.is_set_high() }
            DigitalOutput::LOUT1 => { self.lout1_pin.is_set_high() }
            DigitalOutput::LOUT2 => { self.lout2_pin.is_set_high() }
            DigitalOutput::LOUT3 => { self.lout3_pin.is_set_high() }
            DigitalOutput::LOUT4 => { self.lout4_pin.is_set_high() }
            DigitalOutput::LOUT5 => { self.lout5_pin.is_set_high() }
            DigitalOutput::LOUT6 => { self.lout6_pin.is_set_high() }
            // TODO: M* pins
        };

        if value != old_value {
            info!("HardwareImplementation::set_digital_output: {:?} {:?} -> {:?}",
                    output, old_value, value);
        }

        match output {
            DigitalOutput::Wakeup => { self.wakeup_output_pin.set_state(value.into()) }
            DigitalOutput::HOUT1 => { self.hout1_pin.set_state(value.into()) }
            DigitalOutput::HOUT2 => { self.hout2_pin.set_state(value.into()) }
            DigitalOutput::HOUT3 => { self.hout3_pin.set_state(value.into()) }
            DigitalOutput::HOUT4 => { self.hout4_pin.set_state(value.into()) }
            DigitalOutput::HOUT5 => { self.hout5_pin.set_state(value.into()) }
            DigitalOutput::HOUT6 => { self.hout6_pin.set_state(value.into()) }
            DigitalOutput::HOUT7 => { self.hout7_pin.set_state(value.into()) }
            DigitalOutput::HOUT8 => { self.hout8_pin.set_state(value.into()) }
            DigitalOutput::HOUT9 => { self.hout9_pin.set_state(value.into()) }
            DigitalOutput::HOUT10 => { self.hout10_pin.set_state(value.into()) }
            DigitalOutput::HOUT11 => { self.hout11_pin.set_state(value.into()) }
            DigitalOutput::HOUT12 => { self.hout12_pin.set_state(value.into()) }
            DigitalOutput::LOUT1 => { self.lout1_pin.set_state(value.into()) }
            DigitalOutput::LOUT2 => { self.lout2_pin.set_state(value.into()) }
            DigitalOutput::LOUT3 => { self.lout3_pin.set_state(value.into()) }
            DigitalOutput::LOUT4 => { self.lout4_pin.set_state(value.into()) }
            DigitalOutput::LOUT5 => { self.lout5_pin.set_state(value.into()) }
            DigitalOutput::LOUT6 => { self.lout6_pin.set_state(value.into()) }
            // TODO: M* pins
        }
    }

    fn set_pwm_output(&mut self, output: PwmOutput, value: f32) {
        match output {
            PwmOutput::LCUR1 => { set_lcur1(value, &mut self.tim4_pwm) }
            PwmOutput::SPWM1 => { set_spwm1(value, &mut self.tim4_pwm) }
            PwmOutput::SPWM2 => { set_spwm2(value, &mut self.tim4_pwm) }
            PwmOutput::LPWM2 => { set_lpwm2(value, &mut self.tim3_pwm) }
            PwmOutput::LPWM3 => { set_lpwm3(value, &mut self.tim3_pwm) }
        }
    }
}

// Panic output and input methods

static mut PANIC_TX: Option<hal::serial::Tx<hal::pac::USART1, u8>> = None;
static mut PANIC_BOOT0_CONTROL_PIN: Option<Boot0ControlPin> = None;

// RTIC application

#[rtic::app(device = hal::pac, peripherals = true, dispatchers = [UART5, I2C3_ER, USART6])]
mod rtic_app {
    use super::*;

    #[shared]
    struct Shared {
        usb_dev: UsbDevice<'static, otg_fs::UsbBusType>,
        usb_serial: usbd_serial::SerialPort<'static, otg_fs::UsbBusType>,
        console_rxbuf: ConstGenericRingBuffer<u8, CONSOLE_RX_BUF_SIZE>,
        mainboard_rxbuf: ConstGenericRingBuffer<u8, MAINBOARD_RX_BUF_SIZE>,
        mainboard_txbuf: ConstGenericRingBuffer<u8, MAINBOARD_TX_BUF_SIZE>,
        can1: bxcan::Can<CAN1>,
        can_rx_buf: ConstGenericRingBuffer<bxcan::Frame, 10>,
        can_tx_buf: ConstGenericRingBuffer<bxcan::Frame, 10>,
        adc_result_vbat: f32,
        adc_result_tpcb: f32,
        adc_result_current_vbat: u16,
    }

    #[local]
    struct Local {
        usart1_rx: hal::serial::Rx<hal::pac::USART1, u8>,
        usart1_tx: &'static mut hal::serial::Tx<hal::pac::USART1, u8>,
        //usart2_rx: hal::serial::Rx<hal::pac::USART2, u8>,
        //usart2_tx: hal::serial::Tx<hal::pac::USART2, u8>,
        //usart3_rx: hal::serial::Rx<hal::pac::USART3, u8>,
        //usart3_tx: hal::serial::Tx<hal::pac::USART3, u8>,
        command_accumulator: CommandAccumulator<50>,
        i2c1: hal::i2c::I2c<hal::pac::I2C1>,
        adc1: Adc<pac::ADC1>,
        // Analog input pins
        adc_pa1: gpio::Pin<'A', 1, gpio::Analog>,
        adc_pa2: gpio::Pin<'A', 2, gpio::Analog>,
        adc_pa3: gpio::Pin<'A', 3, gpio::Analog>,
        adc_pb1: gpio::Pin<'B', 1, gpio::Analog>,
        // Digital input pins
        // Output pins
        // (See HardwareImplementation)
        // Other
        hw: HardwareImplementation,
    }

    #[init()]
    fn init(mut cx: init::Context) -> (Shared, Local) {
        static mut EP_MEMORY: [u32; 1024] = [0; 1024];
        static mut USB_BUS: Option<usb_device::bus::UsbBusAllocator<otg_fs::UsbBusType>> = None;
        static mut SPI3_SHARED: Option<Mutex<RefCell<hal::spi::Spi<hal::pac::SPI3>>>> = None;

        // System clock

        // Enable CAN1
        cx.device.RCC.apb1enr.modify(|_, w| w.can1en().enabled());

        let rcc = cx.device.RCC.constrain();
        let clocks = rcc
            .cfgr
            .use_hse(8.MHz()) // Use external crystal (HSE)
            .hclk(168.MHz())
            .pclk1(42.MHz())
            .pclk2(84.MHz())
            .sysclk(168.MHz()) // Set system clock (SYSCLK)
            .freeze(); // Apply the configuration

        let mut syscfg = cx.device.SYSCFG.constrain();

        // Pin assignments

        let gpioa = cx.device.GPIOA.split();
        let gpiob = cx.device.GPIOB.split();
        let gpioc = cx.device.GPIOC.split();
        let gpiod = cx.device.GPIOD.split();
        let gpioe = cx.device.GPIOE.split();

        // Input pins

        let mut group1oc_pin = gpiod.pd8.into_input();
        let mut group2oc_pin = gpiod.pd9.into_input();
        let mut group3oc_pin = gpioc.pc8.into_input();
        let mut group4oc_pin = gpioc.pc9.into_input();

        let mut ign_input_pin = gpiod.pd15.into_input();

        // Output pins

        let boot0_control_pin = gpiob.pb8.into_push_pull_output();
        let boot0_control_pin = unsafe {
            PANIC_BOOT0_CONTROL_PIN = Some(boot0_control_pin);
            PANIC_BOOT0_CONTROL_PIN.as_mut().unwrap()
        };

        let mut wakeup_output_pin = gpioa.pa15.into_push_pull_output();

        let mut hout1_pin = gpioe.pe0.into_push_pull_output();
        let mut hout2_pin = gpioe.pe1.into_push_pull_output();
        let mut hout3_pin = gpioe.pe2.into_push_pull_output();
        let mut hout4_pin = gpioe.pe3.into_push_pull_output();
        let mut hout5_pin = gpioe.pe4.into_push_pull_output();
        let mut hout6_pin = gpioe.pe5.into_push_pull_output();
        let mut hout7_pin = gpioe.pe6.into_push_pull_output();
        let mut hout8_pin = gpiod.pd2.into_push_pull_output();
        let mut hout9_pin = gpiod.pd3.into_push_pull_output();
        let mut hout10_pin = gpiod.pd4.into_push_pull_output();
        let mut hout11_pin = gpiod.pd7.into_push_pull_output();
        let mut hout12_pin = gpioc.pc12.into_push_pull_output();
        let mut lout1_pin = gpioe.pe10.into_push_pull_output();
        let mut lout2_pin = gpioe.pe11.into_push_pull_output();
        let mut lout3_pin = gpioe.pe12.into_push_pull_output();
        let mut lout4_pin = gpioe.pe13.into_push_pull_output();
        let mut lout5_pin = gpioe.pe14.into_push_pull_output();
        let mut lout6_pin = gpioe.pe15.into_push_pull_output();

        // SysTick

        let systick_token = rtic_monotonics::create_systick_token!();
        Systick::start(cx.core.SYST, 168_000_000, systick_token); // Eats SYST peripheral

        // Software utilities

        init_logger();

        info!("-!- ipdmrust boot");

        // TIM3 (PWM generation)

        let lpwm2_ch: hal::timer::ChannelBuilder<hal::pac::TIM3, 0, false> =
            hal::timer::Channel1::new(gpioc.pc6);
        let lpwm3_ch: hal::timer::ChannelBuilder<hal::pac::TIM3, 1, false> =
            hal::timer::Channel2::new(gpioc.pc7);

        let mut tim3_pwm = cx
            .device
            .TIM3
            .pwm_hz((lpwm2_ch, lpwm3_ch), 1000.Hz(), &clocks);

        tim3_pwm.enable(hal::timer::Channel::C1);
        tim3_pwm.enable(hal::timer::Channel::C2);
        set_lpwm2(0.0, &mut tim3_pwm);
        set_lpwm3(0.0, &mut tim3_pwm);

        // TIM4 (PWM generation)

        let lcur1_ch: hal::timer::ChannelBuilder<hal::pac::TIM4, 0, false> =
            hal::timer::Channel1::new(gpiod.pd12);
        let spwm1_ch: hal::timer::ChannelBuilder<hal::pac::TIM4, 1, false> =
            hal::timer::Channel2::new(gpiod.pd13);
        let spwm2_ch: hal::timer::ChannelBuilder<hal::pac::TIM4, 2, false> =
            hal::timer::Channel3::new(gpiod.pd14);

        let mut tim4_pwm = cx
            .device
            .TIM4
            .pwm_hz((lcur1_ch, spwm1_ch, spwm2_ch), 1000.Hz(), &clocks);

        tim4_pwm.enable(hal::timer::Channel::C1);
        tim4_pwm.enable(hal::timer::Channel::C2);
        tim4_pwm.enable(hal::timer::Channel::C3);
        set_lcur1(0.0, &mut tim4_pwm);
        set_spwm1(0.0, &mut tim4_pwm);
        set_spwm2(0.0, &mut tim4_pwm);

        // USART1 (TX=PA9, RX=PA10): TTL serial on programming header. We
        // provide our serial console here, and also on native USB. Note that
        // PA9 is also USB VBUS detect, because the bootloader wants that there.

        let serial_usart1: serial::Serial<pac::USART1, u8> = serial::Serial::new(
            cx.device.USART1,
            (
                gpioa.pa9.into_alternate::<7>(),
                gpioa.pa10.into_alternate::<7>(),
            ),
            serial::config::Config::default().baudrate(115200.bps()),
            &clocks,
        )
        .unwrap();
        let (usart1_tx, mut usart1_rx) = serial_usart1.split();
        usart1_rx.listen();
        let usart1_tx = unsafe {
            PANIC_TX = Some(usart1_tx);
            PANIC_TX.as_mut().unwrap()
        };

        // USB

        let usb = otg_fs::USB::new(
            (
                cx.device.OTG_FS_GLOBAL,
                cx.device.OTG_FS_DEVICE,
                cx.device.OTG_FS_PWRCLK,
            ),
            (
                gpioa.pa11.into_alternate::<10>(),
                gpioa.pa12.into_alternate::<10>(),
            ),
            &clocks,
        );

        unsafe {
            USB_BUS.replace(otg_fs::UsbBus::new(usb, &mut EP_MEMORY));
        }

        let usb_serial = usbd_serial::SerialPort::new(unsafe { USB_BUS.as_ref().unwrap() });

        let usb_dev = UsbDeviceBuilder::new(
            unsafe { USB_BUS.as_ref().unwrap() },
            //UsbVidPid(0x1209, 0x0001)) // https://pid.codes/1209/0001/
            UsbVidPid(0x0483, 0x5740),
        ) // STMicroelectronics / Virtual COM Port
        .device_class(USB_CLASS_CDC)
        .strings(&[
            StringDescriptors::new(usb_device::descriptor::lang_id::LangID::EN)
                .manufacturer("8Dromeda Productions")
                .product("ipdmrust")
                .serial_number("1337"),
        ])
        .unwrap()
        .build();

        // ADC

        let adc_pa1 = gpioa.pa1.into_analog(); // Version detection
        let adc_pa2 = gpioa.pa2.into_analog(); // Vbat
        let adc_pa3 = gpioa.pa3.into_analog(); // LDR
        let adc_pb1 = gpiob.pb1.into_analog(); // PcbT
                                               // TODO: More pins

        let adc_config = AdcConfig::default()
            .resolution(adc::config::Resolution::Twelve)
            .clock(adc::config::Clock::Pclk2_div_8);

        let adc1 = Adc::adc1(cx.device.ADC1, true, adc_config);

        // I2C
        // There's a 24C02 EEPROM chip on this bus

        let mut i2c1 = hal::i2c::I2c::new(
            cx.device.I2C1,
            (gpiob.pb6, gpiob.pb7),
            hal::i2c::Mode::Standard {
                frequency: 400.kHz(),
            },
            &clocks,
        );

        // CAN

        let _pins = (
            gpiod.pd1.into_alternate::<9>(), // CAN1 TX
            gpiod.pd0.into_alternate::<9>(), // CAN1 RX
        );

        let mut can1 = bxcan::Can::builder(CAN1 { _private: () })
            .set_loopback(CAN_ENABLE_LOOPBACK_MODE)
            .set_bit_timing(0x00090006) // 500kbps at 42MHz pclk1
            .enable();

        can1.modify_filters()
            .enable_bank(0, bxcan::Fifo::Fifo0, bxcan::filter::Mask32::accept_all())
            .enable_bank(1, bxcan::Fifo::Fifo1, bxcan::filter::Mask32::accept_all());

        can1.enable_interrupt(bxcan::Interrupt::Fifo0MessagePending);
        can1.enable_interrupt(bxcan::Interrupt::Fifo1MessagePending);
        can1.enable_interrupt(bxcan::Interrupt::TransmitMailboxEmpty);

        unsafe {
            pac::NVIC::unmask(pac::Interrupt::CAN1_RX0);
            pac::NVIC::unmask(pac::Interrupt::CAN1_RX1);
            pac::NVIC::unmask(pac::Interrupt::CAN1_TX);
            pac::NVIC::unmask(pac::Interrupt::CAN1_SCE);
        }

        // Hardware abstraction

        let hw = HardwareImplementation {
            boot0_control_pin,
            wakeup_output_pin,
            can_tx_buf: ConstGenericRingBuffer::new(),
            adc_result_vbat: f32::NAN,
            adc_result_tpcb: f32::NAN,
            tim3_pwm,
            tim4_pwm,
            group1oc_pin,
            group2oc_pin,
            group3oc_pin,
            group4oc_pin,
            ign_input_pin,
            hout1_pin,
            hout2_pin,
            hout3_pin,
            hout4_pin,
            hout5_pin,
            hout6_pin,
            hout7_pin,
            hout8_pin,
            hout9_pin,
            hout10_pin,
            hout11_pin,
            hout12_pin,
            lout1_pin,
            lout2_pin,
            lout3_pin,
            lout4_pin,
            lout5_pin,
            lout6_pin,
        };

        // Schedule tasks

        logic_task::spawn().ok();
        adc_task::spawn().ok();

        // Initialize context

        (
            Shared {
                console_rxbuf: ConstGenericRingBuffer::new(),
                mainboard_rxbuf: ConstGenericRingBuffer::new(),
                mainboard_txbuf: ConstGenericRingBuffer::new(),
                usb_dev: usb_dev,
                usb_serial: usb_serial,
                can1: can1,
                can_rx_buf: ConstGenericRingBuffer::new(),
                can_tx_buf: ConstGenericRingBuffer::new(),
                adc_result_vbat: 0.0,
                adc_result_tpcb: 0.0,
                adc_result_current_vbat: 0,
            },
            Local {
                usart1_rx: usart1_rx,
                usart1_tx: usart1_tx,
                //usart2_rx: usart2_rx,
                //usart2_tx: usart2_tx,
                //usart3_rx: usart3_rx,
                //usart3_tx: usart3_tx,
                command_accumulator: CommandAccumulator::new(),
                i2c1: i2c1,
                adc1: adc1,
                adc_pa1: adc_pa1,
                adc_pa2: adc_pa2,
                adc_pa3: adc_pa3,
                adc_pb1: adc_pb1,
                hw,
            },
        )
    }

    #[idle(
        shared = [
        ]
    )]
    fn idle(mut cx: idle::Context) -> ! {
        loop {
            short_busywait();
            //cx.shared.debug_pin.lock(|pin| { pin.toggle(); });
        }
    }

    #[task(priority = 1,
        shared = [
            console_rxbuf,
            mainboard_rxbuf,
            mainboard_txbuf,
            can1,
            can_rx_buf,
            can_tx_buf,
            adc_result_vbat,
            adc_result_tpcb,
            adc_result_current_vbat,
        ],
        local = [
            command_accumulator,
            hw,
        ]
    )]
    async fn logic_task(mut cx: logic_task::Context) {
        let mut state = app::MainState::new();

        loop {
            // Update values
            cx.local.hw.adc_result_vbat = cx.shared.adc_result_vbat.lock(|v| *v);
            cx.local.hw.adc_result_tpcb = cx.shared.adc_result_tpcb.lock(|v| *v);

            state.update(cx.local.hw);

            // Handle CAN receive buffer
            while let Some(received_frame) =
                cx.shared.can_rx_buf.lock(|can_rx_buf| can_rx_buf.dequeue())
            {
                state.on_can(received_frame);
            }
            // Handle CAN transmit buffer
            while let Some(frame) = cx.local.hw.can_tx_buf.dequeue() {
                cx.shared
                    .can_tx_buf
                    .lock(|can_tx_buf| can_tx_buf.push(frame));
                pac::NVIC::pend(pac::Interrupt::CAN1_TX);
            }

            // Handle console commands
            while let Some(b) = cx.shared.console_rxbuf.lock(|rxbuf| rxbuf.dequeue()) {
                if let Some(command) = cx.local.command_accumulator.put(b as char) {
                    info!("Command: {:?}", command);
                    if state.on_console_command(&command, cx.local.hw) {
                        // Higher level logic handled the command
                    } else {
                        info!(
                            "-> {:?} is an unknown command. Available commands:",
                            command
                        );
                        state.list_console_commands();
                    }
                }
            }

            Systick::delay(15.millis()).await;
        }
    }

    #[task(priority = 2,
        shared = [
            adc_result_current_vbat,
            adc_result_vbat,
            adc_result_tpcb,
        ],
        local = [
            adc1,
            adc_pa1,
            adc_pa2,
            adc_pa3,
            adc_pb1,
        ]
    )]
    async fn adc_task(mut cx: adc_task::Context) {
        let mut mux_channel: usize = 0;
        loop {
            // NOTE: DMA seemed to work, until it stopped after an essentially
            // random amount of time. Thus, we are doing it this way.

            //let adc_result_hwver = cx.local.adc.convert(cx.local.adc_pa1, SampleTime::Cycles_480);

            let adc_result_vbat =
                cx.local
                    .adc1
                    .convert(cx.local.adc_pa3, SampleTime::Cycles_480) as f32
                    * 0.00881;

            // Assign with lowpass
            cx.shared.adc_result_vbat.lock(|v| *v = *v * 0.98 + adc_result_vbat * 0.02);

            // TODO: Current measurements on PA4..PA7
            /*let adc_result_current_vbat = cx
                .local
                .adc1
                .convert(cx.local.adc_pa4, SampleTime::Cycles_480);

            cx.shared.adc_result_current_vbat.lock(|v| *v = adc_result_current_vbat);*/

            // Correct conversion from ADC to DegC (10k NTC thermistor with 10k
            // pull-up resistor)
            let adc_value = cx.local.adc1.convert(cx.local.adc_pb1, SampleTime::Cycles_480) as f32;
            let r_ntc = 10000.0 * adc_value / (4095.0 - adc_value);
            let ln_r = (r_ntc / 10000.0).ln();
            let t_inv = ln_r / 3950.0 + 1.0 / 298.15;
            let t_kelvin = 1.0 / t_inv;
            let t_celsius = t_kelvin - 273.15;
            let adc_result_tpcb = t_celsius;

            // Assign with lowpass
            cx.shared.adc_result_tpcb.lock(|v| *v = *v * 0.98 + adc_result_tpcb * 0.02);

            Systick::delay(20.millis()).await;
        }
    }

    #[task(
        priority = 5,
        binds = USART1,
        shared = [
            console_rxbuf,
        ],
        local = [
            usart1_rx,
            usart1_tx,
            usart1_txbuf: ConstGenericRingBuffer<u8, LOG_BUFFER_SIZE> =
                    ConstGenericRingBuffer::new(),
        ])
    ]
    fn usart1(mut cx: usart1::Context) {
        // Check if there is something to receive, and if so, receive it into
        // somewhere
        if let Ok(b) = cx.local.usart1_rx.read() {
            trace!("USART1/console: Received: {:?}", b);
            //cx.local.usart1_txbuf.push(b); // Echo
            cx.shared.console_rxbuf.lock(|rxbuf| {
                rxbuf.push(b);
            });
        }
        if cx.local.usart1_txbuf.is_empty() {
            // Copy MULTI_LOGGER's buffer to usart1_txbuf
            // NOTE: This assumes there are only single-byte characters in the
            // buffer. Otherwise it won't fully fit in our byte-based usart1_txbuf
            let logger_usart1_txbuf_option = MULTI_LOGGER.get_uart_buffer();
            if let Some(logger_usart1_txbuf) = logger_usart1_txbuf_option {
                for b in logger_usart1_txbuf.bytes() {
                    cx.local.usart1_txbuf.push(b);
                }
            }
            if cx.local.usart1_txbuf.is_empty() {
                cx.local.usart1_tx.unlisten();
            }
        }
        if let Some(b) = cx.local.usart1_txbuf.front() {
            match cx.local.usart1_tx.write(*b) {
                Ok(_) => {
                    cx.local.usart1_txbuf.dequeue();
                }
                Err(_) => {}
            }
        }
        if !cx.local.usart1_txbuf.is_empty() {
            cx.local.usart1_tx.listen();
        }
    }

    /*#[task(
        priority = 6,
        binds = USART3,
        shared = [mainboard_rxbuf, mainboard_txbuf],
        local = [
            usart3_rx,
            usart3_tx,
        ])
    ]
    fn usart3(mut cx: usart3::Context) {
        // Receive to buffer
        if let Ok(b) = cx.local.usart3_rx.read() {
            trace!("USART3/mainboard: Received: {:?}", b);
            cx.shared.mainboard_rxbuf.lock(|rxbuf| {
                rxbuf.push(b);
            });
        }
        // Transmit from buffer
        cx.shared.mainboard_txbuf.lock(|txbuf| {
            if let Some(b) = txbuf.front() {
                match cx.local.usart3_tx.write(*b) {
                    Ok(_) => {
                        txbuf.dequeue();
                    },
                    Err(_) => {},
                }
            }
            if txbuf.is_empty() {
                cx.local.usart3_tx.unlisten();
            } else {
                cx.local.usart3_tx.listen();
            }
        });
    }*/

    /*#[task(
        priority = 9,
        binds = USART2,
        shared = [usart2_rxbuf, usart2_txbuf],
        local = [
            usart2_rx,
            usart2_tx,
        ])
    ]
    fn usart2(mut cx: usart2::Context) {
        // Receive to buffer
        while let Ok(b) = cx.local.usart2_rx.read() {
            //trace!("USART2/SIM7600: Received: {:?}", b);
            cx.shared.usart2_rxbuf.lock(|rxbuf| {
                rxbuf.push(b);
            });
        }
        // Transmit from buffer
        cx.shared.usart2_txbuf.lock(|txbuf| {
            if let Some(b) = txbuf.front() {
                match cx.local.usart2_tx.write(*b) {
                    Ok(_) => {
                        txbuf.dequeue();
                    }
                    Err(_) => {}
                }
            }
            if txbuf.is_empty() {
                cx.local.usart2_tx.unlisten();
            } else {
                cx.local.usart2_tx.listen();
            }
        });
    }*/

    #[task(
        priority = 7,
        binds = OTG_FS,
        shared = [
            usb_dev,
            usb_serial,
            console_rxbuf
        ],
        local = [
            usb_serial_txbuf: ConstGenericRingBuffer<u8, LOG_BUFFER_SIZE> =
                    ConstGenericRingBuffer::new(),
        ],
    )]
    fn otg_fs_int(cx: otg_fs_int::Context) {
        let otg_fs_int::SharedResources {
            __rtic_internal_marker,
            mut usb_dev,
            mut usb_serial,
            mut console_rxbuf,
        } = cx.shared;

        // Fill up usb_serial_txbuf
        if cx.local.usb_serial_txbuf.is_empty() {
            // NOTE: This assumes there are only single-byte characters in the
            // buffer. Otherwise it won't fully fit in our byte-based usb_serial_txbuf
            let logger_usb_serial_txbuf_option = MULTI_LOGGER.get_usb_buffer();
            if let Some(logger_usb_serial_txbuf) = logger_usb_serial_txbuf_option {
                for b in logger_usb_serial_txbuf.bytes() {
                    cx.local.usb_serial_txbuf.push(b);
                }
            }
        }

        // Write
        (&mut usb_serial).lock(|usb_serial| {
            if let Some(b) = cx.local.usb_serial_txbuf.front() {
                let buf: [u8; 1] = [*b];
                match usb_serial.write(&buf) {
                    Ok(n_written) => {
                        for _ in 0..n_written {
                            cx.local.usb_serial_txbuf.dequeue();
                        }
                    }
                    _ => {}
                }
            }
        });

        // Read
        (&mut usb_dev, &mut usb_serial, &mut console_rxbuf).lock(
            |usb_dev, usb_serial, console_rxbuf| {
                if usb_dev.poll(&mut [usb_serial]) {
                    let mut buf = [0u8; 64];
                    match usb_serial.read(&mut buf) {
                        Ok(count) if count > 0 => {
                            for i in 0..count {
                                //cx.local.usb_serial_txbuf.push(buf[i]); // Echo
                                console_rxbuf.push(buf[i]);
                            }
                        }
                        _ => {}
                    }
                }
            },
        );
    }

    #[task(
        priority = 8,
        binds = CAN1_RX0,
        shared = [
            can1,
            can_rx_buf,
        ]
    )]
    fn can1_rx0(cx: can1_rx0::Context) {
        (cx.shared.can1, cx.shared.can_rx_buf).lock(|can1, can_rx_buf| {
            if let Ok(frame) = can1.receive() {
                trace!("CAN1 << {:?} {:?}", frame.id(), frame.data());
                can_rx_buf.push(frame);
            }
        });
    }

    #[task(
        priority = 8,
        binds = CAN1_RX1,
        shared = [
            can1,
            can_rx_buf,
        ]
    )]
    fn can1_rx1(cx: can1_rx1::Context) {
        (cx.shared.can1, cx.shared.can_rx_buf).lock(|can1, can_rx_buf| {
            if let Ok(frame) = can1.receive() {
                trace!("CAN1 << {:?} {:?}", frame.id(), frame.data());
                can_rx_buf.push(frame);
            }
        });
    }

    #[task(
        priority = 8,
        binds = CAN1_TX,
        shared = [
            can1,
            can_tx_buf,
        ]
    )]
    fn can1_tx(cx: can1_tx::Context) {
        (cx.shared.can1, cx.shared.can_tx_buf).lock(|can1, can_tx_buf| {
            can1.clear_tx_interrupt();
            if let Some(frame) = can_tx_buf.dequeue() {
                trace!("-!- CAN1 >> {:?} {:?}", frame.id(), frame.data());
                let _ = can1.transmit(&frame);
                short_busywait(); // NOTE: HACK: For some reson messages get
                                  // dropped from a long TX queue without this
            }
        });
    }
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    if let Some(panic_tx) = unsafe { PANIC_TX.as_mut() } {
        _ = panic_tx.write_str("\r\n");
        if let Some(location) = info.location() {
            _ = write!(
                panic_tx,
                "Panic at {}:{}:{}: ",
                location.file(),
                location.line(),
                location.column()
            );
        }
        _ = core::fmt::write(panic_tx, format_args!("{}", info.message()));
        _ = panic_tx.write_str("\r\n\r\n");
        _ = panic_tx.flush();

        // Wait for some time so that it all actually gets printed out
        long_busywait();
    }

    cortex_m::peripheral::SCB::sys_reset();
}

fn short_busywait() {
    for _ in 0..20000 {
        cortex_m::asm::nop();
    }
}
fn medium_busywait() {
    for _ in 0..1000000 {
        cortex_m::asm::nop();
    }
}
fn long_busywait() {
    for _ in 0..10000000 {
        cortex_m::asm::nop();
    }
}
