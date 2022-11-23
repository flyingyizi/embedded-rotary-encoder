

rotary encoder for embedded. Suitable for gray-code incremental encoders, 
complay with embedded-hal (https://docs.rs/embedded-hal/0.2.7/embedded_hal)

# examples

for arduino and stm32f401 mcus, provide interrupt sample and busy loop sample.

# attention

- rotary pins should be pull-up input, on other word, With no external circuit pulling the pin low, it will be read high.

- when finding direction is opposite as you wishing, you should switch the pin-clk, pin-dt.


# rotary encoder algorithm

rotary encoder produces are based on a 2-bit gray code available on 2 digital data signal lines.
There are 4 possible combinations of on and off on these 2 signal lines, 
related gray code shows below:

|Position	|Bit A	|Bit B|
|-----------|-------|-----|
|0	        |0      |0    |
|1/4        |1      |0    |
|1/2        |1      |1    |
|3/4        |0      |1    |
|1          |0      |0    |

// cw:
```text
   a   b   c   d
   │   |   |   |   │               │          
 ──┼───┐       ┌───┼───┐       ┌───┼───┐       ┌───────┐
   │   │       │   │   │       │   │   │       │       │          pin2
   │   └───────┘   │   └───────┘   │   └───────┘       └────
   ├───────┐       ├───────┐       ├───────┐
   │       │       │       │       │       │                      pin1
 ──┤       └───────┤       └───────┤       └───────
   │               │               │

//   if ((old_state == 2) && (thisState == 3))  {
//     knobPosition++;
//   } else if ((old_state == 3) && (thisState == 1)) {
//     knobPosition++;
//   } else if ((old_state == 1) && (thisState == 0)){
//     knobPosition++;
//   } else if ((old_state == 0) && (thisState == 2)) {
//     knobPosition++;
```

// ccw:
```text
   │               │               │         
 ──┼───┐       ┌───┼───┐       ┌───┼───┐       ┌───────┐
   │   │       │   │   │       │   │   │       │       │        pin2
   │   └───────┘   │   └───────┘   │   └───────┘       └────
 ──┤       ┌───────┤       ┌───────┤       ┌───────┐
   │       │       │       │       │       │       │            pin1
   ├───────┘       ├───────┘       ├───────┘       └────
   │               │               │

//   } else if ((old_state == 3) && (thisState == 2)) {
//     knobPosition--;
//   } else if ((old_state == 2) && (thisState == 0)) {
//     knobPosition--;
//   } else if ((old_state == 0) && (thisState == 1)) {
//     knobPosition--;
//   } else if ((old_state == 1) && (thisState == 3)) {
//     knobPosition--;
//   }
//
```

