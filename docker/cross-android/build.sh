#!/bin/sh

umask 0000

cd android_macroquad
exec cargo quad-apk build --release
