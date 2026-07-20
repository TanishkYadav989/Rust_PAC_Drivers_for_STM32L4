# Rust_PAC_Drivers_for_STM32L4
Rust PAC-level drivers for STM32L4 series (Cortex-M4), covering most on-chip peripherals at register level. Bare-metal, no HAL — direct register and bitfield access via the Peripheral Access Crate, with atomic operations where required by the peripheral.

# Why Rust?
Rust gives memory safety, thread/data-race safety, and prevents undefined-behavior-causing instruction reordering around volatile/atomic accesses — while still allowing the same full bare-metal, register-level control over the chip as C. No runtime cost for these guarantees; they're enforced at compile time.
# What is a PAC?
A Peripheral Access Crate is generated from a chip's SVD file (vendor-released or self-authored ). It binds every peripheral, register, and bit field into a type-safe system — each register and bit gets its own typed accessor/method, so invalid bit-field values, wrong register widths, or overwriting reserved bits become compile-time errors instead of silent runtime bugs.

Everything else follows standard bare-metal practice: reference manuals, datasheets, errata sheets, and programming manuals are used exactly as in C-based bare-metal development. The only difference is that register and bit-level access is wrapped in a type-safe layer, preventing invalid writes, incorrect bit-field access, or accidental register overwrites at compile time rather than relying on manual discipline alone.
# Toolchain
PAC generated via svd2rust from the chip's SVD file; cortex-m-rt provides the reset/vector-table runtime; memory.x defines flash/RAM layout for the linker; Cargo.toml configured for no_std, target-specific build (thumbv7em-none-eabihf); probe-rs used for flashing and on-target debugging.

*Register-level bare-metal C drivers (same peripherals/methodology, no HAL):- https://github.com/TanishkYadav989/Bare_Metal_Drivers_for_STM32L4*

*Reference :*
*https://docs.rust-embedded.org/book/*
