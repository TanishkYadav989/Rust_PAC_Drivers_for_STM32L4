# STM32L4 Bare-Metal Rust: UART-DMA Driver

This driver demonstrates variable length buffer receptions with RX and echoing it back with TX.

Uses PA2 (USART-2 TX) & PA3 (USART-2 RX) mapped as Alternate Functions.

### Baud-Rate Formula

* **USARTDIV:** `0x1A0` $\implies$ $416$ (decimal) for $9600$ baud rate.
* **Formula:**
  $$\text{USARTDIV} = (1 + (1 + \text{OVER8})) \times \frac{4\text{ MHz}}{9600}$$
* **Oversampling Modes:**
  * $\text{OVER8} = 0 \implies$ Oversampling by 16.
  * $\text{OVER8} = 1 \implies$ Oversampling by 8. *(If oversampling by 8 is used, the last hexadecimal digit of the converted decimal value must be right-shifted by 1 bit).*

### Key Features

* **Variable Length Buffers:** Handles variable-length receptions dynamically using USART Idle Line Interrupts.
* **Dynamic Frame Calculation:** On Idle Line detection, the ISR reads the remaining DMA counter value, subtracts it from the total buffer capacity ($100$ bytes) to get the exact received byte count, and immediately triggers TX echo.

### Key Takeaways

* **Ideal Application:** This Idle Line + DMA combo is best suited for streaming continuous sensor data from one peripheral/sensor to another without CPU intervention.

### Key Trade-offs

* **Unsuitable for CLI/AT Commands:** Human typing speed is too slow. A short pause between keystrokes will trigger an Idle Line interrupt (which detects idle states every 1.04 ms), causing unnecessary ISR overhead.
* **Alternative for Interactive Shells:** For CLI commands, use an `RXNE` interrupt with a manual circular buffer instead. This avoids DMA reconfiguration overhead for single bytes while handling asynchronous human typing efficiently.
