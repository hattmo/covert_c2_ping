// trick to not include stdio.h
#define _STDIO_H
#define FILE void

// generate all types and functions
// except those using FILE
#include <nftables/libnftables.h>