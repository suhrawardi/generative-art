#!/bin/sh


PNGS=($(ls -r second/twee/*.png))

for i in {1..5}
do
  j=$(expr $(expr $i - 1) \* 254)
  SLICEDPNGS=${PNGS[@]:${j}:254}
  echo "\n"
  echo "$i $j \n"
  echo $SLICEDPNGS
  k=$(printf "%05d" $i)
  echo "second/twee/movie${k}.mp4"
  # ffmpeg -i $SLICEDPNGS -r 1 -c:v libx264 -y "second/twee/movie${k}.mp4"
  ffmpeg -framerate 25 -i $SLICEDPNGS -c:v libx264 -pix_fmt yuv420p -b:v 100M -y "second/twee/movie${k}.mp4"
done

# ffmpeg -f concat -safe 0 -i <$(ls -r second/twee/movie*.mp4) -c copy output.mp4
