#ifndef __ZK_B64_H
#define __ZK_B64_H

#ifdef __cplusplus
extern "C"
{
#endif // __cplusplus

char* zkB64Encode(const uint8_t* data,
                  int input_length,
                  int* output_length);

uint8_t* zkB64Decode(const char *data,
                     int input_length,
                     int* output_length);

#ifdef __cplusplus
}
#endif // __cplusplus

#endif // __ZK_B64_H
