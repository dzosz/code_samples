// README
// create virtual joystick device that is fed from other devices
// by default uses mouse wheel to output right gamepad stick


#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <fcntl.h>
#include <errno.h>
#include <linux/input.h>
#include <linux/uinput.h>

// jesus
#define MAX(x, y) (((x) > (y)) ? (x) : (y))
#define MIN(x, y) (((x) < (y)) ? (x) : (y))

// user configuration
const char* MOUSE_DEVICE="/dev/input/event6";
const int MWHEEL_AXIS_TRANSLATION = ABS_RY;
const int MWHEEL_STEP_NUMBER = 10; // depends how smooth mouse wheel is
const int JOY_MAX = 512;
const int ABS_FLAT = 15;

void emit(int fd, int type, int code, int val)
{
   struct input_event ie;

   ie.type = type;
   ie.code = code;
   ie.value = val;
   ie.time.tv_sec = 0;
   ie.time.tv_usec = 0;

   int err = write(fd, &ie, sizeof(ie));
   if (err < 0) {
      printf("ERR event emit err=%d, type=%d, errno=%d", err, type, errno);
      exit(1);
   }
}

int init_mouse_fd()
{
    printf("Starting to listen on device %s\n", MOUSE_DEVICE);
    fflush(stdout);
    int fd = open(MOUSE_DEVICE, O_RDONLY);
    if(fd < 0 )
    {
        printf("ERR opening mouse device %s %d\n", MOUSE_DEVICE, fd);
        exit(1);
    }
    return fd;
}

struct input_event read_mouse_event(int fd)
{
    struct input_event ev;
    while (1) {
        if (read(fd, &ev, sizeof(ev)) > 0) {
            //printf("type : %d, code : %d, value : %d \n", ev.type, ev.code, ev.value);
            return ev;
        }
    }
}


int init_virtual_joystick_fd() {
  int fd = open("/dev/uinput", O_WRONLY | O_NONBLOCK);
  if (fd < 0) {
    printf("ERR Opening of uinput failed!\n");
    exit(1);
  }
  ioctl(fd, UI_SET_EVBIT, EV_KEY);

  // button configuration
  ioctl(fd, UI_SET_KEYBIT, BTN_A);
  ioctl(fd, UI_SET_KEYBIT, BTN_B);
  ioctl(fd, UI_SET_KEYBIT, BTN_X);
  ioctl(fd, UI_SET_KEYBIT, BTN_Y);
  ioctl(fd, UI_SET_KEYBIT, BTN_TL);
  ioctl(fd, UI_SET_KEYBIT, BTN_TR);
  ioctl(fd, UI_SET_KEYBIT, BTN_TL2);
  ioctl(fd, UI_SET_KEYBIT, BTN_TR2);
  ioctl(fd, UI_SET_KEYBIT, BTN_START);
  ioctl(fd, UI_SET_KEYBIT, BTN_SELECT);
  ioctl(fd, UI_SET_KEYBIT, BTN_THUMBL);
  ioctl(fd, UI_SET_KEYBIT, BTN_THUMBR);
  ioctl(fd, UI_SET_KEYBIT, BTN_DPAD_UP);
  ioctl(fd, UI_SET_KEYBIT, BTN_DPAD_DOWN);
  ioctl(fd, UI_SET_KEYBIT, BTN_DPAD_LEFT);
  ioctl(fd, UI_SET_KEYBIT, BTN_DPAD_RIGHT);

  ioctl(fd, UI_SET_EVBIT, EV_ABS);
  ioctl(fd, UI_SET_ABSBIT, ABS_X);
  ioctl(fd, UI_SET_ABSBIT, ABS_Y);
  ioctl(fd, UI_SET_ABSBIT, ABS_RX);
  ioctl(fd, UI_SET_ABSBIT, ABS_RY);

  struct uinput_user_dev uidev;
  memset(&uidev, 0, sizeof(uidev));
  snprintf(uidev.name, UINPUT_MAX_NAME_SIZE, "Generic Gamepad");
  uidev.id.bustype = BUS_USB;
  uidev.id.vendor  = 0x3;
  uidev.id.product = 0x3;
  uidev.id.version = 2;

  // left/right sticks
  uidev.absmax[ABS_X] = JOY_MAX;
  uidev.absmin[ABS_X] = -JOY_MAX;
  uidev.absfuzz[ABS_X] = 0;
  uidev.absflat[ABS_X] = ABS_FLAT;
  uidev.absmax[ABS_Y] = JOY_MAX; 
  uidev.absmin[ABS_Y] = -JOY_MAX;
  uidev.absfuzz[ABS_Y] = 0;
  uidev.absflat[ABS_Y] = ABS_FLAT;
  uidev.absmax[ABS_RX] = JOY_MAX;
  uidev.absmin[ABS_RX] = -JOY_MAX;
  uidev.absfuzz[ABS_RX] = 0;
  uidev.absflat[ABS_RX] = ABS_FLAT;
  uidev.absmax[ABS_RY] = JOY_MAX;
  uidev.absmin[ABS_RY] = -JOY_MAX;
  uidev.absfuzz[ABS_RY] = 0;
  uidev.absflat[ABS_RY] = ABS_FLAT;

  if(write(fd, &uidev, sizeof(uidev)) < 0)
  {
    printf("ERR uidev write");
    exit(1);
  }

  if(ioctl(fd, UI_DEV_CREATE) < 0)
  {
    printf("ERR ioctl");
    exit(1);
  }

  return fd;
}

void translate_mouse_to_joy(int fd, struct input_event* ev) {
    static int currentPedalPosition = 0;
    if (ev->type == EV_REL && ev->code == ABS_WHEEL)
    {
        currentPedalPosition += (ev->value * JOY_MAX*2/MWHEEL_STEP_NUMBER);
        currentPedalPosition = MAX(-JOY_MAX, MIN(JOY_MAX, currentPedalPosition));
        emit(fd, EV_ABS, MWHEEL_AXIS_TRANSLATION, currentPedalPosition);
        emit(fd, EV_SYN, SYN_REPORT, 0);
        //printf("current pos: %d\n", currentPedalPosition);
        //fflush(stdout);
    }
    /*
    if (ev.type == EV_KEY && ev.code == BTN_LEFT)
    {
        emit(joy_fd, EV_KEY, BTN_X, ev.value);
        emit(joy_fd, EV_SYN, SYN_REPORT, 0);
    }
    */
}


int main(void)
{ 
  int joy_fd = init_virtual_joystick_fd();
  int mouse_fd = init_mouse_fd();

  while(1)
  {
    struct input_event ev = read_mouse_event(mouse_fd);
    translate_mouse_to_joy(joy_fd, &ev);
  }

  if(ioctl(joy_fd, UI_DEV_DESTROY) < 0)
  {
    printf("ERR joy_fd ioctl destroy");
    exit(1);
  }
  close(joy_fd);
  close(mouse_fd);
  return 0;
}
