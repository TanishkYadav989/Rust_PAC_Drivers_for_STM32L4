//Importing NVIC, DMA2, RCC, Timer-3, I2C1, GPIO-B
use cortex_m::peripheral::NVIC;
use stm32l476_pac::{Dma2, Gpiob, I2c1, Rcc, Tim3};
//Importing a Mutex,RefCell buffer type
use cortex_m::interrupt::Mutex;
use core::cell::RefCell;
//Importing Atomic booleans, Unsigned 8-bit, 32-bit variables
use core::sync::atomic::{AtomicBool,AtomicU8,AtomicU32,Ordering};
//TX_BUFFER for sending REG address & its data
pub static TX_BUFFER:Mutex<RefCell<[u8;2]>>=Mutex::new(RefCell::new([0;2]));
//RX_BUFFER for visible + IR & RX_BUFFER for IR readings
pub static RX_BUFFER:Mutex<RefCell<[u8;2]>>=Mutex::new(RefCell::new([0;2]));
pub static RX_BUFFER_IR:Mutex<RefCell<[u8;2]>>=Mutex::new(RefCell::new([0;2]));
//Flag for ISR indicating STOP condition
pub static DONE: AtomicBool=AtomicBool::new(false); 
//Counter for state transitions
pub static CNT: AtomicU8=AtomicU8::new(1);
//Filtered data from TSL
static FILTERED_VALUE:AtomicU32=AtomicU32::new(0);
//Wrapper struct aggregating peripheral references
pub struct I2C<'a> {
    pub rcc: &'a Rcc,
    pub portb: &'a Gpiob,
    pub i2c: &'a I2c1,
    pub dma: &'a Dma2,
}
//Method block for I2C struct
impl<'a> I2C<'a> {
    pub fn igpio_init(&self) {
    //Enabling clock for GPIO-B, DMA2 & I2C1 and also resetting I2C1    
        self.rcc.ahb2enr().modify(|_r, w| w.gpioben().set_bit());
        self.rcc.ahb1enr().modify(|_r, w| w.dma2en().set_bit());
        self.rcc.apb1enr1().modify(|_r, w| w.i2c1en().set_bit());
        self.rcc.apb1rstr1().modify(|_r, w| w.i2c1rst().set_bit());
        self.rcc.apb1rstr1().modify(|_r, w| w.i2c1rst().clear_bit());
        
    //Configuring PB8 & PB9 for Alternate Function, setting open-drain & internal pull-up    
        self.portb.moder().modify(|_r, w| unsafe { w.moder8().bits(0x2).moder9().bits(0x2) });
        self.portb.otyper().modify(|_r,w| w.ot8().set_bit().ot9().set_bit());
        self.portb.pupdr().modify(|_r, w| unsafe { w.pupdr8().bits(0x1).pupdr9().bits(0x1) });
        self.portb.afrh().modify(|_r, w| unsafe { w.afrh8().bits(0x4).afrh9().bits(0x4) });
    }
    pub fn i2c_dma_init(&self, nv: &mut NVIC) {
    //Configuring I2C1 by setting TIMING REG & enabling STOP interrupt    
        self.i2c.cr1().write(|w| unsafe { w.bits(0) }); //Atomically clearing CR1
        self.i2c.cr1().modify(|_r, w| w.stopie().set_bit());
        self.i2c.timingr().write(|w| unsafe{
            w.presc().bits(0)
             .sdadel().bits(4)
             .scldel().bits(2)        
             .scll().bits(19)
             .sclh().bits(15)
        });

    //Configuring DMA2 CH_6 with Memory increment & priority as very_high
        self.dma.ccr6().write(|w| unsafe { w.bits(0) });
        self.dma.ccr6().modify(|_r, w| w.minc().set_bit());
        self.dma.ccr6().modify(|_r, w| unsafe { w.pl().bits(0x3) });
    //Configuring DMA2 CH_7 with Memory increment, Direction as read from memory & priority as high    
        self.dma.ccr7().write(|w| unsafe { w.bits(0) });
        self.dma.ccr7().modify(|_r, w| w.minc().set_bit().dir().set_bit());
        self.dma.ccr7().modify(|_r, w| unsafe { w.pl().bits(0x2) });
    //Mapping CH_6 & 7 to I2C1 TX & RX
        self.dma.cselr().modify(|_r,w| unsafe{w.c6s().bits(0x5).c7s().bits(0x5)});
    //Unmasking I2C1_EV interrupt to access the ISR & setting priority
        unsafe {
            NVIC::unmask(stm32l476_pac::Interrupt::I2C1_EV);
            nv.set_priority(stm32l476_pac::Interrupt::I2C1_EV, 0x00);
        }
    }
}
//Function for initiating I2C transmission
pub fn i2c_tx(i2c:&I2c1,dma:&Dma2, buf: *const u8, len: u8, slave_addr: u8) {
        dma.ccr7().modify(|_r, w| w.en().clear_bit());
    //Assigning address of TX data reg to CH_7 as the destination of transfer   
        dma.cpar7().write(|w| unsafe { w.bits(i2c.txdr().as_ptr() as u32) });
    //Assigning buffer address as the source of transfer       
        dma.cmar7().write(|w| unsafe { w.bits(buf as u32) });
    //Assigning no. of bytes to the DMA counter reg, then enabling DMA2 CH_7   
        dma.cndtr7().write(|w| unsafe { w.bits(len as u32) });
        dma.ccr7().modify(|_r, w| w.en().set_bit());
    //Enabling DMA request from I2C1 TX    
        i2c.cr1().modify(|_r,w| w.txdmaen().set_bit().pe().set_bit());

    //Atomic configuration of CR2 to initiate transfer as suggested in RM    
        i2c.cr2().write(|w| unsafe {
            w.sadd().bits((slave_addr as u16) << 1)
             .nbytes().bits(len)
             .autoend().set_bit()
             .start().set_bit()
        });
    }
//Function for initiating I2C reception    
pub fn i2c_rx(i2c:&I2c1,dma:&Dma2, buf: *mut u8, len: u8, slave_addr: u8) {
    dma.ccr6().modify(|_r, w| w.en().clear_bit());
    //Assigning address of RX data reg to CH_6 as the source of transfer   
    dma.cpar6().write(|w| unsafe { w.bits(i2c.rxdr().as_ptr() as u32) });
    //Assigning buffer address as the destination of transfer       
    dma.cmar6().write(|w| unsafe { w.bits(buf as u32) });
    //Assigning no. of bytes to the DMA counter reg, then enabling DMA2 CH_6   
    dma.cndtr6().write(|w| unsafe { w.bits(len as u32) });
    dma.ccr6().modify(|_r, w| w.en().set_bit());
    //Enabling DMA request from I2C1 RX    
    i2c.cr1().modify(|_r,w| w.rxdmaen().set_bit().pe().set_bit());

    //Atomic configuration of CR2 to initiate reception as suggested in RM    
    i2c.cr2().write(|w| unsafe {
        w.sadd().bits((slave_addr as u16) << 1)
         .nbytes().bits(len)
         .rd_wrn().set_bit()
         .autoend().set_bit()
         .start().set_bit()
    });
}
pub fn tsl_write(i2c:&I2c1,dma:&Dma2,reg:u8,data:u8,len:u8){
    //Safely accessing TX Mutex buffer & sending REG configuration
    cortex_m::interrupt::free(|cs|{
        let mut buffer=TX_BUFFER.borrow(cs).borrow_mut();
        let ptr=buffer.as_ptr();
        buffer[0]=reg;
        buffer[1]=data;
        i2c_tx(i2c,dma,ptr,len,0x39);
    })
}
pub fn tsl_read(i2c:&I2c1,dma:&Dma2,buff:&Mutex<RefCell<[u8;2]>> ,len:u8){
    //Safely extracting the address of RX Mutex buffer
    cortex_m::interrupt::free(|cs|{
        let mut buffer=buff.borrow(cs).borrow_mut();
        let ptr:*mut u8=buffer.as_mut_ptr();
        i2c_rx(i2c,dma,ptr,len,0x39);
    })
}
pub fn state_machine(i2c:&I2c1,dma:&Dma2,tim:&Tim3){
    //Counter based state machine
    match CNT.load(Ordering::SeqCst){
        1=>{
        //Integration time of TSL readings set to 101ms & 1x Gain    
           tsl_write(i2c,dma,0x81,0x01,2); 
           CNT.store(2,Ordering::SeqCst);
        }
        2=>{
        //Sending data register add. of Visible + IR reading  
            tsl_write(i2c,dma,0xAC,0x00,1);
            CNT.store(3,Ordering::SeqCst);
        }
        3=>{
        //Initiating read sequence for Visible + IR reading    
            tsl_read(i2c,dma,&RX_BUFFER,2);
            CNT.store(4,Ordering::SeqCst);
        }
        4=>{
        //Sending data register add. of IR reading     
            tsl_write(i2c, dma, 0xAE, 0x00, 1);
            CNT.store(5,Ordering::SeqCst);
        }
        5=>{
        //Initiating read sequence for IR reading        
            tsl_read(i2c,dma,&RX_BUFFER_IR,2);
            CNT.store(6,Ordering::SeqCst);
        }
        6=>{
        //Returning the visible value by subtraction IR from Visible + IR    
            let raw_visible = cortex_m::interrupt::free(|cs|{
                let buff1=RX_BUFFER.borrow(cs).borrow_mut();
                let buff2=RX_BUFFER_IR.borrow(cs).borrow_mut();
        //Parsing the little endian value 8-bit value into 16-bit    
                let ch0:u16=((buff1[1] as u16)<<8) | (buff1[0] as u16);
                let ch1:u16=((buff2[1] as u16)<<8) | (buff2[0] as u16);
        //'saturating_sub()' prevent wrap-around of 16-bit value if overflow occurs   
                ch0.saturating_sub(ch1)
            });
        //Keeping the Max visible value at 10000 for 1x Gain       
            let raw_visible:u32= raw_visible.min(10000) as u32;
            let prev=FILTERED_VALUE.load(Ordering::Relaxed);

            let filtered_val= if prev==0{
        //for initial value map raw_visible directly        
                raw_visible
            }    
            else{
        //Implementing EMA filter to get smoother transitions        
                ((prev * 15)+raw_visible)/16 
            };
        //Updating value of Atomic variable    
            FILTERED_VALUE.store(filtered_val,Ordering::Relaxed);
        //Mapping the filtered value to CCR (duty cycle)    
            let ccr = 1 + ((filtered_val * 1999) / 10000);
            tim.ccr2().write(|w| unsafe{w.bits(ccr as u32)});
            CNT.store(2,Ordering::SeqCst);    
            DONE.store(true,Ordering::SeqCst);
        }
        _=>{}
    } 
}
#[unsafe(no_mangle)]
//I2C_EV_IRQHandler
pub extern "C" fn I2C1_EV(){
    let mp=unsafe{stm32l476_pac::Peripherals::steal()};
    if mp.i2c1.isr().read().stopf().bit_is_set(){
    //Clearing STOP Flag & setting DONE flag,
    //indicating I2C is ready for a state transition   
        mp.i2c1.icr().write(|w| w.stopcf().set_bit());
        DONE.store(true,Ordering::SeqCst);
    }
}