// SPDX-FileCopyrightText: Copyright The arm-mhu Contributors.
// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::Error;
use bitflags::bitflags;
use safe_mmio::{
    UniqueMmioPointer, field, field_shared,
    fields::{ReadPure, ReadPureWrite},
};
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout};

/// Postbox/Mailbox Control register value.
#[derive(Clone, Eq, FromBytes, Immutable, IntoBytes, KnownLayout, PartialEq)]
#[repr(transparent)]
struct Ctrl(u32);

bitflags! {
    impl Ctrl: u32 {
        /// Operational Request. Controls whether the MHUS is required to remain in an operational
        /// state.
        const OP_REQ = 1 << 0;

        /// Channel Operational Mask. Controls whether channels need to be idle to allow a
        /// controlled entry of the MHUS into a non-operational state.
        const CH_OP_MSK = 1 << 1;
    }
}

/// Postbox/Mailbox Doorbell Channel Configuration 0 register value.
#[derive(Clone, Eq, FromBytes, Immutable, IntoBytes, KnownLayout, PartialEq)]
#[repr(transparent)]
pub struct DbchCfg0(u32);

impl DbchCfg0 {
    /// Returns the number of Doorbell Channels.
    pub fn channel_count(&self) -> usize {
        ((self.0 & 0x7f) + 1) as usize
    }
}

/// Postbox/Mailbox FIFO Channel Configuration 0 register value.
#[derive(Clone, Eq, FromBytes, Immutable, IntoBytes, KnownLayout, PartialEq)]
#[repr(transparent)]
pub struct FfchCfg0(u32);

bitflags! {
    impl FfchCfg0: u32 {
        /// 8-bit Access Support
        const P8BA_SPT = 1 << 8;
        /// 16-bit Access Support
        const P16BA_SPT = 1 << 9;
        /// 32-bit Access Support
        const P32BA_SPT = 1 << 10;
        /// 64-bit Access Support
        const P64BA_SPT = 1 << 11;
    }
}

impl FfchCfg0 {
    /// Returns the number of FIFO Channels.
    pub fn channel_count(&self) -> usize {
        ((self.0 & 0x3f) + 1) as usize
    }

    /// Returns the FIFO Channel depth.
    pub fn depth(&self) -> usize {
        (((self.0 >> 16) & 0x3ff) + 1) as usize
    }
}

/// Postbox/Mailbox Fast Channel Configuration 0 register value.
#[derive(Clone, Eq, FromBytes, Immutable, IntoBytes, KnownLayout, PartialEq)]
#[repr(transparent)]
pub struct FchCfg0(u32);

bitflags! {
    impl FchCfg0: u32 {
        /// Mailbox only. Fast Channel Group Interrupt Support. Indicates whether Fast Channel Group
        /// Transfer interrupt is implemented for each FCGs.
        const FCGI_SPT = 1 << 10;
    }
}

impl FchCfg0 {
    /// Returns the number of Fast Channels.
    pub fn channel_count(&self) -> usize {
        ((self.0 & 0x3ff) + 1) as usize
    }

    /// Returns the number of Fast Channel Groups.
    pub fn group_count(&self) -> usize {
        (((self.0 >> 11) & 0x1f) + 1) as usize
    }

    /// Returns number of Fast Channels per Fast Channel Group
    pub fn channels_per_group(&self) -> usize {
        (((self.0 >> 16) & 0x1f) + 1) as usize
    }

    /// Returns the Fast Channel Word-Size.
    pub fn word_size(&self) -> usize {
        ((self.0 >> 21) & 0xff) as usize
    }
}

/// Postbox/Mailbox Architecture Identification Register value.
#[derive(Clone, Eq, FromBytes, Immutable, IntoBytes, KnownLayout, PartialEq)]
#[repr(transparent)]
pub struct Aidr(u32);

impl Aidr {
    /// Returns the MHU Architecture Major Revision.
    pub fn arch_major_rev(&self) -> u8 {
        (((self.0 >> 4) & 0xf) + 1) as u8
    }

    /// Returns the MHU Architecture Minor Revision.
    pub fn arch_minor_rev(&self) -> u8 {
        (self.0 & 0xf) as u8
    }
}

/// Postbox control page registers.
///
/// See C2.1.1.1 PBX_CTRL_page, Postbox CTRL page.
#[derive(Clone, Eq, FromBytes, Immutable, IntoBytes, KnownLayout, PartialEq)]
#[repr(C)]
pub struct MhuPostboxControlRegisters {
    /// 0x000: Postbox Block Identifier */
    pbx_blk_id: ReadPure<u32>,
    /// 0x004 - 0x00C
    reserved_4: [u32; 3],
    /// 0x010: Postbox Feature Support 0
    pbx_feat_spt0: ReadPure<u32>,
    /// 0x014: Postbox Feature Support 1
    pbx_feat_spt1: ReadPure<u32>,
    /// 0x018 - 0x01C
    reserved_18: [u32; 2],
    /// 0x020: Postbox Doorbell Channel Configuration 0
    pbx_dbch_cfg0: ReadPure<DbchCfg0>,
    /// 0x024 - 0x02C
    reserved_24: [u32; 3],
    /// 0x030: Postbox FIFO Channel Configuration 0
    pbx_ffch_cfg0: ReadPure<FfchCfg0>,
    /// 0x034 - 0x3c
    reserved_34: [u32; 3],
    /// 0x040: Postbox Fast Channel Configuration 0
    pbx_fch_cfg0: ReadPure<FchCfg0>,
    /// 0x044 - 0x0FC
    reserved_44: [u32; 47],
    /// 0x100: Postbox control
    pbx_ctrl: ReadPureWrite<Ctrl>,
    /// 0x104 - 0x3FC
    reserved_104: [u32; 191],
    /// 0x400: Postbox Doorbell Channel Interrupt Status n,
    pbx_dbch_int_st: [ReadPure<u32>; 4],
    /// 0x410: Postbox FIFO Channel Interrupt Status n,
    pbx_ffch_int_st: [ReadPure<u32>; 2],
    /// 0x418 - 0xFC4
    reserved_418: [u32; 748],
    /// 0xFC8: Postbox Implementer Identification Register
    pbx_iidr: ReadPure<u32>,
    /// 0xFCC: Postbox Architecture Identification Register
    pbx_aidr: ReadPure<Aidr>,
    /// 0xFD0: Postbox Implementation Defined Identification
    impl_def_id: [ReadPure<u32>; 12],
}

/// Mailbox control page registers.
///
/// See C2.2.1.1 MBX_CTRL_page, Mailbox Control page.
#[derive(Clone, Eq, FromBytes, Immutable, IntoBytes, KnownLayout, PartialEq)]
#[repr(C)]
pub struct MhuMailboxControlRegisters {
    /// 0x000: Mailbox Block Identifier
    mbx_blk_id: ReadPure<u32>,
    /// 0x004 - 0x00C
    reserved_4: [u32; 3],
    /// 0x010: Mailbox Feature Support 0
    mbx_feat_spt0: ReadPure<u32>,
    /// 0x014: Mailbox Feature Support 1
    mbx_feat_spt1: ReadPure<u32>,
    /// 0x018 - 0x01C
    reserved_18: [u32; 2],
    /// 0x020: Mailbox Doorbell Channel Configuration 0
    mbx_dbch_cfg0: ReadPure<DbchCfg0>,
    /// 0x024 - 0x02C
    reserved_24: [u32; 3],
    /// 0x030; Mailbox FIFO Channel Configuration 0
    mbx_ffch_cfg0: ReadPure<FfchCfg0>,
    /// 0x034 - 0x03C
    reserved_34: [u32; 3],
    /// 0x040: Mailbox Fast Channel Configuration 0
    mbx_fch_cfg0: ReadPure<FchCfg0>,
    /// 0x044: - 0x0fc
    reserved_44: [u32; 47],
    /// 0x100: Mailbox control
    mbx_ctrl: ReadPureWrite<Ctrl>,
    /// 0x104 - 0x13C
    reserved_104: [u32; 15],
    /// 0x140: Mailbox Fast Channel control
    mbx_fch_ctrl: ReadPureWrite<u32>,
    /// 0x144: Mailbox Fast Channel Group Interrupt Enable
    mbx_fcg_int_en: ReadPureWrite<u32>,
    /// 0x148 - 0x400
    reserved_148: [u32; 174],
    /// 0x400: Mailbox Doorbell Channel Interrupt Status n,
    mbx_dbch_int_st: [ReadPure<u32>; 4],
    ///0x410: Mailbox FIFO Channel Interrupt Status n
    mbx_ffch_int_st: [ReadPure<u32>; 2],
    /// 0x418 - 0x46c
    reserved_418: [u32; 22],
    /// 0x470: Mailbox Fast Channel Group Interrupt Status
    mbx_fcg_int_st: ReadPure<u32>,
    /// 0x474 - 0x47C
    reserved_9: [u32; 3],
    /// 0x480: Mailbox Fast Channel Group Interrupt Status,
    mbx_fch_grp_int_st: [ReadPure<u32>; 32],
    /// 0x500 - 0xFC4
    reserved_10: [u32; 690],
    /// 0xFC8: Mailbox Implementer Identification Register
    mbx_iidr: ReadPure<u32>,
    /// 0xFCC: Mailbox Architecture Identification Register
    mbx_aidr: ReadPure<Aidr>,
    /// 0xFD0: Mailbox Implementation Defined Identification
    impl_def_id: [ReadPure<u32>; 12],
}

/// Postbox Doorbell Channel driver.
pub struct MhuPostboxControl<'a> {
    regs: UniqueMmioPointer<'a, MhuPostboxControlRegisters>,
}

impl<'a> MhuPostboxControl<'a> {
    /// Creates new instance.
    pub fn new(regs: UniqueMmioPointer<'a, MhuPostboxControlRegisters>) -> Self {
        Self { regs }
    }

    /// Checks if the the version of the peripherals and enables the postbox if supported.
    pub fn enable(&mut self, enable: bool) -> Result<(), Error> {
        let aidr = self.aidr();
        if aidr.arch_major_rev() != 3 || aidr.arch_minor_rev() != 0 {
            return Err(Error::UnsupportedMhuVersion);
        }

        field!(self.regs, pbx_ctrl).write(if enable { Ctrl::OP_REQ } else { Ctrl::empty() });

        Ok(())
    }

    /// Returns Doorbell Channel configuration.
    pub fn doorbell_config(&self) -> DbchCfg0 {
        field_shared!(self.regs, pbx_dbch_cfg0).read()
    }

    /// Returns FIFO Channel configuration.
    pub fn fifo_config(&self) -> FfchCfg0 {
        field_shared!(self.regs, pbx_ffch_cfg0).read()
    }

    /// Retuns the Fast Channel configuration.
    pub fn fast_channel_config(&self) -> FchCfg0 {
        field_shared!(self.regs, pbx_fch_cfg0).read()
    }

    /// Returns Architecture Identification Register value.
    pub fn aidr(&self) -> Aidr {
        field_shared!(self.regs, pbx_aidr).read()
    }
}

/// Postbox Doorbell Channel driver.
pub struct MhuMailboxControl<'a> {
    regs: UniqueMmioPointer<'a, MhuMailboxControlRegisters>,
}

impl<'a> MhuMailboxControl<'a> {
    /// Creates new instance.
    pub fn new(regs: UniqueMmioPointer<'a, MhuMailboxControlRegisters>) -> Self {
        Self { regs }
    }

    /// Checks if the the version of the peripherals and enables the mailbox if supported.
    pub fn enable(&mut self, enable: bool) -> Result<(), Error> {
        let aidr = self.aidr();
        if aidr.arch_major_rev() != 3 || aidr.arch_minor_rev() != 0 {
            return Err(Error::UnsupportedMhuVersion);
        }

        field!(self.regs, mbx_ctrl).write(if enable { Ctrl::OP_REQ } else { Ctrl::empty() });

        Ok(())
    }

    /// Returns Doorbell Channel configuration.
    pub fn doorbell_config(&self) -> DbchCfg0 {
        field_shared!(self.regs, mbx_dbch_cfg0).read()
    }

    /// Returns FIFO Channel configuration.
    pub fn fifo_config(&self) -> FfchCfg0 {
        field_shared!(self.regs, mbx_ffch_cfg0).read()
    }

    /// Retuns the Fast Channel configuration.
    pub fn fast_channel_config(&self) -> FchCfg0 {
        field_shared!(self.regs, mbx_fch_cfg0).read()
    }

    /// Returns Architecture Identification Register value.
    pub fn aidr(&self) -> Aidr {
        field_shared!(self.regs, mbx_aidr).read()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::define_fake_regs;
    use zerocopy::transmute_mut;

    define_fake_regs!(
        FakePostboxControlRegisters,
        1024,
        MhuPostboxControlRegisters,
        MhuPostboxControl
    );

    define_fake_regs!(
        FakeMailboxControlRegisters,
        1024,
        MhuMailboxControlRegisters,
        MhuMailboxControl
    );

    #[test]
    fn regs_size() {
        assert_eq!(0x1000, size_of::<MhuPostboxControlRegisters>());
        assert_eq!(0x1000, size_of::<MhuMailboxControlRegisters>());
    }

    #[test]
    fn postbox_enable() {
        let mut regs = FakePostboxControlRegisters::new();

        {
            let mut instance = regs.instance_for_test();

            assert_eq!(Err(Error::UnsupportedMhuVersion), instance.enable(true));
        }

        regs.clear();
        regs.reg_write(0xfcc, 0b00100000);

        {
            let mut instance = regs.instance_for_test();

            assert_eq!(Ok(()), instance.enable(true));
        }

        assert_eq!(0x1, regs.reg_read(0x100));

        {
            let mut instance = regs.instance_for_test();

            assert_eq!(Ok(()), instance.enable(false));
        }

        assert_eq!(0x0, regs.reg_read(0x100));
    }

    #[test]
    fn mailbox_enable() {
        let mut regs = FakeMailboxControlRegisters::new();

        {
            let mut instance = regs.instance_for_test();

            assert_eq!(Err(Error::UnsupportedMhuVersion), instance.enable(true));
        }

        regs.clear();
        regs.reg_write(0xfcc, 0b00100000);

        {
            let mut instance = regs.instance_for_test();

            assert_eq!(Ok(()), instance.enable(true));
        }

        assert_eq!(0x1, regs.reg_read(0x100));

        {
            let mut instance = regs.instance_for_test();

            assert_eq!(Ok(()), instance.enable(false));
        }

        assert_eq!(0x0, regs.reg_read(0x100));
    }

    #[test]
    fn postbox_config() {
        let mut regs = FakePostboxControlRegisters::new();

        regs.reg_write(0x020, 0x7f);
        regs.reg_write(0x030, 0x03ff_0a3f);
        regs.reg_write(0x040, 0x041f_fbff);

        let instance = regs.instance_for_test();

        assert_eq!(128, instance.doorbell_config().channel_count());

        let fifo_config = instance.fifo_config();
        assert_eq!(64, fifo_config.channel_count());
        assert!(fifo_config.contains(FfchCfg0::P64BA_SPT | FfchCfg0::P16BA_SPT));
        assert_eq!(1024, fifo_config.depth());

        let fast_channel_config = instance.fast_channel_config();
        assert_eq!(1024, fast_channel_config.channel_count());
        assert_eq!(32, fast_channel_config.group_count());
        assert_eq!(32, fast_channel_config.channels_per_group());
        assert_eq!(32, fast_channel_config.word_size());
    }

    #[test]
    fn mailbox_config() {
        let mut regs = FakeMailboxControlRegisters::new();

        regs.reg_write(0x020, 0x7f);
        regs.reg_write(0x030, 0x03ff_0a3f);
        regs.reg_write(0x040, 0x041f_fbff);

        let instance = regs.instance_for_test();

        assert_eq!(128, instance.doorbell_config().channel_count());

        let fifo_config = instance.fifo_config();
        assert_eq!(64, fifo_config.channel_count());
        assert!(fifo_config.contains(FfchCfg0::P64BA_SPT | FfchCfg0::P16BA_SPT));
        assert_eq!(1024, fifo_config.depth());

        let fast_channel_config = instance.fast_channel_config();
        assert_eq!(1024, fast_channel_config.channel_count());
        assert_eq!(32, fast_channel_config.group_count());
        assert_eq!(32, fast_channel_config.channels_per_group());
        assert_eq!(32, fast_channel_config.word_size());
    }
}
