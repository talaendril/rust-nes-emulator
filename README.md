# NES Emulator in Rust
NES Emulator written with this guide: https://bugzmanov.github.io/nes_ebook

## Requirements
- rustc >= 1.68.2 with `stable-x86_64-pc-windows-gnu` toolchain (for proper linking without needing VS build tools)
- SDL2 >= 2.0.8

## SDL2 Installation
Follow guide here: https://crates.io/crates/sdl2
Tested using the MSVC variant.

## Problems
### WSL
WSL does not provide an easy way to show GUI apps. Using Xming and xterm might work but it would require more time to be put in.
Another thing is the operting system. I am using Windows 10 and I hear that Windows 11 might be better suited for such a 
problem. I have tried this https://www.reddit.com/r/bashonubuntuonwindows/comments/hvmc2t/gui_apps_in_wsl2/ and it didn't work
immediately. The basis of the problem is that SDL2 does not find an _available video device_ and thus panics.
Input lag might also be a problem with this approach since just funnel the events back to Windows to show UI (I think anyway).

`XDG_RUNTIME_DIR not set in the environment` was also an error thrown during WSL tryouts though that one might be easily fixable.

### Performance
With the snake example the performance was really bad. The snake needed around 2-3 seconds between moves, I don't think this is
because of my rust implementation but I am unsure. I ran the guide code as well and the snake needed only around 1 second between 
moves, so it might just be my implementation. Just writing it down here so I don't forget it...
