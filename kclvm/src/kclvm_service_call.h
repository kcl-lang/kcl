#pragma once
#ifndef KCLVM_SERVICE_CALL
#define KCLVM_SERVICE_CALL

#ifdef __cplusplus
extern "C" {
#endif


typedef struct kclvm_service kclvm_service;

kclvm_service * kclvm_service_new();

void kclvm_service_delete(kclvm_service *);

void kclvm_service_free_result(const char * res);

const char* kclvm_service_call(kclvm_service* c,const char * method,const char * args);


#ifdef __cplusplus
} // extern "C"
#endif

#endif 