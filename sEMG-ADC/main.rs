#![no_std]
#![no_main]
//Importing Peripheral addresses from PAC
use stm32l476_pac::Peripherals;
//Importing RTT functions
use rtt_target::{rtt_init_print,rprintln};
use cortex_m_rt::entry;
use panic_halt as _;
use core::ptr::addr_of;
mod adc;
mod motor_pwm;
//Importing rust atomics
use core::sync::atomic::{AtomicU32,Ordering};
//Including EMG_READINGS buffer & PRINT flag from adc.rs
use adc::{EMG_READINGS,START_FILTERING};
static FILTER:AtomicU32=AtomicU32::new(0);
#[entry]
fn main()->!{
    //Initialising RTT block in SRAM
    rtt_init_print!();
    //'mp' contains MCU's peripheral instances
    let mp=unsafe{Peripherals::steal()};
    //'cp' contains Cortex-M peripheral instances
    let mut cp=unsafe{cortex_m::Peripherals::steal()};
    //Mapping REG addresses to EMG struct variables
    let emg=adc::EMG{
      rcc: &mp.rcc,
      flash: &mp.flash,
      adc123: &mp.adc_common,
      adc: &mp.adc1,
      dma: &mp.dma1,
      tim: &mp.tim6,
      gpio: &mp.gpioa,
    };
    //Mapping REG addresses to TIM struct variables
    let tim=motor_pwm::TIM{
      rcc:&mp.rcc,
      tim:&mp.tim3,
      gpioc:&mp.gpioc
    };
    //Initialising Flash, Clocks, GPIO pins, ADC1, DMA1_CH1 & NVIC for ISR
    emg.gpio_clock_inits();
    emg.timer_init();
    tim.led_init();
    tim.timer_pwm_init();
    emg.adc_dma_init(&mut cp.NVIC);
    emg.adc_start(addr_of!(EMG_READINGS) as *mut u16, 4);
    loop{
       //Flag set by DMA1_CH1 TC interrupt   
       if START_FILTERING.swap(false,Ordering::SeqCst){
        unsafe{
          let prev:u32=FILTER.load(Ordering::Relaxed);
          //Taking average of 4 ADC samples 
          let raw:u32= ((EMG_READINGS[0]+EMG_READINGS[1]+EMG_READINGS[2]+EMG_READINGS[3])>>2) as u32;
          //Removes DC baseline & performs full-wave rectification
          let rectified=raw.abs_diff(2048);
          //Implementing an EMA filter on the average reading to smooth the transition
          let filtered:u32= if prev==0 {
              rectified
          }else{
            ((prev * 31)+ rectified) >> 5 
          };
          //Subtracts noise floor
          let least=filtered.saturating_sub(220);
          //Storing EMA value in atomic variable
          FILTER.store(filtered,Ordering::Relaxed);
          let scaled_duty:u32=50+((least*78)/60).min(78);
          //Mapping to PWM duty cycle
          mp.tim3.ccr2().write(|w| w.bits(scaled_duty));  
          //RTT prints to terminal       
          rprintln!("{},{}",raw, scaled_duty);
        }
       }
    }
}