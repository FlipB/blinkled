# Blinking LED for WeAct STM32F411 MiniF4 

Blink onboard LED.

# Prereqs
 
rust target

```sh
$ rustup target install thumbv7em-none-eabihf
```

cargo-binutils

```sh
$ cargo install cargo-binutils
$ rustup component add llvm-tools-preview
```

dfu-util

```sh
$ brew install dfu-util
```

# Building

```sh
$ cargo objcopy --release -- -O binary out.bin
```

# Flashing

## dfu-util

Flashing with dfu requires the following procedure.
1. Disconnect board from USB C.
2. Short A9 and A10 (You can leave these connected).
3. Connect USB C to board.
4. Press BOOT0, NRST.
5. Release NRST, wait 500ms, Release BOOT0.
6. Flash with dfu-util with the following command.

```sh
$ dfu-util -a0 -s 0x08000000  -D out.bin
```

## ST-Link

Using the ST-link with probe-run to flash and run with `cargo run`.

### Install deps

```sh
$ brew install libftdi
```

```sh
$ cargo install cargo-flash
```

```sh
$ cargo install probe-run
```

### Run with probe-run

probe-run is already set as the default cargo runner - just run `cargo run`.

### Problems with probe-run

ST-Link seems to drop connection when the MCU enters deep sleep state (wfi - wait for interrupts).
