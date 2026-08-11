#![no_std]
#![no_main]
/*Importing MCU & Cortex-M 'Peripherals' instances from PAC*/
use stm32l476_pac::Peripherals; 
use cortex_m::Peripherals as nv; 
/*Importing 'entry' attribute*/
use cortex_m_rt::entry; 
use panic_halt as _;
/*Including uart_dma module*/
mod uart_dma;
/*Common buffer for reception & echo transmission*/ 
use uart_dma::RX; 
use uart_dma::SIZE; /*Size of the buffer i.e 100 bytes*/
/*For fetching raw pointer to RX buffer*/
use core::ptr::{addr_of}; 
#[entry]
fn main()->!{
/*'dp' contains the instance address of MCU peripherals*/
    let dp=unsafe{Peripherals::steal()}; 
/*'cp' contains the instance of Cortex-M peripherals*/    
    let mut cp=unsafe{nv::steal()}; 
/*9600 baud rate with 4MHz clock & Oversampling by 16x*/    
    let uart1=uart_dma::UART{ 
       baud:0x1A0 /*USARTDIV*/
    };
/*Configuring PA2, PA3, USART2 & NVIC*/
    uart1.uart_init(&dp.rcc,&dp.gpioa,&dp.usart2,&mut cp.NVIC); 
/*Configuring DMA CH6 for RX & initialising reception*/
    uart_dma::udma_rx(&dp.usart2,&dp.dma1,addr_of!(RX) as *const u8, SIZE as u8);
    loop{}
}