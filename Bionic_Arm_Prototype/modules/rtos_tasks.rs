//Importing FreeRTOS crate, Tim3 & RTT 
use freertos_rust::*;
use stm32l476_pac::Tim3;
use rtt_target::rprintln;
use core::{sync::atomic::{AtomicU32,AtomicPtr,Ordering},slice::from_raw_parts};
//DSP_TASK_HANDLE declared as an Atomic c_void pointer 
pub static DSP_TASK_HANDLE:AtomicPtr<core::ffi::c_void>=AtomicPtr::new(core::ptr::null_mut());
pub fn start_filtering(buff_ptr: *const u32, filter1:&AtomicU32, filter2:&AtomicU32, tim:&Tim3){
   //Converting buff_ptr into a buffer to fetch index 
   let buff=unsafe{from_raw_parts(buff_ptr, 4)}; 
   loop{ 
    //Suspending Task until notified
    let notification_val=CurrentTask::take_notification(true, Duration::infinite());
    //Checking if Notification occured
       if notification_val>0{
        //Obtaining previous filtered reading
        let prev1:u32=filter1.load(Ordering::Relaxed);
        let prev2:u32=filter2.load(Ordering::Relaxed);
    
        //Unpacking readings of ADC 1,2 & averaging them 
        let raw1:u32= ((buff[0] & 0xFFFF)+(buff[1] & 0xFFFF)+(buff[2] & 0xFFFF)+(buff[3] & 0xFFFF)) >> 2;
        let raw2:u32= ((buff[0]>>16)+(buff[1]>>16)+(buff[2]>>16)+(buff[3]>>16)) >> 2;

        //Performing Full-wave Rectification on AC reading and converting to DC
        let rectified1=raw1.abs_diff(2048);
        let rectified2=raw2.abs_diff(2048);

        //Implementing EMA filter on the Rectified value
        let filtered1:u32= if prev1==0 {
              rectified1
          }else{
              ((prev1 * 31)+ rectified1) >> 5 
        };
        let filtered2:u32= if prev2==0 {
              rectified2
          }else{
              ((prev2 * 31)+ rectified2) >> 5 
        };
        
       //Subtracting Noise floor
        let least1=filtered1.saturating_sub(220);
        let least2=filtered2.saturating_sub(220);
 
        //Storing current filtered values
        filter1.store(filtered1,Ordering::Relaxed);
        filter2.store(filtered2,Ordering::Relaxed);

        //Converting into Duty cycle with 50 baseline
        let scaled_duty1:u32=50+((least1*82)/60).min(82);
        let scaled_duty2:u32=50+((least2*82)/60).min(82);
  
        //Printing to terminal using RTT  
        rprintln!("{},{}",scaled_duty1,scaled_duty2);

        //Mapping scaled_duty1 to PWM output connected to motors
        tim.ccr1().write(|w| unsafe{w.bits(scaled_duty1)});
        tim.ccr2().write(|w| unsafe{w.bits(scaled_duty1)});
        tim.ccr3().write(|w| unsafe{w.bits(scaled_duty1)});
        tim.ccr4().write(|w| unsafe{w.bits(scaled_duty1)});
      } 
    }
}