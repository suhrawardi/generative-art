#!/bin/sh

cargo run --release
convert -delay 25 -loop 0 $(ls -tr second/*.png) animated.gif
