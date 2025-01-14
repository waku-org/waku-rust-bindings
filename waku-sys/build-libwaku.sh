#!/bin/bash

## This is needed to manually build libwaku and copy the needed files
## into the waku-sys/libwaku folder.
## The waky-sys folder will take the files contained in waku-sys/libwaku/, which are git-versioned.

cd vendor/nwaku/

## Create vendor/nwaku/build/libwaku.a
make POSTGRES=1 STATIC=true libwaku -j8

echo "Stripping the library"
## Reduce the libwaku.a size
strip --strip-debug build/libwaku.a

## Go back to the previous folder
cd -

echo "Copying generated files to waku-sys/libwaku"

cp vendor/nwaku/build/libwaku.a libwaku/
cp vendor/nwaku/library/libwaku.h libwaku/
cp vendor/nwaku/vendor/nim-libbacktrace/libbacktrace_wrapper.o libwaku/
cp vendor/nwaku/vendor/nim-libbacktrace/install/usr/lib/libbacktrace.a libwaku/
cp vendor/nwaku/vendor/nim-nat-traversal/vendor/miniupnp/miniupnpc/build/libminiupnpc.a libwaku/
cp vendor/nwaku/vendor/nim-nat-traversal/vendor/libnatpmp-upstream/libnatpmp.a libwaku/

