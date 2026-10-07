//Importing FreeRTOS crate & TIM struct 
use crate::timer::TIM; 
use freertos_rust::*;
//Function for task1 using Struct reference & Semaphores
pub fn task1(timer:&TIM<'static>, sem1:&Semaphore, sem2:&Semaphore){

    loop{
        //Blocks & yields CPU execution until task2 gives sem1
        sem1.take(Duration::infinite()).ok();
        //Increasing intensity of LED1
        for i in 0..999{
            timer.tim.ccr2().write(|w| unsafe{w.bits(i)});
            //Yeilds CPU to scheduler between step changes
            CurrentTask::delay(Duration::ms(1));
        }
        //Decreasing intensity of LED1
        for i in (0..999).rev(){
            timer.tim.ccr2().write(|w| unsafe{w.bits(i)});
            CurrentTask::delay(Duration::ms(1));
        }
        //Signals task2 that task1 is finished
        sem2.give();
    }
}
pub fn task2(timer:&TIM<'static>, sem2:&Semaphore, sem1:&Semaphore){
    loop{
        //Blocks & wait for sem2
        sem2.take(Duration::infinite()).ok();
        //Increasing intensity of LED1
        for i in 0..999{
            timer.tim.ccr1().write(|w| unsafe{w.bits(i)});
            CurrentTask::delay(Duration::ms(1));
        }
        //Decreasing intensity of LED1
        for i in (0..999).rev(){
            timer.tim.ccr1().write(|w| unsafe{w.bits(i)});
            CurrentTask::delay(Duration::ms(1));
        }  
        //Signals task2 that task1 is finished 
        sem1.give(); 
    }
}