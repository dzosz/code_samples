#include <fcntl.h>
#include <stdio.h>
#include <sys/mman.h>
#include <sys/stat.h>

int main(int argc, char ** argv) {
  int fd = open(argv[1], O_RDONLY);
  struct stat stats;
  fstat(fd, &stats);
  posix_fadvise(fd, 0, stats.st_size, POSIX_FADV_DONTNEED);
  char * map = (char *) mmap(NULL, stats.st_size, PROT_READ, MAP_SHARED, fd, 0);
  if (map == MAP_FAILED) {
    perror("Failed to mmap");
    return 1;
  }
  int result = 0;
  int i;
  for (i = 0; i < stats.st_size; i++) {
    result += map[i];
  }
  munmap(map, stats.st_size);
  return result;
}
