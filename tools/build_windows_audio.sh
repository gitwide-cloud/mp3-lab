#!/usr/bin/env bash
set -euo pipefail
root="$PWD"
work="$root/audio-build"
prefix="$work/prefix"
mkdir -p "$work/sources" "$prefix" "$root/windows-runtime/licenses" "$root/dist"
curl -fL --retry 3 https://codeload.github.com/FFmpeg/FFmpeg/tar.gz/refs/tags/n8.1.3 -o "$work/sources/ffmpeg-n8.1.3.tar.gz"
curl -fL --retry 3 https://github.com/libsdl-org/SDL/releases/download/release-2.32.10/SDL2-2.32.10.tar.gz -o "$work/sources/SDL2-2.32.10.tar.gz"
curl -fL --retry 3 https://codeload.github.com/arthenica/lame/tar.gz/refs/tags/RELEASE__3_100 -o "$work/sources/lame-3.100.tar.gz"
for f in "$work"/sources/*.tar.gz; do tar xf "$f" -C "$work"; done
cd "$work/lame-RELEASE__3_100"
./configure --host=x86_64-w64-mingw32 --prefix="$prefix" --disable-shared --enable-static --disable-frontend
make -j4
make install
cp COPYING "$root/windows-runtime/licenses/LAME-COPYING.txt"
cd "$work/SDL2-2.32.10"
./configure --host=x86_64-w64-mingw32 --prefix="$prefix" --disable-shared --enable-static --disable-haptic --disable-joystick --disable-sensor
make -j4
make install
cp LICENSE.txt "$root/windows-runtime/licenses/SDL2-LICENSE.txt"
cd "$work/FFmpeg-n8.1.3"
export PKG_CONFIG_LIBDIR="$prefix/lib/pkgconfig"
./configure --target-os=mingw32 --arch=x86_64 --cross-prefix=x86_64-w64-mingw32- \
  --enable-cross-compile --prefix="$prefix" --disable-autodetect --disable-network \
  --disable-doc --disable-debug --disable-shared --enable-static --enable-libmp3lame \
  --enable-sdl2 --pkg-config=pkg-config --pkg-config-flags=--static \
  --extra-cflags="-I$prefix/include" --extra-ldflags="-L$prefix/lib -static -static-libgcc -static-libstdc++"
make -j4
cp ffmpeg.exe ffprobe.exe ffplay.exe "$root/windows-runtime/"
cp COPYING* LICENSE.md "$root/windows-runtime/licenses/"
cp ffbuild/config.log "$root/windows-runtime/licenses/FFmpeg-config.log"
cd "$root"
cp tools/build_windows_audio.sh "$work/sources/"
cp -R windows-runtime/licenses "$work/sources/"
sha256sum "$work"/sources/*.tar.gz > "$work/sources/SHA256SUMS.txt"
tar -czf dist/windows-audio-sources.tar.gz -C "$work" sources
