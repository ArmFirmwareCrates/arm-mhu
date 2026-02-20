// SPDX-FileCopyrightText: Copyright The arm-mhu Contributors.
// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::{Error, control::FchCfg0};
use safe_mmio::{
    UniqueMmioPointer, field,
    fields::{ReadPureWrite, ReadWrite},
};
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout};

/// Postbox Fast Channel register block for handling both 32-bit and 64-bit register layout as an
/// opaque type.
#[derive(Clone, Eq, FromBytes, Immutable, IntoBytes, KnownLayout, PartialEq)]
#[repr(C)]
pub struct MhuPostboxFastChannelOpaqueRegisters {
    regs: [u32; 1024],
}

/// Postbox Fast Channel 32-bit register block.
#[derive(Clone, Eq, FromBytes, Immutable, IntoBytes, KnownLayout, PartialEq)]
#[repr(C)]
pub struct MhuPostboxFastChannel32Registers {
    pfcw_pay32: [ReadPureWrite<u32>; 1024],
}

/// Postbox Fast Channel 64-bit register block.
#[derive(Clone, Eq, FromBytes, Immutable, IntoBytes, KnownLayout, PartialEq)]
#[repr(C)]
pub struct MhuPostboxFastChannel64Registers {
    pfcw_pay64: [ReadPureWrite<u64>; 512],
}

/// Mailbox Fast Channel register block for handling both 32-bit and 64-bit register layout as an
/// opaque type.
#[derive(Clone, Eq, FromBytes, Immutable, IntoBytes, KnownLayout, PartialEq)]
#[repr(C)]
pub struct MhuMailboxFastChannelOpaqueRegisters {
    regs: [u32; 1024],
}

/// Mailbox Fast Channel 32-bit register block.
#[derive(Clone, Eq, FromBytes, Immutable, IntoBytes, KnownLayout, PartialEq)]
#[repr(C)]
pub struct MhuMailboxFastChannel32Registers {
    mfcw_pay32: [ReadWrite<u32>; 1024],
}

/// Mailbox Fast Channel 64-bit register block.
#[derive(Clone, Eq, FromBytes, Immutable, IntoBytes, KnownLayout, PartialEq)]
#[repr(C)]
pub struct MhuMailboxFastChannel64Registers {
    mfcw_pay64: [ReadWrite<u64>; 512],
}

/// Postbox Fast Channel 32-bit driver.
pub struct MhuPostboxFastChannel32<'a> {
    pub(super) regs: UniqueMmioPointer<'a, MhuPostboxFastChannel32Registers>,
    config: FchCfg0,
}

impl<'a> MhuPostboxFastChannel32<'a> {
    /// Creates new instance.
    pub fn new(
        regs: UniqueMmioPointer<'a, MhuPostboxFastChannel32Registers>,
        config: FchCfg0,
    ) -> Self {
        assert_eq!(32, config.word_size());

        Self { regs, config }
    }

    /// Returns the configuration of the fast channels.
    pub fn config(&self) -> FchCfg0 {
        self.config
    }

    /// Writes a value to a fast channel.
    pub fn write_channel(&mut self, group: usize, index: usize, value: u32) -> Result<(), Error> {
        if group >= self.config.group_count() || index >= self.config.channels_per_group() {
            return Err(Error::InvalidChannelIndex);
        }

        let index = group * self.config.channels_per_group() + index;
        field!(self.regs, pfcw_pay32)
            .get(index)
            .unwrap()
            .write(value);

        Ok(())
    }
}

/// Postbox Fast Channel 64-bit driver.
pub struct MhuPostboxFastChannel64<'a> {
    pub(super) regs: UniqueMmioPointer<'a, MhuPostboxFastChannel64Registers>,
    config: FchCfg0,
}

impl<'a> MhuPostboxFastChannel64<'a> {
    /// Creates new instance.
    pub fn new(
        regs: UniqueMmioPointer<'a, MhuPostboxFastChannel64Registers>,
        config: FchCfg0,
    ) -> Self {
        assert_eq!(64, config.word_size());

        Self { regs, config }
    }

    /// Returns the configuration of the fast channels.
    pub fn config(&self) -> FchCfg0 {
        self.config
    }

    /// Writes a value to a fast channel.
    pub fn write_channel(&mut self, group: usize, index: usize, value: u64) -> Result<(), Error> {
        if group >= self.config.group_count() || index >= self.config.channels_per_group() {
            return Err(Error::InvalidChannelIndex);
        }

        let index = group * self.config.channels_per_group() + index;
        field!(self.regs, pfcw_pay64)
            .get(index)
            .unwrap()
            .write(value);

        Ok(())
    }
}

/// Opaque Postbox Fast Channel type for allowing passing either 32-bit or 64-bit Fast Channel
/// variants.
pub enum MhuPostboxFastChannel<'a> {
    FastChannel32(MhuPostboxFastChannel32<'a>),
    FastChannel64(MhuPostboxFastChannel64<'a>),
}

impl<'a> MhuPostboxFastChannel<'a> {
    /// Creates new instance.
    pub fn new(
        mut regs: UniqueMmioPointer<MhuPostboxFastChannelOpaqueRegisters>,
        config: FchCfg0,
    ) -> Self {
        match config.word_size() {
            32 => {
                const {
                    assert!(
                        size_of::<[u32; 1024]>() == size_of::<MhuPostboxFastChannel32Registers>()
                    )
                };
                let ptr = regs
                    .ptr_nonnull()
                    .cast::<MhuPostboxFastChannel32Registers>();
                assert!(ptr.is_aligned());

                // Safety: `regs` is guaranteed to be a valid pointer, and the conversion to the
                // specific type is checked for size/alignment correctness, thus `ptr` is valid.
                let regs = unsafe { UniqueMmioPointer::new(ptr) };
                let fast_channel = MhuPostboxFastChannel32::new(regs, config);
                Self::FastChannel32(fast_channel)
            }
            64 => {
                const {
                    assert!(
                        size_of::<[u32; 1024]>() == size_of::<MhuPostboxFastChannel64Registers>()
                    )
                };
                let ptr = regs
                    .ptr_nonnull()
                    .cast::<MhuPostboxFastChannel64Registers>();
                assert!(ptr.is_aligned());

                // Safety: `regs` is guaranteed to be a valid pointer, and the conversion to the
                // specific type is checked for size/alignment correctness, thus `ptr` is valid.
                let regs = unsafe { UniqueMmioPointer::new(ptr) };
                let fast_channel = MhuPostboxFastChannel64::new(regs, config);
                Self::FastChannel64(fast_channel)
            }
            _ => panic!("Invalid Fast Channel word size"),
        }
    }
}

/// Mailbox Fast Channel 32-bit driver.
pub struct MhuMailboxFastChannel32<'a> {
    pub(super) regs: UniqueMmioPointer<'a, MhuMailboxFastChannel32Registers>,
    config: FchCfg0,
}

impl<'a> MhuMailboxFastChannel32<'a> {
    /// Creates new instance.
    pub fn new(
        regs: UniqueMmioPointer<'a, MhuMailboxFastChannel32Registers>,
        config: FchCfg0,
    ) -> Self {
        assert_eq!(32, config.word_size());

        Self { regs, config }
    }

    /// Returns the configuration of the fast channels.
    pub fn config(&self) -> FchCfg0 {
        self.config
    }

    /// Reads a value from a fast channel.
    pub fn read_channel(&mut self, group: usize, index: usize) -> Result<u32, Error> {
        if group >= self.config.group_count() || index >= self.config.channels_per_group() {
            return Err(Error::InvalidChannelIndex);
        }

        let index = group * self.config.channels_per_group() + index;
        Ok(field!(self.regs, mfcw_pay32).get(index).unwrap().read())
    }
}

/// Mailbox Fast Channel 64-bit driver.
pub struct MhuMailboxFastChannel64<'a> {
    pub(super) regs: UniqueMmioPointer<'a, MhuMailboxFastChannel64Registers>,
    config: FchCfg0,
}

impl<'a> MhuMailboxFastChannel64<'a> {
    /// Creates new instance.
    pub fn new(
        regs: UniqueMmioPointer<'a, MhuMailboxFastChannel64Registers>,
        config: FchCfg0,
    ) -> Self {
        assert_eq!(64, config.word_size());

        Self { regs, config }
    }

    /// Returns the configuration of the fast channels.
    pub fn config(&self) -> FchCfg0 {
        self.config
    }

    /// Reads a value from a fast channel.
    pub fn read_channel(&mut self, group: usize, index: usize) -> Result<u64, Error> {
        if group >= self.config.group_count() || index >= self.config.channels_per_group() {
            return Err(Error::InvalidChannelIndex);
        }

        let index = group * self.config.channels_per_group() + index;
        Ok(field!(self.regs, mfcw_pay64).get(index).unwrap().read())
    }
}

/// Opaque Mailbox Fast Channel type for allowing passing either 32-bit or 64-bit Fast Channel
/// variants.
pub enum MhuMailboxFastChannel<'a> {
    FastChannel32(MhuMailboxFastChannel32<'a>),
    FastChannel64(MhuMailboxFastChannel64<'a>),
}

impl<'a> MhuMailboxFastChannel<'a> {
    /// Creates new instance.
    pub fn new(
        mut regs: UniqueMmioPointer<MhuMailboxFastChannelOpaqueRegisters>,
        config: FchCfg0,
    ) -> Self {
        match config.word_size() {
            32 => {
                const {
                    assert!(
                        size_of::<[u32; 1024]>() == size_of::<MhuMailboxFastChannel32Registers>()
                    )
                };
                let ptr = regs
                    .ptr_nonnull()
                    .cast::<MhuMailboxFastChannel32Registers>();
                assert!(ptr.is_aligned());

                // Safety: `regs` is guaranteed to be a valid pointer, and the conversion to the
                // specific type is checked for size/alignment correctness, thus `ptr` is valid.
                let regs = unsafe { UniqueMmioPointer::new(ptr) };
                let fast_channel = MhuMailboxFastChannel32::new(regs, config);
                MhuMailboxFastChannel::FastChannel32(fast_channel)
            }
            64 => {
                const {
                    assert!(
                        size_of::<[u32; 1024]>() == size_of::<MhuMailboxFastChannel64Registers>()
                    )
                };
                let ptr = regs
                    .ptr_nonnull()
                    .cast::<MhuMailboxFastChannel64Registers>();
                assert!(ptr.is_aligned());

                // Safety: `regs` is guaranteed to be a valid pointer, and the conversion to the
                // specific type is checked for size/alignment correctness, thus `ptr` is valid.
                let regs = unsafe { UniqueMmioPointer::new(ptr) };
                let fast_channel = MhuMailboxFastChannel64::new(regs, config);
                MhuMailboxFastChannel::FastChannel64(fast_channel)
            }
            _ => panic!("Invalid Fast Channel word size"),
        }
    }
}

#[cfg(test)]
mod tests {
    use core::ptr::NonNull;

    use super::*;
    use crate::tests::define_fake_regs;

    const GROUP_COUNT: usize = 2;
    const CHANNEL_PER_GROUP: usize = 4;
    const CHANNEL_COUNT: usize = GROUP_COUNT * CHANNEL_PER_GROUP;

    const CONFIG_BASE: u32 =
        ((CHANNEL_COUNT - 1) | ((GROUP_COUNT - 1) << 11) | ((CHANNEL_PER_GROUP - 1) << 16)) as u32;

    const CONFIG32: FchCfg0 = FchCfg0(CONFIG_BASE | (32 << 21));
    const CONFIG64: FchCfg0 = FchCfg0(CONFIG_BASE | (64 << 21));

    define_fake_regs!(
        FakePostboxFastChannel32Registers,
        1024,
        MhuPostboxFastChannel32Registers,
        MhuPostboxFastChannel32,
        CONFIG32
    );

    define_fake_regs!(
        FakePostboxFastChannel64Registers,
        1024,
        MhuPostboxFastChannel64Registers,
        MhuPostboxFastChannel64,
        CONFIG64
    );

    define_fake_regs!(
        FakeMailboxFastChannel32Registers,
        1024,
        MhuMailboxFastChannel32Registers,
        MhuMailboxFastChannel32,
        CONFIG32
    );

    define_fake_regs!(
        FakeMailboxFastChannel64Registers,
        1024,
        MhuMailboxFastChannel64Registers,
        MhuMailboxFastChannel64,
        CONFIG64
    );

    #[test]
    fn regs_size() {
        assert_eq!(0x1000, size_of::<MhuPostboxFastChannel32Registers>());
        assert_eq!(0x1000, size_of::<MhuPostboxFastChannel64Registers>());
        assert_eq!(0x1000, size_of::<MhuMailboxFastChannel32Registers>());
        assert_eq!(0x1000, size_of::<MhuMailboxFastChannel64Registers>());
    }

    #[test]
    fn postbox_config() {
        assert_eq!(
            CONFIG32,
            FakePostboxFastChannel32Registers::new()
                .instance_for_test()
                .config()
        );

        assert_eq!(
            CONFIG64,
            FakePostboxFastChannel64Registers::new()
                .instance_for_test()
                .config()
        );

        assert_eq!(
            CONFIG32,
            FakeMailboxFastChannel32Registers::new()
                .instance_for_test()
                .config()
        );

        assert_eq!(
            CONFIG64,
            FakeMailboxFastChannel64Registers::new()
                .instance_for_test()
                .config()
        );
    }

    #[test]
    #[should_panic]
    fn postbox_invalid_word_size() {
        // Safety: The pointer will not be dereferenced, the test only check whether `new` panics on invalid word size.
        let regs = unsafe { UniqueMmioPointer::new(NonNull::new(0x1000 as *mut _).unwrap()) };
        let config = FchCfg0(1 << 21);

        MhuPostboxFastChannel::new(regs, config);
    }

    #[test]
    fn postbox_fast_channel32_write() {
        let mut regs = FakePostboxFastChannel32Registers::new();

        {
            let mut instance = regs.instance_for_test();

            assert_eq!(
                Err(Error::InvalidChannelIndex),
                instance.write_channel(GROUP_COUNT, 0, 0)
            );

            assert_eq!(
                Err(Error::InvalidChannelIndex),
                instance.write_channel(0, CHANNEL_PER_GROUP, 0)
            );

            assert_eq!(
                Ok(()),
                instance.write_channel(GROUP_COUNT - 1, CHANNEL_PER_GROUP - 1, 0x1234_5678)
            );
        }

        assert_eq!(0x1234_5678, regs.reg_read((CHANNEL_COUNT - 1) * 4));
    }

    #[test]
    fn postbox_fast_channel64_write() {
        let mut regs = FakePostboxFastChannel64Registers::new();

        {
            let mut instance = regs.instance_for_test();

            assert_eq!(
                Err(Error::InvalidChannelIndex),
                instance.write_channel(GROUP_COUNT, 0, 0)
            );

            assert_eq!(
                Err(Error::InvalidChannelIndex),
                instance.write_channel(0, CHANNEL_PER_GROUP, 0)
            );

            assert_eq!(
                Ok(()),
                instance.write_channel(
                    GROUP_COUNT - 1,
                    CHANNEL_PER_GROUP - 1,
                    0x1234_5678_90ab_cdef
                )
            );
        }

        assert_eq!(0x90ab_cdef, regs.reg_read((CHANNEL_COUNT - 1) * 8));
        assert_eq!(0x1234_5678, regs.reg_read((CHANNEL_COUNT - 1) * 8 + 4));
    }

    #[test]
    #[should_panic]
    fn mailbox_invalid_word_size() {
        // Safety: The pointer will not be dereferenced, the test only check whether `new` panics on invalid word size.
        let regs = unsafe { UniqueMmioPointer::new(NonNull::new(0x1000 as *mut _).unwrap()) };
        let config = FchCfg0(1 << 21);

        MhuMailboxFastChannel::new(regs, config);
    }

    #[test]
    fn mailbox_fast_channel32_read() {
        let mut regs = FakeMailboxFastChannel32Registers::new();

        regs.reg_write((CHANNEL_COUNT - 1) * 4, 0x1234_5678);

        let mut instance = regs.instance_for_test();

        assert_eq!(
            Err(Error::InvalidChannelIndex),
            instance.read_channel(GROUP_COUNT, 0)
        );

        assert_eq!(
            Err(Error::InvalidChannelIndex),
            instance.read_channel(0, CHANNEL_PER_GROUP)
        );

        assert_eq!(
            Ok(0x1234_5678),
            instance.read_channel(GROUP_COUNT - 1, CHANNEL_PER_GROUP - 1)
        );
    }

    #[test]
    fn mailbox_fast_channel64_read() {
        let mut regs = FakeMailboxFastChannel64Registers::new();

        regs.reg_write((CHANNEL_COUNT - 1) * 8, 0x90ab_cdef);
        regs.reg_write((CHANNEL_COUNT - 1) * 8 + 4, 0x1234_5678);

        let mut instance = regs.instance_for_test();

        assert_eq!(
            Err(Error::InvalidChannelIndex),
            instance.read_channel(GROUP_COUNT, 0)
        );

        assert_eq!(
            Err(Error::InvalidChannelIndex),
            instance.read_channel(0, CHANNEL_PER_GROUP)
        );

        assert_eq!(
            Ok(0x1234_5678_90ab_cdef),
            instance.read_channel(GROUP_COUNT - 1, CHANNEL_PER_GROUP - 1)
        );
    }
}
