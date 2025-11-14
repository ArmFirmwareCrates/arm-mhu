// SPDX-FileCopyrightText: Copyright The arm-mhu Contributors.
// SPDX-License-Identifier: MIT OR Apache-2.0

use bitflags::bitflags;
use safe_mmio::{
    UniqueMmioPointer, field, field_shared,
    fields::{ReadPure, ReadPureWrite, WriteOnly},
};
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout};

/// Postbox Doorbell interrupt register value.
#[derive(Clone, Eq, FromBytes, Immutable, IntoBytes, KnownLayout, PartialEq)]
#[repr(transparent)]
pub struct DoorbellInterrupt(u32);

bitflags! {
    impl DoorbellInterrupt: u32 {
        /// Transfer Acknowledge
        const TFR_ACK = 1 << 0;
    }
}

/// Postbox/Mailbox control register value.
#[derive(Clone, Eq, FromBytes, Immutable, IntoBytes, KnownLayout, PartialEq)]
#[repr(transparent)]
pub struct DoorbellControl(u32);

bitflags! {
    impl DoorbellControl: u32 {
        /// Doorbell interrupts contribute to the Postbox/Mailbox Combined interrupt.
        const COMB_EN = 1 << 0;
    }
}

/// Postbox doorbell channel window page structure
///
/// See C2.1.1.2 PDBCW_page, Postbox Doorbell Channel Window Page
#[derive(Clone, Eq, FromBytes, Immutable, IntoBytes, KnownLayout, PartialEq)]
#[repr(C)]
pub struct MhuPostboxDoorbellRegisters {
    // 0x00: Postbox Doorbell Channel Window Status
    pdbcw_st: ReadPure<u32>,
    /// 0x04 - 0x08
    reserved_4: [u32; 2],
    /// 0x0c: Postbox Doorbell Channel Window Set
    pdbcw_set: WriteOnly<u32>,
    /// 0x10: Postbox Doorbell Channel Window Interrupt Status
    pdbcw_int_st: ReadPure<DoorbellInterrupt>,
    /// 0x14: Postbox Doorbell Channel Window Interrupt Clear
    pdbcw_int_clr: WriteOnly<DoorbellInterrupt>,
    /// 0x18: Postbox Doorbell Channel Window Interrupt Enable
    pdbcw_int_en: ReadPureWrite<DoorbellInterrupt>,
    /// 0x1C: Postbox Doorbell Channel Window Control
    pdbcw_ctrl: ReadPureWrite<DoorbellControl>,
}

/// Mailbox doorbell channel window page structure
///
/// See C2.2.1.2 MDBCW_page, Mailbox Doorbell Channel Windows Page.
#[derive(Clone, Eq, FromBytes, Immutable, IntoBytes, KnownLayout, PartialEq)]
#[repr(C)]
pub struct MhuMailboxDoorbellRegisters {
    /// 0x00: Mailbox Doorbell Channel Window Status
    mdbcw_st: ReadPure<u32>,
    /// 0x04: Mailbox Doorbell Channel Window Status Masked
    mdbcw_st_msk: ReadPure<u32>,
    /// 0x08: Mailbox Doorbell Channel Window Clear
    mdbcw_clr: WriteOnly<u32>,
    /// 0x0C
    reserved_c: u32,
    /// 0x10: Mailbox Doorbell Channel Window Mask Status
    mdbcw_msk_st: ReadPure<u32>,
    /// 0x14: Mailbox Doorbell Channel Window Mask Set
    mdbcw_msk_set: WriteOnly<u32>,
    /// 0x18: Mailbox Doorbell Channel Window Mask Clear
    mdbcw_msk_clr: WriteOnly<u32>,
    /// 0x1C: Mailbox Doorbell Channel Window Control
    mdbcw_ctrl: ReadPureWrite<DoorbellControl>,
}

/// Postbox Doorbell Channel driver.
pub struct MhuPostboxDoorbell<'a> {
    regs: UniqueMmioPointer<'a, MhuPostboxDoorbellRegisters>,
}

impl<'a> MhuPostboxDoorbell<'a> {
    /// Creates new instance.
    pub fn new(regs: UniqueMmioPointer<'a, MhuPostboxDoorbellRegisters>) -> Self {
        Self { regs }
    }

    /// Enables/disables TFR_ACK interrupt and the doorbell channel interrupts to contribute to the
    /// Postbox Combined interrupt.
    pub fn enable_interrupt(&mut self, enable: bool) {
        if enable {
            field!(self.regs, pdbcw_int_en).write(DoorbellInterrupt::TFR_ACK);
            field!(self.regs, pdbcw_ctrl).write(DoorbellControl::COMB_EN);
        } else {
            field!(self.regs, pdbcw_int_en).write(DoorbellInterrupt::empty());
            field!(self.regs, pdbcw_int_clr).write(DoorbellInterrupt::TFR_ACK);
            field!(self.regs, pdbcw_ctrl).write(DoorbellControl::empty());
        }
    }

    /// Clears TFR_ACK interrupt.
    pub fn clear_interrupt(&mut self) {
        field!(self.regs, pdbcw_int_clr).write(DoorbellInterrupt::TFR_ACK);
    }

    /// Returns flags of the doorbell.
    pub fn flags(&self) -> u32 {
        field_shared!(self.regs, pdbcw_st).read()
    }

    /// Sets the flags of the doorbell.
    pub fn set_flags(&mut self, flags: u32) {
        field!(self.regs, pdbcw_set).write(flags);
    }
}

/// Mailbox Doorbell Channel driver.
pub struct MhuMailboxDoorbell<'a> {
    regs: UniqueMmioPointer<'a, MhuMailboxDoorbellRegisters>,
}

impl<'a> MhuMailboxDoorbell<'a> {
    /// Creates new instance.
    pub fn new(regs: UniqueMmioPointer<'a, MhuMailboxDoorbellRegisters>) -> Self {
        Self { regs }
    }

    /// Enables/disables doorbell channel interrupts to contribute to the Mailbox Combined
    /// interrupt.
    pub fn enable_interrupt(&mut self, enable: bool) {
        let value = if enable {
            DoorbellControl::COMB_EN
        } else {
            DoorbellControl::empty()
        };

        field!(self.regs, mdbcw_ctrl).write(value);
    }

    /// Returns the mask of the doorbell.
    pub fn mask(&self) -> u32 {
        field_shared!(self.regs, mdbcw_msk_st).read()
    }

    /// Sets bits in the mask of the doorbell.
    pub fn set_mask(&mut self, mask: u32) {
        field!(self.regs, mdbcw_msk_set).write(mask);
    }

    /// Clears bits in the mask of the doorbell.
    pub fn clear_mask(&mut self, mask: u32) {
        field!(self.regs, mdbcw_msk_clr).write(mask);
    }

    /// Returns flags of the doorbell.
    pub fn flags(&self) -> u32 {
        field_shared!(self.regs, mdbcw_st).read()
    }

    /// Clears flags of the doorbell.
    pub fn clear_flags(&mut self, flags: u32) {
        field!(self.regs, mdbcw_clr).write(flags);
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

    /// Splits Doorbells into individual Doorbell objects. Returns `None` item in the array if the
    /// channel is not implemented.
    pub fn split<const N: usize>(mut self) -> [Option<MhuPostboxDoorbell<'a>>; N] {
        const { assert!(N <= 128) };

        core::array::from_fn(|channel| {
            if channel < self.count {
                let item_pointer = self.regs.get(channel).unwrap().ptr_mut();

                // Safety: `split_child` is called only once on each item and the original
                // `UniqueMmioPointer` is consumed by this function.
                let doorbell_regs = unsafe {
                    self.regs
                        .split_child(core::ptr::NonNull::new(item_pointer).unwrap())
                };

                Some(MhuPostboxDoorbell::new(doorbell_regs))
            } else {
                None
            }
        })
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

    /// Splits Doorbells into individual Doorbell objects. Returns `None` item in the array if the
    /// channel is not implemented.
    pub fn split<const N: usize>(mut self) -> [Option<MhuMailboxDoorbell<'a>>; N] {
        const { assert!(N <= 128) };

        core::array::from_fn(|channel| {
            if channel < self.count {
                let item_pointer = self.regs.get(channel).unwrap().ptr_mut();

                // Safety: `split_child` is called only once on each item and the original
                // `UniqueMmioPointer` is consumed by this function.
                let doorbell_regs = unsafe {
                    self.regs
                        .split_child(core::ptr::NonNull::new(item_pointer).unwrap())
                };

                Some(MhuMailboxDoorbell::new(doorbell_regs))
            } else {
                None
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use core::ptr::NonNull;

    use super::*;
    use crate::tests::define_fake_regs;
    use zerocopy::transmute_mut;

    define_fake_regs!(
        FakePostboxRegisters,
        8,
        MhuPostboxDoorbellRegisters,
        MhuPostboxDoorbell
    );

    define_fake_regs!(
        FakeMailboxRegisters,
        8,
        MhuMailboxDoorbellRegisters,
        MhuMailboxDoorbell
    );

    #[test]
    fn regs_size() {
        assert_eq!(0x20, size_of::<MhuPostboxDoorbellRegisters>());
        assert_eq!(0x20, size_of::<MhuMailboxDoorbellRegisters>());
    }

    #[test]
    fn postbox_enable_interrupts() {
        let mut regs = FakePostboxRegisters::new();

        {
            let mut instance = regs.instance_for_test();
            instance.enable_interrupt(true);
        }

        assert_eq!(0x1, regs.reg_read(0x1c));
        assert_eq!(0x1, regs.reg_read(0x18));

        {
            let mut instance = regs.instance_for_test();
            instance.enable_interrupt(false);
        }

        assert_eq!(0x1, regs.reg_read(0x14));
        assert_eq!(0x0, regs.reg_read(0x1c));
        assert_eq!(0x0, regs.reg_read(0x18));
    }

    #[test]
    fn postbox_clear_interrupt() {
        let mut regs = FakePostboxRegisters::new();

        {
            let mut instance = regs.instance_for_test();
            instance.clear_interrupt();
        }

        assert_eq!(0x1, regs.reg_read(0x14));
    }

    #[test]
    fn postbox_flags() {
        let mut regs = FakePostboxRegisters::new();

        {
            let mut instance = regs.instance_for_test();
            instance.set_flags(0xabcd1234);
        }

        assert_eq!(0xabcd1234, regs.reg_read(0xc));

        regs.clear();
        regs.reg_write(0x0, 0x56789ef);

        let instance = regs.instance_for_test();
        assert_eq!(0x56789ef, instance.flags());
    }

    #[test]
    fn mailbox_enable_interrupts() {
        let mut regs = FakeMailboxRegisters::new();

        {
            let mut instance = regs.instance_for_test();
            instance.enable_interrupt(true);
        }

        assert_eq!(0x1, regs.reg_read(0x1c));

        {
            let mut instance = regs.instance_for_test();
            instance.enable_interrupt(false);
        }

        assert_eq!(0x0, regs.reg_read(0x1c));
    }

    #[test]
    fn mailbox_mask() {
        let mut regs = FakeMailboxRegisters::new();

        {
            let mut instance = regs.instance_for_test();
            instance.set_mask(0xabcdef01);
        }

        assert_eq!(0xabcdef01, regs.reg_read(0x14));
        regs.clear();

        {
            let mut instance = regs.instance_for_test();
            instance.clear_mask(0xabcdef01);
        }

        assert_eq!(0xabcdef01, regs.reg_read(0x18));
        regs.clear();
        regs.reg_write(0x10, 0xabcdef01);

        let instance = regs.instance_for_test();
        assert_eq!(0xabcdef01, instance.mask());
    }

    #[test]
    fn mailbox_flags() {
        let mut regs = FakeMailboxRegisters::new();

        {
            let mut instance = regs.instance_for_test();
            instance.clear_flags(0xabcd1234);
        }

        assert_eq!(0xabcd1234, regs.reg_read(0x8));

        regs.clear();
        regs.reg_write(0x0, 0x56789ef);

        let instance = regs.instance_for_test();
        assert_eq!(0x56789ef, instance.flags());
    }

    #[test]
    fn postbox_doorbells() {
        // Safety: The test only validates the offsets and does not dereference the pointer.
        let doorbell_regs = unsafe {
            UniqueMmioPointer::new(
                NonNull::new(0x2000_0000 as *mut [MhuPostboxDoorbellRegisters; 128]).unwrap(),
            )
        };

        let mut doorbells = MhuPostboxDoorbells::new(doorbell_regs, 64);

        assert_eq!(
            0x2000_0000,
            doorbells.doorbell(0).unwrap().regs.ptr() as usize
        );
        assert_eq!(
            0x2000_0020,
            doorbells.doorbell(1).unwrap().regs.ptr() as usize
        );

        assert!(doorbells.doorbell(64).is_none());
        assert_eq!(0x2000_07e0, doorbells.take(63).unwrap().regs.ptr() as usize);

        // Safety: The test only validates the offsets and does not dereference the pointer.
        let doorbell_regs = unsafe {
            UniqueMmioPointer::new(
                NonNull::new(0x2000_0000 as *mut [MhuPostboxDoorbellRegisters; 128]).unwrap(),
            )
        };

        let doorbells = MhuPostboxDoorbells::new(doorbell_regs, 2);
        assert!(doorbells.take(64).is_none());

        // Safety: The test only validates the offsets and does not dereference the pointer.
        let doorbell_regs = unsafe {
            UniqueMmioPointer::new(
                NonNull::new(0x2000_0000 as *mut [MhuPostboxDoorbellRegisters; 128]).unwrap(),
            )
        };

        let doorbells = MhuPostboxDoorbells::new(doorbell_regs, 2);

        let [a, b, c] = doorbells.split();

        assert_eq!(0x2000_0000, a.unwrap().regs.ptr() as usize);
        assert_eq!(0x2000_0020, b.unwrap().regs.ptr() as usize);
        assert!(c.is_none());
    }

    #[test]
    fn mailbox_doorbells() {
        // Safety: The test only validates the offsets and does not dereference the pointer.
        let doorbell_regs = unsafe {
            UniqueMmioPointer::new(
                NonNull::new(0x2000_0000 as *mut [MhuMailboxDoorbellRegisters; 128]).unwrap(),
            )
        };

        let mut doorbells = MhuMailboxDoorbells::new(doorbell_regs, 64);

        assert_eq!(
            0x2000_0000,
            doorbells.doorbell(0).unwrap().regs.ptr() as usize
        );
        assert_eq!(
            0x2000_0020,
            doorbells.doorbell(1).unwrap().regs.ptr() as usize
        );

        assert!(doorbells.doorbell(64).is_none());
        assert_eq!(0x2000_07e0, doorbells.take(63).unwrap().regs.ptr() as usize);

        // Safety: The test only validates the offsets and does not dereference the pointer.
        let doorbell_regs = unsafe {
            UniqueMmioPointer::new(
                NonNull::new(0x2000_0000 as *mut [MhuMailboxDoorbellRegisters; 128]).unwrap(),
            )
        };

        let doorbells = MhuMailboxDoorbells::new(doorbell_regs, 2);
        assert!(doorbells.take(64).is_none());

        // Safety: The test only validates the offsets and does not dereference the pointer.
        let doorbell_regs = unsafe {
            UniqueMmioPointer::new(
                NonNull::new(0x2000_0000 as *mut [MhuMailboxDoorbellRegisters; 128]).unwrap(),
            )
        };

        let doorbells = MhuMailboxDoorbells::new(doorbell_regs, 2);

        let [a, b, c] = doorbells.split();

        assert_eq!(0x2000_0000, a.unwrap().regs.ptr() as usize);
        assert_eq!(0x2000_0020, b.unwrap().regs.ptr() as usize);
        assert!(c.is_none());
    }
}
