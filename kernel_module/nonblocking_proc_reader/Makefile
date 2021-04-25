obj-m	:= sleep.o
 
KERNELDIR ?= /lib/modules/$(shell uname -r)/build
PWD       := $(shell pwd)
 
all: default cat_noblock
 
default:
	$(MAKE) -C $(KERNELDIR) M=$(PWD) modules

cat_noblock: cat_noblock.c
	gcc cat_noblock.c -o cat_noblock
 
clean:
	rm *.o *.ko *.mod *.mod.c Module.* modules.*
