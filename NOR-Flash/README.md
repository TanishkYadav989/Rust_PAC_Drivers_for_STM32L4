# STM32L4 Bare-Metal Rust: SPI NOR Flash Driver (W25Qxx)

This driver demonstrates non-blocking memory operations on SPI NOR Flash (W25Qxx) using dual state machines with DMA offloading, splitting page programs, reads, and sector erases without CPU stalling.

Uses SPI1 and DMA1 Channel 2 (TX) / Channel 3 (RX) mapped as Alternate Functions.

## SPI Timing Configuration (2 MHz Baud Rate @ 4 MHz System Clock)

* **PRESC:** `fPCLK / 2` (2 MHz Baud Rate on 4 MHz System Core Clock)
* **SPI Mode:** Mode 0 (`CPOL = 0`, `CPHA = 0`)
* **Target Address:** Sector 15 (`0x00F000`)

## Key Features

* **Thread-Safe State Sharing:** Uses Rust atomics (`Ordering::SeqCst`) to guarantee lock-free memory safety between the main loop and `DMA1_CH2` ISR contexts.
* **Non-Blocking Super-Loop FSM:** The main loop evaluates atomic completion flags and hardware status bits, executing write, read, and sector erase sequences without CPU blocking loops.
* **Hardware Status Interrogation:** Interrogates the SPI peripheral `bsy` flag and WIP (Write In Progress) bit in Status Register 1 (SR1) to advance state transitions cleanly.

## Key Takeaways

* **Ideal Application:** Non-volatile telemetry logging, parameter storage & firmware updates in real-time edge devices.
* **Execution Metric:** Eliminates up to 40 ms of CPU stalling during physical sector erase cycles.

## Key Trade-offs

* **Super-Loop Overhead vs. RTOS:** State machine flags require evaluation on each super-loop pass, trading minimal CPU execution cycles for complete deterministic RTOS independence.
