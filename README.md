# Two fixes:

## Missing platform feature — added "x11" to gpui_platform features in Cargo.toml, since Linux Mint runs X11 and the crate needs an explicit backend enabled.
## Missing system library — installed libxkbcommon-x11-dev via apt, which the X11 backend links against.

`sudo apt install libxkbcommon-x11-dev`
