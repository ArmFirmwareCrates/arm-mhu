// SPDX-FileCopyrightText: Copyright The arm-mhu Contributors.
// SPDX-License-Identifier: MIT OR Apache-2.0

#![no_std]
#![doc = include_str!("../README.md")]
#![deny(clippy::undocumented_unsafe_blocks)]
#![deny(unsafe_op_in_unsafe_fn)]

//!
//! The implementation is based on Message Handling Unit Architecture version 3.0 (ARM-AES-0072 A.b).

/// Control page driver module.
pub mod control;
/// Doorbell driver module.
pub mod doorbell;
/// Fast channel driver module.
pub mod fast_channel;
/// FIFO driver module.
pub mod fifo;
/// Sender/Receiver Security Control module.
pub mod security_control;

use crate::{
    control::{
        FfchCfg0, MhuMailboxControl, MhuMailboxControlRegisters, MhuPostboxControl,
        MhuPostboxControlRegisters,
    },
    doorbell::{
        MhuMailboxDoorbell, MhuMailboxDoorbellRegisters, MhuPostboxDoorbell,
        MhuPostboxDoorbellRegisters,
    },
    fast_channel::{
        MhuMailboxFastChannel, MhuMailboxFastChannelOpaqueRegisters, MhuPostboxFastChannel,
        MhuPostboxFastChannelOpaqueRegisters,
    },
    fifo::{MhuMailboxFifo, MhuMailboxFifoRegisters, MhuPostboxFifo, MhuPostboxFifoRegisters},
};
use safe_mmio::{UniqueMmioPointer, field, split_fields};
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout};

/// MHU error type.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    /// The MHU peripheral's version is not supported by the crate.
    UnsupportedMhuVersion,
    /// The operation is not supported because the feature is not implemented
    /// by the MHU instance.
    OperationNotSupported,
    /// The channel index is greater than the number of implemented channels.
    InvalidChannelIndex,
}

/// MHU Postbox register block.
///
/// See C2.1.1 PBX, Postbox.
#[derive(Clone, Eq, FromBytes, Immutable, IntoBytes, KnownLayout, PartialEq)]
#[repr(C)]
pub struct MhuPostboxRegisters {
    /// Postbox Control
    pbx_ctrl_page: MhuPostboxControlRegisters,
    /// Postbox Doorbell Channel Window
    pdbcw_page: [MhuPostboxDoorbellRegisters; 128],
    /// Postbox FIFO Channel Windows Page
    pffcw_page: [MhuPostboxFifoRegisters; 64],
    /// Postbox Fast Channel Windows Page
    pfcw_page: MhuPostboxFastChannelOpaqueRegisters,
    reserved_4000: [u32; 0x2C00],
    /// Postbox Implementation Defined page
    pbx_impl_def_page: [u32; 1024],
}

/// MHU Mailbox register block.
///
/// See C2.2.1 MBX, Mailbox.
#[derive(Clone, Eq, FromBytes, Immutable, IntoBytes, KnownLayout, PartialEq)]
#[repr(C)]
pub struct MhuMailboxRegisters {
    /// Mailbox control
    mbx_ctrl_page: MhuMailboxControlRegisters,
    /// Mailbox Doorbell Channel Window
    mdbcw_page: [MhuMailboxDoorbellRegisters; 128],
    /// Mailbox FIFO Channel Window Page
    mffcw_page: [MhuMailboxFifoRegisters; 64],
    /// Mailbox Fast Channel Windows Page
    mfcw_page: MhuMailboxFastChannelOpaqueRegisters,
    reserved_4000: [u32; 0x2C00],
    /// Mailbox Implementation Defined page
    mbx_impl_def_page: [u32; 1024],
}

/// MHU Postbox driver.
pub struct MhuPostbox<'a> {
    regs: UniqueMmioPointer<'a, MhuPostboxRegisters>,
}

impl<'a> MhuPostbox<'a> {
    /// Creates new Postbox instance.
    pub fn new(regs: UniqueMmioPointer<'a, MhuPostboxRegisters>) -> Self {
        Self { regs }
    }

    /// Returns Control block driver.
    pub fn control<'regs>(&'regs mut self) -> MhuPostboxControl<'regs> {
        MhuPostboxControl::new(field!(self.regs, pbx_ctrl_page))
    }

    /// Returns all Doorbell Channel instances.
    pub fn doorbells(&mut self) -> Option<MhuPostboxDoorbells<'_>> {
        if let Some(config) = self.control().doorbell_config() {
            Some(MhuPostboxDoorbells::new(
                field!(self.regs, pdbcw_page),
                config.channel_count(),
            ))
        } else {
            None
        }
    }

    /// Returns Doorbell Channel or None if the channel is not implemented.
    pub fn doorbell<'regs>(&'regs mut self, channel: usize) -> Option<MhuPostboxDoorbell<'regs>> {
        let doorbell_count = self
            .control()
            .doorbell_config()
            .map_or(0, |config| config.channel_count());

        if channel < doorbell_count {
            Some(MhuPostboxDoorbell::new(
                field!(self.regs, pdbcw_page).take(channel).unwrap(),
            ))
        } else {
            None
        }
    }

    /// Returns all FIFO Channel instances.
    pub fn fifos(&mut self) -> Option<MhuPostboxFifos<'_>> {
        if let Some(config) = self.control().fifo_config() {
            Some(MhuPostboxFifos::new(field!(self.regs, pffcw_page), config))
        } else {
            None
        }
    }

    /// Returns FIFO Channel or None if the channel is not implemented.
    pub fn fifo<'regs>(&'regs mut self, channel: usize) -> Option<MhuPostboxFifo<'regs>> {
        if let Some(config) = self.control().fifo_config()
            && channel < config.channel_count()
        {
            Some(MhuPostboxFifo::new(
                field!(self.regs, pffcw_page).take(channel).unwrap(),
                config,
            ))
        } else {
            None
        }
    }

    /// Returns Fast Channel instance.
    pub fn fast_channel(&mut self) -> Option<MhuPostboxFastChannel<'_>> {
        if let Some(config) = self.control().fast_channel_config() {
            Some(MhuPostboxFastChannel::new(
                field!(self.regs, pfcw_page),
                config,
            ))
        } else {
            None
        }
    }

    /// Splits Postbox into Control block, Doorbell, FIFO and Fast Channel units.
    pub fn split(
        self,
    ) -> (
        MhuPostboxControl<'a>,
        Option<MhuPostboxDoorbells<'a>>,
        Option<MhuPostboxFifos<'a>>,
        Option<MhuPostboxFastChannel<'a>>,
    ) {
        // Safety: Each field name is only passed once.
        let (control_regs, doorbell_regs, fifo_regs, fast_channel_regs) =
            unsafe { split_fields!(self.regs, pbx_ctrl_page, pdbcw_page, pffcw_page, pfcw_page) };

        let control = MhuPostboxControl::new(control_regs);

        let doorbells = control
            .doorbell_config()
            .map(|config| MhuPostboxDoorbells::new(doorbell_regs, config.channel_count()));

        let fifos = control
            .fifo_config()
            .map(|config| MhuPostboxFifos::new(fifo_regs, config));

        let fast_channel = control
            .fast_channel_config()
            .map(|config| MhuPostboxFastChannel::new(fast_channel_regs, config));

        (control, doorbells, fifos, fast_channel)
    }
}

/// MHU Mailbox driver.
pub struct MhuMailbox<'a> {
    regs: UniqueMmioPointer<'a, MhuMailboxRegisters>,
}

impl<'a> MhuMailbox<'a> {
    /// Creates new Mailbox instance.
    pub fn new(regs: UniqueMmioPointer<'a, MhuMailboxRegisters>) -> Self {
        Self { regs }
    }

    /// Returns Control block driver.
    pub fn control<'regs>(&'regs mut self) -> MhuMailboxControl<'regs> {
        MhuMailboxControl::new(field!(self.regs, mbx_ctrl_page))
    }

    /// Returns all Doorbell Channel instances.
    pub fn doorbells(&mut self) -> Option<MhuMailboxDoorbells<'_>> {
        if let Some(config) = self.control().doorbell_config() {
            Some(MhuMailboxDoorbells::new(
                field!(self.regs, mdbcw_page),
                config.channel_count(),
            ))
        } else {
            None
        }
    }

    /// Returns Doorbell Channel or None if the channel is not implemented.
    pub fn doorbell<'regs>(&'regs mut self, channel: usize) -> Option<MhuMailboxDoorbell<'regs>> {
        let doorbell_count = self
            .control()
            .doorbell_config()
            .map_or(0, |config| config.channel_count());

        if channel < doorbell_count {
            Some(MhuMailboxDoorbell::new(
                field!(self.regs, mdbcw_page).take(channel).unwrap(),
            ))
        } else {
            None
        }
    }

    /// Returns all FIFO Channel instances.
    pub fn fifos(&mut self) -> Option<MhuMailboxFifos<'_>> {
        if let Some(config) = self.control().fifo_config() {
            Some(MhuMailboxFifos::new(field!(self.regs, mffcw_page), config))
        } else {
            None
        }
    }

    /// Returns FIFO Channel or None if the channel is not implemented.
    pub fn fifo<'regs>(&'regs mut self, channel: usize) -> Option<MhuMailboxFifo<'regs>> {
        if let Some(config) = self.control().fifo_config()
            && channel < config.channel_count()
        {
            Some(MhuMailboxFifo::new(
                field!(self.regs, mffcw_page).take(channel).unwrap(),
                config,
            ))
        } else {
            None
        }
    }

    /// Returns Fast Channel instance.
    pub fn fast_channel(&mut self) -> Option<MhuMailboxFastChannel<'_>> {
        if let Some(config) = self.control().fast_channel_config() {
            Some(MhuMailboxFastChannel::new(
                field!(self.regs, mfcw_page),
                config,
            ))
        } else {
            None
        }
    }

    /// Splits Mailbox into Control block, Doorbell, FIFO and Fast Channel units.
    pub fn split(
        self,
    ) -> (
        MhuMailboxControl<'a>,
        Option<MhuMailboxDoorbells<'a>>,
        Option<MhuMailboxFifos<'a>>,
        Option<MhuMailboxFastChannel<'a>>,
    ) {
        // Safety: Each field name is only passed once.
        let (control_regs, doorbell_regs, fifo_regs, fast_channel_regs) =
            unsafe { split_fields!(self.regs, mbx_ctrl_page, mdbcw_page, mffcw_page, mfcw_page) };

        let control = MhuMailboxControl::new(control_regs);

        let doorbells = control
            .doorbell_config()
            .map(|config| MhuMailboxDoorbells::new(doorbell_regs, config.channel_count()));

        let fifos = control
            .fifo_config()
            .map(|config| MhuMailboxFifos::new(fifo_regs, config));

        let fast_channel = control
            .fast_channel_config()
            .map(|config| MhuMailboxFastChannel::new(fast_channel_regs, config));

        (control, doorbells, fifos, fast_channel)
    }
}

/// Handles all Postbox Doorbell instances.
pub struct MhuPostboxDoorbells<'a> {
    regs: UniqueMmioPointer<'a, [MhuPostboxDoorbellRegisters; 128]>,
    count: usize,
}

impl<'a> MhuPostboxDoorbells<'a> {
    /// Creates new Doorbells instance.
    pub fn new(
        regs: UniqueMmioPointer<'a, [MhuPostboxDoorbellRegisters; 128]>,
        count: usize,
    ) -> Self {
        Self { regs, count }
    }

    /// Borrows a single Doorbell instance. Returns `None` if the channel is not implemented.
    pub fn doorbell<'regs>(&'regs mut self, channel: usize) -> Option<MhuPostboxDoorbell<'regs>> {
        if channel < self.count {
            Some(MhuPostboxDoorbell::new(self.regs.get(channel).unwrap()))
        } else {
            None
        }
    }

    /// Takes a single Doorbell instance and consume Doorbells object. Returns `None` if the channel
    /// is not implemented.
    pub fn take(self, channel: usize) -> Option<MhuPostboxDoorbell<'a>> {
        if channel < self.count {
            Some(MhuPostboxDoorbell::new(self.regs.take(channel).unwrap()))
        } else {
            None
        }
    }

    /// Splits the chosen Doorbells into individual Doorbell objects.
    pub fn split_some<const N: usize>(self, chosen: [usize; N]) -> [MhuPostboxDoorbell<'a>; N] {
        assert!(chosen.iter().all(|i| *i < self.count));
        self.regs.split_some(chosen).map(MhuPostboxDoorbell::new)
    }
}

/// Handles all Mailbox Doorbell instances.
pub struct MhuMailboxDoorbells<'a> {
    regs: UniqueMmioPointer<'a, [MhuMailboxDoorbellRegisters; 128]>,
    count: usize,
}

impl<'a> MhuMailboxDoorbells<'a> {
    /// Creates new Doorbells instance.
    pub fn new(
        regs: UniqueMmioPointer<'a, [MhuMailboxDoorbellRegisters; 128]>,
        count: usize,
    ) -> Self {
        Self { regs, count }
    }

    /// Borrows a single Doorbell instance. Returns `None` if the channel is not implemented.
    pub fn doorbell<'regs>(&'regs mut self, channel: usize) -> Option<MhuMailboxDoorbell<'regs>> {
        if channel < self.count {
            Some(MhuMailboxDoorbell::new(self.regs.get(channel).unwrap()))
        } else {
            None
        }
    }

    /// Takes a single Doorbell instance and consume Doorbells object. Returns `None` if the channel
    /// is not implemented.
    pub fn take(self, channel: usize) -> Option<MhuMailboxDoorbell<'a>> {
        if channel < self.count {
            Some(MhuMailboxDoorbell::new(self.regs.take(channel).unwrap()))
        } else {
            None
        }
    }

    /// Splits the chosen Doorbells into individual Doorbell objects.
    pub fn split_some<const N: usize>(self, chosen: [usize; N]) -> [MhuMailboxDoorbell<'a>; N] {
        assert!(chosen.iter().all(|i| *i < self.count));
        self.regs.split_some(chosen).map(MhuMailboxDoorbell::new)
    }
}

/// Handles all Postbox FIFO instances.
pub struct MhuPostboxFifos<'a> {
    regs: UniqueMmioPointer<'a, [MhuPostboxFifoRegisters; 64]>,
    config: FfchCfg0,
}

impl<'a> MhuPostboxFifos<'a> {
    /// Creates new FIFOs instance.
    pub fn new(
        regs: UniqueMmioPointer<'a, [MhuPostboxFifoRegisters; 64]>,
        config: FfchCfg0,
    ) -> Self {
        Self { regs, config }
    }

    /// Borrows a single FIFO instance. Returns `None` if the channel is not implemented.
    pub fn fifo<'regs>(&'regs mut self, channel: usize) -> Option<MhuPostboxFifo<'regs>> {
        if channel < self.config.channel_count() {
            Some(MhuPostboxFifo::new(
                self.regs.get(channel).unwrap(),
                self.config,
            ))
        } else {
            None
        }
    }

    /// Takes a single FIFO instance and consume FIFOs object. Returns `None` if the channel
    /// is not implemented.
    pub fn take(self, channel: usize) -> Option<MhuPostboxFifo<'a>> {
        if channel < self.config.channel_count() {
            Some(MhuPostboxFifo::new(
                self.regs.take(channel).unwrap(),
                self.config,
            ))
        } else {
            None
        }
    }

    /// Splits the chosen FIFOs into individual FIFO objects.
    pub fn split_some<const N: usize>(self, chosen: [usize; N]) -> [MhuPostboxFifo<'a>; N] {
        assert!(chosen.iter().all(|i| *i < self.config.channel_count()));
        self.regs
            .split_some(chosen)
            .map(|regs| MhuPostboxFifo::new(regs, self.config))
    }
}

/// Handles all Mailbox FIFO instances.
pub struct MhuMailboxFifos<'a> {
    regs: UniqueMmioPointer<'a, [MhuMailboxFifoRegisters; 64]>,
    config: FfchCfg0,
}

impl<'a> MhuMailboxFifos<'a> {
    /// Creates new FIFOs instance.
    pub fn new(
        regs: UniqueMmioPointer<'a, [MhuMailboxFifoRegisters; 64]>,
        config: FfchCfg0,
    ) -> Self {
        Self { regs, config }
    }

    /// Borrows a single FIFO instance. Returns `None` if the channel is not implemented.
    pub fn fifo<'regs>(&'regs mut self, channel: usize) -> Option<MhuMailboxFifo<'regs>> {
        if channel < self.config.channel_count() {
            Some(MhuMailboxFifo::new(
                self.regs.get(channel).unwrap(),
                self.config,
            ))
        } else {
            None
        }
    }

    /// Takes a single FIFO instance and consume FIFOs object. Returns `None` if the channel
    /// is not implemented.
    pub fn take(self, channel: usize) -> Option<MhuMailboxFifo<'a>> {
        if channel < self.config.channel_count() {
            Some(MhuMailboxFifo::new(
                self.regs.take(channel).unwrap(),
                self.config,
            ))
        } else {
            None
        }
    }

    /// Splits the chosen FIFOs into individual FIFO objects.
    pub fn split_some<const N: usize>(self, chosen: [usize; N]) -> [MhuMailboxFifo<'a>; N] {
        assert!(chosen.iter().all(|i| *i < self.config.channel_count()));
        self.regs
            .split_some(chosen)
            .map(|regs| MhuMailboxFifo::new(regs, self.config))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! define_fake_regs {
        ($name:ident, $count:literal, $regs:ty, $driver:tt $(, $init_params:tt)?) => {
            pub struct $name {
                pub regs: [u64; $count / 2],
            }

            #[allow(dead_code)]
            impl $name {
                pub fn new() -> Self {
                    Self {
                        regs: [0u64; $count / 2],
                    }
                }
                pub fn clear(&mut self) {
                    self.regs.fill(0);
                }

                pub fn reg_write(&mut self, offset: usize, value: u32) {
                    let regs: &mut [u32; $count] = zerocopy::transmute_mut!(&mut self.regs);
                    regs[offset / 4] = value;
                }

                pub fn reg_read(&self, offset: usize) -> u32 {
                    let regs: &[u32; $count] = zerocopy::transmute_ref!(&self.regs);
                    regs[offset / 4]
                }

                fn get(&mut self) -> UniqueMmioPointer<'_, $regs> {
                    UniqueMmioPointer::from(zerocopy::transmute_mut!(&mut self.regs))
                }

                pub fn instance_for_test(&mut self) -> $driver<'_> {
                    <$driver>::new(self.get(), $( $init_params )?)
                }
            }
        };
    }

    pub(crate) use define_fake_regs;

    macro_rules! assert_offset {
        ($expected_offset:expr, $reg:expr, $base:expr) => {
            assert_eq!($expected_offset, $reg.ptr() as usize - $base);
        };
    }

    define_fake_regs!(FakePostboxRegisters, 16384, MhuPostboxRegisters, MhuPostbox);
    define_fake_regs!(FakeMailboxRegisters, 16384, MhuMailboxRegisters, MhuMailbox);

    impl FakePostboxRegisters {
        pub fn enable_all_features(&mut self) {
            self.reg_write(0x010, 0x0011_1111);
        }
    }

    impl FakeMailboxRegisters {
        pub fn enable_all_features(&mut self) {
            self.reg_write(0x010, 0x0011_1111);
        }
    }

    const DOORBELL_COUNT: usize = 16;

    define_fake_regs!(
        FakePostboxDoorbellsRegisters,
        1024,
        [MhuPostboxDoorbellRegisters; 128],
        MhuPostboxDoorbells,
        DOORBELL_COUNT
    );
    define_fake_regs!(
        FakeMailboxDoorbellsRegisters,
        1024,
        [MhuMailboxDoorbellRegisters; 128],
        MhuMailboxDoorbells,
        DOORBELL_COUNT
    );

    const FIFO_COUNT: usize = 8;
    const FIFO_CONFIG: FfchCfg0 = FfchCfg0::F8BA_SPT
        .union(FfchCfg0::F16BA_SPT)
        .union(FfchCfg0::F32BA_SPT)
        .union(FfchCfg0::F64BA_SPT)
        .union(FfchCfg0::from_bits_retain(FIFO_COUNT as u32 - 1));

    define_fake_regs!(
        FakePostboxFifosRegisters,
        1024,
        [MhuPostboxFifoRegisters; 64],
        MhuPostboxFifos,
        FIFO_CONFIG
    );
    define_fake_regs!(
        FakeMailboxFifosRegisters,
        1024,
        [MhuMailboxFifoRegisters; 64],
        MhuMailboxFifos,
        FIFO_CONFIG
    );

    #[test]
    fn regs_size() {
        assert_eq!(0x1_0000, size_of::<MhuPostboxRegisters>());
        assert_eq!(0x1_0000, size_of::<MhuMailboxRegisters>());
    }

    #[test]
    fn postbox_doorbell() {
        let mut regs = FakePostboxRegisters::new();
        regs.enable_all_features();

        let base = regs.regs.as_ptr() as usize;

        {
            let mut instance = regs.instance_for_test();
            assert!(instance.doorbell(FIFO_COUNT - 1).is_none());

            let doorbell = instance.doorbell(0).unwrap();
            assert_offset!(0x1000, doorbell.regs, base);
        }

        // Set doorbell count to DOORBELL_COUNT
        regs.reg_write(0x20, DOORBELL_COUNT as u32 - 1);

        {
            let mut instance = regs.instance_for_test();
            assert!(
                instance
                    .doorbells()
                    .unwrap()
                    .doorbell(DOORBELL_COUNT)
                    .is_none()
            );

            let mut doorbells = instance.doorbells().unwrap();
            let doorbell = doorbells.doorbell(DOORBELL_COUNT - 1).unwrap();
            assert_offset!(0x1000 + 0x20 * (DOORBELL_COUNT - 1), doorbell.regs, base);
        }
    }

    #[test]
    fn postbox_fifo() {
        let mut regs = FakePostboxRegisters::new();
        regs.enable_all_features();

        let base = regs.regs.as_ptr() as usize;

        {
            let mut instance = regs.instance_for_test();
            assert!(instance.fifo(FIFO_COUNT).is_none());

            let fifo = instance.fifo(0).unwrap();
            assert_offset!(0x2000, fifo.regs, base);
        }

        // Set FIFO count to FIFO_COUNT
        regs.reg_write(0x30, FIFO_COUNT as u32 - 1);

        {
            let mut instance = regs.instance_for_test();
            assert!(instance.fifos().unwrap().fifo(FIFO_COUNT).is_none());

            let mut fifos = instance.fifos().unwrap();
            let fifo = fifos.fifo(FIFO_COUNT - 1).unwrap();
            assert_offset!(0x2000 + 0x40 * (FIFO_COUNT - 1), fifo.regs, base);
        }
    }

    #[test]
    fn postbox_fast_channel() {
        let mut regs = FakePostboxRegisters::new();
        regs.enable_all_features();

        let base = regs.regs.as_ptr() as usize;

        // Set fast channel width to 32 bits
        regs.reg_write(0x40, 32 << 21);

        {
            let mut instance = regs.instance_for_test();
            let Some(MhuPostboxFastChannel::FastChannel32(fast_channel)) = instance.fast_channel()
            else {
                panic!("Invalid fast channel type")
            };

            assert_offset!(0x3000, fast_channel.regs, base);
        }
    }

    #[test]
    fn postbox_split() {
        let mut regs = FakePostboxRegisters::new();
        regs.enable_all_features();

        regs.reg_write(0x020, 4 - 1); // FIFO channel count = 4
        regs.reg_write(0x040, 32 << 21); // 32-bit Fast channel
        regs.reg_write(0x1000, 0xabcd_ef01);

        let base = regs.regs.as_ptr() as usize;

        {
            let instance = regs.instance_for_test();

            let (control, doorbells, fifos, fast_channel) = instance.split();

            assert_offset!(0x0000, control.regs, base);
            assert_offset!(0x1000, doorbells.unwrap().regs, base);
            assert_offset!(0x2000, fifos.unwrap().regs, base);

            let Some(MhuPostboxFastChannel::FastChannel32(fc)) = fast_channel else {
                panic!("Invalid fast channel type");
            };
            assert_offset!(0x3000, fc.regs, base);
        }

        regs.reg_write(0x040, 64 << 21); // 64-bit Fast channel

        {
            let instance = regs.instance_for_test();

            let (control, doorbells, fifos, fast_channel) = instance.split();

            assert_offset!(0x0000, control.regs, base);
            assert_offset!(0x1000, doorbells.unwrap().regs, base);
            assert_offset!(0x2000, fifos.unwrap().regs, base);

            let Some(MhuPostboxFastChannel::FastChannel64(fc)) = fast_channel else {
                panic!("Invalid fast channel type");
            };
            assert_offset!(0x3000, fc.regs, base);
        }
    }

    #[test]
    fn postbox_doorbells() {
        let mut regs = FakePostboxDoorbellsRegisters::new();
        let base = regs.regs.as_ptr() as usize;

        {
            let mut doorbells = regs.instance_for_test();
            assert_offset!(0x0000, doorbells.doorbell(0).unwrap().regs, base);

            assert_offset!(0x0020, doorbells.doorbell(1).unwrap().regs, base);

            assert_offset!(
                0x0020 * (DOORBELL_COUNT - 1),
                doorbells.doorbell(DOORBELL_COUNT - 1).unwrap().regs,
                base
            );

            assert!(doorbells.doorbell(DOORBELL_COUNT).is_none());
        }

        {
            let doorbells = regs.instance_for_test();

            assert_offset!(
                0x20 * (DOORBELL_COUNT - 1),
                doorbells.take(DOORBELL_COUNT - 1).unwrap().regs,
                base
            );
        }

        {
            let doorbells = regs.instance_for_test();

            assert!(doorbells.take(DOORBELL_COUNT).is_none());
        }

        {
            let doorbells = regs.instance_for_test();

            let [a, b, c] = doorbells.split_some([0, 1, 3]);

            assert_offset!(0x0000, a.regs, base);
            assert_offset!(0x0020, b.regs, base);
            assert_offset!(0x0060, c.regs, base);
        }
    }

    #[test]
    fn postbox_fifos() {
        let mut regs = FakePostboxFifosRegisters::new();
        let base = regs.regs.as_ptr() as usize;

        {
            let mut fifos = regs.instance_for_test();
            assert_offset!(0x0000, fifos.fifo(0).unwrap().regs, base);

            assert_offset!(0x0040, fifos.fifo(1).unwrap().regs, base);

            assert_offset!(
                0x0040 * (FIFO_COUNT - 1),
                fifos.fifo(FIFO_COUNT - 1).unwrap().regs,
                base
            );

            assert!(fifos.fifo(FIFO_COUNT).is_none());
        }

        {
            let fifos = regs.instance_for_test();

            assert_offset!(
                0x40 * (FIFO_COUNT - 1),
                fifos.take(FIFO_COUNT - 1).unwrap().regs,
                base
            );
        }

        {
            let fifos = regs.instance_for_test();

            assert!(fifos.take(FIFO_COUNT).is_none());
        }

        {
            let fifos = regs.instance_for_test();

            let [a, b, c] = fifos.split_some([0, 1, 3]);

            assert_offset!(0x0000, a.regs, base);
            assert_offset!(0x0040, b.regs, base);
            assert_offset!(0x00c0, c.regs, base);
        }
    }

    #[test]
    fn postbox_not_supported() {
        let mut regs = FakePostboxRegisters::new();

        let mut instance = regs.instance_for_test();
        assert!(instance.doorbells().is_none());
        assert!(instance.fifos().is_none());
        assert!(instance.fast_channel().is_none());
    }

    #[test]
    fn mailbox_doorbell() {
        let mut regs = FakeMailboxRegisters::new();
        regs.enable_all_features();

        let base = regs.regs.as_ptr() as usize;

        {
            let mut instance = regs.instance_for_test();
            assert!(instance.doorbell(1).is_none());

            let doorbell = instance.doorbell(0).unwrap();
            assert_offset!(0x1000, doorbell.regs, base);
        }

        // Set doorbell count to DOORBELL_COUNT
        regs.reg_write(0x20, DOORBELL_COUNT as u32 - 1);

        {
            let mut instance = regs.instance_for_test();
            assert!(
                instance
                    .doorbells()
                    .unwrap()
                    .doorbell(DOORBELL_COUNT)
                    .is_none()
            );

            let mut doorbells = instance.doorbells().unwrap();
            let doorbell = doorbells.doorbell(DOORBELL_COUNT - 1).unwrap();
            assert_offset!(0x1000 + 0x20 * (DOORBELL_COUNT - 1), doorbell.regs, base);
        }
    }

    #[test]
    fn mailbox_fifo() {
        let mut regs = FakeMailboxRegisters::new();
        regs.enable_all_features();

        let base = regs.regs.as_ptr() as usize;

        {
            let mut instance = regs.instance_for_test();
            assert!(instance.fifo(1).is_none());

            let fifo = instance.fifo(0).unwrap();
            assert_offset!(0x2000, fifo.regs, base);
        }

        // Set FIFO count to FIFO_COUNT
        regs.reg_write(0x30, FIFO_COUNT as u32 - 1);

        {
            let mut instance = regs.instance_for_test();
            assert!(instance.fifos().unwrap().fifo(FIFO_COUNT).is_none());

            let mut fifos = instance.fifos().unwrap();
            let fifo = fifos.fifo(FIFO_COUNT - 1).unwrap();
            assert_offset!(0x2000 + 0x40 * (FIFO_COUNT - 1), fifo.regs, base);
        }
    }

    #[test]
    fn mailbox_fast_channel() {
        let mut regs = FakeMailboxRegisters::new();
        regs.enable_all_features();

        let base = regs.regs.as_ptr() as usize;

        // Set fast channel width to 32 bits
        regs.reg_write(0x40, 32 << 21);

        {
            let mut instance = regs.instance_for_test();
            let Some(MhuMailboxFastChannel::FastChannel32(fast_channel)) = instance.fast_channel()
            else {
                panic!("Invalid fast channel type")
            };

            assert_offset!(0x3000, fast_channel.regs, base);
        }
    }

    #[test]
    fn mailbox_split() {
        let mut regs = FakeMailboxRegisters::new();
        regs.enable_all_features();

        regs.reg_write(0x020, 4 - 1); // FIFO channel count = 4
        regs.reg_write(0x040, 32 << 21); // 32-bit Fast channel
        regs.reg_write(0x1000, 0xabcd_ef01);

        let base = regs.regs.as_ptr() as usize;

        {
            let instance = regs.instance_for_test();

            let (control, doorbells, fifos, fast_channel) = instance.split();

            assert_offset!(0x0000, control.regs, base);
            assert_offset!(0x1000, doorbells.unwrap().regs, base);
            assert_offset!(0x2000, fifos.unwrap().regs, base);

            let Some(MhuMailboxFastChannel::FastChannel32(fc)) = fast_channel else {
                panic!("Invalid fast channel type");
            };
            assert_offset!(0x3000, fc.regs, base);
        }

        regs.reg_write(0x040, 64 << 21); // 64-bit Fast channel

        {
            let instance = regs.instance_for_test();

            let (control, doorbells, fifos, fast_channel) = instance.split();

            assert_offset!(0x0000, control.regs, base);
            assert_offset!(0x1000, doorbells.unwrap().regs, base);
            assert_offset!(0x2000, fifos.unwrap().regs, base);

            let Some(MhuMailboxFastChannel::FastChannel64(fc)) = fast_channel else {
                panic!("Invalid fast channel type");
            };
            assert_offset!(0x3000, fc.regs, base);
        }
    }

    #[test]
    fn mailbox_doorbells() {
        let mut regs = FakeMailboxDoorbellsRegisters::new();
        let base = regs.regs.as_ptr() as usize;

        {
            let mut doorbells = regs.instance_for_test();
            assert_offset!(0x0000, doorbells.doorbell(0).unwrap().regs, base);

            assert_offset!(0x0020, doorbells.doorbell(1).unwrap().regs, base);

            assert_offset!(
                0x0020 * (DOORBELL_COUNT - 1),
                doorbells.doorbell(DOORBELL_COUNT - 1).unwrap().regs,
                base
            );

            assert!(doorbells.doorbell(DOORBELL_COUNT).is_none());
        }

        {
            let doorbells = regs.instance_for_test();

            assert_offset!(
                0x20 * (DOORBELL_COUNT - 1),
                doorbells.take(DOORBELL_COUNT - 1).unwrap().regs,
                base
            );
        }

        {
            let doorbells = regs.instance_for_test();

            assert!(doorbells.take(DOORBELL_COUNT).is_none());
        }

        {
            let doorbells = regs.instance_for_test();

            let [a, b, c] = doorbells.split_some([0, 1, 3]);

            assert_offset!(0x0000, a.regs, base);
            assert_offset!(0x0020, b.regs, base);
            assert_offset!(0x0060, c.regs, base);
        }
    }

    #[test]
    fn mailbox_fifos() {
        let mut regs = FakeMailboxFifosRegisters::new();
        let base = regs.regs.as_ptr() as usize;

        {
            let mut fifos = regs.instance_for_test();
            assert_offset!(0x0000, fifos.fifo(0).unwrap().regs, base);

            assert_offset!(0x0040, fifos.fifo(1).unwrap().regs, base);

            assert_offset!(
                0x0040 * (FIFO_COUNT - 1),
                fifos.fifo(FIFO_COUNT - 1).unwrap().regs,
                base
            );

            assert!(fifos.fifo(FIFO_COUNT).is_none());
        }

        {
            let fifos = regs.instance_for_test();

            assert_offset!(
                0x40 * (FIFO_COUNT - 1),
                fifos.take(FIFO_COUNT - 1).unwrap().regs,
                base
            );
        }

        {
            let fifos = regs.instance_for_test();

            assert!(fifos.take(FIFO_COUNT).is_none());
        }

        {
            let fifos = regs.instance_for_test();

            let [a, b, c] = fifos.split_some([0, 1, 3]);

            assert_offset!(0x0000, a.regs, base);
            assert_offset!(0x0040, b.regs, base);
            assert_offset!(0x00c0, c.regs, base);
        }
    }

    #[test]
    fn mailbox_not_supported() {
        let mut regs = FakeMailboxRegisters::new();

        let mut instance = regs.instance_for_test();
        assert!(instance.doorbells().is_none());
        assert!(instance.fifos().is_none());
        assert!(instance.fast_channel().is_none());
    }
}
