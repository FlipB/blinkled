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


# Arduino IDE

STM32 can be used in arduino IDE with stm32duino or a different "core" by some similar name.
It has to be added in preferences in `Board manager` by adding a custom url (I used `https://github.com/stm32duino/BoardManagerFiles/raw/master/STM32/package_stm_index.json`, stm32duino 1.9.0 (deprecated)).

## USB HID Bootloader

A bootloader can be installed (does not replace DFU) which enables flashing over USB as well as serial over USB and with integration in Arduino IDE.
This was helpful to get serial output for debugging sketches.

### Flashing USB HID Bootloader

Bootloader was gotten from attachmen in thread at `https://github.com/Serasidis/STM32_HID_Bootloader/issues/23`.
File is present in this git commit at `hid_bootloader_STM32F411.bin`.

Flash with `dfu-util`:

`dfu-util -a 0 -s 0x08000000 -D hid_bootloader_STM32F411.bin`

If that doesn't work, try:

`dfu-util -a 0 -d 0483:df11 -s 0x8000000:mass-erase:force -D hid_bootloader_STM32F411.bin -t 1024`


#### Alternative bootloader

WeAct has a closed source USB HID bootloader as well. I think it might require patched hid-flash though.
It's probably usable if you update the hid_flash tool used by Ardduino IDE `~/Library/Arduino15/packages/STM32/tools/STM32Tools/1.4.0/tools/macosx/hid-flash`, but it didnt have any Mac version prebuilt so I didn't try this.

More about that here: https://hardwareliberopinerolo.github.io/site/blackpill/

## Using USB HID bootloader

Once flashed, it must be entered by holding the "KEY" button when starting the device - the LED should come on and the USB device detectable "STM32 HID bootloader" (ProductID:	0xbeba VendorID:	0x1209).

macOS did not automatically create a tty device under /dev/tty* (/dev/tty.usbmodem3450356C35391) until AFTER
a sketch had been uploaded in Arduino IDE (hid-flash CLI was NOT sufficient).
So this is very finicky to get working.

### Arduino IDE USB HID settings

Use following settings in Arduino IDE:

Board: Generic STM32F4
Board Part Number: BlackPill STM32F411CE
U(S)ART Support: Enabled (Generic 'Serial')
USB Support (if avail.): CDC (generic 'Serial' supersede U(S)ART)
USB Speed: Low/Full speed
Upload Method: HID Bootloader 2.2
Port: <See text below>

So (atleast on macOS) the usb tty device is not created correctly, and so the correct port could not be chosen.
Just pick ANY port here (as long as it's something), hid-flash will fail, and then scan and find the usb device using USB VID:PID. Once flashing is done, the USB tty device was created, and I could update the Port setting.

Once the correct USB tty is created and selected as Port in Arduino IDE you can open the Serial Monitor window to see the output.

## Troubleshooting

### Uploading sketch stuck on `Toggling DTR...`

You didn't hold KEY pressed while starting the MCU. Hold down KEY and press NRST to restart the MCU and enter the bootloader. Upload should complete.
Upload might complete and output errors about being unable to open the USB serial device.
Try uploading sketch again and see if that resolves it.