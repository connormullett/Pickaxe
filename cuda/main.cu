
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <cuda.h>
#include "sha256.cuh"
#include <dirent.h>
#include <ctype.h>

__global__ void sha256_cuda(BYTE *data, BYTE *digest, int n)
{
	// perform sha256 calculation here
	SHA256_CTX ctx;
	sha256_init(&ctx);
	sha256_update(&ctx, data, n);
	sha256_final(&ctx, digest);
}

// void sha_256(BYTE *data, BYTE *digest, int n)
// {
// 	compy symbols
// 	checkCudaErrors(cudaMemcpyToSymbol(dev_k, host_k, sizeof(host_k), 0, cudaMemcpyHostToDevice));
// 	sha256_cuda(data, digest, n);
// }
