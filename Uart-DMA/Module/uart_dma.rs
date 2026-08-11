/*Importing USART2, RCC ,GPIOA & DMA1 from PAC*/
use stm32l476_pac::{Usart2,Rcc,Dma1,Gpioa}; 
/*Importing NVIC from cortex_m*/
use cortex_m::peripheral::NVIC; 
/*fetches raw pointer to the RX buffer*/
use core::ptr::{addr_of}; 
/*SIZE constant for Buffer & functions*/
pub const SIZE:usize=100; 
/*Common buffer for reception & echo transmission with 100 bytes*/
pub static mut RX:[u8;SIZE]=[0u8;100]; 

pub struct UART{ /*Defining struct for baud rate*/
   pub baud:u16
}
impl UART{
    pub fn uart_init(&self,rcc:&Rcc,gpioa:&Gpioa,uart:&Usart2,nv: &mut NVIC){
/*Enabling Clock of GPIO port-A, USART-2 & DMA-1*/ 
       rcc.ahb2enr().modify(|_r,w| w.gpioaen().set_bit());
       rcc.apb1enr1().modify(|_r,w| w.usart2en().set_bit());
       rcc.ahb1enr().modify(|_r,w| w.dma1en().set_bit());

/*Configuring PA2, PA3 & mapping them to USART2 TX & RX via Alternate Func. Register-Low (AFRL)*/
       gpioa.moder().modify(|_r,w| unsafe{w.moder2().bits(0b10).moder3().bits(0b10)});
/*Internal 40kΩ pull-up for active-high & prevents floating noise*/
       gpioa.pupdr().modify(|_r,w| unsafe{w.pupdr2().bits(0b01).pupdr3().bits(0b01)});
       gpioa.afrl().modify(|_r,w| unsafe{w.afrl2().bits(0x7).afrl3().bits(0x7)});

       uart.cr1().write(|w| unsafe{w.bits(0)}); /*Atomic clearing of CR1*/
/*Enabling Transmitter, Receiver & Idle line interrupt*/
       uart.cr1().modify(|_r,w| w.te().set_bit().re().set_bit().idleie().set_bit());
/*Atomic clearing of CR2 & CR3*/
       uart.cr2().write(|w| unsafe{w.bits(0)});
       uart.cr3().write(|w| unsafe{w.bits(0)});
/*Enabling USART2 DMA request when RXNE = 1(data in RDR) & TXE = 1(TDR empty)*/
       uart.cr3().modify(|_r,w| w.dmat().set_bit().dmar().set_bit());
/*Assigning Baud Rate/USARTDIV to BRR Register*/
       uart.brr().write(|w| unsafe{w.bits(self.baud as u32)}); 
       unsafe{ 
/*Unmasking USART-2 interrupt for the Interrupt Controller & assigning priority*/
          NVIC::unmask(stm32l476_pac::Interrupt::USART2);
          nv.set_priority(stm32l476_pac::Interrupt::USART2, 0x00);
       }
    }       
}
pub fn udma_tx(uart:&Usart2,dma:&Dma1,buf:*const u8,len:u8,nv:&mut NVIC){
    dma.ccr7().write(|w| unsafe{w.bits(0)}); /*Atomic clearing of CH_7 CCR*/
/*Setting Memory increment, Read-from-memory & Transfer Complete interrupt bits*/
    dma.ccr7().modify(|_r,w| w.minc().set_bit().dir().set_bit().tcie().set_bit());
    dma.ccr7().modify(|_r,w| unsafe{w.pl().bits(0b10)}); /*CH_7 priority as high*/
/*Mapping USART-2 TX (PA2) to DMA1 CH_7*/
    dma.cselr().modify(|_r,w| unsafe{w.c7s().bits(0b0010)}); 
/*Assigning address of USART2 TDR as the destination of transfer*/
    dma.cpar7().write(|w| unsafe{w.bits(uart.tdr().as_ptr() as u32)});
/*Assigning address of buffer as a func. argument as the source of transfer*/
    dma.cmar7().write(|w| unsafe{w.bits(buf as u32)});
/*Assigning length of the buffer (determines transfer complete)*/
    dma.cndtr7().write(|w| unsafe{w.bits(len as u32)});
/*Enabling DMA1 CH_7*/
    dma.ccr7().modify(|_r,w| w.en().set_bit());
    unsafe{
/*Unmasking DMA1 CH_7 interrupt for the Interrupt Controller & assigning priority*/
        NVIC::unmask(stm32l476_pac::Interrupt::DMA1_CH7);
        nv.set_priority(stm32l476_pac::Interrupt::DMA1_CH7, 0x10);
    }
/*Enabling USART-2*/
    uart.cr1().modify(|_r,w| w.ue().set_bit());
}  
pub fn udma_rx(uart:&Usart2,dma:&Dma1,buf:*const u8,len:u8){
    dma.ccr6().write(|w| unsafe{w.bits(0)}); /*Atomic clearing of CH_6 CCR*/

    dma.ccr6().modify(|_r,w| w.minc().set_bit().circ().set_bit());
/*Setting Memory increment & Circular mode to prevent buffer overflow*/
    dma.ccr6().modify(|_r,w| unsafe{w.pl().bits(0b11)}); /*CH_6 priority as very high*/
/*Mapping USART-2 RX (PA3) to DMA1 CH_6*/
    dma.cselr().modify(|_r,w| unsafe{w.c6s().bits(0b0010)});
/*Assigning address of USART2 RDR as the source of transfer*/
    dma.cpar6().write(|w| unsafe{w.bits(uart.rdr().as_ptr() as u32)});
/*Assigning address of buffer as a func. argument as the destination of transfer*/
    dma.cmar6().write(|w| unsafe{w.bits(buf as u32)});
/*Assigning length of the buffer*/
    dma.cndtr6().write(|w| unsafe{w.bits(len as u32)});
/*Enabling DMA1 CH_6*/
    dma.ccr6().modify(|_r,w| w.en().set_bit());
/*Enabling USART-2*/
    uart.cr1().modify(|_r,w| w.ue().set_bit());
}
#[no_mangle]
pub extern "C" fn USART2(){
/*'mp' & 'cp' contains the instance of MCU & Cortex-M peripherals*/    
    let mp=unsafe{stm32l476_pac::Peripherals::steal()}; 
    let mut cp=unsafe{cortex_m::Peripherals::steal()};
    let bytes:u8; 
/*Checking if IDLE flag is set*/
    if mp.usart2.isr().read().idle().bit_is_set(){
/*Clearing the IDLE flag*/
        mp.usart2.icr().write(|w| w.idlecf().set_bit());
/*Reads remaining transfer count fro CH_6 CNDTR to calculate received bytes*/
        let remaining:u8=mp.dma1.cndtr6().read().bits() as u8; 
/*'bytes' contains the total no. of bytes received*/
        bytes=(SIZE as u8)-remaining;
        if bytes > 0 {
            let last=unsafe{RX[(bytes as usize)-1]}; /*Contains the last byte received*/
/*Checking if last byte is CR or LF*/
            if last==b'\r' || last==b'\n' { 
/*Triggers echo back of received data to the terminal*/
                udma_tx(&mp.usart2,&mp.dma1,addr_of!(RX) as *const u8,bytes,&mut cp.NVIC);
            }
        }
    }
} 
#[no_mangle]
pub extern "C" fn DMA1_CH7(){ /*DMA1_CH7_IRQHandler (ISR)*/
    let mp=unsafe{stm32l476_pac::Peripherals::steal()}; /*'mp' contains the REG instances of MCU peripherals*/
/*Checking if the Transfer Complete flag for CH_7 is set*/
    if mp.dma1.isr().read().tcif7().bit_is_set(){ 
/*Clearing the TC flag via IFCR*/
        mp.dma1.ifcr().write(|w| w.ctcif7().set_bit());
/*Triggering Reception again upon echoing back*/
        udma_rx(&mp.usart2,&mp.dma1,addr_of!(RX) as *const u8,SIZE as u8);
    }
}
