#include <stdio.h>
#include <pthread.h>

__thread int my_thread_local = 0xdeadbeef;

int main () {

  printf("my_thread_local: %d @ %x\n", my_thread_local, &my_thread_local);

}
