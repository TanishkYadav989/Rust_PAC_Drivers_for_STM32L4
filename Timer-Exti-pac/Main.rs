#![no_std]
#![no_main]

use panic_halt as _;
/*Importing all the Peripherals from the Peripheral Access Crate generated from SVD*/
use stm32l476_pac::{Peripherals, Tim2}; 
/*Importing Entry point for the program for executing main*/
use cortex_m_rt::entry; 
/*Including timer.rs module containing timer-2 configurations and functions*/
mod timer; 
/*Including exti.rs module containing EXTI_0 configurations and functions for PC0*/
mod exti; 
/*Signifies the no. of button presses*/
use exti::COUNTER; 
/*Flag set upon execution of the Timer-2 ISR*/
use timer::TOGGLE; 
/*Defined Speed Variants for State Machine structure*/
enum Speed{ 
  Slow,
  Medium,
  High,
  VeryHigh
}
/*'entry' attributes indicates start of main function*/
#[entry] 
fn main()->!{
  let timconfig=timer::TIM{ /*Assigning values to TIM struct variables from timer.rs*/
    psc:3999, 
    arr:1000
  };
  /*'dp' contains the REG addresses of MCU Peripherals struct from the PAC*/
  let dp = unsafe{Peripherals::steal()}; 
  /*'cp' contains the REG addresses of Cortex-M Peripherals*/
  let mut cp=unsafe{cortex_m::Peripherals::steal()}; 
  /*Enabling Clock of GPIO Port-A on AHB-2 bus*/
  dp.rcc.ahb2enr().modify(|_r, w| w.gpioaen().set_bit());
  //*Setting PA0 to General Purpose Output Mode */
  dp.gpioa.moder().modify(|_r, w| unsafe{ w.moder0().bits(0b01) }); 
  /*Initialising PC0 for EXTI0 Line*/
  exti::gpio_init(&dp.rcc,&dp.gpioc); 
   /*Configuring EXTI0*/
  exti::button_init(&dp.syscfg,&dp.exti,&mut cp.NVIC);
  /*Configuring Timer-2*/
  timconfig.timer_init(&dp.tim2,&dp.rcc,&mut cp.NVIC); 
  loop{
    /*Checking if Timer-2 ISR flag is set and atomically setting it to False*/
    if TOGGLE.swap(false,core::sync::atomic::Ordering::SeqCst) { 
      /*Deploying State-Machine method from Speed impl block*/
      Speed::state_machine(&dp.tim2); 
      /*Toggling PA0 (LED) according to the current delay*/
      dp.gpioa.odr().modify(|_r,w| unsafe{w.bits(_r.bits() ^ (1u32))}); 
    }
  }
}

impl Speed{ /*Method implementations for Speed enum*/
  /*Function for assigning COUNTER value to Enum variants & returning the corresponding variant*/
  fn count()->Speed{ 
    match COUNTER.load(core::sync::atomic::Ordering::SeqCst) { 
      0=>Speed::Slow, /*COUNTER values mapped directly to different Blink Speed*/
      1=>Speed::Medium,
      2=>Speed::High,
      3=>Speed::VeryHigh,
      _=>Speed::Slow /*Exceptions results in Slow speed*/
    }
  }
  fn state_machine(tim:&Tim2){ /*Function for assigning delay values to their respective enum variants*/
    let s=Speed::count(); /*'s' holds the current enum variant acc. to the COUNTER*/
    match s{ /*Enum variants mapped to generate specific delays corresponding to the COUNTER values*/
      Speed::Slow=>timer::delay_hw(1000,tim),
      Speed::Medium=>timer::delay_hw(500,tim),
      Speed::High=>timer::delay_hw(200,tim),
      Speed::VeryHigh=>timer::delay_hw(90,tim),
    }
  }
}
