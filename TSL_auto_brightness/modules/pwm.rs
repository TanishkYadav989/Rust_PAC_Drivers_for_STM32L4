//Importing RCC, GPIO-C & TIM3 
use stm32l476_pac::{Rcc,Gpioc,Tim3};
//Wrapper struct aggregating peripheral references
pub struct TIM<'a>{
   pub rcc: &'a Rcc,
   pub gpioc: &'a Gpioc,
   pub tim: &'a Tim3
}
//Methods block for TIM struct
impl <'a> TIM<'a>{
    pub fn led_init(&self){
    //Enabling clock of GPIO port C & Timer-3   
        self.rcc.ahb2enr().modify(|_r,w| w.gpiocen().set_bit());
        self.rcc.apb1enr1().modify(|_r,w| w.tim3en().set_bit());

    //Configuring PC7 as Alternate function for PWM output from TIM3 CH_2   
        self.gpioc.moder().modify(|_r,w| unsafe{w.moder7().bits(0x2)});
        self.gpioc.afrl().modify(|_r,w| unsafe{w.afrl7().bits(0x2)});
    }
    pub fn timer_pwm_init(&self){
    //Center-Alligned mode for PWM    
        self.tim.cr1().modify(|_r,w| unsafe {w.cms().bits(0x1)});
    //Setting PWM mode-1    
        self.tim.ccmr1_output().modify(|_r,w| unsafe{w.oc2m().bits(0x6).oc2pe().set_bit()});
    //Prescaler 0 for 4MHz clock i.e 0.25us per clock cycle
        self.tim.psc().write(|w| unsafe{w.bits(0)});
    //Auto Reload value set to 2000 as max limit    
        self.tim.arr().write(|w| unsafe{w.bits(2000)});
    //Duty cycle set to 0 (will be manipulated by TSL's reading)    
        self.tim.ccr2().write(|w| unsafe{w.bits(0)});
    //Enabling CH_2 output & Timer-3    
        self.tim.ccer().modify(|_r,w| w.cc2e().set_bit());
        self.tim.cr1().modify(|_r,w| w.cen().set_bit());
    }
} 