// SPDX-FileCopyrightText: Copyright The arm-mhu Contributors.
// SPDX-License-Identifier: MIT OR Apache-2.0

/// Control page driver model.
pub mod control;
/// Doorbell driver module.
pub mod doorbell;

use crate::mhu_v3::{
    control::{
        MhuMailboxControl, MhuMailboxControlRegisters, MhuPostboxControl,
        MhuPostboxControlRegisters,
    },
    doorbell::{
        MhuMailboxDoorbell, MhuMailboxDoorbellRegisters, MhuMailboxDoorbells, MhuPostboxDoorbell,
        MhuPostboxDoorbellRegisters, MhuPostboxDoorbells,
    },
};
use safe_mmio::{UniqueMmioPointer, field, split_fields};
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout};

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
    pffcw_page: [u32; 1024],
    /// Postbox Fast Channel Windows Page
    pfcw_page: [u32; 1024],
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
    mffcw_page: [u32; 1024],
    /// Mailbox Fast Channel Windows Page
    mfcw_page: [u32; 1024],
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

    /// Returns Doorbell Channel instance.
    pub fn doorbells(&mut self) -> MhuPostboxDoorbells<'_> {
        let count = self.control().doorbell_config().channel_count();
        MhuPostboxDoorbells::new(field!(self.regs, pdbcw_page), count)
    }

    /// Returns Doorbell Channel or None if the channel is not implemented.
    pub fn doorbell<'regs>(&'regs mut self, channel: usize) -> Option<MhuPostboxDoorbell<'regs>> {
        if channel < self.control().doorbell_config().channel_count() {
            Some(MhuPostboxDoorbell::new(
                field!(self.regs, pdbcw_page).take(channel).unwrap(),
            ))
        } else {
            None
        }
    }

    /// Splits Postbox into Control block and Doorbell units.
    pub fn split(self) -> (MhuPostboxControl<'a>, MhuPostboxDoorbells<'a>) {
        // Safety: Each field name is only passed once.
        let (control_regs, doorbell_regs) =
            unsafe { split_fields!(self.regs, pbx_ctrl_page, pdbcw_page) };

        let control = MhuPostboxControl::new(control_regs);
        let doorbells =
            MhuPostboxDoorbells::new(doorbell_regs, control.doorbell_config().channel_count());

        (control, doorbells)
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

    /// Returns Doorbell Channel instance.
    pub fn doorbells(&mut self) -> MhuMailboxDoorbells<'_> {
        let count = self.control().doorbell_config().channel_count();
        MhuMailboxDoorbells::new(field!(self.regs, mdbcw_page), count)
    }

    /// Returns Doorbell Channel or None if the channel is not implemented.
    pub fn doorbell<'regs>(&'regs mut self, channel: usize) -> Option<MhuMailboxDoorbell<'regs>> {
        if channel < self.control().doorbell_config().channel_count() {
            Some(MhuMailboxDoorbell::new(
                field!(self.regs, mdbcw_page).take(channel).unwrap(),
            ))
        } else {
            None
        }
    }

    /// Splits Mailbox into Control block and Doorbell units.
    pub fn split(self) -> (MhuMailboxControl<'a>, MhuMailboxDoorbells<'a>) {
        // Safety: Each field name is only passed once.
        let (control_regs, doorbell_regs) =
            unsafe { split_fields!(self.regs, mbx_ctrl_page, mdbcw_page) };

        let control = MhuMailboxControl::new(control_regs);
        let doorbells =
            MhuMailboxDoorbells::new(doorbell_regs, control.doorbell_config().channel_count());

        (control, doorbells)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::define_fake_regs;
    use zerocopy::transmute_mut;

    define_fake_regs!(FakePostboxRegisters, 16384, MhuPostboxRegisters, MhuPostbox);
    define_fake_regs!(FakeMailboxRegisters, 16384, MhuMailboxRegisters, MhuMailbox);

    #[test]
    fn regs_size() {
        assert_eq!(0x1_0000, size_of::<MhuPostboxRegisters>());
        assert_eq!(0x1_0000, size_of::<MhuMailboxRegisters>());
    }

    #[test]
    fn postbox_doorbell() {
        let mut regs = FakePostboxRegisters::new();

        {
            let mut instance = regs.instance_for_test();
            assert!(instance.doorbell(1).is_none());

            let mut doorbell = instance.doorbell(0).unwrap();
            doorbell.set_flags(0x1234678);
        }

        assert_eq!(0x1234678, regs.reg_read(0x100c));
        regs.clear();

        // Set doorbell count to 32
        regs.reg_write(0x20, 32 - 1);

        {
            let mut instance = regs.instance_for_test();
            assert!(instance.doorbells().doorbell(32).is_none());

            let mut doorbells = instance.doorbells();
            let mut doorbell = doorbells.doorbell(31).unwrap();
            doorbell.set_flags(0x1234678);
        }

        assert_eq!(0x1234678, regs.reg_read(0x13ec));
    }

    #[test]
    fn postbox_split() {
        let mut regs = FakePostboxRegisters::new();
        regs.reg_write(0x020, 4 - 1);
        regs.reg_write(0x1000, 0xabcd_ef01);

        let instance = regs.instance_for_test();

        let (control, mut doorbells) = instance.split();
        assert_eq!(4, control.doorbell_config().channel_count());
        assert_eq!(0xabcd_ef01, doorbells.doorbell(0).unwrap().flags());
        assert!(doorbells.doorbell(4).is_none());
    }

    #[test]
    fn mailbox_doorbell() {
        let mut regs = FakeMailboxRegisters::new();

        {
            let mut instance = regs.instance_for_test();
            assert!(instance.doorbell(1).is_none());

            let mut doorbell = instance.doorbell(0).unwrap();
            doorbell.clear_flags(0x1234678);
        }

        assert_eq!(0x1234678, regs.reg_read(0x1008));
        regs.clear();

        // Set doorbell count to 32
        regs.reg_write(0x20, 32 - 1);

        {
            let mut instance = regs.instance_for_test();
            assert!(instance.doorbells().doorbell(32).is_none());

            let mut doorbells = instance.doorbells();
            let mut doorbell = doorbells.doorbell(31).unwrap();
            doorbell.clear_flags(0x1234678);
        }

        assert_eq!(0x1234678, regs.reg_read(0x13e8));
    }

    #[test]
    fn mailbox_split() {
        let mut regs = FakeMailboxRegisters::new();
        regs.reg_write(0x020, 4 - 1);
        regs.reg_write(0x1000, 0xabcd_ef01);

        let instance = regs.instance_for_test();

        let (control, mut doorbells) = instance.split();
        assert_eq!(4, control.doorbell_config().channel_count());
        assert_eq!(0xabcd_ef01, doorbells.doorbell(0).unwrap().flags());
        assert!(doorbells.doorbell(4).is_none());
    }
}
