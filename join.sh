#!/bin/sh


convert -delay 4 -loop 0 $(ls -tr second/twee/*.png) animated.gif

ffmpeg -f gif -i animated.gif animation.mp4

