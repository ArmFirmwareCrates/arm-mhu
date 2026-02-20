// SPDX-FileCopyrightText: Copyright The arm-mhu Contributors.
// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::{Error, control::FfchCfg0};
use bitflags::bitflags;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use safe_mmio::{
    UniqueMmioPointer, field, field_shared,
    fields::{ReadOnly, ReadPure, ReadPureWrite, WriteOnly},
};
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout};

/// Postbox FIFO Channel flag register.
#[derive(Clone, Copy, Debug, Eq, FromBytes, Immutable, IntoBytes, KnownLayout, PartialEq)]
#[repr(transparent)]
pub struct PostboxFifoFlag(u32);

bitflags! {
    impl PostboxFifoFlag: u32 {
        /// The ACK flag requests that when the Receiver pops the byte from the FIFO, and that byte
        /// is the last byte of the Transfer, a Transfer Acknowledge event is generated.
        const ACK = 1 << 0;
        /// The SOT flag indicates that the next push operation to the FIFO will contain the first
        /// byte of a Transfer.
        const SOT = 1 << 1;
        /// The EOT field indicates that the next push operation to the FIFO will contain the last
        /// byte of a Transfer.
        const EOT = 1 << 2;
    }
}

/// Postbox/Mailbox FIFO Channel interrupt register.
#[derive(Clone, Copy, Debug, Eq, FromBytes, Immutable, IntoBytes, KnownLayout, PartialEq)]
#[repr(transparent)]
pub struct FifoInterrupt(u32);

bitflags! {
    impl FifoInterrupt: u32 {
        /// Transfer Acknowledge
        const TFR_ACK = 1 << 0;
        /// FIFO Low Tidemark
        const FLT = 1 << 1;
        /// FIFO High Tidemark
        const FHT = 1 << 2;
        /// FIFO Flush
        const FF = 1 << 31;
    }
}

/// Transfer Delineation Mode selects whether the MHU or software or a combination of both manages
/// the SOT and EOT flags.
#[derive(Clone, Copy, Debug, Eq, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(u32)]
pub enum TransferDelineationMode {
    SoftwareFlag = 0b00,
    PartialFlag = 0b01,
    AutoFlag = 0b10,
}

/// Postbox FIFO Channel control register.
#[derive(Clone, Copy, Debug, Eq, FromBytes, Immutable, IntoBytes, KnownLayout, PartialEq)]
pub struct PostboxFifoControl(u32);

bitflags! {
    impl PostboxFifoControl: u32 {
        /// Postbox Combined Enable
        const PBX_COMB_EN = 1 << 0;
        /// Most Significant Byte First
        const MSBF = 1 << 1;
        /// FIFO Flush
        const FF = 1 << 31;
    }
}

impl PostboxFifoControl {
    const TDM_SHIFT: u32 = 2;
    const TDM_MASK: u32 = 0b11;

    /// Returns the Transfer Delineation Mode.
    pub fn transfer_delineation_mode(&self) -> Option<TransferDelineationMode> {
        let tdm_bits = (self.0 >> Self::TDM_SHIFT) & Self::TDM_MASK;
        tdm_bits.try_into().ok()
    }

    /// Sets the Transfer Delineation Mode.
    pub fn set_transfer_delineation_mode(&mut self, mode: TransferDelineationMode) {
        self.0 &= !(Self::TDM_MASK << Self::TDM_SHIFT);
        self.0 |= u32::from(mode) << Self::TDM_SHIFT;
    }
}

/// Mailbox FIFO Channel control register.
#[derive(Clone, Copy, Debug, Eq, FromBytes, Immutable, IntoBytes, KnownLayout, PartialEq)]
pub struct MailboxFifoControl(u32);

bitflags! {
    impl MailboxFifoControl: u32 {
        /// Mailbox Combined Enable
        const MBX_COMB_EN = 1 << 0;
        /// Most Significant Byte First
        const MSBF = 1 << 1;
        /// Controls whether Read to acknowledge is enabled.
        const RA_EN = 1 << 2;
        /// Future Transfer Auto Buffering
        const FTAB = 1 << 3;
        /// FIFO Flush
        const FF = 1 << 31;
    }
}

/// Postbox FIFO Channel status register.
#[derive(Clone, Copy, Debug, Eq, FromBytes, Immutable, IntoBytes, KnownLayout, PartialEq)]
pub struct PostboxFifoStatus(u32);

bitflags! {
    impl PostboxFifoStatus: u32 {
        /// Previous Push Error
        const PPE = 1 << 16;
        /// FIFO Flush
        const FF = 1 << 31;
    }
}

impl PostboxFifoStatus {
    const FFS_SHIFT: u32 = 0;
    const FFS_MASK: u32 = 0b111_1111_1111;

    /// Returns the number of invalid bytes in the FIFO.
    pub const fn fifo_free_space(&self) -> usize {
        ((self.0 >> Self::FFS_SHIFT) & Self::FFS_MASK) as usize
    }
}

/// Mailbox FIFO Channel status register.
#[derive(Clone, Copy, Debug, Eq, FromBytes, Immutable, IntoBytes, KnownLayout, PartialEq)]
pub struct MailboxFifoStatus(u32);

bitflags! {
    impl MailboxFifoStatus: u32 {
        /// FIFO Flush
        const FF = 1 << 31;
    }
}

impl MailboxFifoStatus {
    const FFL_SHIFT: u32 = 0;
    const FFL_MASK: u32 = 0b111_1111_1111;

    /// Returns the number of valid bytes in the FIFO.
    pub const fn fifo_fill_level(&self) -> usize {
        ((self.0 >> Self::FFL_SHIFT) & Self::FFL_MASK) as usize
    }
}

/// Postbox FIFO Channel acknowledge counter register.
#[derive(Clone, Copy, Debug, Eq, FromBytes, Immutable, IntoBytes, KnownLayout, PartialEq)]
pub struct PostboxFifoAckCnt(u32);

bitflags! {
    impl PostboxFifoAckCnt: u32 {
        /// Acknowledge Counter Overflow
        const ACK_CNT_OVRFLW = 1 << 11;
    }
}

impl PostboxFifoAckCnt {
    const ACK_CNT_SHIFT: u32 = 0;
    const ACK_CNT_MASK: u32 = 0b111_1111_1111;

    /// Returns the Acknowledge Count.
    pub const fn acknowledge_count(&self) -> usize {
        ((self.0 >> Self::ACK_CNT_SHIFT) & Self::ACK_CNT_MASK) as usize
    }
}

/// Mailbox FIFO pop request register.
#[derive(Clone, Copy, Debug, Eq, FromBytes, Immutable, IntoBytes, KnownLayout, PartialEq)]
pub struct MailboxFifoPop(u32);

impl MailboxFifoPop {
    const POP_SHIFT: u32 = 0;
    const POP_MASK: u32 = 0b111;

    /// Creates new instance
    pub const fn new(pop_count: usize) -> Self {
        Self((pop_count as u32 & Self::POP_MASK) << Self::POP_SHIFT)
    }
}

/// Postbox/Mailbox FIFO Channel tidemark register.
#[derive(Clone, Copy, Debug, Eq, FromBytes, Immutable, IntoBytes, KnownLayout, PartialEq)]
pub struct FifoTidemark(u32);

impl FifoTidemark {
    const LOW_SHIFT: u32 = 0;
    const LOW_MASK: u32 = 0b11_1111_1111;
    const HIGH_SHIFT: u32 = 16;
    const HIGH_MASK: u32 = 0b11_1111_1111;

    /// Creates new instance.
    pub const fn new(high: usize, low: usize) -> Self {
        assert!(high <= Self::HIGH_MASK as usize);
        assert!(low <= Self::LOW_MASK as usize);

        Self(
            (((high as u32) & Self::HIGH_MASK) << Self::HIGH_SHIFT)
                | ((low as u32) & Self::LOW_MASK) << Self::LOW_SHIFT,
        )
    }

    /// Returns the FIFO high tidemark.
    pub const fn high(&self) -> usize {
        ((self.0 >> Self::HIGH_SHIFT) & Self::HIGH_MASK) as usize
    }

    /// Sets the FIFO high tidemark.
    pub const fn set_high(&mut self, high: usize) {
        assert!(high <= Self::HIGH_MASK as usize);

        self.0 &= !(Self::HIGH_MASK << Self::HIGH_SHIFT);
        self.0 |= ((high as u32) & Self::HIGH_MASK) << Self::HIGH_SHIFT;
    }

    /// Returns the FIFO low tidemark.
    pub const fn low(&self) -> usize {
        ((self.0 >> Self::LOW_SHIFT) & Self::LOW_MASK) as usize
    }

    /// Sets the FIFO low tidemark.
    pub const fn set_low(&mut self, low: usize) {
        assert!(low <= Self::LOW_MASK as usize);

        self.0 &= !(Self::LOW_MASK << Self::LOW_SHIFT);
        self.0 |= ((low as u32) & Self::LOW_MASK) << Self::LOW_SHIFT;
    }
}

/// Postbox FIFO Channel window registers.
#[derive(Clone, Eq, FromBytes, Immutable, IntoBytes, KnownLayout, PartialEq)]
#[repr(C)]
pub struct MhuPostboxFifoRegisters {
    /// 0x00: Payload
    pffcw_pay: u64,
    /// 0x08: Postbox FIFO Channel Window Flag
    pffcw_flg: ReadPureWrite<PostboxFifoFlag>,
    /// 0x0C
    reserved_c: u32,
    /// 0x10: Postbox FIFO Channel Window Interrupt Status
    pffcw_int_st: ReadPure<FifoInterrupt>,
    /// 0x14: Postbox FIFO Channel Window Interrupt Clear
    pffcw_int_clr: WriteOnly<FifoInterrupt>,
    /// 0x18: Postbox FIFO Channel Window Interrupt Enable
    pffcw_int_en: ReadPureWrite<FifoInterrupt>,
    /// 0x1C
    reserved_1c: u32,
    /// 0x20: Postbox FIFO Channel Window Control
    pffcw_ctlr: ReadPureWrite<PostboxFifoControl>,
    /// 0x24: Postbox FIFO Channel Window Status
    pffcw_st: ReadPure<PostboxFifoStatus>,
    /// 0x28: Postbox FIFO Channel Window Acknowledge Counter
    pffcw_ack_cnt: ReadOnly<PostboxFifoAckCnt>,
    /// 0x2C: Postbox FIFO Channel Window Tidemark
    pffcw_tide: ReadPureWrite<FifoTidemark>,
    /// 0x30 - 0x3C
    reserved_30: [u32; 4],
}

/// Mailbox FIFO Channel window registers.
#[derive(Clone, Eq, FromBytes, Immutable, IntoBytes, KnownLayout, PartialEq)]
#[repr(C)]
pub struct MhuMailboxFifoRegisters {
    /// 0x00: Payload
    mffcw_pay: u64,
    /// 0x08: Mailbox FIFO Channel Window Flag
    mffcw_flg: u64,
    /// 0x10: Mailbox FIFO Channel Window Interrupt Status
    mffcw_int_st: ReadPure<FifoInterrupt>,
    /// 0x14: Mailbox FIFO Channel Window Interrupt Clear
    mffcw_int_clr: WriteOnly<FifoInterrupt>,
    /// 0x18: Mailbox FIFO Channel Window Interrupt Enable
    mffcw_int_en: ReadPureWrite<FifoInterrupt>,
    /// 0x1C
    reserved_1c: u32,
    /// 0x20: Mailbox FIFO Channel Window Control
    mffcw_ctlr: ReadPureWrite<MailboxFifoControl>,
    /// 0x24: Mailbox FIFO Channel Window Status
    mffcw_st: ReadPure<MailboxFifoStatus>,
    /// 0x28: Mailbox FIFO Channel FIFO POP
    mffcw_fifo_pop: WriteOnly<MailboxFifoPop>,
    /// 0x2C: Mailbox FIFO Channel Window Tidemark
    mffcw_tide: ReadPureWrite<FifoTidemark>,
    /// 0x30 - 0x3C
    reserved_30: [u32; 4],
}

/// Postbox FIFO Channel driver.
pub struct MhuPostboxFifo<'a> {
    pub(super) regs: UniqueMmioPointer<'a, MhuPostboxFifoRegisters>,
    config: FfchCfg0,
}

impl<'a> MhuPostboxFifo<'a> {
    /// Creates new instance.
    pub fn new(regs: UniqueMmioPointer<'a, MhuPostboxFifoRegisters>, config: FfchCfg0) -> Self {
        Self { regs, config }
    }

    /// Returns the configuration of the FIFO.
    pub fn config(&self) -> FfchCfg0 {
        self.config
    }

    /// Writes a byte to the FIFO.
    pub fn write8(&mut self, data: u8) -> Result<(), Error> {
        if self.config.contains(FfchCfg0::F8BA_SPT) {
            self.write(data);
            Ok(())
        } else {
            Err(Error::OperationNotSupported)
        }
    }

    /// Writes a 16-bit word to the FIFO.
    pub fn write16(&mut self, data: u16) -> Result<(), Error> {
        if self.config.contains(FfchCfg0::F16BA_SPT) {
            self.write(data);
            Ok(())
        } else {
            Err(Error::OperationNotSupported)
        }
    }

    /// Writes a 32-bit word to the FIFO.
    pub fn write32(&mut self, data: u32) -> Result<(), Error> {
        if self.config.contains(FfchCfg0::F32BA_SPT) {
            self.write(data);
            Ok(())
        } else {
            Err(Error::OperationNotSupported)
        }
    }

    /// Writes a 64-bit word to the FIFO.
    pub fn write64(&mut self, data: u64) -> Result<(), Error> {
        if self.config.contains(FfchCfg0::F64BA_SPT) {
            self.write(data);
            Ok(())
        } else {
            Err(Error::OperationNotSupported)
        }
    }

    /// Returns flags of the FIFO.
    pub fn flags(&self) -> PostboxFifoFlag {
        field_shared!(self.regs, pffcw_flg).read()
    }

    /// Sets the flags of the FIFO.
    pub fn set_flags(&mut self, flags: PostboxFifoFlag) {
        field!(self.regs, pffcw_flg).write(flags);
    }

    /// Enables/disables interrupts and the FIFO Channel interrupts to contribute to the Postbox
    /// Combined interrupt.
    pub fn configure_interrupts(&mut self, interrupts: Option<FifoInterrupt>) {
        if let Some(interrupts) = interrupts {
            field!(self.regs, pffcw_int_en).write(interrupts);
        } else {
            field!(self.regs, pffcw_int_en).write(FifoInterrupt::empty());
            self.clear_interrupts(FifoInterrupt::all());
        }

        self.modify_ctlr(|ctlr| ctlr.set(PostboxFifoControl::PBX_COMB_EN, interrupts.is_some()));
    }

    /// Reads interrupt status.
    pub fn interrupt_status(&self) -> FifoInterrupt {
        field_shared!(self.regs, pffcw_int_st).read()
    }

    /// Clears interrupts.
    pub fn clear_interrupts(&mut self, interrupt: FifoInterrupt) {
        field!(self.regs, pffcw_int_clr).write(interrupt);
    }

    /// Return true if the most significant byte is the first in the FIFO.
    pub fn is_msb_first(&self) -> bool {
        field_shared!(self.regs, pffcw_ctlr)
            .read()
            .contains(PostboxFifoControl::MSBF)
    }

    /// Sets the most significant byte to be the first in the FIFO.
    pub fn set_msb_first(&mut self, msb_first: bool) {
        self.modify_ctlr(|ctlr| ctlr.set(PostboxFifoControl::MSBF, msb_first));
    }

    /// Returns the Transfer Delineation Mode.
    pub fn transfer_delineation_mode(&self) -> TransferDelineationMode {
        let ctlr = field_shared!(self.regs, pffcw_ctlr).read();
        ctlr.transfer_delineation_mode().unwrap()
    }

    /// Sets the Transfer Delineation Mode.
    pub fn set_transfer_delineation_mode(&mut self, mode: TransferDelineationMode) {
        self.modify_ctlr(|ctlr| ctlr.set_transfer_delineation_mode(mode));
    }

    /// Flushes the FIFO and waits for completion.
    pub fn flush(&mut self) {
        self.modify_ctlr(|ctlr| *ctlr |= PostboxFifoControl::FF);

        let st = field_shared!(self.regs, pffcw_st);
        while !st.read().contains(PostboxFifoStatus::FF) {
            core::hint::spin_loop();
        }

        self.modify_ctlr(|ctlr| *ctlr -= PostboxFifoControl::FF);
    }

    /// Returns true if the previous push caused an error.
    pub fn previous_push_error(&self) -> bool {
        field_shared!(self.regs, pffcw_st)
            .read()
            .contains(PostboxFifoStatus::PPE)
    }

    /// Returns the available free space in the FIFO.
    pub fn free_space(&self) -> usize {
        field_shared!(self.regs, pffcw_st).read().fifo_free_space()
    }

    /// Returns whether the acknowledge counter overflowed and the acknowledge count. The function
    /// returns the count since the last call to this function, i.e. it clears the register by
    /// reading it.
    pub fn acknowledge_count(&mut self) -> (bool, usize) {
        let ack_cnt = field!(self.regs, pffcw_ack_cnt).read();

        (
            ack_cnt.contains(PostboxFifoAckCnt::ACK_CNT_OVRFLW),
            ack_cnt.acknowledge_count(),
        )
    }

    /// Returns the FIFO tidemark.
    pub fn tide(&self) -> FifoTidemark {
        field_shared!(self.regs, pffcw_tide).read()
    }

    /// Sets the FIFO tidemark.
    pub fn set_tide(&mut self, tide: FifoTidemark) {
        field!(self.regs, pffcw_tide).write(tide);
    }

    /// Writes the payload using the requested access width.
    fn write<T>(&mut self, data: T)
    where
        T: FromBytes + IntoBytes + Immutable,
    {
        // Safety: This function is only called internally with u8, u16, u32 or u64 types,
        // after checking whether pffcw_pay supports the requested access width.
        let mut payload = unsafe {
            UniqueMmioPointer::new(
                field!(self.regs, pffcw_pay)
                    .ptr_nonnull()
                    .cast::<ReadPureWrite<T>>(),
            )
        };

        payload.write(data)
    }

    /// Updates the control register with a modifier function.
    fn modify_ctlr<F>(&mut self, f: F)
    where
        F: Fn(&mut PostboxFifoControl),
    {
        let mut ctlr = field_shared!(self.regs, pffcw_ctlr).read();
        f(&mut ctlr);
        field!(self.regs, pffcw_ctlr).write(ctlr);
    }
}

/// Mailbox FIFO Channel driver.
pub struct MhuMailboxFifo<'a> {
    pub(super) regs: UniqueMmioPointer<'a, MhuMailboxFifoRegisters>,
    config: FfchCfg0,
}

impl<'a> MhuMailboxFifo<'a> {
    /// Creates new instance.
    pub fn new(regs: UniqueMmioPointer<'a, MhuMailboxFifoRegisters>, config: FfchCfg0) -> Self {
        Self { regs, config }
    }

    /// Returns the configuration of the FIFO.
    pub fn config(&self) -> FfchCfg0 {
        self.config
    }

    /// Reads a byte from the FIFO.
    pub fn read8(&mut self) -> Result<u8, Error> {
        if self.config.contains(FfchCfg0::F8BA_SPT) {
            Ok(self.read())
        } else {
            Err(Error::OperationNotSupported)
        }
    }

    /// Reads a 16-bit word from the FIFO.
    pub fn read16(&mut self) -> Result<u16, Error> {
        if self.config.contains(FfchCfg0::F16BA_SPT) {
            Ok(self.read())
        } else {
            Err(Error::OperationNotSupported)
        }
    }

    /// Reads a 32-bit word from the FIFO.
    pub fn read32(&mut self) -> Result<u32, Error> {
        if self.config.contains(FfchCfg0::F32BA_SPT) {
            Ok(self.read())
        } else {
            Err(Error::OperationNotSupported)
        }
    }

    /// Reads a 64-bit word from the FIFO.
    pub fn read64(&mut self) -> Result<u64, Error> {
        if self.config.contains(FfchCfg0::F64BA_SPT) {
            Ok(self.read())
        } else {
            Err(Error::OperationNotSupported)
        }
    }

    /// Enables/disables interrupts and the FIFO Channel interrupts to contribute to the Mailbox
    /// Combined interrupt.
    pub fn configure_interrupts(&mut self, interrupts: Option<FifoInterrupt>) {
        if let Some(interrupts) = interrupts {
            field!(self.regs, mffcw_int_en).write(interrupts);
        } else {
            field!(self.regs, mffcw_int_en).write(FifoInterrupt::empty());
            self.clear_interrupts(FifoInterrupt::all());
        }

        self.modify_ctlr(|ctlr| ctlr.set(MailboxFifoControl::MBX_COMB_EN, interrupts.is_some()));
    }

    /// Reads interrupt status.
    pub fn interrupt_status(&self) -> FifoInterrupt {
        field_shared!(self.regs, mffcw_int_st).read()
    }

    /// Clears interrupts.
    pub fn clear_interrupts(&mut self, interrupt: FifoInterrupt) {
        field!(self.regs, mffcw_int_clr).write(interrupt);
    }

    /// Return true if the most significant byte is the first in the FIFO.
    pub fn is_msb_first(&self) -> bool {
        field_shared!(self.regs, mffcw_ctlr)
            .read()
            .contains(MailboxFifoControl::MSBF)
    }

    /// Sets the most significant byte to be the first in the FIFO.
    pub fn set_msb_first(&mut self, msb_first: bool) {
        self.modify_ctlr(|ctlr| ctlr.set(MailboxFifoControl::MSBF, msb_first));
    }

    /// Flushes the FIFO and waits for completion.
    pub fn flush(&mut self) {
        self.modify_ctlr(|ctlr| *ctlr |= MailboxFifoControl::FF);

        let st = field_shared!(self.regs, mffcw_st);
        while !st.read().contains(MailboxFifoStatus::FF) {
            core::hint::spin_loop();
        }

        self.modify_ctlr(|ctlr| *ctlr -= MailboxFifoControl::FF);
    }

    /// Returns the number of valid bytes in the FIFO.
    pub fn fill_level(&self) -> usize {
        field_shared!(self.regs, mffcw_st).read().fifo_fill_level()
    }

    /// Pops bytes from the FIFO based on the configured access width.
    pub fn pop(&mut self, count: usize) -> Result<(), Error> {
        if count == 0 {
            return Ok(());
        }

        let ok = match count {
            1 => self.config.contains(FfchCfg0::F8BA_SPT),
            2 => self.config.contains(FfchCfg0::F16BA_SPT),
            4 => self.config.contains(FfchCfg0::F32BA_SPT),
            8 => self.config.contains(FfchCfg0::F64BA_SPT),
            _ => false,
        };

        if ok {
            if field!(self.regs, mffcw_ctlr)
                .read()
                .contains(MailboxFifoControl::RA_EN)
            {
                match count {
                    1 => {
                        self.read::<u8>();
                    }
                    2 => {
                        self.read::<u16>();
                    }
                    4 => {
                        self.read::<u32>();
                    }

                    8 => {
                        self.read::<u64>();
                    }
                    _ => unreachable!(),
                }
            } else {
                field!(self.regs, mffcw_fifo_pop).write(MailboxFifoPop::new(count - 1));
            }

            Ok(())
        } else {
            Err(Error::OperationNotSupported)
        }
    }

    /// Returns the FIFO tidemark.
    pub fn tide(&self) -> FifoTidemark {
        field_shared!(self.regs, mffcw_tide).read()
    }

    /// Sets the FIFO tidemark.
    pub fn set_tide(&mut self, tide: FifoTidemark) {
        field!(self.regs, mffcw_tide).write(tide);
    }

    /// Reads the payload using the requested access width.
    fn read<T>(&mut self) -> T
    where
        T: FromBytes + IntoBytes,
    {
        // Safety: This function is only called internally with u8, u16, u32 or u64 types,
        // after checking whether mffcw_pay supports the requested access width.
        let mut payload = unsafe {
            UniqueMmioPointer::new(
                field!(self.regs, mffcw_pay)
                    .ptr_nonnull()
                    .cast::<ReadOnly<T>>(),
            )
        };

        payload.read()
    }

    /// Updates the control register with a modifier function.
    fn modify_ctlr<F>(&mut self, f: F)
    where
        F: Fn(&mut MailboxFifoControl),
    {
        let mut ctlr = field_shared!(self.regs, mffcw_ctlr).read();
        f(&mut ctlr);
        field!(self.regs, mffcw_ctlr).write(ctlr);
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::define_fake_regs;

    use super::*;

    const FIFO_CONFIG: FfchCfg0 = FfchCfg0::F8BA_SPT
        .union(FfchCfg0::F16BA_SPT)
        .union(FfchCfg0::F32BA_SPT)
        .union(FfchCfg0::F64BA_SPT);

    define_fake_regs!(
        FakePostboxRegisters,
        16,
        MhuPostboxFifoRegisters,
        MhuPostboxFifo,
        FIFO_CONFIG
    );

    define_fake_regs!(
        FakeMailboxRegisters,
        16,
        MhuMailboxFifoRegisters,
        MhuMailboxFifo,
        FIFO_CONFIG
    );

    #[test]
    fn regs_size() {
        assert_eq!(0x40, size_of::<MhuPostboxFifoRegisters>());
        assert_eq!(0x40, size_of::<MhuMailboxFifoRegisters>());
    }

    #[test]
    fn config() {
        let mut regs = FakePostboxRegisters::new();
        assert_eq!(FIFO_CONFIG, regs.instance_for_test().config());

        let mut regs = FakeMailboxRegisters::new();
        assert_eq!(FIFO_CONFIG, regs.instance_for_test().config());
    }

    #[test]
    fn postbox_write() {
        let mut regs = FakePostboxRegisters::new();

        {
            let mut instance = regs.instance_for_test();
            assert!(instance.write8(0xab).is_ok());
        }

        assert_eq!(0xab, regs.reg_read(0x00));

        {
            let mut instance = regs.instance_for_test();
            assert!(instance.write16(0xabcd).is_ok());
        }

        assert_eq!(0xabcd, regs.reg_read(0x00));

        {
            let mut instance = regs.instance_for_test();
            assert!(instance.write32(0xabcdef01).is_ok());
        }

        assert_eq!(0xabcdef01, regs.reg_read(0x00));

        {
            let mut instance = regs.instance_for_test();
            assert!(instance.write64(0xabcdef01_23456789).is_ok());
        }

        assert_eq!(0x23456789, regs.reg_read(0x00));
        assert_eq!(0xabcdef01, regs.reg_read(0x04));
    }

    #[test]
    fn postbox_flags() {
        let mut regs = FakePostboxRegisters::new();

        {
            let mut instance = regs.instance_for_test();
            instance.set_flags(PostboxFifoFlag::ACK | PostboxFifoFlag::EOT);
        }

        assert_eq!(0b101, regs.reg_read(0x08));

        regs.clear();
        regs.reg_write(0x08, 0b010);

        let instance = regs.instance_for_test();
        assert_eq!(PostboxFifoFlag::SOT, instance.flags());
    }

    #[test]
    fn postbox_interrupts() {
        let mut regs = FakePostboxRegisters::new();

        {
            let mut instance = regs.instance_for_test();
            instance.configure_interrupts(Some(FifoInterrupt::TFR_ACK | FifoInterrupt::FHT));
        }

        assert_eq!(0b101, regs.reg_read(0x18));
        assert_eq!(0x01, regs.reg_read(0x20));

        {
            let mut instance = regs.instance_for_test();
            instance.configure_interrupts(None);
        }

        assert_eq!(0x8000_0007, regs.reg_read(0x14));
        assert_eq!(0x0, regs.reg_read(0x18));
        assert_eq!(0x0, regs.reg_read(0x20));

        {
            let mut instance = regs.instance_for_test();
            instance.clear_interrupts(FifoInterrupt::TFR_ACK);
        }

        assert_eq!(0b001, regs.reg_read(0x14));

        regs.reg_write(0x10, 0b101);

        let instance = regs.instance_for_test();
        assert_eq!(
            FifoInterrupt::TFR_ACK | FifoInterrupt::FHT,
            instance.interrupt_status()
        );
    }

    #[test]
    fn postbox_control() {
        let mut regs = FakePostboxRegisters::new();

        {
            let mut instance = regs.instance_for_test();
            assert!(!instance.is_msb_first());
            instance.set_msb_first(true);
            assert!(instance.is_msb_first());
        }

        assert_eq!(PostboxFifoControl::MSBF.bits(), regs.reg_read(0x20));

        regs.clear();

        {
            let mut instance = regs.instance_for_test();
            instance.set_transfer_delineation_mode(TransferDelineationMode::AutoFlag);
        }

        assert_eq!(0b10 << 2, regs.reg_read(0x20));

        let instance = regs.instance_for_test();
        assert_eq!(
            TransferDelineationMode::AutoFlag,
            instance.transfer_delineation_mode()
        );

        regs.clear();

        regs.reg_write(0x24, 1 << 31);

        {
            let mut instance = regs.instance_for_test();
            instance.flush();
        }

        assert_eq!(0, regs.reg_read(0x20));
    }

    #[test]
    fn postbox_status_and_acknowledge() {
        let mut regs = FakePostboxRegisters::new();

        regs.reg_write(0x24, 0x0001_0155);
        let instance = regs.instance_for_test();
        assert_eq!(0x155, instance.free_space());
        assert!(instance.previous_push_error());
    }

    #[test]
    fn postbox_acknowledge_count() {
        let mut regs = FakePostboxRegisters::new();

        regs.reg_write(0x28, (1 << 11) | 0x321);

        let mut instance = regs.instance_for_test();
        assert_eq!((true, 0x321), instance.acknowledge_count());
    }

    #[test]
    fn postbox_tidemark() {
        let mut regs = FakePostboxRegisters::new();
        let mut tide = FifoTidemark::new(0x123, 0x198);

        {
            let mut instance = regs.instance_for_test();
            instance.set_tide(tide);
        }

        assert_eq!(0x0123_0198, regs.reg_read(0x2c));

        tide.set_high(0x356);
        tide.set_low(0x378);

        {
            let mut instance = regs.instance_for_test();
            instance.set_tide(tide);
        }

        assert_eq!(0x0356_0378, regs.reg_read(0x2c));

        regs.clear();
        regs.reg_write(0x2c, 0x0345_0321);

        let instance = regs.instance_for_test();
        assert_eq!(0x345, instance.tide().high());
        assert_eq!(0x321, instance.tide().low());
    }

    #[test]
    fn mailbox_read() {
        let mut regs = FakeMailboxRegisters::new();

        regs.reg_write(0x00, 0xab);
        {
            let mut instance = regs.instance_for_test();
            assert_eq!(Ok(0xab), instance.read8());
        }

        regs.reg_write(0x00, 0xabcd);
        {
            let mut instance = regs.instance_for_test();
            assert_eq!(Ok(0xabcd), instance.read16());
        }

        regs.reg_write(0x00, 0xabcdef01);
        {
            let mut instance = regs.instance_for_test();
            assert_eq!(Ok(0xabcdef01), instance.read32());
        }

        regs.reg_write(0x00, 0x23456789);
        regs.reg_write(0x04, 0xabcdef01);
        {
            let mut instance = regs.instance_for_test();
            assert_eq!(Ok(0xabcdef01_23456789), instance.read64());
        }
    }

    #[test]
    fn mailbox_interrupts() {
        let mut regs = FakeMailboxRegisters::new();

        {
            let mut instance = regs.instance_for_test();
            instance.configure_interrupts(Some(FifoInterrupt::TFR_ACK | FifoInterrupt::FHT));
        }

        assert_eq!(0b101, regs.reg_read(0x18));
        assert_eq!(0x01, regs.reg_read(0x20));

        {
            let mut instance = regs.instance_for_test();
            instance.configure_interrupts(None);
        }

        assert_eq!(0x8000_0007, regs.reg_read(0x14));
        assert_eq!(0x0, regs.reg_read(0x18));
        assert_eq!(0x0, regs.reg_read(0x20));

        {
            let mut instance = regs.instance_for_test();
            instance.clear_interrupts(FifoInterrupt::TFR_ACK);
        }

        assert_eq!(0b001, regs.reg_read(0x14));

        regs.reg_write(0x10, 0b101);

        let instance = regs.instance_for_test();
        assert_eq!(
            FifoInterrupt::TFR_ACK | FifoInterrupt::FHT,
            instance.interrupt_status()
        );
    }

    #[test]
    fn mailbox_control() {
        let mut regs = FakeMailboxRegisters::new();

        {
            let mut instance = regs.instance_for_test();
            assert!(!instance.is_msb_first());
            instance.set_msb_first(true);
            assert!(instance.is_msb_first());
        }

        assert_eq!(PostboxFifoControl::MSBF.bits(), regs.reg_read(0x20));

        regs.clear();

        regs.reg_write(0x24, 1 << 31);

        {
            let mut instance = regs.instance_for_test();
            instance.flush();
        }

        assert_eq!(0, regs.reg_read(0x20));
    }

    #[test]
    fn mailbox_status() {
        let mut regs = FakeMailboxRegisters::new();

        regs.reg_write(0x24, 0x1ab);
        let instance = regs.instance_for_test();
        assert_eq!(0x1ab, instance.fill_level());
    }

    #[test]
    fn mailbox_pop_ra_en() {
        let mut regs = FakeMailboxRegisters::new();

        // Set MFFCW_CTRL.RA_EN
        regs.reg_write(0x020, 0x04);

        let mut instance = regs.instance_for_test();

        assert_eq!(Ok(()), instance.pop(0));
        assert_eq!(Ok(()), instance.pop(1));
        assert_eq!(Ok(()), instance.pop(2));
        assert_eq!(Ok(()), instance.pop(4));
        assert_eq!(Ok(()), instance.pop(8));
        assert_eq!(Err(Error::OperationNotSupported), instance.pop(3));
    }

    #[test]
    fn mailbox_pop() {
        let mut regs = FakeMailboxRegisters::new();

        assert_eq!(0x3, MailboxFifoPop::new(0x3).0);
        assert_eq!(0x7, MailboxFifoPop::new(0xf).0);

        {
            let mut instance = regs.instance_for_test();
            assert_eq!(Ok(()), instance.pop(0));
            assert_eq!(Ok(()), instance.pop(1));
        }

        assert_eq!(0x0, regs.reg_read(0x28));

        {
            let mut instance = regs.instance_for_test();
            assert_eq!(Ok(()), instance.pop(2));
        }

        assert_eq!(0x1, regs.reg_read(0x28));

        {
            let mut instance = regs.instance_for_test();
            assert_eq!(Ok(()), instance.pop(4));
        }

        assert_eq!(0x3, regs.reg_read(0x28));

        {
            let mut instance = regs.instance_for_test();
            assert_eq!(Ok(()), instance.pop(8));
        }

        assert_eq!(0x7, regs.reg_read(0x28));

        let mut instance = regs.instance_for_test();
        assert_eq!(Err(Error::OperationNotSupported), instance.pop(3));
    }

    #[test]
    fn mailbox_tidemark() {
        let mut regs = FakeMailboxRegisters::new();
        let mut tide = FifoTidemark::new(0x123, 0x198);

        {
            let mut instance = regs.instance_for_test();
            instance.set_tide(tide);
        }

        assert_eq!(0x0123_0198, regs.reg_read(0x2c));

        tide.set_high(0x356);
        tide.set_low(0x378);

        {
            let mut instance = regs.instance_for_test();
            instance.set_tide(tide);
        }

        assert_eq!(0x0356_0378, regs.reg_read(0x2c));

        regs.clear();
        regs.reg_write(0x2c, 0x0345_0321);

        let instance = regs.instance_for_test();
        assert_eq!(0x345, instance.tide().high());
        assert_eq!(0x321, instance.tide().low());
    }
}
