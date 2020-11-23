#!/bin/bash

echo RELEASE MODE
cargo build --release || exit 1

[ ! -d $HOME/.lv2 ] && mkdir $HOME/.lv2
[ ! -d $HOME/.lv2/carsonsynth.lv2 ] && mkdir $HOME/.lv2/carsonsynth.lv2

cp target/release/libcarsonsynth_lv2.so $HOME/.lv2/carsonsynth.lv2/ || exit
#cp target/release/libcarsonsynth_lv2_ui.so $HOME/.lv2/carsonsynth.lv2/ || exit
cp lv2/*ttl $HOME/.lv2/carsonsynth.lv2/ || exit

echo
echo carsonsynth.lv2 successfully installed