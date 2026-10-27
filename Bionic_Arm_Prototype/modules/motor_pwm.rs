//Importing required MCU peripherals
use stm32l476_pac::{Rcc,Gpioc,Tim3};
//Wrapper struct aggregating peripheral references
pub struct TIM<'a>{
   pub rcc: &'a Rcc,
   pub gpioc: &'a Gpioc,
   pub tim: &'a Tim3

}
//Methods block for TIM struct
impl <'a> TIM<'a>{
    pub fn motor_gpio_init(&self){
    //Enabling clock of GPIO port C & Timer-3   
        self.rcc.ahb2enr().modify(|_r,w| w.gpiocen().set_bit());
        self.rcc.apb1enr1().modify(|_r,w| w.tim3en().set_bit());

    //Configuring PC6, PC7, PC8 & PC9 as Alternate function for PWM output from TIM3   
        self.gpioc.moder().modify(|_r,w| unsafe{
            w.moder7().bits(0x2)
             .moder6().bits(0x2)
             .moder8().bits(0x2)
             .moder9().bits(0x2)
        });
        self.gpioc.afrl().modify(|_r,w| unsafe{w.afrl7().bits(0x2).afrl6().bits(0x2)});
        self.gpioc.afrh().modify(|_r,w| unsafe{w.afrh8().bits(0x2).afrh9().bits(0x2)});
    }
    pub fn timer_pwm_init(&self){
    //Edge-Alligned mode for PWM    
        self.tim.cr1().modify(|_r,w| unsafe {w.cms().bits(0)});
    //Setting PWM mode-1    
        self.tim.ccmr1_output().modify(|_r,w| unsafe{
            w.oc2m().bits(0x6).oc2pe().set_bit()
             .oc1m().bits(0x6).oc1pe().set_bit()
        });
        self.tim.ccmr2_output().modify(|_r,w| unsafe{
            w.oc3m().bits(0x6).oc3pe().set_bit()
             .oc4m().bits(0x6).oc4pe().set_bit()
        });
    //Prescaler 0 for 48MHz clock i.e 20us per clock cycle
        self.tim.psc().write(|w| unsafe{w.bits(959)});
    //Auto Reload value set to 1000 
        self.tim.arr().write(|w| unsafe{w.bits(999)});
    //Duty cycle of all channels set to 0 (mapped to sEMG reading)    
        self.tim.ccr1().write(|w| unsafe{w.bits(0)});
        self.tim.ccr2().write(|w| unsafe{w.bits(0)});
        self.tim.ccr3().write(|w| unsafe{w.bits(0)});
        self.tim.ccr4().write(|w| unsafe{w.bits(0)});
    //Enabling all channel output & reversing their polarity   
        self.tim.ccer().modify(|_r,w| w
            .cc1e().set_bit().cc1p().set_bit().cc1np().clear_bit()
            .cc2e().set_bit().cc2p().set_bit().cc2np().clear_bit()
            .cc3e().set_bit().cc3p().set_bit().cc3np().clear_bit()
            .cc4e().set_bit().cc4p().set_bit().cc4np().clear_bit()
        );
    //Enabling TIM-3    
        self.tim.cr1().modify(|_r,w| w.cen().set_bit());
    }
}