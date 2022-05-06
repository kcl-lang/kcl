// Copyright 2021 The KCL Authors. All rights reserved.

#include "kclvm_plugin.h"

#include <assert.h>
#include <string.h>

static const int32_t kDefaultBufferSize = 1024*1024*10;

static _kclvm_plugin_AppContextBase* g_self_ = NULL;
static uint64_t g_rust_invoke_json_ptr_ = 0;

static const char* _invoke_json_proxy(
    const char* method,
    const char* args_json,
    const char* kwargs_json
) {
    if(g_self_ == NULL) { return ""; }

    static std::string jsonResult;
    jsonResult = g_self_->_call_py_method(method, args_json, kwargs_json);
    return jsonResult.c_str();
}

_kclvm_plugin_AppContextBase::_kclvm_plugin_AppContextBase(uint64_t rust_invoke_json_ptr) {
    g_rust_invoke_json_ptr_ = rust_invoke_json_ptr;
    assert(g_self_ == NULL);
    g_self_ = this;
}
_kclvm_plugin_AppContextBase::~_kclvm_plugin_AppContextBase() {
    g_rust_invoke_json_ptr_ = 0;
    g_self_ = NULL;
}

void _kclvm_plugin_AppContextBase::_clear_options() {
    this->option_keys_.clear();
    this->option_values_.clear();
}
void _kclvm_plugin_AppContextBase::_add_option(const std::string& key, const std::string& value) {
    this->option_keys_.push_back(key);
    this->option_values_.push_back(value);
}

std::string _kclvm_plugin_AppContextBase::_run_app(
    uint64_t _start_fn_ptr,
    uint64_t _kclvm_main_ptr, // main.k => kclvm_main
    int32_t strict_range_check,
    int32_t disable_none,
    int32_t disable_schema_check,
    int32_t list_option_mode,
    int32_t debug_mode,
    int32_t buffer_size
) {
    typedef int32_t (*kcl_run_t)(
        uint64_t _kclvm_main_ptr, // main.k => kclvm_main
        int32_t option_len,
        const char** option_keys,
        const char** option_values,
        int32_t strict_range_check,
        int32_t disable_none,
        int32_t disable_schema_check,
        int32_t list_option_mode,
        int32_t debug_mode,
        int32_t result_buffer_len,
        char* result_buffer,
        int32_t warn_buffer_len,
        char* warn_buffer
    );

    int32_t _option_len = this->option_keys_.size();
    std::vector<char*> _option_keys(_option_len);
    std::vector<char*> _option_values(_option_len);

    for(size_t i = 0; i < this->option_keys_.size(); i++) {
        _option_keys[i] = (char*)this->option_keys_[i].c_str();
        _option_values[i] = (char*)this->option_values_[i].c_str();
    }

    this->buffer_.clear();
    this->warn_buffer_.clear();

    if(buffer_size > 0) {
        this->buffer_.resize(buffer_size, '\0');
    } else {
        this->buffer_.resize(kDefaultBufferSize, '\0');
    }

    this->warn_buffer_.resize(10*1024*1924, '\0');

    kcl_run_t _kcl_run = (kcl_run_t)(_start_fn_ptr);
    int32_t result_len = _kcl_run(
        _kclvm_main_ptr,
        _option_len,
        (const char**)(_option_keys.data()),
        (const char**)(_option_values.data()),
        strict_range_check,
        disable_none,
        disable_schema_check,
        list_option_mode,
        debug_mode,
        this->buffer_.size()-1,
        &this->buffer_[0],
        this->warn_buffer_.size()-1,
        &this->warn_buffer_[0]
    );

    if(result_len > 0) {
        this->buffer_.resize(result_len);
    } else if (result_len == 0) {
        this->buffer_ = "{}";
    } else {
        this->buffer_ = "{\"error\": \"buffer size limit\"}";
    }

    this->warn_buffer_.resize(strlen(&this->warn_buffer_[0]));

    return this->buffer_;
}

std::string _kclvm_plugin_AppContextBase::_get_warn() {
    return this->warn_buffer_;
}

uint64_t _kclvm_plugin_AppContextBase::_get_cxx_invoke_proxy_ptr() {
    return uint64_t(_invoke_json_proxy);
}

std::string _kclvm_plugin_AppContextBase::_call_rust_method(
    const std::string& name,
    const std::string& args_json,
    const std::string& kwargs_json
) {
    typedef const char* (*invoke_fn_t)(
        const char* method,
        const char* args_json,
        const char* kwargs_json
    );
    invoke_fn_t fn = (invoke_fn_t)(g_rust_invoke_json_ptr_);
    return fn(name.c_str(), args_json.c_str(), kwargs_json.c_str());
}

std::string _kclvm_plugin_AppContextBase::_call_py_method(
    const std::string& name,
    const std::string& args_json,
    const std::string& kwargs_json
) {
    return "implemented in Python!!!";
}
