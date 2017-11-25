.PHONY: build

build:
	as -static -arch i386 -o build/00.exit.o examples/00.exit.s
	ld -static -arch i386 -o build/00.exit build/00.exit.o