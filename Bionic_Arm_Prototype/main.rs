#![no_std]
#![no_main]
use cortex_m::singleton;
use stm32l476_pac::Peripherals;
use rtt_target::{rtt_init_print};
use cortex_m_rt::entry;
use panic_halt as _;
use core::{ffi::c_void, ptr::addr_of};
//Importing all modules
mod adc;
mod motor_pwm;
mod asm;
mod rtos_tasks;\
//Importing FreeRTOS crate
use freertos_rust::*;
use core::sync::atomic::{AtomicU32,Ordering};
//Importing EMG_READINGS Buffer containing readings from ADC Common DR
use adc::{EMG_READINGS};
use rtos_tasks::DSP_TASK_HANDLE;
//Atomic Variable for storing previous iteration values
static FILTER1:AtomicU32=AtomicU32::new(0);
static FILTER2:AtomicU32=AtomicU32::new(0);
//Allocating Heap space for RTOS kernel
#[global_allocator]
static GLOBAL:FreeRtosAllocator=FreeRtosAllocator;
#[entry]
#[allow(unreachable_code)]
fn main()->!{
    //Initialising RTT block in SRAM
    rtt_init_print!();
    //'mp' & 'cp' contains MCU & Cortex-M peripheral instances respectively
    let mp=unsafe{Peripherals::steal()};
    let mut cp=unsafe{cortex_m::Peripherals::steal()};
    //Mapping REG addresses to EMG struct variables
    let emg=adc::EMG{
      rcc: &mp.rcc,
      flash: &mp.flash,
      adc123: &mp.adc_common,
      adc: &mp.adc1,
      adc2: &mp.adc2,
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
    //Initalising MSI, GPIOC, Timer-6 (TRGO) & Timer-3
    emg.gpio_clock_inits();
    emg.timer_init();
    tim.motor_gpio_init();
    tim.timer_pwm_init();
    //Taking ownership of Timer-3 via singleton    
    let own_timer=singleton!(: stm32l476_pac::Tim3=mp.tim3).unwrap();
    //Allocating a new Task for performing DSP on sEMG readings
    let dsp_task=Task::new()
      .name("DSP_Filter") //Assigning a name to the Task
      .stack_size(512)  //Allocating stack size for the task i.e 512 bytes
      .priority(TaskPriority(1)) //Assigning Priority 1 to the task
      .start(move |_| rtos_tasks::start_filtering(addr_of!(EMG_READINGS) as *const u32,&FILTER1,&FILTER2,own_timer)); //Passing Task function
    
    //Obtaining the DSP_Task handle for Task_Notifications
    if let Ok(task)=dsp_task{
      let handle=task.raw_handle() as *mut c_void;
      DSP_TASK_HANDLE.store(handle,Ordering::Relaxed);
    };
    //Initialising ADC 1,2 & enabling sampling
    emg.adc_dma_init(&mut cp.NVIC);
    emg.adc_start(addr_of!(EMG_READINGS) as *mut u32, 4);
    //Enabling RTOS Task_Scheduler
    FreeRtosUtils::start_scheduler();
    loop{}
}