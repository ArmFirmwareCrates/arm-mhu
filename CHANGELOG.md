# Changelog

## 0.2.0

### Bugfixes

- Fix `Mhu*boxFastChannel::new` lifetimes so returned drivers cannot outlive their register storage.

### Breaking changes

- Make `Mhu*boxFastChannel::new` unsafe: callers must ensure the configuration matches the
  hardware's access width and implemented channel layout.
- Make `Mhu*boxFifo::new` and `Mhu*boxFifos::new` unsafe: callers must ensure the configuration
  matches the hardware's supported payload access widths and implemented channels.
- Updated to `safe-mmio` 0.4.0.

## 0.1.1

### Improvements

- Update `bitflags`, `num_enum`, and `safe-mmio` dependencies.

### Bugfixes

- Fix Miri integer-to-pointer warnings in fast-channel tests.

## 0.1.0

Initial MHU v3 driver with postbox and mailbox support, doorbell, FIFO, and fast channels, and
TrustZone and Realm Management Extensions support. Provides standalone component drivers and a
top-level driver with borrow, take, and split APIs.
