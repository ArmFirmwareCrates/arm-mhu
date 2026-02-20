// SPDX-FileCopyrightText: Copyright The arm-mhu Contributors.
// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::{
    Error,
    control::{Aidr, DbchCfg0, FchCfg0, FeatSpt0, FeatSpt1, FfchCfg0},
};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use safe_mmio::{
    UniqueMmioPointer, field, field_shared,
    fields::{ReadPure, ReadPureWrite},
};
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout};

/// Security group selector.
#[derive(Clone, Copy, Debug, Eq, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(u32)]
pub enum SecurityGroup {
    Secure = 0b00,
    NonSecure = 0b01,
    Root = 0b10,
    Realm = 0b11,
}

/// Sender Security Control page registers.
///
/// C2.1.2.1 SSC_CTRL_page, Sender Security Control Page
#[derive(Clone, Eq, FromBytes, Immutable, IntoBytes, KnownLayout, PartialEq)]
#[repr(C)]
pub struct MhuSenderSecurityControlRegisters {
    /// 0x000: Sender Security Block Identifier
    ssc_blk_id: ReadPure<u32>,
    /// 0x004 - 0x00C
    reserved_4: [u32; 3],
    /// 0x010: Sender Security Feature Support 0
    ssc_feat_spt0: ReadPure<FeatSpt0>,
    /// 0x014: Sender Security Feature Support 1
    ssc_feat_spt1: ReadPure<FeatSpt1>,
    /// 0x018 - 0x01C
    reserved_18: [u32; 2],
    /// 0x020: Sender Security Doorbell Channel Configuration 0
    ssc_dbch_cfg0: ReadPure<DbchCfg0>,
    /// 0x024 - 0x02C
    reserved_24: [u32; 3],
    /// 0x030: Sender Security FIFO Channel Configuration 0
    ssc_ffch_cfg0: ReadPure<FfchCfg0>,
    /// 0x034 - 0x3c
    reserved_34: [u32; 3],
    /// 0x040: Sender Security Fast Channel Configuration 0
    ssc_fch_cfg0: ReadPure<FchCfg0>,
    /// 0x044 - 0x010C
    reserved_44: [u32; 51],
    /// 0x110: Postbox control
    ssc_pbx_sg: ReadPureWrite<u32>,
    /// 0x114 - 0xFC4
    reserved_114: [u32; 941],
    /// 0xFC8: Postbox Implementer Identification Register
    ssc_iidr: ReadPure<u32>,
    /// 0xFCC: Postbox Architecture Identification Register
    ssc_aidr: ReadPure<Aidr>,
    /// 0xFD0: Postbox Implementation Defined Identification
    impl_def_id: [ReadPure<u32>; 12],
}

/// Receiver Security Control page registers.
///
/// See C2.2.2.1 RSC_CTRL_page, Receiver Security Control Page
#[derive(Clone, Eq, FromBytes, Immutable, IntoBytes, KnownLayout, PartialEq)]
#[repr(C)]
pub struct MhuReceiverSecurityControlRegisters {
    /// 0x000: Mailbox Block Identifier
    rsc_blk_id: ReadPure<u32>,
    /// 0x004 - 0x00C
    reserved_4: [u32; 3],
    /// 0x010: Mailbox Feature Support 0
    rsc_feat_spt0: ReadPure<FeatSpt0>,
    /// 0x014: Mailbox Feature Support 1
    rsc_feat_spt1: ReadPure<FeatSpt1>,
    /// 0x018 - 0x01C
    reserved_18: [u32; 2],
    /// 0x020: Mailbox Doorbell Channel Configuration 0
    rsc_dbch_cfg0: ReadPure<DbchCfg0>,
    /// 0x024 - 0x02C
    reserved_24: [u32; 3],
    /// 0x030; Mailbox FIFO Channel Configuration 0
    rsc_ffch_cfg0: ReadPure<FfchCfg0>,
    /// 0x034 - 0x03C
    reserved_34: [u32; 3],
    /// 0x040: Mailbox Fast Channel Configuration 0
    rsc_fch_cfg0: ReadPure<FchCfg0>,
    /// 0x044: - 0x0fc
    reserved_44: [u32; 51],
    /// 0x110: Mailbox control
    rsc_mbx_sg: ReadPureWrite<u32>,
    /// 0x114 - 0xFC4
    reserved_114: [u32; 941],
    /// 0xFC8: Mailbox Implementer Identification Register
    rsc_iidr: ReadPure<u32>,
    /// 0xFCC: Mailbox Architecture Identification Register
    rsc_aidr: ReadPure<Aidr>,
    /// 0xFD0: Mailbox Implementation Defined Identification
    impl_def_id: [ReadPure<u32>; 12],
}

/// Sender Security Control driver.
pub struct MhuSenderSecurityControl<'a> {
    regs: UniqueMmioPointer<'a, MhuSenderSecurityControlRegisters>,
}

impl<'a> MhuSenderSecurityControl<'a> {
    /// Creates new instance.
    pub fn new(regs: UniqueMmioPointer<'a, MhuSenderSecurityControlRegisters>) -> Self {
        Self { regs }
    }

    /// Returns Feature Support 0.
    pub fn features0(&self) -> FeatSpt0 {
        field_shared!(self.regs, ssc_feat_spt0).read()
    }

    /// Returns Feature Support 1.
    pub fn features1(&self) -> FeatSpt1 {
        field_shared!(self.regs, ssc_feat_spt1).read()
    }

    /// Returns Doorbell Channel configuration.
    pub fn doorbell_config(&self) -> DbchCfg0 {
        field_shared!(self.regs, ssc_dbch_cfg0).read()
    }

    /// Returns FIFO Channel configuration.
    pub fn fifo_config(&self) -> FfchCfg0 {
        field_shared!(self.regs, ssc_ffch_cfg0).read()
    }

    /// Returns the Fast Channel configuration.
    pub fn fast_channel_config(&self) -> FchCfg0 {
        field_shared!(self.regs, ssc_fch_cfg0).read()
    }

    /// Returns the current security group.
    pub fn security_group(&self) -> SecurityGroup {
        field_shared!(self.regs, ssc_pbx_sg)
            .read()
            .try_into()
            .unwrap()
    }

    /// Sets the security group if supported.
    pub fn set_security_group(&mut self, group: SecurityGroup) -> Result<(), Error> {
        let features = self.features0();

        match (
            features.trustzone_supported(),
            features.rme_supported(),
            group,
        ) {
            (false, _, _) | (_, false, SecurityGroup::Root) | (_, false, SecurityGroup::Realm) => {
                Err(Error::OperationNotSupported)
            }
            (_, _, group) => {
                field!(self.regs, ssc_pbx_sg).write(group.into());
                Ok(())
            }
        }
    }

    /// Returns Architecture Identification Register value.
    pub fn aidr(&self) -> Aidr {
        field_shared!(self.regs, ssc_aidr).read()
    }
}

/// Receiver Security Control driver.
pub struct MhuReceiverSecurityControl<'a> {
    regs: UniqueMmioPointer<'a, MhuReceiverSecurityControlRegisters>,
}

impl<'a> MhuReceiverSecurityControl<'a> {
    /// Creates new instance.
    pub fn new(regs: UniqueMmioPointer<'a, MhuReceiverSecurityControlRegisters>) -> Self {
        Self { regs }
    }

    /// Returns Feature Support 0.
    pub fn features0(&self) -> FeatSpt0 {
        field_shared!(self.regs, rsc_feat_spt0).read()
    }

    /// Returns Feature Support 1.
    pub fn features1(&self) -> FeatSpt1 {
        field_shared!(self.regs, rsc_feat_spt1).read()
    }

    /// Returns Doorbell Channel configuration.
    pub fn doorbell_config(&self) -> DbchCfg0 {
        field_shared!(self.regs, rsc_dbch_cfg0).read()
    }

    /// Returns FIFO Channel configuration.
    pub fn fifo_config(&self) -> FfchCfg0 {
        field_shared!(self.regs, rsc_ffch_cfg0).read()
    }

    /// Returns the Fast Channel configuration.
    pub fn fast_channel_config(&self) -> FchCfg0 {
        field_shared!(self.regs, rsc_fch_cfg0).read()
    }

    /// Returns the current security group.
    pub fn security_group(&self) -> SecurityGroup {
        field_shared!(self.regs, rsc_mbx_sg)
            .read()
            .try_into()
            .unwrap()
    }

    /// Sets the security group if supported.
    pub fn set_security_group(&mut self, group: SecurityGroup) -> Result<(), Error> {
        let features = self.features0();

        match (
            features.trustzone_supported(),
            features.rme_supported(),
            group,
        ) {
            (false, _, _) | (_, false, SecurityGroup::Root) | (_, false, SecurityGroup::Realm) => {
                Err(Error::OperationNotSupported)
            }
            (_, _, group) => {
                field!(self.regs, rsc_mbx_sg).write(group.into());
                Ok(())
            }
        }
    }

    /// Returns Architecture Identification Register value.
    pub fn aidr(&self) -> Aidr {
        field_shared!(self.regs, rsc_aidr).read()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::define_fake_regs;

    define_fake_regs!(
        FakeSenderSecurityControlRegisters,
        1024,
        MhuSenderSecurityControlRegisters,
        MhuSenderSecurityControl
    );

    define_fake_regs!(
        FakeMhuReceiverSecurityControl,
        1024,
        MhuReceiverSecurityControlRegisters,
        MhuReceiverSecurityControl
    );

    #[test]
    fn regs_size() {
        assert_eq!(0x1000, size_of::<MhuSenderSecurityControlRegisters>());
        assert_eq!(0x1000, size_of::<MhuReceiverSecurityControlRegisters>());
    }

    #[test]
    fn sender_config() {
        let mut regs = FakeSenderSecurityControlRegisters::new();

        regs.reg_write(0x010, 0x0123_4567);
        regs.reg_write(0x014, 0x89ab_cdef);
        regs.reg_write(0x020, 0x7f);
        regs.reg_write(0x030, 0x03ff_0a3f);
        regs.reg_write(0x040, 0x041f_fbff);
        regs.reg_write(0xfcc, 0x24);

        let instance = regs.instance_for_test();

        assert_eq!(0x0123_4567, instance.features0().0);
        assert_eq!(0x89ab_cdef, instance.features1().0);
        assert_eq!(0x7f, instance.doorbell_config().0);
        assert_eq!(0x03ff_0a3f, instance.fifo_config().0);
        assert_eq!(0x041f_fbff, instance.fast_channel_config().0);

        let aidr = instance.aidr();
        assert_eq!(3, aidr.arch_major_rev());
        assert_eq!(4, aidr.arch_minor_rev());
    }

    #[test]
    fn receiver_config() {
        let mut regs = FakeMhuReceiverSecurityControl::new();

        regs.reg_write(0x010, 0x0123_4567);
        regs.reg_write(0x014, 0x89ab_cdef);
        regs.reg_write(0x020, 0x7f);
        regs.reg_write(0x030, 0x03ff_0a3f);
        regs.reg_write(0x040, 0x041f_fbff);
        regs.reg_write(0xfcc, 0x24);

        let instance = regs.instance_for_test();

        assert_eq!(0x0123_4567, instance.features0().0);
        assert_eq!(0x89ab_cdef, instance.features1().0);
        assert_eq!(0x7f, instance.doorbell_config().0);
        assert_eq!(0x03ff_0a3f, instance.fifo_config().0);
        assert_eq!(0x041f_fbff, instance.fast_channel_config().0);

        let aidr = instance.aidr();
        assert_eq!(3, aidr.arch_major_rev());
        assert_eq!(4, aidr.arch_minor_rev());
    }

    #[test]
    fn sender_security_group_read() {
        let mut regs = FakeSenderSecurityControlRegisters::new();

        {
            regs.reg_write(0x110, 0b11);

            let instance = regs.instance_for_test();
            assert_eq!(SecurityGroup::Realm, instance.security_group());
        }

        regs.clear();

        {
            let mut instance = regs.instance_for_test();

            assert_eq!(
                Err(Error::OperationNotSupported),
                instance.set_security_group(SecurityGroup::Secure)
            );
        }

        regs.reg_write(0x010, 0b0001 << 12);

        {
            let mut instance = regs.instance_for_test();
            assert_eq!(Ok(()), instance.set_security_group(SecurityGroup::Secure));
        }
        assert_eq!(0b00, regs.reg_read(0x110));

        {
            let mut instance = regs.instance_for_test();
            assert_eq!(
                Err(Error::OperationNotSupported),
                instance.set_security_group(SecurityGroup::Root)
            );
        }

        regs.reg_write(0x010, (0b0001 << 12) | (0b0001 << 16));

        let mut instance = regs.instance_for_test();
        assert_eq!(Ok(()), instance.set_security_group(SecurityGroup::Root));
        assert_eq!(0b10, regs.reg_read(0x110));
    }

    #[test]
    fn receiver_security_group_read() {
        let mut regs = FakeMhuReceiverSecurityControl::new();

        {
            regs.reg_write(0x110, 0b11);

            let instance = regs.instance_for_test();
            assert_eq!(SecurityGroup::Realm, instance.security_group());
        }

        regs.clear();

        {
            let mut instance = regs.instance_for_test();

            assert_eq!(
                Err(Error::OperationNotSupported),
                instance.set_security_group(SecurityGroup::Secure)
            );
        }

        regs.reg_write(0x010, 0b0001 << 12);

        {
            let mut instance = regs.instance_for_test();
            assert_eq!(Ok(()), instance.set_security_group(SecurityGroup::Secure));
        }
        assert_eq!(0b00, regs.reg_read(0x110));

        {
            let mut instance = regs.instance_for_test();
            assert_eq!(
                Err(Error::OperationNotSupported),
                instance.set_security_group(SecurityGroup::Root)
            );
        }

        regs.reg_write(0x010, (0b0001 << 12) | (0b0001 << 16));

        let mut instance = regs.instance_for_test();
        assert_eq!(Ok(()), instance.set_security_group(SecurityGroup::Root));
        assert_eq!(0b10, regs.reg_read(0x110));
    }
}
