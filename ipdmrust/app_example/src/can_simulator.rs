use common::bxcan;

#[allow(unused_imports)]
use log::{info, warn};
use ringbuffer::{ConstGenericRingBuffer, RingBuffer};

pub struct CanSimulator {
    pub txbuf: ConstGenericRingBuffer<bxcan::Frame, 10>,
    i: u64,
}

impl CanSimulator {
    pub fn new() -> Self {
        Self {
            txbuf: ConstGenericRingBuffer::new(),
            i: 0,
        }
    }

    pub fn update(&mut self, millis: u64) {
        // You can generate these using util/generate_can_simulator_txframe.py

        if self.i % 27 == 0 {
            self.txbuf.push(bxcan::Frame::new_data(
                bxcan::StandardId::new(0x50).unwrap(),
                bxcan::Data::new(b"\x17\x20\x00\x00\x00\x00\x96\x00").unwrap(),
            ));
        }
        if self.i % 27 == 1 {
            self.txbuf.push(bxcan::Frame::new_data(
                bxcan::StandardId::new(0x51).unwrap(),
                bxcan::Data::new(b"\x00\x00\xff\xff\x00\x00\x00\x00").unwrap(),
            ));
        }
        if self.i % 27 == 2 {
            self.txbuf.push(bxcan::Frame::new_data(
                bxcan::StandardId::new(0x52).unwrap(),
                bxcan::Data::new(b"\x15\x1d\x1a\x12\x00\x00\x00\x00").unwrap(),
            ));
        }
        if self.i % 27 == 3 {
            // ipdm1 (includes pm state and pm contactor reason)
            self.txbuf.push(bxcan::Frame::new_data(
                bxcan::StandardId::new(0x550).unwrap(),
                bxcan::Data::new(b"\x41\x00\x00\x32\xd0\x15\x00\xe6").unwrap(),
            ));
        }
        if self.i % 27 == 4 {
            // BMS (includes main contactor status)
            self.txbuf.push(bxcan::Frame::new_data(
                bxcan::StandardId::new(0x30).unwrap(),
                bxcan::Data::new(b"\x07\x0b\xd1\x00\x00\x00\x00\x00").unwrap(),
            ));
        }
        if self.i % 27 == 5 {
            // BMS (includes cell voltages and temperatures)
            self.txbuf.push(bxcan::Frame::new_data(
                bxcan::StandardId::new(0x31).unwrap(),
                bxcan::Data::new(b"\x18\x21\x85\x06\x0a\x00\x00\x09").unwrap(),
            ));
        }
        if self.i % 27 == 6 {
            // BMS (includes SoC)
            self.txbuf.push(bxcan::Frame::new_data(
                bxcan::StandardId::new(0x32).unwrap(),
                bxcan::Data::new(b"\x00\xa3\x00\x3c\x0d\xac\xb1\x00").unwrap(),
            ));
        }
        if self.i % 27 == 7 {
            self.txbuf.push(bxcan::Frame::new_data(
                bxcan::StandardId::new(0x33).unwrap(),
                bxcan::Data::new(b"\x00\x00\x00\x00\x03\x01\x05\x3e").unwrap(),
            ));
        }
        if self.i % 27 == 8 {
            self.txbuf.push(bxcan::Frame::new_data(
                bxcan::StandardId::new(0x34).unwrap(),
                bxcan::Data::new(b"\x10\x18\x00\x00\x00\x00\x00\x00").unwrap(),
            ));
        }
        if self.i % 27 == 9 {
            self.txbuf.push(bxcan::Frame::new_data(
                bxcan::StandardId::new(0x377).unwrap(),
                bxcan::Data::new(b"\x05\x9c\x01\x12\x45\x4b\x40\x22").unwrap(),
            ));
        }
        if self.i % 27 == 10 {
            // BMS (includes main contactor status)
            self.txbuf.push(bxcan::Frame::new_data(
                bxcan::StandardId::new(0x389).unwrap(),
                bxcan::Data::new(b"\x97\x00\x01\x3f\x40\xd0\x00\x00").unwrap(),
            ));
        }
        if self.i % 27 == 11 {
            self.txbuf.push(bxcan::Frame::new_data(
                bxcan::StandardId::new(0x38A).unwrap(),
                bxcan::Data::new(b"\x44\x3f\x81\x1b\x04\x00\x00\x00").unwrap(),
            ));
        }
        if self.i % 27 == 12 {
            self.txbuf.push(bxcan::Frame::new_data(
                bxcan::StandardId::new(0x569).unwrap(),
                bxcan::Data::new(b"\xaa\xa8\xa2\x2a\x00\x00\x00\x00").unwrap(),
            ));
        }
        if self.i % 27 == 13 {
            self.txbuf.push(bxcan::Frame::new_data(
                bxcan::StandardId::new(0x568).unwrap(),
                bxcan::Data::new(b"\x8a\x3a\xaa\x8a\xaa\x8a\xa8\xaa").unwrap(),
            ));
        }
        if self.i % 27 == 14 {
            self.txbuf.push(bxcan::Frame::new_data(
                bxcan::StandardId::new(0x563).unwrap(),
                bxcan::Data::new(b"\xaa\xaa\xaa\xaa\x82\x2a\x00\x00").unwrap(),
            ));
        }
        if self.i % 27 == 15 {
            self.txbuf.push(bxcan::Frame::new_data(
                bxcan::StandardId::new(0x664).unwrap(),
                bxcan::Data::new(b"\x02\x83\x06\x32\xf3\x00\x00\x00").unwrap(),
            ));
        }
        if self.i % 27 == 16 {
            self.txbuf.push(bxcan::Frame::new_data(
                bxcan::StandardId::new(0x663).unwrap(),
                bxcan::Data::new(b"\x01\x31\x30\x31\x00\x00\x00\x00").unwrap(),
            ));
        }
        if self.i % 27 == 17 {
            // Outlander heater
            self.txbuf.push(bxcan::Frame::new_data(
                bxcan::StandardId::new(0x398).unwrap(),
                bxcan::Data::new(b"\x01\x1c\x13\x5e\x5f\x28\x00\x00").unwrap(),
            ));
        }
        if self.i % 27 == 18 {
            self.txbuf.push(bxcan::Frame::new_data(
                bxcan::StandardId::new(0x630).unwrap(),
                bxcan::Data::new(b"\x00\x00\x00\x00\x00\x00\x00\x00").unwrap(),
            ));
        }
        if self.i % 27 == 19 {
            self.txbuf.push(bxcan::Frame::new_data(
                bxcan::StandardId::new(0x62D).unwrap(),
                bxcan::Data::new(b"\x00\x00\x00\x00\x00\x00\x00\x00").unwrap(),
            ));
        }
        if self.i % 27 == 20 {
            self.txbuf.push(bxcan::Frame::new_data(
                bxcan::StandardId::new(0x6BD).unwrap(),
                bxcan::Data::new(b"\x00\x00\x00\x00\x00\x00\x00\x00").unwrap(),
            ));
        }
        if self.i % 27 == 21 {
            self.txbuf.push(bxcan::Frame::new_data(
                bxcan::StandardId::new(0x500).unwrap(),
                bxcan::Data::new(b"\x00\x08\x00\x00\x00\x00\x00\x1e").unwrap(),
            ));
        }
        if self.i % 27 == 22 {
            self.txbuf.push(bxcan::Frame::new_data(
                bxcan::StandardId::new(0x506).unwrap(),
                bxcan::Data::new(b"\x00\x00\x00\x00\x00\x00\x00\x00").unwrap(),
            ));
        }
        if self.i % 27 == 23 {
            self.txbuf.push(bxcan::Frame::new_data(
                bxcan::StandardId::new(0x521).unwrap(),
                bxcan::Data::new(b"\x00\x09\xff\xff\xd9\x4f").unwrap(),
            ));
        }
        if self.i % 27 == 24 {
            self.txbuf.push(bxcan::Frame::new_data(
                bxcan::StandardId::new(0x525).unwrap(),
                bxcan::Data::new(b"\x04\x0b\x00\x00\x00\x71").unwrap(),
            ));
        }
        if self.i % 27 == 25 {
            self.txbuf.push(bxcan::Frame::new_data(
                bxcan::StandardId::new(0x527).unwrap(),
                bxcan::Data::new(b"\x06\x08\x00\x00\x8f\xca").unwrap(),
            ));
        }
        if self.i % 27 == 26 {
            self.txbuf.push(bxcan::Frame::new_data(
                bxcan::StandardId::new(0x50).unwrap(),
                bxcan::Data::new(b"\x17\x20\x00\x00\x00\x00\x96\x00").unwrap(),
            ));
        }

        self.i += 1;
    }
}
