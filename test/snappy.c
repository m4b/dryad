#include<snappy-c.h>
#include<math.h>
#include<stdio.h>
#include<malloc.h>

int main () {

  char* input = "hello, world";
  int input_length = 12;
  size_t output_length = snappy_max_compressed_length(input_length);
  char* output = (char*)malloc(output_length);

  snappy_compress(input, input_length, output, &output_length);

  printf("snappy: %s\n", output);
  printf("sin: %f\n", sin(2.5));

  return 0;
}
