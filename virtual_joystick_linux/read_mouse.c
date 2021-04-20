#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <fcntl.h>
#include <errno.h>
#include <linux/input.h>
#include <linux/uinput.h>
int main()
{

    int fd;
    struct input_event ev;
    const char* pFile = "/dev/input/event6";

    fd = open(pFile, O_RDONLY);
    if(fd < 0 )
    {
        printf("ERROR Opening %s\n", pFile);
        return -1;
    }

    while (1) {
        if (read(fd, &ev, sizeof(ev)) > 0) {
            printf("type : %d, code : %d, value : %d \n", ev.type, ev.code, ev.value);
            fflush(stdout);
        }
    }
    close(fd);
    return 0;
}
