#![no_std]
#![no_main]
use stm32l476_pac::Peripherals;
use panic_halt as _;
use cortex_m_rt::entry;
//Including Modules
mod timer;
mod task_file;
mod asm;
//Importing FreeRTOS crate
use freertos_rust::*;
use cortex_m::singleton;
use timer::TIM;
extern crate alloc;
use alloc::boxed::Box;
//Initialising RTOS allocator in heap memory
#[global_allocator]
static GLOBAL: FreeRtosAllocator=FreeRtosAllocator;
#[entry]
#[allow(unreachable_code)]
fn main()->!{
    //'mp' contains MCU peripheral instances
    let mp=unsafe{Peripherals::steal()};
    //Singleton ensures thread-safe, single instance initialisation without dynamic heap allocation
    let clock = singleton!(:stm32l476_pac::Rcc=mp.rcc).unwrap();
    let io = singleton!(:stm32l476_pac::Gpioc=mp.gpioc).unwrap();
    let tim3 = singleton!(:stm32l476_pac::Tim3=mp.tim3).unwrap();
    let timer: &'static TIM<'static>= singleton!(: TIM<'static> = TIM {
        rcc: clock,
        gpioc: io,
        tim: tim3
    }).unwrap();

    //Creating Binary Semaphores for task1 & task2
    //Box::leak prvents RUST from erasing them, also giving us a static pointer 
    let sem1:&Semaphore=Box::leak(Box::new(Semaphore::new_binary().unwrap()));
    let sem2:&Semaphore=Box::leak(Box::new(Semaphore::new_binary().unwrap()));
    //Give initial token to task1 to be executed first
    sem1.give();
    //Timer-2 & PC6 & PC7 iniitialisation
    timer.led_init();
    timer.timer_pwm_init();
    //Creating LED1 task for PC7
    Task::new()
        .name("LED1")
        .stack_size(512)
        .priority(TaskPriority(1))
        .start(move |_| task_file::task1(&timer, sem1, sem2)).unwrap();
    //Creating LED2 task for PC6
    Task::new()
        .name("LED2")
        .stack_size(512)
        .priority(TaskPriority(1))
        .start(move |_| task_file::task2(&timer, sem2, sem1)).unwrap();
    //Starting RTOS scheduler
    FreeRtosUtils::start_scheduler();
    loop{}
}