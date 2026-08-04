# STM32L4 Bare-Metal Rust: EXTI & TIM2 Blinker

This is a driver controlling PA0 (LED) blink speed via PC0 (EXTI0 button interrupt) and TIM2 hardware downcounting.

### 4MHz Clock & Timer Formula
* **Prescaler (PSC):** $3999 \implies f_{\text{TIM}} = \frac{4\text{ MHz}}{4000} = 1\text{ kHz}$ ($1\text{ tick} = 1\text{ ms}$).
* **Delay Logic:** Writing $N$ to `TIM2->CNT` creates an exact $N\text{ ms}$ downcount delay until the Update Interrupt (`UIF`) fires.

### Speed States

| State | Mode | CNT Value | Delay |
| :---: | :---: | :---: | :---: |
| 0 | Slow | 1000 | 1000 ms |
| 1 | Medium | 500 | 500 ms |
| 2 | High | 200 | 200 ms |
| 3 | VeryHigh | 90 | 90 ms |

### Key Features
* **Non-blocking:** Reloads `CNT` directly without blocking loop delays.
* **Thread-safe:** Atomic flags (`AtomicBool`, `AtomicU8`) share state cleanly between ISRs and main.
