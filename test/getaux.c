#include <stdlib.h>
#include <stdio.h>
#include <sys/auxv.h>

int main (){

  long int addr = getauxval(AT_ENTRY);
  printf ("AT_ENTRY: 0x%Lu\n", addr);

  return 0;
}
