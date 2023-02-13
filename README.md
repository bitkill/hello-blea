# hello-blea - now in rust

 > ⚠️ This project is in progress

This aims to replace my previous project, [blea2mqtt](https://github.com/bitkill/blea2mqtt), with a more stable and compatible bluetooth library.

## Instructions
### macOS

To use Bluetooth on macOS Big Sur (11) or later, you need to either package your
binary into an application bundle with an `Info.plist` including
`NSBluetoothAlwaysUsageDescription`, or (for a command-line application such as
the examples included with `btleplug`) enable the Bluetooth permission for your
terminal. You can do the latter by going to _System Preferences_ → _Security &
Privacy_ → _Privacy_ → _Bluetooth_, clicking the '+' button, and selecting
'Terminal' (or iTerm or whichever terminal application you use).

## TODO
 - Send data to a mqtt server
 - Cleanup this readme
 - Generate a mac app (w/`Info.plist`)
 - Have a demo with the prometheus aggregator & grafana
