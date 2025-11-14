// SPDX-FileCopyrightText: Copyright The arm-mhu Contributors.
// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::Error;
use bitflags::bitflags;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use safe_mmio::{
    UniqueMmioPointer, field, field_shared,
    fields::{ReadPure, ReadPureWrite},
};
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout};

/// RAS extension support status.
#[derive(Clone, Copy, Debug, Eq, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(u32)]
pub enum RasSupport {
    /// MHU does not implement the RAS extension
    NotImplemented = 0b0000,
    /// MHU implements RAS but does not follow recommendations in B10.7
    ImplementedNonCompliant = 0b0010,
    /// MHU implements RAS and follows recommendations in B10.7
    ImplementedCompliant = 0b0011,
}

/// Postbox/Mailbox Feature Support 0 register value.
#[derive(Clone, Copy, Debug, Eq, FromBytes, Immutable, IntoBytes, KnownLayout, PartialEq)]
#[repr(transparent)]
pub struct FeatSpt0(pub(crate) u32);

impl FeatSpt0 {
    const DBE_SPT_SHIFT: u32 = 0;
    const DBE_SPT_MASK: u32 = 0b1111;
    const DBE_SPT_IMPLEMENTED: u32 = 0b0001;

    const FE_SPT_SHIFT: u32 = 4;
    const FE_SPT_MASK: u32 = 0b1111;
    const FE_SPT_IMPLEMENTED: u32 = 0b0001;

    const FCE_SPT_SHIFT: u32 = 8;
    const FCE_SPT_MASK: u32 = 0b1111;
    const FCE_SPT_IMPLEMENTED: u32 = 0b0001;

    const TZE_SPT_SHIFT: u32 = 12;
    const TZE_SPT_MASK: u32 = 0b1111;
    const TZE_SPT_IMPLEMENTED: u32 = 0b0001;

    const RME_SPT_SHIFT: u32 = 16;
    const RME_SPT_MASK: u32 = 0b1111;
    const RME_SPT_IMPLEMENTED: u32 = 0b0001;

    const RASE_SPT_SHIFT: u32 = 20;
    const RASE_SPT_MASK: u32 = 0b1111;

    /// Returns whether Doorbell channels are supported.
    pub fn doorbell_supported(&self) -> bool {
        (self.0 >> Self::DBE_SPT_SHIFT) & Self::DBE_SPT_MASK == Self::DBE_SPT_IMPLEMENTED
    }

    /// Returns whether FIFO channels are supported.
    pub fn fifo_supported(&self) -> bool {
        (self.0 >> Self::FE_SPT_SHIFT) & Self::FE_SPT_MASK == Self::FE_SPT_IMPLEMENTED
    }

    /// Returns whether Fast Channels are supported.
    pub fn fast_channel_supported(&self) -> bool {
        (self.0 >> Self::FCE_SPT_SHIFT) & Self::FCE_SPT_MASK == Self::FCE_SPT_IMPLEMENTED
    }

    /// Returns whether TrustZone is supported.
    pub fn trustzone_supported(&self) -> bool {
        (self.0 >> Self::TZE_SPT_SHIFT) & Self::TZE_SPT_MASK == Self::TZE_SPT_IMPLEMENTED
    }

    /// Returns whether RME is supported.
    pub fn rme_supported(&self) -> bool {
        (self.0 >> Self::RME_SPT_SHIFT) & Self::RME_SPT_MASK == Self::RME_SPT_IMPLEMENTED
    }

    /// Returns the RAS extension support status.
    pub fn ras_support(&self) -> RasSupport {
        let bits = (self.0 >> Self::RASE_SPT_SHIFT) & Self::RASE_SPT_MASK;
        bits.try_into().unwrap()
    }
}

/// Postbox/Mailbox Feature Support 1 register value.
#[derive(Clone, Copy, Debug, Eq, FromBytes, Immutable, IntoBytes, KnownLayout, PartialEq)]
#[repr(transparent)]
pub struct FeatSpt1(pub(crate) u32);

impl FeatSpt1 {
    const AUTO_OP_SPT_SHIFT: u32 = 0;
    const AUTO_OP_SPT_MASK: u32 = 0b1111;
    const AUTO_OP_SPT_IMPLEMENTED: u32 = 0b0001;

    /// Returns whether Auto-Operation is supported.
    pub fn auto_op_supported(&self) -> bool {
        (self.0 >> Self::AUTO_OP_SPT_SHIFT) & Self::AUTO_OP_SPT_MASK
            == Self::AUTO_OP_SPT_IMPLEMENTED
    }
}

/// Postbox/Mailbox Doorbell Channel Configuration 0 register value.
#[derive(Clone, Copy, Debug, Eq, FromBytes, Immutable, IntoBytes, KnownLayout, PartialEq)]
#[repr(transparent)]
pub struct DbchCfg0(pub(crate) u32);

impl DbchCfg0 {
    /// Returns the number of Doorbell Channels.
    pub const fn channel_count(&self) -> usize {
        ((self.0 & 0x7f) + 1) as usize
    }
}

/// Postbox/Mailbox FIFO Channel Configuration 0 register value.
#[derive(Clone, Copy, Debug, Eq, FromBytes, Immutable, IntoBytes, KnownLayout, PartialEq)]
#[repr(transparent)]
pub struct FfchCfg0(pub(crate) u32);

bitflags! {
    impl FfchCfg0: u32 {
        /// 8-bit Access Support
        const F8BA_SPT = 1 << 8;
        /// 16-bit Access Support
        const F16BA_SPT = 1 << 9;
        /// 32-bit Access Support
        const F32BA_SPT = 1 << 10;
        /// 64-bit Access Support
        const F64BA_SPT = 1 << 11;
    }
}

impl FfchCfg0 {
    const NUM_FFCH_SHIFT: u32 = 0;
    const NUM_FFCH_MASK: u32 = 0x3f;
    const FFCH_DEPTH_SHIFT: u32 = 16;
    const FFCH_DEPTH_MASK: u32 = 0x3ff;

    /// Returns the number of FIFO Channels.
    pub const fn channel_count(&self) -> usize {
        (((self.0 >> Self::NUM_FFCH_SHIFT) & Self::NUM_FFCH_MASK) + 1) as usize
    }

    /// Returns the FIFO Channel depth.
    pub const fn depth(&self) -> usize {
        (((self.0 >> Self::FFCH_DEPTH_SHIFT) & Self::FFCH_DEPTH_MASK) + 1) as usize
    }
}

/// Postbox/Mailbox Fast Channel Configuration 0 register value.
#[derive(Clone, Copy, Debug, Eq, FromBytes, Immutable, IntoBytes, KnownLayout, PartialEq)]
#[repr(transparent)]
pub struct FchCfg0(pub(crate) u32);

bitflags! {
    impl FchCfg0: u32 {
        /// Mailbox only (RES0 in postbox). Fast Channel Group Interrupt Support. Indicates whether
        /// Fast Channel Group Transfer interrupt is implemented for each FCGs.
        const FCGI_SPT = 1 << 10;
    }
}

impl FchCfg0 {
    const NUM_FCH_SHIFT: u32 = 0;
    const NUM_FCH_MASK: u32 = 0x3ff;
    const NUM_FCG_SHIFT: u32 = 11;
    const NUM_FCG_MASK: u32 = 0x1f;
    const NUM_FCH_PER_FCG_SHIFT: u32 = 16;
    const NUM_FCH_PER_FCG_MASK: u32 = 0x1f;
    const FCH_WS_SHIFT: u32 = 21;
    const FCH_WS_MASK: u32 = 0xff;

    /// Returns the number of Fast Channels.
    pub const fn channel_count(&self) -> usize {
        (((self.0 >> Self::NUM_FCH_SHIFT) & Self::NUM_FCH_MASK) + 1) as usize
    }

    /// Returns the number of Fast Channel Groups.
    pub const fn group_count(&self) -> usize {
        (((self.0 >> Self::NUM_FCG_SHIFT) & Self::NUM_FCG_MASK) + 1) as usize
    }

    /// Returns number of Fast Channels per Fast Channel Group
    pub const fn channels_per_group(&self) -> usize {
        (((self.0 >> Self::NUM_FCH_PER_FCG_SHIFT) & Self::NUM_FCH_PER_FCG_MASK) + 1) as usize
    }

    /// Returns the Fast Channel Word-Size in bits (32/64).
    pub const fn word_size(&self) -> usize {
        ((self.0 >> Self::FCH_WS_SHIFT) & Self::FCH_WS_MASK) as usize
    }
}

/// Postbox/Mailbox Control register value.
#[derive(Clone, Copy, Debug, Eq, FromBytes, Immutable, IntoBytes, KnownLayout, PartialEq)]
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

/// Mailbox Fast Channel Control register value.
#[derive(Clone, Copy, Debug, Eq, FromBytes, Immutable, IntoBytes, KnownLayout, PartialEq)]
#[repr(transparent)]
pub struct FchCtlr(u32);

bitflags! {
    impl FchCtlr: u32 {
        /// Interrupt enable
        const INT_EN = 1 << 2;
    }
}

/// Postbox/Mailbox Architecture Identification Register value.
#[derive(Clone, Copy, Debug, Eq, FromBytes, Immutable, IntoBytes, KnownLayout, PartialEq)]
#[repr(transparent)]
pub struct Aidr(u32);

impl Aidr {
    /// Returns the MHU Architecture Major Revision.
    pub const fn arch_major_rev(&self) -> u8 {
        (((self.0 >> 4) & 0xf) + 1) as u8
    }

    /// Returns the MHU Architecture Minor Revision.
    pub const fn arch_minor_rev(&self) -> u8 {
        (self.0 & 0xf) as u8
    }
}

/// Postbox Control page registers.
///
/// See C2.1.1.1 PBX_CTRL_page, Postbox CTRL page.
#[derive(Clone, Eq, FromBytes, Immutable, IntoBytes, KnownLayout, PartialEq)]
#[repr(C)]
pub struct MhuPostboxControlRegisters {
    /// 0x000: Postbox Block Identifier
    pbx_blk_id: ReadPure<u32>,
    /// 0x004 - 0x00C
    reserved_4: [u32; 3],
    /// 0x010: Postbox Feature Support 0
    pbx_feat_spt0: ReadPure<FeatSpt0>,
    /// 0x014: Postbox Feature Support 1
    pbx_feat_spt1: ReadPure<FeatSpt1>,
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
    /// 0x400: Postbox Doorbell Channel Interrupt Status n
    pbx_dbch_int_st: [ReadPure<u32>; 4],
    /// 0x410: Postbox FIFO Channel Interrupt Status n
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

/// Mailbox Control page registers.
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
    mbx_feat_spt0: ReadPure<FeatSpt0>,
    /// 0x014: Mailbox Feature Support 1
    mbx_feat_spt1: ReadPure<FeatSpt1>,
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
    mbx_fch_ctrl: ReadPureWrite<FchCtlr>,
    /// 0x144: Mailbox Fast Channel Group Interrupt Enable
    mbx_fcg_int_en: ReadPureWrite<u32>,
    /// 0x148 - 0x400
    reserved_148: [u32; 174],
    /// 0x400: Mailbox Doorbell Channel Interrupt Status n
    mbx_dbch_int_st: [ReadPure<u32>; 4],
    ///0x410: Mailbox FIFO Channel Interrupt Status n
    mbx_ffch_int_st: [ReadPure<u32>; 2],
    /// 0x418 - 0x46c
    reserved_418: [u32; 22],
    /// 0x470: Mailbox Fast Channel Group Interrupt Status
    mbx_fcg_int_st: ReadPure<u32>,
    /// 0x474 - 0x47C
    reserved_474: [u32; 3],
    /// 0x480: Mailbox Fast Channel Group Interrupt Status
    mbx_fch_grp_int_st: [ReadPure<u32>; 32],
    /// 0x500 - 0xFC4
    reserved_500: [u32; 690],
    /// 0xFC8: Mailbox Implementer Identification Register
    mbx_iidr: ReadPure<u32>,
    /// 0xFCC: Mailbox Architecture Identification Register
    mbx_aidr: ReadPure<Aidr>,
    /// 0xFD0: Mailbox Implementation Defined Identification
    impl_def_id: [ReadPure<u32>; 12],
}

/// Postbox Control driver.
pub struct MhuPostboxControl<'a> {
    pub(super) regs: UniqueMmioPointer<'a, MhuPostboxControlRegisters>,
}

impl<'a> MhuPostboxControl<'a> {
    /// Creates new instance.
    pub fn new(regs: UniqueMmioPointer<'a, MhuPostboxControlRegisters>) -> Self {
        Self { regs }
    }

    /// Checks the version of the peripherals and enables the postbox if supported.
    pub fn enable(&mut self, enable: bool) -> Result<(), Error> {
        let aidr = self.aidr();
        if aidr.arch_major_rev() != 3 || aidr.arch_minor_rev() != 0 {
            return Err(Error::UnsupportedMhuVersion);
        }

        field!(self.regs, pbx_ctrl).write(if enable { Ctrl::OP_REQ } else { Ctrl::empty() });

        Ok(())
    }

    /// Returns Feature Support 0.
    pub fn features0(&self) -> FeatSpt0 {
        field_shared!(self.regs, pbx_feat_spt0).read()
    }

    /// Returns Feature Support 1.
    pub fn features1(&self) -> FeatSpt1 {
        field_shared!(self.regs, pbx_feat_spt1).read()
    }

    /// Returns Doorbell Channel configuration if the extension is supported.
    pub fn doorbell_config(&self) -> Option<DbchCfg0> {
        if self.features0().doorbell_supported() {
            Some(field_shared!(self.regs, pbx_dbch_cfg0).read())
        } else {
            None
        }
    }

    /// Returns FIFO Channel configuration if the extension is supported.
    pub fn fifo_config(&self) -> Option<FfchCfg0> {
        if self.features0().fifo_supported() {
            Some(field_shared!(self.regs, pbx_ffch_cfg0).read())
        } else {
            None
        }
    }

    /// Returns the Fast Channel configuration if the extension is supported.
    pub fn fast_channel_config(&self) -> Option<FchCfg0> {
        if self.features0().fast_channel_supported() {
            Some(field_shared!(self.regs, pbx_fch_cfg0).read())
        } else {
            None
        }
    }

    /// Returns Architecture Identification Register value.
    pub fn aidr(&self) -> Aidr {
        field_shared!(self.regs, pbx_aidr).read()
    }
}

// Mailbox Fast Channel control
pub struct MhuMailboxFastChannelControl<'a, 'regs> {
    regs: &'a mut UniqueMmioPointer<'regs, MhuMailboxControlRegisters>,
}

impl<'a, 'b> MhuMailboxFastChannelControl<'a, 'b> {
    /// Returns the Fast Channel Control value.
    pub fn control(&self) -> FchCtlr {
        field_shared!(self.regs, mbx_fch_ctrl).read()
    }

    /// Sets the Fast Channel Control value.
    pub fn set_control(&mut self, ctlr: FchCtlr) {
        field!(self.regs, mbx_fch_ctrl).write(ctlr);
    }

    /// Returns whether a Fast Channel Group interrupt is enabled.
    pub fn is_interrupt_enabled(&self, index: usize) -> bool {
        assert!(index < 32);

        let interrupts = field_shared!(self.regs, mbx_fcg_int_en).read();
        interrupts & (1 << index) != 0
    }

    /// Enables/disables a Fast Channel Control interrupt.
    pub fn configure_interrupt(&mut self, index: usize, enable: bool) {
        assert!(index < 32);

        let mut interrupts = field_shared!(self.regs, mbx_fcg_int_en).read();

        if enable {
            interrupts |= 1 << index;
        } else {
            interrupts &= !(1 << index);
        }

        field!(self.regs, mbx_fcg_int_en).write(interrupts);
    }
}

/// Mailbox Control driver.
pub struct MhuMailboxControl<'a> {
    pub(super) regs: UniqueMmioPointer<'a, MhuMailboxControlRegisters>,
}

impl<'a> MhuMailboxControl<'a> {
    /// Creates new instance.
    pub fn new(regs: UniqueMmioPointer<'a, MhuMailboxControlRegisters>) -> Self {
        Self { regs }
    }

    /// Checks the version of the peripherals and enables the mailbox if supported.
    pub fn enable(&mut self, enable: bool) -> Result<(), Error> {
        let aidr = self.aidr();
        if aidr.arch_major_rev() != 3 || aidr.arch_minor_rev() != 0 {
            return Err(Error::UnsupportedMhuVersion);
        }

        field!(self.regs, mbx_ctrl).write(if enable { Ctrl::OP_REQ } else { Ctrl::empty() });

        Ok(())
    }

    /// Returns Feature Support 0.
    pub fn features0(&self) -> FeatSpt0 {
        field_shared!(self.regs, mbx_feat_spt0).read()
    }

    /// Returns Feature Support 1.
    pub fn features1(&self) -> FeatSpt1 {
        field_shared!(self.regs, mbx_feat_spt1).read()
    }

    /// Returns Doorbell Channel configuration if the extension is supported.
    pub fn doorbell_config(&self) -> Option<DbchCfg0> {
        if self.features0().doorbell_supported() {
            Some(field_shared!(self.regs, mbx_dbch_cfg0).read())
        } else {
            None
        }
    }

    /// Returns FIFO Channel configuration if the extension is supported.
    pub fn fifo_config(&self) -> Option<FfchCfg0> {
        if self.features0().fifo_supported() {
            Some(field_shared!(self.regs, mbx_ffch_cfg0).read())
        } else {
            None
        }
    }

    /// Returns the Fast Channel configuration if the extension is supported.
    pub fn fast_channel_config(&self) -> Option<FchCfg0> {
        if self.features0().fast_channel_supported() {
            Some(field_shared!(self.regs, mbx_fch_cfg0).read())
        } else {
            None
        }
    }

    /// Returns Fast Channel control object if the extension is supported.
    pub fn fast_channel_control(&mut self) -> Option<MhuMailboxFastChannelControl<'_, 'a>> {
        if self.features0().fast_channel_supported() {
            Some(MhuMailboxFastChannelControl {
                regs: &mut self.regs,
            })
        } else {
            None
        }
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
    fn feat_spt0_flags() {
        let feat = FeatSpt0(0);

        assert!(!feat.doorbell_supported());
        assert!(!feat.fifo_supported());
        assert!(!feat.fast_channel_supported());
        assert!(!feat.trustzone_supported());
        assert!(!feat.rme_supported());
        assert_eq!(RasSupport::NotImplemented, feat.ras_support());

        let value = (0b0001 << 0)
            | (0b0001 << 4)
            | (0b0001 << 8)
            | (0b0001 << 12)
            | (0b0001 << 16)
            | (0b0011 << 20);
        let feat = FeatSpt0(value);

        assert!(feat.doorbell_supported());
        assert!(feat.fifo_supported());
        assert!(feat.fast_channel_supported());
        assert!(feat.trustzone_supported());
        assert!(feat.rme_supported());
        assert_eq!(RasSupport::ImplementedCompliant, feat.ras_support());
    }

    #[test]
    fn feat_spt1() {
        assert!(FeatSpt1(0b0001).auto_op_supported());
        assert!(!FeatSpt1(0b0010).auto_op_supported());
    }

    #[test]
    fn doorbell_config() {
        let config = DbchCfg0(0x7f);

        assert_eq!(128, config.channel_count());
    }

    #[test]
    fn fifo_config() {
        let config = FfchCfg0(0x03ff_0a3f);

        assert_eq!(64, config.channel_count());
        assert!(config.contains(FfchCfg0::F64BA_SPT | FfchCfg0::F16BA_SPT));
        assert_eq!(1024, config.depth());
    }

    #[test]
    fn fast_channel_config() {
        let config = FchCfg0(0x041f_fbff);

        assert_eq!(1024, config.channel_count());
        assert_eq!(32, config.group_count());
        assert_eq!(32, config.channels_per_group());
        assert_eq!(32, config.word_size());
    }

    #[test]
    fn postbox_config() {
        let mut regs = FakePostboxControlRegisters::new();

        regs.reg_write(0x010, 0x0011_1111);
        regs.reg_write(0x014, 0x89ab_cdef);
        regs.reg_write(0x020, 0x7f);
        regs.reg_write(0x030, 0x03ff_0a3f);
        regs.reg_write(0x040, 0x041f_fbff);
        regs.reg_write(0xfcc, 0x24);

        {
            let instance = regs.instance_for_test();

            assert_eq!(0x0011_1111, instance.features0().0);
            assert_eq!(0x89ab_cdef, instance.features1().0);
            assert_eq!(0x7f, instance.doorbell_config().unwrap().0);
            assert_eq!(0x03ff_0a3f, instance.fifo_config().unwrap().0);
            assert_eq!(0x041f_fbff, instance.fast_channel_config().unwrap().0);

            let aidr = instance.aidr();
            assert_eq!(3, aidr.arch_major_rev());
            assert_eq!(4, aidr.arch_minor_rev());
        }

        regs.clear();
        {
            let instance = regs.instance_for_test();

            assert!(instance.doorbell_config().is_none());
            assert!(instance.fifo_config().is_none());
            assert!(instance.fast_channel_config().is_none());
        }
    }

    #[test]
    fn mailbox_config() {
        let mut regs = FakeMailboxControlRegisters::new();

        regs.reg_write(0x010, 0x0011_1111);
        regs.reg_write(0x014, 0x89ab_cdef);
        regs.reg_write(0x020, 0x7f);
        regs.reg_write(0x030, 0x03ff_0a3f);
        regs.reg_write(0x040, 0x041f_fbff);
        regs.reg_write(0xfcc, 0x24);

        {
            let instance = regs.instance_for_test();

            assert_eq!(0x0011_1111, instance.features0().0);
            assert_eq!(0x89ab_cdef, instance.features1().0);
            assert_eq!(0x7f, instance.doorbell_config().unwrap().0);
            assert_eq!(0x03ff_0a3f, instance.fifo_config().unwrap().0);
            assert_eq!(0x041f_fbff, instance.fast_channel_config().unwrap().0);

            let aidr = instance.aidr();
            assert_eq!(3, aidr.arch_major_rev());
            assert_eq!(4, aidr.arch_minor_rev());
        }

        {
            let mut instance = regs.instance_for_test();

            let mut fast_channel_control = instance.fast_channel_control().unwrap();
            assert_eq!(FchCtlr::empty(), fast_channel_control.control());
            fast_channel_control.set_control(FchCtlr::INT_EN);
        }
        assert_eq!(0x04, regs.reg_read(0x140));

        {
            let mut instance = regs.instance_for_test();

            let mut fast_channel_control = instance.fast_channel_control().unwrap();

            fast_channel_control.configure_interrupt(1, true);
            fast_channel_control.configure_interrupt(2, false);
            fast_channel_control.configure_interrupt(3, true);

            assert!(fast_channel_control.is_interrupt_enabled(1));
            assert!(!fast_channel_control.is_interrupt_enabled(2));
            assert!(fast_channel_control.is_interrupt_enabled(3));
        }

        assert_eq!(0b1010, regs.reg_read(0x144));

        regs.clear();

        {
            let mut instance = regs.instance_for_test();

            assert!(instance.doorbell_config().is_none());
            assert!(instance.fifo_config().is_none());
            assert!(instance.fast_channel_config().is_none());
            assert!(instance.fast_channel_control().is_none());
        }
    }
}
