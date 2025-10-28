(module
  ;; Import host functions
  (import "env" "adc_read" (func $adc_read (param i32) (result i32)))
  (import "env" "uart_write" (func $uart_write (param i32 i32)))
  (import "env" "delay_ms" (func $delay_ms (param i32)))
  
  ;; Memory for data
  (memory (export "memory") 1)
  (data (i32.const 0) "Temperature Monitor\n")
  (data (i32.const 20) "Reading: ")
  
  ;; Convert temperature value to ASCII string
  (func $int_to_string (param $value i32) (param $buffer i32)
    (local $digit i32)
    
    ;; Simple conversion (2 digits)
    (local.set $digit (i32.div_u (local.get $value) (i32.const 10)))
    (i32.store8 (local.get $buffer) (i32.add (local.get $digit) (i32.const 48)))
    
    (local.set $digit (i32.rem_u (local.get $value) (i32.const 10)))
    (i32.store8 (i32.add (local.get $buffer) (i32.const 1)) 
                 (i32.add (local.get $digit) (i32.const 48)))
  )
  
  ;; Main function
  (func (export "main") (result i32)
    (local $temp i32)
    (local $counter i32)
    
    ;; Startup message
    (call $uart_write (i32.const 0) (i32.const 20))
    
    ;; Read temperature in loop
    (loop $read_loop
      ;; Read ADC channel 0 (temperature sensor)
      (local.set $temp (call $adc_read (i32.const 0)))
      
      ;; Convert to Celsius (simplified: raw_value / 10)
      (local.set $temp (i32.div_u (local.get $temp) (i32.const 10)))
      
      ;; Write "Reading: "
      (call $uart_write (i32.const 20) (i32.const 9))
      
      ;; Convert temp to string and write
      (call $int_to_string (local.get $temp) (i32.const 100))
      (call $uart_write (i32.const 100) (i32.const 2))
      
      ;; Wait 2 seconds
      (call $delay_ms (i32.const 2000))
      
      ;; Continue for 30 readings
      (local.set $counter (i32.add (local.get $counter) (i32.const 1)))
      (br_if $read_loop (i32.lt_u (local.get $counter) (i32.const 30)))
    )
    
    (i32.const 0)
  )
)
