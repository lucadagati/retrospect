(module
  ;; Import host functions
  (import "env" "gpio_read" (func $gpio_read (param i32) (result i32)))
  (import "env" "gpio_write" (func $gpio_write (param i32 i32)))
  (import "env" "delay_ms" (func $delay_ms (param i32)))
  (import "env" "log" (func $log (param i32 i32)))
  
  ;; Memory
  (memory (export "memory") 1)
  (data (i32.const 0) "Button Counter App\n")
  (data (i32.const 20) "Button pressed! Count: ")
  
  ;; Global counter
  (global $press_count (mut i32) (i32.const 0))
  
  ;; Main function
  (func (export "main") (result i32)
    (local $button_state i32)
    (local $prev_state i32)
    (local $counter i32)
    
    ;; Startup
    (call $log (i32.const 0) (i32.const 19))
    
    ;; Monitor button (GPIO pin 2)
    (loop $monitor_loop
      (local.set $button_state (call $gpio_read (i32.const 2)))
      
      ;; Detect button press (transition from 0 to 1)
      (if (i32.and
            (i32.eq (local.get $button_state) (i32.const 1))
            (i32.eq (local.get $prev_state) (i32.const 0)))
        (then
          ;; Button pressed!
          (global.set $press_count 
            (i32.add (global.get $press_count) (i32.const 1)))
          
          ;; Log press count
          (call $log (i32.const 20) (i32.const 23))
          
          ;; Toggle LED (pin 0)
          (call $gpio_write (i32.const 0) (global.get $press_count))
        )
      )
      
      (local.set $prev_state (local.get $button_state))
      (call $delay_ms (i32.const 50))
      
      ;; Run for a while
      (local.set $counter (i32.add (local.get $counter) (i32.const 1)))
      (br_if $monitor_loop (i32.lt_u (local.get $counter) (i32.const 2000)))
    )
    
    (i32.const 0)
  )
)
