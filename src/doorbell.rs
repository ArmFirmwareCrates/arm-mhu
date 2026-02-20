// SPDX-FileCopyrightText: Copyright The arm-mhu Contributors.
// SPDX-License-Identifier: MIT OR Apache-2.0

use bitflags::bitflags;
use safe_mmio::{
    UniqueMmioPointer, field, field_shared,
    fields::{ReadPure, ReadPureWrite, WriteOnly},
};
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout};

/// Postbox Doorbell interrupt register value.
#[derive(Clone, Copy, Debug, Eq, FromBytes, Immutable, IntoBytes, KnownLayout, PartialEq)]
#[repr(transparent)]
pub struct DoorbellInterrupt(u32);

bitflags! {
    impl DoorbellInterrupt: u32 {
        /// Transfer Acknowledge
        const TFR_ACK = 1 << 0;
    }
}

/// Postbox/Mailbox control register value.
#[derive(Clone, Copy, Debug, Eq, FromBytes, Immutable, IntoBytes, KnownLayout, PartialEq)]
#[repr(transparent)]
pub struct DoorbellControl(u32);

bitflags! {
    impl DoorbellControl: u32 {
        /// Doorbell interrupts contribute to the Postbox/Mailbox Combined interrupt.
        const COMB_EN = 1 << 0;
    }
}

/// Postbox Doorbell Channel window page structure
///
/// See C2.1.1.2 PDBCW_page, Postbox Doorbell Channel Window Page
#[derive(Clone, Eq, FromBytes, Immutable, IntoBytes, KnownLayout, PartialEq)]
#[repr(C)]
pub struct MhuPostboxDoorbellRegisters {
    /// 0x00: Postbox Doorbell Channel Window Status
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

/// Mailbox Doorbell Channel window page structure
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
    pub(super) regs: UniqueMmioPointer<'a, MhuPostboxDoorbellRegisters>,
}

impl<'a> MhuPostboxDoorbell<'a> {
    /// Creates new instance.
    pub fn new(regs: UniqueMmioPointer<'a, MhuPostboxDoorbellRegisters>) -> Self {
        Self { regs }
    }

    /// Enables/disables interrupts and the doorbell channel interrupts to contribute to the
    /// Postbox Combined interrupt.
    pub fn configure_interrupts(&mut self, interrupts: Option<DoorbellInterrupt>) {
        if let Some(interrupts) = interrupts {
            field!(self.regs, pdbcw_int_en).write(interrupts);
        } else {
            field!(self.regs, pdbcw_int_en).write(DoorbellInterrupt::empty());
            self.clear_interrupts(DoorbellInterrupt::all());
        }

        self.modify_ctlr(|ctlr| ctlr.set(DoorbellControl::COMB_EN, interrupts.is_some()));
    }

    /// Reads the doorbell interrupt status.
    pub fn interrupt_status(&self) -> DoorbellInterrupt {
        field_shared!(self.regs, pdbcw_int_st).read()
    }

    /// Clears the specified doorbell interrupts.
    pub fn clear_interrupts(&mut self, interrupts: DoorbellInterrupt) {
        field!(self.regs, pdbcw_int_clr).write(interrupts);
    }

    /// Returns flags of the doorbell.
    pub fn flags(&self) -> u32 {
        field_shared!(self.regs, pdbcw_st).read()
    }

    /// Sets the flags of the doorbell.
    pub fn set_flags(&mut self, flags: u32) {
        field!(self.regs, pdbcw_set).write(flags);
    }

    /// Updates the control register with a modifier function.
    fn modify_ctlr<F>(&mut self, f: F)
    where
        F: Fn(&mut DoorbellControl),
    {
        let mut ctlr = field_shared!(self.regs, pdbcw_ctrl).read();
        f(&mut ctlr);
        field!(self.regs, pdbcw_ctrl).write(ctlr);
    }
}

/// Mailbox Doorbell Channel driver.
pub struct MhuMailboxDoorbell<'a> {
    pub(super) regs: UniqueMmioPointer<'a, MhuMailboxDoorbellRegisters>,
}

impl<'a> MhuMailboxDoorbell<'a> {
    /// Creates new instance.
    pub fn new(regs: UniqueMmioPointer<'a, MhuMailboxDoorbellRegisters>) -> Self {
        Self { regs }
    }

    /// Enables/disables interrupts and the doorbell channel interrupts to contribute to the
    /// Mailbox Combined interrupt.
    pub fn configure_interrupts(&mut self, interrupts: Option<DoorbellInterrupt>) {
        self.modify_ctlr(|ctlr| ctlr.set(DoorbellControl::COMB_EN, interrupts.is_some()));
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

    /// Updates the control register with a modifier function.
    fn modify_ctlr<F>(&mut self, f: F)
    where
        F: Fn(&mut DoorbellControl),
    {
        let mut ctlr = field_shared!(self.regs, mdbcw_ctrl).read();
        f(&mut ctlr);
        field!(self.regs, mdbcw_ctrl).write(ctlr);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::define_fake_regs;

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
    fn postbox_configure_interrupts() {
        let mut regs = FakePostboxRegisters::new();

        {
            let mut instance = regs.instance_for_test();
            instance.configure_interrupts(Some(DoorbellInterrupt::TFR_ACK));
        }

        assert_eq!(0x1, regs.reg_read(0x1c));
        assert_eq!(0x1, regs.reg_read(0x18));

        {
            let mut instance: MhuPostboxDoorbell<'_> = regs.instance_for_test();
            instance.configure_interrupts(None);
        }

        assert_eq!(0x1, regs.reg_read(0x14));
        assert_eq!(0x0, regs.reg_read(0x1c));
        assert_eq!(0x0, regs.reg_read(0x18));
    }

    #[test]
    fn postbox_clear_status() {
        let mut regs = FakePostboxRegisters::new();
        regs.reg_write(0x10, 0x01);

        let instance = regs.instance_for_test();

        assert_eq!(DoorbellInterrupt::TFR_ACK, instance.interrupt_status());
    }

    #[test]
    fn postbox_clear_interrupt() {
        let mut regs = FakePostboxRegisters::new();

        {
            let mut instance = regs.instance_for_test();
            instance.clear_interrupts(DoorbellInterrupt::all());
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
    fn mailbox_configure_interrupts() {
        let mut regs = FakeMailboxRegisters::new();

        {
            let mut instance = regs.instance_for_test();
            instance.configure_interrupts(Some(DoorbellInterrupt::TFR_ACK));
        }

        assert_eq!(0x1, regs.reg_read(0x1c));

        {
            let mut instance = regs.instance_for_test();
            instance.configure_interrupts(None);
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
}
