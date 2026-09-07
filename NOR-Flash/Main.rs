#![no_std]
#![no_main]
//Importing MCU Peripheral instances
use stm32l476_pac::Peripherals;
use cortex_m_rt::entry;
use panic_halt as _;
//Including 'spi' driver 
mod spi_nor_flash;
//Flag indicating Erase sequence is finished
use spi_nor_flash::ERASE_DONE;
#[entry]
fn main()->!{
    //Contains MCU peripherals' instances
    let mp=unsafe{Peripherals::steal()}; 
    //Contains Cortex-M peripherals' instances
    let mut cp=unsafe{cortex_m::Peripherals::steal()}; 
    //Struct variable assignment with Peripheral addresses
    let spi_x=spi::SPI{
       rcc: &mp.rcc,
       gpioa: &mp.gpioa,
       gpiob: &mp.gpiob,
       spi: &mp.spi1,
       dma: &mp.dma1
    };
    //GPIO pins & clock configurations
    spi_x.gpio_clock_init();
    //SPI & DMA channel configurations 
    spi_x.spi_dma_init(&mut cp.NVIC);
    //Looping through Erase & Write-Read FSMs
    loop{
        if mp.spi1.sr().read().bsy().bit_is_clear() && ERASE_DONE.load(core::sync::atomic::Ordering::SeqCst)==false{
            spi_x.erase_initiate(0x00, 0xF0, 0x00);
        }
        else{
            spi_x.write_initiate(0x00, 0xF0, 0x00);
        }
    } 
}