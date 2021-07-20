# T-Watch-2020 with Rust

This project tries to provide a minimal firmware implementation for the 
[T-Watch-2020](http://www.lilygo.cn/prod_view.aspx?TypeId=50053&Id=1290&FId=t3:50053:3)
but in Rust.

## What's working ?

 - [x] PMU: AXP202 - Partial, see https://github.com/pyaillet/axp20x-rs
 - [ ] Power button
 - [x] Vibration
 - [x] ST7789V - Partial, see https://github.com/almindor/st7789
 - [ ] BMA423 Axis Sensor
 - [ ] I2S Class Max98357A
 - [ ] IR
 - [ ] Touch board
 - [X] Real time clock - see https://github.com/nebelgrau77/pcf8563-rs
 - [ ] BLE
 - [ ] WiFi


## Reference

This project started with a fork from https://github.com/MabezDev/xtensa-rust-quickstart

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
