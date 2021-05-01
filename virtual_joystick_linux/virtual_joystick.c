// README
// create virtual joystick device that is fed from other devices
// default configuration:
// * uses mouse wheel to output right gamepad stick
// * when right click is pressed outputs left gamepad stick


#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <fcntl.h>
#include <errno.h>
#include <linux/input.h>
#include <linux/uinput.h>
#include <math.h> // round

// jesus
#define MAX(x, y) (((x) > (y)) ? (x) : (y))
#define MIN(x, y) (((x) < (y)) ? (x) : (y))

// user configuration
const char* MOUSE_DEVICE="/dev/input/event5"; // cat /proc/bus/input/devices and look for mouse
const int MOUSEWHEEL_STEPS = 10; // depends how smooth mouse wheel is
const int JOY_MAX = 512;
const int JOY_MIN = -512;
const int ABS_FLAT = 15;
const static float FREELOOK_SENSITIVITY = 0.2;

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
    if (read(fd, &ev, sizeof(ev)) > 0) {
        //printf("type : %d, code : %d, value : %d \n", ev.type, ev.code, ev.value);
        return ev;
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
  //ioctl(fd, UI_SET_ABSBIT, ABS_RX);
  //ioctl(fd, UI_SET_ABSBIT, ABS_RY);
  ioctl(fd, UI_SET_ABSBIT, ABS_GAS);
  ioctl(fd, UI_SET_ABSBIT, ABS_BRAKE);
  //ioctl(fd, UI_SET_ABSBIT, ABS_WHEEL);

  struct uinput_user_dev uidev;
  memset(&uidev, 0, sizeof(uidev));
  uidev.id.bustype = BUS_USB;

  if (1) {
      snprintf(uidev.name, UINPUT_MAX_NAME_SIZE, "ThurstMaster T150");
      uidev.id.vendor  = 0x044f; uidev.id.product = 0xb677;
  } else {
      snprintf(uidev.name, UINPUT_MAX_NAME_SIZE, "Generic Gamepad");
      uidev.id.vendor  = 0x3; uidev.id.product = 0x3;
  }

  uidev.id.version = 2;

  // setup controller axis
  uidev.absmax[ABS_X] = JOY_MAX;
  uidev.absmin[ABS_X] = JOY_MIN;
  uidev.absfuzz[ABS_X] = 0;
  uidev.absflat[ABS_X] = ABS_FLAT;

  uidev.absmax[ABS_Y] = JOY_MAX; 
  uidev.absmin[ABS_Y] = JOY_MIN;
  uidev.absfuzz[ABS_Y] = 0;
  uidev.absflat[ABS_Y] = ABS_FLAT;

  uidev.absmax[ABS_GAS] = JOY_MAX;
  uidev.absmin[ABS_GAS] = JOY_MIN;
  uidev.absfuzz[ABS_GAS] = 0;
  uidev.absflat[ABS_GAS] = ABS_FLAT;

  uidev.absmax[ABS_BRAKE] = JOY_MAX;
  uidev.absmin[ABS_BRAKE] = JOY_MIN;
  uidev.absfuzz[ABS_BRAKE] = 0;
  uidev.absflat[ABS_BRAKE] = ABS_FLAT;
  //uidev.absmax[ABS_WHEEL] = JOY_MAX; 
  //uidev.absmin[ABS_WHEEL] = JOY_MIN;
  //uidev.absfuzz[ABS_WHEEL] = 0;
  //uidev.absflat[ABS_WHEEL] = ABS_FLAT;
  //uidev.absmax[ABS_RX] = JOY_MAX;
  //uidev.absmin[ABS_RX] = JOY_MIN;
  //uidev.absfuzz[ABS_RX] = 0;
  //uidev.absflat[ABS_RX] = ABS_FLAT;
  //uidev.absmax[ABS_RY] = JOY_MAX;
  //uidev.absmin[ABS_RY] = JOY_MIN;
  //uidev.absfuzz[ABS_RY] = 0;
  //uidev.absflat[ABS_RY] = ABS_FLAT;

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

    // mouse wheel
    if (ev->type == EV_REL && ev->code == ABS_WHEEL)
    {
        const static float stepChange = 1.0*(JOY_MAX-JOY_MIN)/MOUSEWHEEL_STEPS;
        static float currentPedalPosition = 0;

        currentPedalPosition += ev->value * stepChange;
        currentPedalPosition = MAX(JOY_MIN, MIN(JOY_MAX, currentPedalPosition));
        int newPos = round(currentPedalPosition);
        //emit(fd, EV_ABS, ABS_GAS, newPos); // FIXME ABS_GAS might be detected as same axis as ABS_Y. temporarily use ABS_BRAKE instead
        emit(fd, EV_ABS, ABS_BRAKE, newPos);
        emit(fd, EV_SYN, SYN_REPORT, 0);
        // printf("current pos: %d\n", newPos); fflush(stdout);
        return;
    }

    // freelook activated on right mouse press
    static int freelookEnabled = 0;
    static float freelookX=0, freelookY = 0;
    if (ev->type == EV_REL && freelookEnabled) {
        if (ev->code == REL_X) {
            freelookX += ev->value * FREELOOK_SENSITIVITY;
            freelookX = MAX(JOY_MIN, MIN(JOY_MAX, freelookX));
            emit(fd, EV_ABS, ABS_X, round(freelookX));
            emit(fd, EV_SYN, SYN_REPORT, 0);
        }
        else if (ev->code == REL_Y) {
            freelookY += ev->value * FREELOOK_SENSITIVITY;
            freelookY = MAX(JOY_MIN, MIN(JOY_MAX, (int)freelookY));
            emit(fd, EV_ABS, ABS_Y, round(freelookY));
            emit(fd, EV_SYN, SYN_REPORT, 0);
        }
    }
    if (ev->type == EV_KEY && ev->code == BTN_RIGHT && ev->value == 1)
    {
        freelookEnabled = !freelookEnabled;
        // reset axis
        if (!freelookEnabled) {
            freelookX = 0;
            freelookY = 0;
            emit(fd, EV_ABS, ABS_X, freelookX);
            emit(fd, EV_ABS, ABS_Y, freelookY);
            emit(fd, EV_SYN, SYN_REPORT, 0);
        }
        printf("switching freelook to: %d\n", freelookEnabled); fflush(stdout);
    }
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
