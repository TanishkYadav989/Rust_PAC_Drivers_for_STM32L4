use stm32l476_pac::Exti; /*Brought EXTI, SYSCFG, RCC, GPIOC & NVIC into scope from PAC & cortex_m*/
use stm32l476_pac::Syscfg;
use stm32l476_pac::Rcc;
use stm32l476_pac::Gpioc;
use cortex_m::peripheral::NVIC;
use core::sync::atomic::{AtomicU8,Ordering}; /*Brought ATOMIC u8 in scope for preventing race-conditions with atomic variable writes*/

pub static COUNTER:AtomicU8 = AtomicU8::new(0); /*Global atomic variable for no. of button presses*/

pub fn gpio_init(rcc:&Rcc,c:&Gpioc){ /*Func. for Initialisng PC0 for EXTI-line*/
    rcc.ahb2enr().modify(|_r,w| w.gpiocen().set_bit()); /*Enabling Clock of GPIO Port C on AHB-2 bus*/
    rcc.apb2enr().modify(|_r,w| w.syscfgen().set_bit()); /*Enabling Clock of SYSCFG on APB-2 bus*/
    c.moder().modify(|_r,w| unsafe{w.moder0().bits(0b00)}); /*Setting PC0 to Input mode*/
    c.pupdr().modify(|_r,w| unsafe{w.pupdr0().bits(0b01)}); /*Internal 40kΩ pull-up for Active-High*/
}

pub fn button_init(sys:&Syscfg,ex:&Exti,nv:&mut NVIC){ /*Func. for Configuring EXTI0 for button interrupt*/
    sys.exticr1().modify(|_r,w| unsafe{w.exti0().bits(0b0010)}); /*Mapping EXTI0 to Port C as EXTI no. denotes pin no. and SYSCFG maps to port*/
    ex.ftsr1().modify(|_r,w| w.tr0().set_bit()); /*Falling edge trigger as pin goes low when button is pressed*/
    ex.rtsr1().modify(|_r,w| w.tr0().clear_bit()); /*No rising edge trigger*/
    ex.imr1().modify(|_r,w| w.mr0().set_bit()); /*Unmasking EXTI0*/
    unsafe{
        NVIC::unmask(stm32l476_pac::Interrupt::EXTI0); /*Unmasking EXTI for the Interrupt controller*/
        nv.set_priority(stm32l476_pac::Interrupt::EXTI0, 0x00); /*Asssigning 0x00 as Priority to its Interrupt*/
    }
}
#[no_mangle]
pub extern "C" fn EXTI0(){ /*EXTI0_IRQHandler (ISR)*/
   let ext0=unsafe{stm32l476_pac::Peripherals::steal()}; /*'ext0 contains the REG address of MCU peripherals*/
   if ext0.exti.pr1().read().pr0().bit_is_set() { /*Checking if the pending bit is set when Interrupt triggers*/
      ext0.exti.pr1().write(|w| w.pr0().set_bit()); /*Setting PR0 bit clears the flag*/
      COUNTER.fetch_add(1,Ordering::SeqCst); /*Atomically incrementing COUNTER by 1 each time*/
      if COUNTER.load(Ordering::SeqCst)==4 { 
        COUNTER.store(0,Ordering::SeqCst); /*Resets COUNTER value after completing all 4-states*/
      }
   }
}