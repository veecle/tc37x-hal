## Demo HAL for AURIX™ tc37x lite-kit

Demo implementation of an HAL for the [TC375-Lite kit]. The HAL is very limited at the moment and only supports
- Clock configuration via `pll` and `ccu` wrappers: configurations are hardcoded for the TC375
- Basic handling of time and delay
- CAN-driver: this is more or less working in simplified manner 

For full usage documentation refer to the [veecle-aurix-demo] crate.

[veecle-aurix-demo]: https://github.com/veecle/tc37x-demo
[TC375-Lite kit]: https://www.infineon.com/cms/en/product/promopages/AURIX-microcontroller-boards/low-cost-arduino-kits/aurix-tc375-lite-kit/

#### License

Licensed under <a href="LICENSE">Apache License, Version 2.0</a>.