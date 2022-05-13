// Copyright 2021 The KCL Authors. All rights reserved.

#pragma once

#ifndef __cplusplus
#error "please use C++"
#endif

#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>

#include <string>
#include <vector>

// ----------------------------------------------------------------------------

class _kclvm_plugin_AppContextBase {
    std::string buffer_;
    std::string warn_buffer_;
    std::vector<std::string> option_keys_;
    std::vector<std::string> option_values_;

public:
    _kclvm_plugin_AppContextBase(uint64_t rust_invoke_json_ptr);
    virtual ~_kclvm_plugin_AppContextBase();

    void _clear_options();
    void _add_option(const std::string& key, const std::string& value);

    std::string _run_app(
        uint64_t _start_fn_ptr,
        uint64_t _kclvm_main_ptr, // main.k => kclvm_main
        int32_t strict_range_check,
        int32_t disable_none,
        int32_t disable_schema_check,
        int32_t list_option_mode,
        int32_t debug_mode,
        int32_t buffer_size
    );

    std::string _get_warn();

    uint64_t _get_cxx_invoke_proxy_ptr();

    std::string _call_rust_method(
        const std::string& name,
        const std::string& args_json,
        const std::string& kwargs_json
    );

    virtual std::string _call_py_method(
        const std::string& name,
        const std::string& args_json,
        const std::string& kwargs_json
    );
};

// ----------------------------------------------------------------------------
// END
// ----------------------------------------------------------------------------
