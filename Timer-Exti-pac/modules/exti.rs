/*Brought EXTI, SYSCFG, RCC, GPIOC & NVIC into scope from PAC & cortex_m*/
use stm32l476_pac::Exti; 
use stm32l476_pac::Syscfg;
use stm32l476_pac::Rcc;
use stm32l476_pac::Gpioc;
use cortex_m::peripheral::NVIC;
/*Brought ATOMIC u8 in scope for preventing race-conditions with atomic variable writes*/
use core::sync::atomic::{AtomicU8,Ordering}; 
/*Global atomic variable for no. of button presses*/
pub static COUNTER:AtomicU8 = AtomicU8::new(0); 
/*Function for Initialisng PC0 for EXTI-line*/
pub fn gpio_init(rcc:&Rcc,c:&Gpioc){ 
/*Enabling Clock of GPIO Port C & SYSCFG*/
    rcc.ahb2enr().modify(|_r,w| w.gpiocen().set_bit());
    rcc.apb2enr().modify(|_r,w| w.syscfgen().set_bit()); 
/*Configuring PC0 for Input mode with 40kΩ internal pull-up*/    
    c.moder().modify(|_r,w| unsafe{w.moder0().bits(0b00)}); 
    c.pupdr().modify(|_r,w| unsafe{w.pupdr0().bits(0b01)});
}
/*Function for Configuring EXTI0 for button interrupt*/
pub fn button_init(sys:&Syscfg,ex:&Exti,nv:&mut NVIC){ 
/*Mapping EXTI0 to Port C as EXTI no. denotes pin no. and SYSCFG maps to port*/
    sys.exticr1().modify(|_r,w| unsafe{w.exti0().bits(0b0010)}); 
/*Falling edge trigger as pin goes low when button is pressed*/
    ex.ftsr1().modify(|_r,w| w.tr0().set_bit());
    ex.rtsr1().modify(|_r,w| w.tr0().clear_bit()); /*No rising edge trigger*/
/*Unmasking EXTI0*/
    ex.imr1().modify(|_r,w| w.mr0().set_bit());
/*Unmasking EXTI for the Interrupt controller with 0x00 as its priority*/
    unsafe{
        NVIC::unmask(stm32l476_pac::Interrupt::EXTI0); 
        nv.set_priority(stm32l476_pac::Interrupt::EXTI0, 0x00);
    }
}
#[no_mangle]
pub extern "C" fn EXTI0(){ /*EXTI0_IRQHandler (ISR)*/
/*'mp' contains the REG address of MCU peripherals*/    
   let mp=unsafe{stm32l476_pac::Peripherals::steal()}; 
   if mp.exti.pr1().read().pr0().bit_is_set() { /*Checking if the pending bit is set when Interrupt triggers*/     
/*Setting PR0 bit clears the flag*/       
      mp.exti.pr1().write(|w| w.pr0().set_bit()); 
/*Atomically incrementing COUNTER by 1 each time*/       
      COUNTER.fetch_add(1,Ordering::SeqCst); 
      if COUNTER.load(Ordering::SeqCst)==4 { 
/*Resets COUNTER value after completing all 4-states*/          
        COUNTER.store(0,Ordering::SeqCst); 
      }
   }
}
