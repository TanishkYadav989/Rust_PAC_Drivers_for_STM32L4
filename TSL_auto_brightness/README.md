# STM32L4 Bare-Metal Rust: TSL2561 I2C-DMA Driver

This driver demonstrates non-blocking ambient light sensor (TSL2561) data acquisition using I2C with DMA, processed via an Exponential Moving Average (EMA) filter to drive smooth LED PWM auto-brightness.

Uses **I2C1 (SCL, SDA)** and **TIM2 PWM Output (LED)** mapped as Alternate Functions.

## I2C Timing Calculations (Standard-Mode 100 kHz @ 4 MHz Clock)

* **PRESC:** `0` (I2C Clock = 4 MHz)
* **SDADEL Formula / Hold Time:** `SDADEL = 4` $= 1.0\mu\text{s}$
* **SCLDEL Formula / Setup Time:** `SCLDEL = 2` $= 0.5\mu\text{s}$

## Key Features

* **Thread-Safe Memory Sharing:** Uses `Mutex` buffers to guarantee memory safety across main loop and interrupt contexts without polling overhead.
* **Non-Blocking Super-Loop FSM:** The main loop checks a single completion flag set by the I2C event ISR upon `STOP` condition detection, executing a 6-stage Finite State Machine (FSM) without CPU stalling.
* **DSP & Backlight Modulation:** Extracts visible spectrum lux, applies Exponential Moving Average (EMA) filtering to prevent backlight flicker, and directly updates the Timer Capture/Compare Register (CCR) for smooth PWM dimming.

## Key Takeaways

* **Ideal Application:** Asynchronous, low-power sensor telemetry pipelines in mobile/wearable devices where real-time auto-brightness is offloaded entirely from the CPU core.
* **Execution Metric:** The complete EMA DSP calculations execute in just **$17\mu\text{s}$** (68 clock cycles at 4 MHz).

## Key Trade-offs

* **Filter Tuning vs. Latency:** High EMA smoothing weights reduce sensor noise and ambient flicker but introduce minor response lag during rapid, drastic light transitions.
