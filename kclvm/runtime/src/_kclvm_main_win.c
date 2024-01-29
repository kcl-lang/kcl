// Copyright The KCL Authors. All rights reserved.

extern void* kclvm_main(void* ctx);

extern void kclvm_debug_hello();
extern void kclvm_debug_print(const char* s);

__declspec(dllexport) void* kclvm_main_win(void* ctx) {
    return kclvm_main(ctx);
}

__declspec(dllexport) void kclvm_debug_hello_win() {
    kclvm_debug_hello();
}

__declspec(dllexport) void kclvm_debug_print_win(const char* s) {
    kclvm_debug_print(s);
}
