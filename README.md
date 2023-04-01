# NES Emulator in Rust
NES Emulator written with this guide: https://bugzmanov.github.io/nes_ebook

## Requirements
- rustc >= 1.68.2
- SDL2 >= 2.0.8

## SDL2 Installation
Follow guide here: https://crates.io/crates/sdl2

## Caveats
Currently I am using WSL for testing on my Win 10 machine which isn't ideal when I want to display a GUI application. 
So I am trying this workaround: https://www.reddit.com/r/bashonubuntuonwindows/comments/hvmc2t/gui_apps_in_wsl2/ (didn't work)
This workaround seems to cause a not insignificant input lag, so it might cause problems in the future.
There might be a more efficient way to deal with this problem in Win 11.

`XDG_RUNTIME_DIR not set in the environment` is currently my error I need to fix