// SPDX-FileCopyrightText: Copyright The arm-mhu Contributors.
// SPDX-License-Identifier: MIT OR Apache-2.0

#![no_std]
#![doc = include_str!("../README.md")]
#![deny(clippy::undocumented_unsafe_blocks)]
#![deny(unsafe_op_in_unsafe_fn)]

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

#[cfg(test)]
mod tests {
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
}
