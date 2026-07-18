#![no_std]
#![no_main]

use panic_halt as _;
use stm32l476_pac::{Peripherals, Tim2}; /*Importing all the Peripherals from the Peripheral Access Crate generated from SVD*/
use cortex_m_rt::entry; /*Importing Entry point for the program for executing main*/
mod timer; /*Including timer.rs module containing timer-2 configurations and functions*/
mod exti; /*Including exti.rs module containing EXTI_0 configurations and functions for PC0*/
use exti::COUNTER; /*Signifies the no. of button presses*/
use timer::TOGGLE; /*Flag set upon execution of the Timer-2 ISR*/

enum Speed{ /*Defined Speed Variants for State Machine structure*/
  Slow,
  Medium,
  High,
  VeryHigh
}

#[entry] /*'entry' attributes indicates start of main function*/
fn main()->!{
  let timconfig=timer::TIM{ /*Assigning values to TIM struct variables from timer.rs*/
    psc:3999, 
    arr:1000
  };
  let dp = unsafe{Peripherals::steal()}; /*'dp' contains the REG addresses of MCU Peripherals struct from the PAC*/
  let mut cp=unsafe{cortex_m::Peripherals::steal()}; /*'cp' contains the REG addresses of Cortex-M Peripherals*/
  dp.rcc.ahb2enr().modify(|_r, w| w.gpioaen().set_bit()); /*Enabling Clock of GPIO Port-A on AHB-2 bus*/
  dp.gpioa.moder().modify(|_r, w| unsafe{ w.moder0().bits(0b01) }); //*Setting PA0 to General Purpose Output Mode */
  exti::gpio_init(&dp.rcc,&dp.gpioc); /*Initialising PC0 for EXTI0 Line*/
  exti::button_init(&dp.syscfg,&dp.exti,&mut cp.NVIC); /*Configuring EXTI0*/
  timconfig.timer_init(&dp.tim2,&dp.rcc,&mut cp.NVIC); /*Configuring Timer-2*/
  loop{
    if TOGGLE.swap(false,core::sync::atomic::Ordering::SeqCst) { /*Checking if Timer-2 ISR flag is set and atomically setting it to False*/
      Speed::state_machine( &dp.tim2); /*Deploying State-Machine method from Speed impl block*/
      dp.gpioa.odr().modify(|_r,w| unsafe{w.bits(_r.bits() ^ (1u32))}); /*Toggling PA0 (LED) according to the current delay*/
    }
  }
}

impl Speed{ /*Method implementations for Speed enum*/
  fn count()->Speed{ /*Function for assigning COUNTER value to Enum variants & returning the corresponding variant*/
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
