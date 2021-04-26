#include <stdio.h> // printf
#include <unistd.h> // sleep


#define MAX_STR 100
char my_string[MAX_STR]="this string was not hacked yet";
int main() {
    printf("page size %d\n", getpagesize());
    printf("addr %p\n", my_string);
    printf("addr after page %p\n", (size_t)my_string%getpagesize());
    while (1)
    {
        printf("%.*s\n", 40, my_string);
        sleep(5);
    }
}
