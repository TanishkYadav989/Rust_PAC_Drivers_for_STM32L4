# STM32L4 Bare-Metal Rust: sEMG ADC-DMA & PWM Servo Driver

This driver demonstrates non-blocking surface electromyography (sEMG) bio-signal acquisition using ADC1 with DMA1, triggered via TIM6 hardware TRGO, processed through DC offset removal and Exponential Moving Average (EMA) filtering to drive a PWM micro servo for prosthetic actuation.

Uses ADC1, DMA1 Channel 1, TIM6 (Trigger), and TIM3 CH2 PWM Output mapped as Alternate Functions.

## Signal Processing & Timing Calculations (48 MHz Clock)

* **TIM6 TRGO Trigger Rate (ADC Sampling):** `PSC = 319`, `ARR = 99` => 48 MHz / (320 * 100) = 1.5 kHz sampling rate (666.67 µs period).
* **TIM3 PWM Output (Servo Drive):** `PSC = 959`, `ARR = 999` => 48 MHz / (960 * 1000) = 50 Hz PWM frame (20 ms period, Edge-aligned PWM Mode 1).
* **Pulse Width Mapping:** 20 µs per step resolution, mapping filtered EMG signal levels directly to 1.0 ms - 2.56 ms duty cycles (`CCR2` range: 50 - 128).

## Key Features

* **Thread-Safe Atomic Signaling:** Uses `AtomicBool` flags and `AtomicU32` registers to guarantee memory safety across main loop and interrupt contexts without polling overhead.
* **Hardware-Triggered DMA Pipeline:** TIM6 TRGO continuously triggers ADC1 conversions at 1.5 kHz without CPU stalling, streaming 4-sample buffer frames directly to SRAM via DMA1 Channel 1 in circular mode.
* **DSP & Servo Actuation:** Averages 4 raw ADC samples, applies full-wave rectification (`abs_diff(2048)`), smooths signals via Exponential Moving Average (EMA) filtering, subtracts noise floor, and directly updates TIM3 `CCR2` for smooth PWM actuation.

## Key Takeaways

* **Ideal Application:** Real-time muscle-actuated prosthetic controls and biomedical wearable interfaces where analog sampling and motor driving are completely offloaded from software loops.
* **Execution Metric:** DMA handles 4-sample acquisition bursts automatically every 2.67 ms.

## Key Trade-offs

* **Filter Smoothing vs. Actuation Latency:** High EMA smoothing weights (31/32) eliminate muscle jitter and powerline interference but introduce minor response lag during rapid muscular flexes.
