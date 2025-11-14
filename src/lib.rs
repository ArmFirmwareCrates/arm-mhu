// SPDX-FileCopyrightText: Copyright The arm-mhu Contributors.
// SPDX-License-Identifier: MIT OR Apache-2.0

#![no_std]
#![doc = include_str!("../README.md")]
#![deny(clippy::undocumented_unsafe_blocks)]
#![deny(unsafe_op_in_unsafe_fn)]

/// Message Handling Unit Architecture version 3.0 driver.
pub mod mhu_v3;

/// MHU error type.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    UnsupportedMhuVersion,
}

#[cfg(test)]
mod tests {
    macro_rules! define_fake_regs {
        ($name:ident, $count:literal, $regs:ty, $driver:tt) => {
            pub struct $name {
                regs: [u32; $count],
            }

            impl $name {
                pub fn new() -> Self {
                    Self {
                        regs: [0u32; $count],
                    }
                }
                pub fn clear(&mut self) {
                    self.regs.fill(0);
                }

                pub fn reg_write(&mut self, offset: usize, value: u32) {
                    self.regs[offset / 4] = value;
                }

                pub fn reg_read(&self, offset: usize) -> u32 {
                    self.regs[offset / 4]
                }

                fn get(&mut self) -> UniqueMmioPointer<'_, $regs> {
                    UniqueMmioPointer::from(transmute_mut!(&mut self.regs))
                }

                pub fn instance_for_test(&mut self) -> $driver<'_> {
                    <$driver>::new(self.get())
                }
            }
        };
    }

    pub(crate) use define_fake_regs;
}
