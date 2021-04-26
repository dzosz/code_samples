obj-m += kernel_hack.o

all: target_process
	make -C /lib/modules/$(shell uname -r)/build M=$(PWD) modules

target_process: target_process.c
	gcc target_process.c -o target_process

clean:
	make -C /lib/modules/$(shell uname -r)/build M=$(PWD) clean
	rm -f target_process
