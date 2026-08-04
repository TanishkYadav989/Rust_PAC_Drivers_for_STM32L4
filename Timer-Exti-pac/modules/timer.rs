/*Brought Tim2, RCC & NVIC to scope from PAC & Cortex_m*/
use stm32l476_pac::Tim2; 
use stm32l476_pac::Rcc; 
use cortex_m::peripheral::NVIC;
/*Brought ATOMIC Booleans in scope for preventing race-conditions flag read/write*/
use::core::sync::atomic::{AtomicBool,Ordering}; 
/*TOGGLE Flag for ISR set to true as default*/
pub static TOGGLE:AtomicBool=AtomicBool::new(true); 
/*TIM struct defined with variables for Timer-2 Prescaler & Auto-Reload Value*/
pub struct TIM{ 
   pub psc:u16,
   pub arr:u16
}
impl TIM{ /*Struct Implementations/Methods*/
/*Func. for Timer-2 Configuration*/   
    pub fn timer_init(&self,tim:&Tim2,rcc:&Rcc,nv:&mut NVIC){ 
    /*Enabling Clock of Timer-2 on APB-1 Bus*/   
        rcc.apb1enr1().modify(|_r,w| w.tim2en().set_bit()); 
    /*Disabling Timer-2 & setting direction bit for Down-counting*/
        tim.cr1().modify(|_r,w| w.cen().clear_bit().dir().set_bit()); 
    /*Enabling Update Interrupt for Timer-2 to trigger ISR*/   
        tim.dier().modify(|_r,w| w.uie().set_bit()); 
     /*Update generation bit set to fetch value from shadow registers*/    
        tim.egr().write(|w| w.ug().set_bit());
     /*Clearing any stale Timer-2 Status Flags*/  
        tim.sr().write(|w| unsafe{w.bits(0x0000)}); 
     /*Assigning Prescaler and ARR via Struct variable to scale-down Timer frequency*/
        tim.psc().write(|w| unsafe{w.psc().bits(self.psc)}); 
        tim.arr().write(|w| unsafe{w.arr_l().bits(self.arr)}); 
      /*Unmasking Timer-2 for the Interrupt controller with 0x10 as its priority*/  
        unsafe{
            NVIC::unmask(stm32l476_pac::Interrupt::TIM2);
            nv.set_priority(stm32l476_pac::Interrupt::TIM2, 0x10); 
        }
    }
}
/*Function for Non-blocking Delay*/
pub fn delay_hw(ms:u16,tim:&Tim2){ 
/*Assigning Counter(CNT) value via function argument & enabling Timer-2*/   
    tim.cnt().write(|w| unsafe{w.cnt_l().bits(ms)}); 
    tim.cr1().modify(|_r,w| w.cen().set_bit());
}
#[no_mangle]
pub extern "C" fn TIM2(){ /*TIM2_IRQHandler (ISR)*/
/*'mp' contains the REG address of MCU Peripherals struct from PAC*/   
    let tim=unsafe{stm32l476_pac::Peripherals::steal()}; 
/*Checking & clearing if the Update interrupt flag is set (triggers on CNT underflow)*/   
    if mp.tim2.sr().read().uif().bit_is_set(){ 
        mp.tim2.sr().write(|w| w.uif().clear_bit());
/*TOGGLE is atomically set to True */       
        TOGGLE.store(true,Ordering::SeqCst); 
    }
}
