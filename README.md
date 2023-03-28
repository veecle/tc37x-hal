## Demo HAL for AURIX™ tc37x lite-kit

Demo implementation of a HAL for the [TC375-Lite kit]. The HAL is very limited at the moment and only supports
- Clock configuration via `pll` and `ccu` wrappers: configurations are hard-coded for the TC375
- Basic handling of time and delay
- CAN-driver: this is more or less working in a simplified (and untested) manner 

For usage and documentation refer to the [veecle-aurix-demo] crate.

**This is a basic and likely wrong implementation: this only serves as proof of concept and nothing more; future MRs will transform this to a more correctly designed and validated HAL**

[veecle-aurix-demo]: https://github.com/veecle/tc37x-demo
[TC375-Lite kit]: https://www.infineon.com/cms/en/product/promopages/AURIX-microcontroller-boards/low-cost-arduino-kits/aurix-tc375-lite-kit/

#### License

Licensed under <a href="LICENSE">Apache License, Version 2.0</a>