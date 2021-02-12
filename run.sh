#!/bin/sh

cargo run --release
convert -delay 25 -loop 0 $(ls -tr second/*.png) animated.gif

ffmpeg -framerate 1/5 -i $(ls -tr second/*.png) -c:v libx264 -r 30 -pix_fmt yuv420p out.mp4
