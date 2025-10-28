(module
  ;; Import host function to control LED
  (import "env" "led_set" (func $led_set (param i32 i32)))
  (import "env" "delay_ms" (func $delay_ms (param i32)))
  (import "env" "log" (func $log (param i32 i32)))
  
  ;; Memory for string data
  (memory (export "memory") 1)
  (data (i32.const 0) "LED Blinker started\n")
  (data (i32.const 20) "LED ON\n")
  (data (i32.const 27) "LED OFF\n")
  
  ;; Main function - blink LED forever
  (func (export "main") (result i32)
    (local $counter i32)
    
    ;; Log startup message
    (call $log (i32.const 0) (i32.const 20))
    
    ;; Blink loop
    (loop $blink_loop
      ;; Turn LED ON (pin 0, value 1)
      (call $led_set (i32.const 0) (i32.const 1))
      (call $log (i32.const 20) (i32.const 7))
      (call $delay_ms (i32.const 500))
      
      ;; Turn LED OFF (pin 0, value 0)
      (call $led_set (i32.const 0) (i32.const 0))
      (call $log (i32.const 27) (i32.const 8))
      (call $delay_ms (i32.const 500))
      
      ;; Increment counter
      (local.set $counter (i32.add (local.get $counter) (i32.const 1)))
      
      ;; Continue blinking (infinite loop for demo)
      (br_if $blink_loop (i32.lt_u (local.get $counter) (i32.const 100)))
    )
    
    (i32.const 0)
  )
)
