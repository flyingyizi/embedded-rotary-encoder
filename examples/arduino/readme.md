
connection:  D7/D8 used as pin-clk/pin-dt.

# proteus simulate notes

- program file: elf not suitable, hex is workable. e.g. `examples\avr> avr-objcopy  -j .text -j .data -O ihex target\avr-atmega328p\debug\rotary-encoder.elf rotary-encoder.hex`


# rotary-encoder

based on pin change interrupt. demo rotary encoder

# rotary-encoder-by-loop

based on busy-loop
