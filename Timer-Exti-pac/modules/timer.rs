use stm32l476_pac::Tim2; /*Brought Tim2, RCC & NVIC to scope from PAC & Cortex_m*/
use stm32l476_pac::Rcc; 
use cortex_m::peripheral::NVIC;
use::core::sync::atomic::{AtomicBool,Ordering}; /*Brought ATOMIC Booleans in scope for preventing race-conditions flag read/write*/

pub static TOGGLE:AtomicBool=AtomicBool::new(true); /*TOGGLE Flag for ISR set to true as default*/

pub struct TIM{ /*TIM struct defined with variables for Timer-2 Prescaler & Auto-Reload Value*/
   pub psc:u16,
   pub arr:u16
}
impl TIM{ /*Struct Implementations/Methods*/
    pub fn timer_init(&self,tim:&Tim2,rcc:&Rcc,nv:&mut NVIC){ /*Func. for Timer-2 Configuration*/
        rcc.apb1enr1().modify(|_r,w| w.tim2en().set_bit()); /*Enabling Clock of Timer-2 on APB-1 Bus*/

        tim.cr1().modify(|_r,w| w.cen().clear_bit().dir().set_bit()); /*Disabling Timer-2 & setting direction bit for Down-counting*/
        tim.dier().modify(|_r,w| w.uie().set_bit()); /*Enabling Update Interrupt for Timer-2 to trigger ISR*/
        tim.psc().write(|w| unsafe{w.psc().bits(self.psc)}); /*Assigning Prescaler via Struct variable to scale-down Timer frequency*/
        tim.egr().write(|w| w.ug().set_bit()); /*Update generation bit set to fetch value from shadow registers*/
        tim.sr().write(|w| unsafe{w.bits(0x0000)}); /*Clearing any stale Timer-2 Status Flags*/
        tim.arr().write(|w| unsafe{w.arr_l().bits(self.arr)}); /*Assigning Auto-Reload value via Struct variable*/
        unsafe{
            NVIC::unmask(stm32l476_pac::Interrupt::TIM2); /*Unmasking Timer-2 for the Interrupt controller*/
            nv.set_priority(stm32l476_pac::Interrupt::TIM2, 0x10); /*Asssigning 0x10 as Priority to its Interrupt*/
        }
    }
}
pub fn delay_hw(ms:u16,tim:&Tim2){ /*Func. for Non-blocking Delay*/
    tim.cnt().write(|w| unsafe{w.cnt_l().bits(ms)}); /*Assigning Counter(CNT) value via function argument*/
    tim.cr1().modify(|_r,w| w.cen().set_bit()); /*Enabling Timer-2*/
}
#[no_mangle]
pub extern "C" fn TIM2(){ /*TIM2_IRQHandler (ISR)*/
    let tim=unsafe{stm32l476_pac::Peripherals::steal()}; /*'tim' contains the REG address of MCU Peripherals struct from PAC*/
    if tim.tim2.sr().read().uif().bit_is_set(){ /*Checking if the Update interrupt flag is set (triggers on CNT underflow)*/
        tim.tim2.sr().write(|w| w.uif().clear_bit()); /*Clearing UIF flag*/
        TOGGLE.store(true,Ordering::SeqCst); /*TOGGLE is set to True atomically*/
    }
}