#pragma once
#ifndef __KCLVM_SERVICE_CALL_H__
#define __KCLVM_SERVICE_CALL_H__

#ifdef __cplusplus
extern "C" {
#endif


typedef struct kclvm_service kclvm_service;

kclvm_service * kclvm_service_new();

void kclvm_service_delete(kclvm_service *);

void kclvm_service_free_string(const char * res);

const char* kclvm_service_call(kclvm_service* c,const char * method,const char * args);

const char* kclvm_service_get_error_buffer(kclvm_service* c);

void kclvm_service_clear_error_buffer(kclvm_service* c);


#ifdef __cplusplus
} // extern "C"
#endif

#endif 