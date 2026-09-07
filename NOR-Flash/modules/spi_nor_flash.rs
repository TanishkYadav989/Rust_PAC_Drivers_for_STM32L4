//Importing GPIOA, GPIOB, SPI1, RCC & DMA1 instances
use stm32l476_pac::{Gpioa,Gpiob,Spi1,Rcc,Dma1};
//Importing NVIC from Cortex-M
use cortex_m::peripheral::NVIC;
//Atomic Boolean & uint8_t variables for preventing Race-conditions
use core::sync::atomic::{AtomicU8,AtomicBool,Ordering};
//Fetches raw pointer 
use core::ptr::{addr_of};
pub static mut RX_BUF:[u8;260]=[0;260];
pub static mut TX_BUF:[u8;260]=[0x37;260];
//Constant op-codes for Flash commands
pub const REG_TABLE:[u8;5]=[0x06,0x05,0x20,0x02,0x03];
//For WRITE-READ state machine
pub static COUNTER:AtomicU8=AtomicU8::new(0);
//For Erase state machine
pub static COUNTER_1:AtomicU8=AtomicU8::new(0);
//Flag Guard for function calls
pub static CALL_DONE:AtomicBool=AtomicBool::new(false);
pub static ERASE_DONE:AtomicBool=AtomicBool::new(false);
//Flags for DMA Transfer complete interrupts
pub static TX_DONE:AtomicBool=AtomicBool::new(false);
pub static RX_DONE:AtomicBool=AtomicBool::new(false);

//Wrapper struct aggregating peripheral references
pub struct SPI<'a> {
    pub rcc : &'a Rcc,
    pub gpioa : &'a Gpioa,
    pub gpiob : &'a Gpiob,
    pub spi : &'a Spi1,
    pub dma : &'a Dma1, 
} 
//Struct impl block (methods)
impl<'a> SPI<'a>{
    pub fn gpio_clock_init(&self){
    //Enabling Clocks for GPIOA, GPIOB, DMA1 & SPI1 also resetting SPI1    
        self.rcc.ahb2enr().modify(|_r,w| w.gpioaen().set_bit().gpioben().set_bit());
        self.rcc.ahb1enr().modify(|_r,w| w.dma1en().set_bit());
        self.rcc.apb2enr().modify(|_r,w| w.spi1en().set_bit());
        self.rcc.apb2rstr().modify(|_r,w| w.spi1rst().set_bit());
        self.rcc.apb2rstr().modify(|_r,w| w.spi1rst().clear_bit());
    //Setting PA5, PA6 & PA7 for Alternate Functions
        self.gpioa.moder().modify(|_r,w| unsafe{
            w.moder5().bits(0x2)
             .moder6().bits(0x2)
             .moder7().bits(0x2)
        });
    //GP Output mode for PB6 for CS line    
        self.gpiob.moder().modify(|_r,w| unsafe{w.moder6().bits(0x1)});
        self.gpiob.odr().modify(|_r,w| w.odr6().set_bit());
    //PA5, PA6 & PA7 for SPI1 for SCK, MISO & MOSI line    
        self.gpioa.afrl().modify(|_r,w| unsafe{
            w.afrl5().bits(0x5)
             .afrl6().bits(0x5)
             .afrl7().bits(0x5)
        });
    }
    pub fn spi_dma_init(&self,nv:&mut NVIC){
    //Atomic clearing of Control REG 1 & 2    
        self.spi.cr1().write(|w| unsafe{w.bits(0)});
        self.spi.cr2().write(|w| unsafe{w.bits(0)});
    //Master mode & Software slave management enabled   
        self.spi.cr1().modify(|_r,w| 
            w.ssm().set_bit()
             .ssi().set_bit()
             .mstr().set_bit()
        );
    //FIFO reception threshold set for 8-bit data  
        self.spi.cr2().modify(|_r,w| unsafe{w.ds().bits(0x7).frxth().set_bit()});
    //Atomic clearing of DMA1_CH2 & CH3 Control REG   
        self.dma.ccr2().write(|w| unsafe {w.bits(0)});
        self.dma.ccr3().write(|w| unsafe {w.bits(0)});
    //Enabling Memory increment & CH_2 transfer complete interrupt    
        self.dma.ccr2().modify(|_r,w| 
            w.tcie().set_bit()
             .minc().set_bit()
        ); 
    //Priority set as very_high for CH_2           
        self.dma.ccr2().modify(|_r,w| unsafe{w.pl().bits(0x3)});
    //Enabled TC interrupt, Memory increment & DIR as read from memory    
        self.dma.ccr3().modify(|_r,w| 
            w.minc().set_bit()
             .tcie().set_bit()
             .dir().set_bit()
        );
    //Priority set as high for CH_3               
        self.dma.ccr3().modify(|_r,w| unsafe{w.pl().bits(0x2)});
    //Mapping CH_2 & 3 for SPI1    
        self.dma.cselr().modify(|_r,w| unsafe{w.c2s().bits(0x1).c3s().bits(0x1)});
    //Unmasking & setting priority for CH_2 & CH_3 interrupts    
        unsafe{
            NVIC::unmask(stm32l476_pac::Interrupt::DMA1_CH2);
            NVIC::unmask(stm32l476_pac::Interrupt::DMA1_CH3);
            nv.set_priority(stm32l476_pac::Interrupt::DMA1_CH2,0x00);
            nv.set_priority(stm32l476_pac::Interrupt::DMA1_CH3,0x10);
        }
    }
    pub fn spi_stream_enable(&self,rx_buf:*const u8,tx_buf:*const u8,len_tx:u16,len_rx:u16){ 
    //CS line pulled low     
       self.gpiob.odr().modify(|_r,w| w.odr6().clear_bit()); 
    //Assigning Data register address    
       self.dma.cpar2().write(|w| unsafe{w.bits(self.spi.dr().as_ptr() as u32)});
       self.dma.cpar3().write(|w| unsafe{w.bits(self.spi.dr().as_ptr() as u32)});
    //Assigning length of data to counter   
       self.dma.cndtr3().write(|w| unsafe{w.bits((len_tx) as u32)});
       self.dma.cndtr2().write(|w| unsafe{w.bits((len_rx) as u32)});
    //Assigning memory address of TX & RX Buffers   
       self.dma.cmar2().write(|w| unsafe{w.bits(rx_buf as u32)});
       self.dma.cmar3().write(|w| unsafe{w.bits(tx_buf as u32)});
    //Enabling DMA RX request -> Enabling both channels ->Enabling DMA TX request acc. the RM
       self.spi.cr2().modify(|_r,w| w.rxdmaen().set_bit());
       self.dma.ccr2().modify(|_r,w| w.en().set_bit());
       self.dma.ccr3().modify(|_r,w| w.en().set_bit());
       self.spi.cr2().modify(|_r,w| w.txdmaen().set_bit());
    //Enabling SPI1 at last   
       self.spi.cr1().modify(|_r,w| w.spe().set_bit());
    }
    pub fn spi_stream_disable(&self){
    //Pulling CS line to de-select the chip    
        self.gpiob.odr().modify(|_r,w| w.odr6().set_bit());
    //Disabling SPI & DMA streams acc. to RM    
        self.dma.ccr2().modify(|_r,w| w.en().clear_bit());
        self.dma.ccr3().modify(|_r,w| w.en().clear_bit());
        self.spi.cr1().modify(|_r,w| w.spe().clear_bit());
        self.spi.cr2().modify(|_r,w| w.rxdmaen().clear_bit().txdmaen().clear_bit());    
    }
    pub fn flash_write_enable(&self){
        unsafe{ 
    //First byte of transfer 0x06 for Write enable
            TX_BUF[0]=REG_TABLE[0];
        }
        self.spi_stream_enable(addr_of!(RX_BUF) as *const u8,addr_of!(TX_BUF) as *const u8,1,0);
    }
    pub fn flash_page_program(&self,msb:u8,mid:u8,lsb:u8){
        unsafe{
    //First byte as 0x02 for page program to initiate data storing           
           TX_BUF[0]=REG_TABLE[3];
    //Assigning the 24-bit sector address where the data will be stored       
           TX_BUF[1]=msb;
           TX_BUF[2]=mid;
           TX_BUF[3]=lsb;
        }
        self.spi_stream_enable(addr_of!(RX_BUF) as *const u8,addr_of!(TX_BUF) as *const u8,260,0);
    }
    pub fn flash_sector_erase(&self,msb:u8,mid:u8,lsb:u8){
        unsafe{
    //First byte as 0x20 to initiate data erase                   
           TX_BUF[0]=REG_TABLE[2];
    //Assigning the 24-bit sector address whose data will be erased          
           TX_BUF[1]=msb;
           TX_BUF[2]=mid;
           TX_BUF[3]=lsb;
        }
        self.spi_stream_enable(addr_of!(RX_BUF) as *const u8,addr_of!(TX_BUF) as *const u8,4,0);
    }
    pub fn flash_read(&self,msb:u8,mid:u8,lsb:u8){
        unsafe{
    //First byte as 0x03 to initiate data read                          
           TX_BUF[0]=REG_TABLE[4];
    //Assigning the 24-bit sector address whose data will be read       
           TX_BUF[1]=msb;
           TX_BUF[2]=mid;
           TX_BUF[3]=lsb;
        }   
        self.spi_stream_enable(addr_of!(RX_BUF) as *const u8,addr_of!(TX_BUF) as *const u8,260,260);
    }
    pub fn flash_status_check(&self){
        unsafe{
    //Sending Status register address alongwith one dummy byte        
           TX_BUF[0]=REG_TABLE[1];
           TX_BUF[1]=0x00;
        };
        self.spi_stream_enable(addr_of!(RX_BUF) as *const u8,addr_of!(TX_BUF) as *const u8,2,2); 
    }
//State machine for Writing & reading data from a sector    
    pub fn write_initiate(&self,msb:u8,mid:u8,lsb:u8){
        match COUNTER.load(Ordering::SeqCst) {
            0=>{
        //Calling function with a guard to prevent calling every iteration 
                if !CALL_DONE.swap(true,Ordering::SeqCst){
                    self.flash_write_enable();
                }
        //Checking if SPI1 is not busy & DMA1 CH_3 is done transmitting        
                if self.spi.sr().read().bsy().bit_is_clear() && TX_DONE.swap(false,Ordering::SeqCst){
                    self.spi_stream_disable();   
                    CALL_DONE.store(false,Ordering::SeqCst);
                //Incrementing counter    
                    COUNTER.store(1, Ordering::SeqCst);
                } 
            }
            1=>{
                if !CALL_DONE.swap(true,Ordering::SeqCst){
                    self.flash_page_program(msb,mid,lsb);
                }
                
                if self.spi.sr().read().bsy().bit_is_clear() && TX_DONE.swap(false,Ordering::SeqCst){
                    self.spi_stream_disable();
                    CALL_DONE.store(false,Ordering::SeqCst);
                    COUNTER.store(2, Ordering::SeqCst);
                } 
            }
            2=>{
                if !CALL_DONE.swap(true,Ordering::SeqCst){
                    self.flash_status_check();
                }
                if self.spi.sr().read().bsy().bit_is_clear() && RX_DONE.swap(false,Ordering::SeqCst){
                    self.spi_stream_disable();
                //Checking if Data write is finished then initiating a read sequence    
                    unsafe{
                        if (RX_BUF[1] & 0x03) == 0{
                           CALL_DONE.store(false,Ordering::SeqCst);
                           COUNTER.store(3, Ordering::SeqCst);  
                        } 
                //Polling the busy register until busy is cleared        
                        else{
                            CALL_DONE.store(false,Ordering::SeqCst);
                        }
                    } 
                }
            }
            3=>{
            //Initialising a Read sequence    
                if !CALL_DONE.swap(true,Ordering::SeqCst){
                    self.flash_read(0x00, 0xF0, 0x00);
                }
                if self.spi.sr().read().bsy().bit_is_clear() && RX_DONE.swap(false,Ordering::SeqCst){
                    self.spi_stream_disable();
                    CALL_DONE.store(false,Ordering::SeqCst);
                    COUNTER_1.store(0,Ordering::SeqCst);
                    ERASE_DONE.store(false, Ordering::SeqCst);
                    COUNTER.store(4, Ordering::SeqCst);  
                }
            }
            _=>{}
        }
    }
    //State machine for erasing a sector same as above
    pub fn erase_initiate(&self,msb:u8,mid:u8,lsb:u8){
        match COUNTER_1.load(Ordering::SeqCst) {
            0=>{
                if !CALL_DONE.swap(true,Ordering::SeqCst){
                    self.flash_write_enable();
                }
                if self.spi.sr().read().bsy().bit_is_clear() && TX_DONE.swap(false,Ordering::SeqCst){
                    self.spi_stream_disable();
                    CALL_DONE.store(false,Ordering::SeqCst);
                    COUNTER_1.store(1, Ordering::SeqCst);
                } 
            }
            1=>{
                if !CALL_DONE.swap(true,Ordering::SeqCst){
                    self.flash_sector_erase(msb,mid,lsb);
                }
                if self.spi.sr().read().bsy().bit_is_clear() && TX_DONE.swap(false,Ordering::SeqCst){
                    self.spi_stream_disable();
                    CALL_DONE.store(false,Ordering::SeqCst);
                    COUNTER_1.store(2, Ordering::SeqCst);
                } 
            }
            2=>{
                if !CALL_DONE.swap(true,Ordering::SeqCst){
                    self.flash_status_check();
                }
                if self.spi.sr().read().bsy().bit_is_clear() && RX_DONE.swap(false,Ordering::SeqCst){
                    self.spi_stream_disable();
                    unsafe{
                        if (RX_BUF[1] & 0x03) == 0{
                           COUNTER.store(0,Ordering::SeqCst);
                           ERASE_DONE.store(true,Ordering::SeqCst);
                           COUNTER_1.store(3, Ordering::SeqCst);  
                           CALL_DONE.store(false,Ordering::SeqCst);
                        } 
                        else{
                            CALL_DONE.store(false,Ordering::SeqCst);
                        }
                    } 
                }
            }
            _=>{}
        }
    }    
}
#[unsafe(no_mangle)]
//DMA1_CH2_IRQHandler
pub extern "C" fn DMA1_CH2(){
   let mp=unsafe{stm32l476_pac::Peripherals::steal()};
   if mp.dma1.isr().read().tcif2().bit_is_set(){ 
    //Clearing TC flag & setting RX_DONE flag
        mp.dma1.ifcr().write(|w| w.ctcif2().set_bit());
        RX_DONE.store(true,Ordering::SeqCst);
   }
}
#[unsafe(no_mangle)]
//DMA1_CH3_IRQHandler
pub extern "C" fn DMA1_CH3(){
   let mp=unsafe{stm32l476_pac::Peripherals::steal()};
 if mp.dma1.isr().read().tcif3().bit_is_set(){
    //Clearing TC flag & setting TX_DONE flag
        mp.dma1.ifcr().write(|w| w.ctcif3().set_bit());
        TX_DONE.store(true,Ordering::SeqCst);
   }
}