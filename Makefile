.PHONY: build

CC=clang -lSystem -arch i386 -nostdlib -o
AS=as -static -arch i386 -o
LD=ld -static -arch i386 -o

E02=02.call--noarg

build: build/00.exit build/01.main build/$(E02)

build/00.exit: examples/00.exit.s
	$(AS) build/00.exit.o examples/00.exit.s
	$(LD) build/00.exit build/00.exit.o

build/01.main: examples/01.main.c
	$(CC) build/01.main examples/01.main.c -O0
	$(CC) build/01.main.s examples/01.main.c -O0 -S

build/$(E02): examples/$(E02).c
	$(CC) build/$(E02) examples/$(E02).c -O0
	$(CC) build/$(E02).s examples/$(E02).c -O0 -S