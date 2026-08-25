#![no_std]
#![no_main]
//Importing Peripheral Access Crate (PAC)
use stm32l476_pac::Peripherals;
use panic_halt as _;
use cortex_m_rt::entry;
//Importing i2c & pwm(timer) module
mod i2c;
mod pwm;
//Flag set after every STOP bit
use i2c::DONE;
#[entry]
fn main()->!{
  //'mp' & 'cp' contains the REG instances of MCU & Cortex-M Peripherals  
  let mp=unsafe{Peripherals::steal()};
  let mut cp =unsafe{cortex_m::Peripherals::steal()};
  //Assigning address to struct variables
  let tsl=i2c::I2C{
  rcc: &mp.rcc,
  portb: &mp.gpiob,
  i2c: &mp.i2c1,
  dma: &mp.dma2
  };
  let tim=pwm::TIM{
    rcc: &mp.rcc,
    gpioc: &mp.gpioc,
    tim: &mp.tim3
  };
  //GPIO pins, Timer-3, I2C1 & DMA1 CH_6 & CH_7 initializations  
  tsl.igpio_init();
  tsl.i2c_dma_init(&mut cp.NVIC);
  tim.led_init();
  tim.timer_pwm_init();
  //Transmission for Powering-ON TSL2561
  i2c::tsl_write(&mp.i2c1,&mp.dma2,0x80,0x03,2);
  loop {
    if DONE.swap(false,core::sync::atomic::Ordering::SeqCst){
  //Event-driven state machine for sending REG configs & reading data from TSL
      i2c::state_machine(&mp.i2c1, &mp.dma2, &mp.tim3);
    }
  }
}