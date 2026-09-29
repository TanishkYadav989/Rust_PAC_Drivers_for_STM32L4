//Imoporting MCU & Cortex-M Peripherals 
use stm32l476_pac::{AdcCommon,Adc1,Rcc,Gpioa,Dma1,Flash,Tim6};
use cortex_m::{peripheral::NVIC,asm};
//Importing Rust Atomics
use core::sync::atomic::{AtomicBool,Ordering};
//Memory Address of DMA1 status & clear flag register
const DMA1_ISR: *const u32 = 0x40020000 as *const u32;
const DMA1_IFCR: *mut u32 = 0x40020004 as *mut u32;
//Global Buffer for 4-EMG Samples
pub static mut EMG_READINGS:[u16;4]=[0;4];
//Flag set by DMA1_CH1 handler when 4 samples are collected
pub static START_FILTERING:AtomicBool=AtomicBool::new(false);

//Wrapper struct aggregating peripheral references
pub struct EMG<'a>{
    pub rcc: &'a Rcc,
    pub flash: &'a Flash,
    pub gpio: &'a Gpioa,
    pub tim: &'a Tim6,
    pub adc123: &'a AdcCommon,
    pub adc: &'a Adc1,
    pub dma: &'a Dma1
}
//Struct implementation/method block
impl <'a> EMG<'a>{

   pub fn gpio_clock_inits(&self){
//Flash latency set to Two-wait states
      self.flash.acr().modify(|_r,w| unsafe{w.latency().bits(0x2)});
//MSI clock range set to 48MHz & waiting until MSIRDY flag is set
      self.rcc.cr().modify(|_r,w| unsafe{w.msirange().bits(0xB).msirgsel().set_bit()});
      while self.rcc.cr().read().msirdy().bit_is_clear(){}
//Enabling clocks of DMA1, TIM6, GPIO port-A & ADC  
      self.rcc.ahb1enr().modify(|_r,w| w.dma1en().set_bit());
      self.rcc.apb1enr1().modify(|_r,w| w.tim6en().set_bit());
      self.rcc.ahb2enr().modify(|_r,w| w.gpioaen().set_bit().adcen().set_bit());
//Selecting system clock as ADC clock source  
      self.rcc.ccipr().modify(|_r,w| unsafe{w.adcsel().bits(0x3)});
//PA0 set to analog input mode & enabling its analog switch   
      self.gpio.moder().modify(|_r,w| unsafe{w.moder0().bits(0x3)});
      self.gpio.ascr().modify(|_r,w| w.asc0().set_bit());
    }
    pub fn timer_init(&self){
//Atomically clearing CR1 & CR2 of Timer-6
       self.tim.cr1().write(|w| unsafe{w.bits(0)});
       self.tim.cr2().write(|w| unsafe{w.bits(0)});
//Configuring for timer triggered output (TRGO)     
       self.tim.cr2().modify(|_r,w| unsafe{w.mms().bits(0b010)});
//Prescaler as 319 scaling down to 150kHz       
       self.tim.psc().write(|w| unsafe{w.bits(319)});
//ARR set as 99       
       self.tim.arr().write(|w| unsafe{w.bits(99)});
//Enabling Timer-6       
       self.tim.cr1().modify(|_r,w| w.cen().set_bit());
    }

    pub fn adc_dma_init(&self,nv:&mut NVIC){
//Atomic clearing of ADC-123 Common Control register        
        self.adc123.ccr().write(|w| unsafe{w.bits(0)});
//Atomic clearing of ADC1 Control register        
        self.adc.cr().write(|w| unsafe{w.bits(0)});
//Enabling Voltage regulator & waiting for 20.833us        
        self.adc.cr().modify(|_r,w| w.advregen().set_bit());
        asm::delay(1000);
//Calibrating ADC and waiting until calibration bit cleared        
        self.adc.cr().modify(|_r,w| w.adcal().set_bit());
        while self.adc.cr().read().adcal().bit_is_set(){};

//Mapping External trigger for sampling & enabling it, enabling Overrun mode
//DMA configured for circular continuous mode & enabling DMA mode
        self.adc.cfgr().modify(|_r,w| unsafe{
            w.extsel().bits(0xD)
             .ovrmod().set_bit()
             .dmacfg().set_bit()
             .exten().bits(0x1)
             .dmaen().set_bit()
        });
//Sampling rate set as 247.5 ADC clock cycles & 1st sampling Sequence set for CH-5        
        self.adc.smpr1().modify(|_r,w| unsafe{w.smp5().bits(0b110)});
        self.adc.sqr1().modify(|_r,w| unsafe{w.sq1().bits(0x5)});
    
//DMA1 CH_1 configuration by setting
//Circular mode, Memory increment mode, Peripheral DR & Mem size set to 16-bits
//& enabling Transfer complete interrupt    
        self.dma.ccr1().write(|w| unsafe{w.bits(0)});
        self.dma.ccr1().modify(|_r,w| unsafe{
            w.circ().set_bit()
             .minc().set_bit()
             .psize().bits(0x1)
             .msize().bits(0x1)
             .tcie().set_bit()
        });
//Mapping DMA1_CH1 to ADC1        
        self.dma.cselr().modify(|_r,w| unsafe{w.c1s().bits(0)});
//Unmasking & setting interrupt priority of TC via NVIC        
        unsafe{
            NVIC::unmask(stm32l476_pac::Interrupt::DMA1_CH1);
            nv.set_priority(stm32l476_pac::Interrupt::DMA1_CH1, 0x00);
        }
    }

    pub fn adc_start(&self,buff:*mut u16,len:u32){
//Assigning Source, destination addresses & length of DMA transfer via arguments
        self.dma.cmar1().write(|w| unsafe{w.bits(buff as u32)});
        self.dma.cpar1().write(|w| unsafe{w.bits(self.adc.dr().as_ptr() as u32)});
        self.dma.cndtr1().write(|w| unsafe{w.bits(len)});
//Enabling DMA1_CH1        
        self.dma.ccr1().modify(|_r,w| w.en().set_bit());
        
//Enabling ADC & waiting until ADRDY flag is set         
        self.adc.cr().modify(|_r,w| w.aden().set_bit());
        while self.adc.isr().read().adrdy().bit_is_clear(){}
//Starts sampling by setting ADSTART Bit        
        self.adc.cr().modify(|_r,w| w.adstart().set_bit());
    }
}
#[unsafe(no_mangle)]
//DMA1_CH1 IRQHandler
pub extern "C" fn DMA1_CH1(){
    unsafe{
        let isr=DMA1_ISR.read_volatile();
//Checking if TC flag is set
        if (isr & (1u32 << 1)) != 0 {
//Flag set to initiate DSP of emg values              
            START_FILTERING.store(true,Ordering::SeqCst);
//Writing 1 to clear TC Flag            
            DMA1_IFCR.write_volatile(1u32 << 1);
        }
    }
}