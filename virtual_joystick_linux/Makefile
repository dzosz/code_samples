all: virtual_joystick read_mouse_events
 
virtual_joystick: virtual_joystick.c
	gcc virtual_joystick.c -o virtual_joystick -O2 -std=gnu11 -lm

read_mouse_events: read_mouse.c
	gcc read_mouse.c -o read_mouse_events -O2 -std=gnu11
 
clean:
	rm virtual_joystick read_mouse_events
