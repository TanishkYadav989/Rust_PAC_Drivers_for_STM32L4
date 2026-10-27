# STM32L4 FreeRTOS & Bare-Metal Rust: Dual-Channel sEMG ADC-DMA & Multi-Servo Bionic Arm Controller

This firmware driver implements a real-time, deterministic surface electromyography (sEMG) acquisition and actuation pipeline on the **STM32L476RG Microcontroller** using **Bare-Metal Rust** and **FreeRTOS**. 

Transitioning from a polling/super-loop model, this architecture leverages **Dual Simultaneous ADC Mode** (Master ADC1 / Slave ADC2), hardware-triggered **TIM6 TRGO** sampling, **DMA1 Channel 1** circular buffering, zero-latency **FreeRTOS Task Notifications**, and hardware-driven **TIM3 Multi-Channel PWM** servo control.

---

## Technical Specifications & Signal Processing (48 MHz Core Clock)

### 1. Hardware Timers & Frequency Calculations
* **TIM6 TRGO Trigger Rate (ADC Sampling):**
  $$\text{PSC} = 439, \quad \text{ARR} = 99 \implies f_{\text{TRGO}} = \frac{48\text{ MHz}}{(439 + 1) \times (99 + 1)} = \frac{48\text{ MHz}}{4400} = 1.0909\text{ kHz}$$
  * **Sampling Period ($T_{\text{sample}}$):** $\approx 916.67\ \mu\text{s}$ per dual conversion.
* **DMA Transfer Complete Interrupt Rate:**
  * DMA circular buffer acquires **4 packed 32-bit frames** (holding 4 samples of ADC1 & ADC2 simultaneously).
  * **Interrupt Interval ($T_{\text{TC}}$):** $4 \times 916.67\ \mu\text{s} = 3.667\text{ ms}$ (triggers the DSP Task notification at $\approx 272.7\text{ Hz}$).
* **TIM3 PWM Output (Multi-Servo Actuation):**
  $$\text{PSC} = 959, \quad \text{ARR} = 999 \implies f_{\text{PWM}} = \frac{48\text{ MHz}}{(959 + 1) \times (999 + 1)} = 50\text{ Hz}\ (20\text{ ms frame})$$
* **Pulse Width & Duty Cycle Resolution:**
  * Resolution per count: $20\ \mu\text{s}$.
  * Mapped Duty Cycle Range (`CCR1–CCR4`): **50 to 132** (corresponding to **1.0 ms to 2.64 ms** pulse width across $0^\circ$ to $180^\circ$ servo extension).

---

## Architectural Enhancements over Bare-Metal Super-Loop

### 1. FreeRTOS Task Notification Pipeline
Instead of polling atomic flags in a bare-metal `loop {}`, the **DMA1 Channel 1 Transfer Complete ISR** (`DMA1_CH1`) directly signals the DSP task using lightweight RTOS task notifications (`freertos_rs_task_notify_isr`). This unblocks `DSP_Filter` Task instantly with zero scheduler overhead.

### 2. Dual Simultaneous ADC Mode (Master/Slave)
* **Master (ADC1 - Channel 5 / PA0):** Captures main flexor muscle sEMG signals.
* **Slave (ADC2 - Channel 6 / PA1):** Captures secondary flexor/extensor muscle sEMG signals.
* **Hardware Interleaved Packing:** Data from both ADCs is packed into the 32-bit `ADC123->CDR` (Common Data Register), where the lower 16 bits contain ADC1 data and the upper 16 bits contain ADC2 data.

### 3. Digital Signal Processing (DSP) Chain
Unpacks raw 32-bit dual ADC frames, performs a 4-point block moving average, applies full-wave rectification, runs an Exponential Moving Average (EMA) filter for smoothing, performs noise floor subtraction, and directly maps the resulting value to TIM3 Capture/Compare Registers (CCR) for servo actuation.

---

## Key Features

* **Deterministic Real-Time Execution:** Zero CPU polling; hardware TIM6 triggers ADC conversions, DMA handles memory transfer, and FreeRTOS schedules execution only when new data frames arrive.
* **Thread-Safe Synchronization:** Uses FreeRTOS native notification primitives and atomic primitives (`AtomicU32`, `AtomicPtr`) to pass state safely between interrupt handlers and task execution contexts.
* **Multi-Channel Servo Support:** Simultaneously drives up to 4 hardware PWM channels (`TIM3_CH1` to `TIM3_CH4`) for individual finger and thumb actuation on the arm.

---

## Key Takeaways

* **Ideal Application:** Real-time, bio-signal actuated prosthetic control and wearable robotics requiring microsecond determinism and isolated high-power peripheral actuation.
* **Execution Metric:** DMA handles 4-sample dual acquisition bursts automatically every **3.67 ms**, notifying the DSP pipeline without stalling core processing tasks.

---

## Key Trade-offs

* **Filter Smoothing vs. Actuation Latency:** High EMA smoothing weights ($\alpha = 1/32$) eliminate muscle twitching, EMI noise, and powerline interference, but introduce minor phase lag during ultra-fast muscular flexes.
