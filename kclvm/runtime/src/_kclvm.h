// Copyright The KCL Authors. All rights reserved.

// Auto generated, DONOT EDIT!!!

#pragma once

#ifndef _kclvm_h_
#define _kclvm_h_

#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

// please keep same as 'kclvm/runtime/src/kind/mod.rs#Kind'

enum kclvm_kind_t {
    Invalid = 0,

    // only for value

    Undefined = 1, 
    None = 2,

    // for value & type

    Bool = 3,
    Int = 4,
    Float = 5,
    Str = 6,
    List = 7,
    Dict = 8,

    Schema = 9,
    Error = 10,

    // only for type

    Any = 11,
    Union = 12,

    BoolLit = 13,
    IntLit = 14,
    FloatLit = 15,
    StrLit = 16,

    Func = 17,

    // max num

    Max = 18,
};

typedef int8_t kclvm_bool_t;

typedef struct kclvm_buffer_t kclvm_buffer_t;

typedef char kclvm_char_t;

typedef struct kclvm_context_t kclvm_context_t;

typedef struct kclvm_decorator_value_t kclvm_decorator_value_t;

typedef struct kclvm_eval_scope_t kclvm_eval_scope_t;

typedef double kclvm_float_t;

typedef int64_t kclvm_int_t;

typedef struct kclvm_iterator_t kclvm_iterator_t;

typedef enum kclvm_kind_t kclvm_kind_t;

typedef int32_t kclvm_size_t;

typedef struct kclvm_type_t kclvm_type_t;

typedef struct kclvm_value_ref_t kclvm_value_ref_t;

typedef struct kclvm_value_t kclvm_value_t;

void kclvm_assert(kclvm_context_t* ctx, kclvm_value_ref_t* value, kclvm_value_ref_t* msg);

kclvm_value_ref_t* kclvm_base64_decode(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* _kwargs);

kclvm_value_ref_t* kclvm_base64_encode(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* _kwargs);

kclvm_value_ref_t* kclvm_builtin_abs(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* _kwargs);

kclvm_value_ref_t* kclvm_builtin_all_true(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* _kwargs);

kclvm_value_ref_t* kclvm_builtin_any_true(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* kwargs);

kclvm_value_ref_t* kclvm_builtin_bin(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* _kwargs);

kclvm_value_ref_t* kclvm_builtin_bool(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* _kwargs);

kclvm_value_ref_t* kclvm_builtin_dict(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* kwargs);

kclvm_value_ref_t* kclvm_builtin_float(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* kwargs);

kclvm_value_ref_t* kclvm_builtin_hex(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* _kwargs);

kclvm_value_ref_t* kclvm_builtin_int(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* kwargs);

kclvm_value_ref_t* kclvm_builtin_isunique(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* _kwargs);

kclvm_value_ref_t* kclvm_builtin_len(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* _kwargs);

kclvm_value_ref_t* kclvm_builtin_list(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* _kwargs);

kclvm_value_ref_t* kclvm_builtin_max(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* _kwargs);

kclvm_value_ref_t* kclvm_builtin_min(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* _kwargs);

kclvm_value_ref_t* kclvm_builtin_multiplyof(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* _kwargs);

kclvm_value_ref_t* kclvm_builtin_oct(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* _kwargs);

kclvm_value_ref_t* kclvm_builtin_option(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* kwargs);

void kclvm_builtin_option_init(kclvm_context_t* ctx, char* key, char* value);

kclvm_value_ref_t* kclvm_builtin_option_reset(kclvm_context_t* ctx, kclvm_value_ref_t* _args, kclvm_value_ref_t* _kwargs);

kclvm_value_ref_t* kclvm_builtin_ord(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* _kwargs);

kclvm_value_ref_t* kclvm_builtin_pow(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* _kwargs);

kclvm_value_ref_t* kclvm_builtin_print(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* kwargs);

kclvm_value_ref_t* kclvm_builtin_range(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* _kwargs);

kclvm_value_ref_t* kclvm_builtin_round(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* _kwargs);

kclvm_value_ref_t* kclvm_builtin_sorted(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* kwargs);

kclvm_value_ref_t* kclvm_builtin_str(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* _kwargs);

kclvm_value_ref_t* kclvm_builtin_str_capitalize(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* _kwargs);

kclvm_value_ref_t* kclvm_builtin_str_count(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* _kwargs);

kclvm_value_ref_t* kclvm_builtin_str_endswith(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* _kwargs);

kclvm_value_ref_t* kclvm_builtin_str_find(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* _kwargs);

kclvm_value_ref_t* kclvm_builtin_str_format(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* kwargs);

kclvm_value_ref_t* kclvm_builtin_str_index(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* _kwargs);

kclvm_value_ref_t* kclvm_builtin_str_isalnum(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* _kwargs);

kclvm_value_ref_t* kclvm_builtin_str_isalpha(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* _kwargs);

kclvm_value_ref_t* kclvm_builtin_str_isdigit(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* _kwargs);

kclvm_value_ref_t* kclvm_builtin_str_islower(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* _kwargs);

kclvm_value_ref_t* kclvm_builtin_str_isspace(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* _kwargs);

kclvm_value_ref_t* kclvm_builtin_str_istitle(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* _kwargs);

kclvm_value_ref_t* kclvm_builtin_str_isupper(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* _kwargs);

kclvm_value_ref_t* kclvm_builtin_str_join(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* _kwargs);

kclvm_value_ref_t* kclvm_builtin_str_lower(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* _kwargs);

kclvm_value_ref_t* kclvm_builtin_str_lstrip(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* _kwargs);

kclvm_value_ref_t* kclvm_builtin_str_removeprefix(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* _kwargs);

kclvm_value_ref_t* kclvm_builtin_str_removesuffix(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* _kwargs);

kclvm_value_ref_t* kclvm_builtin_str_replace(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* _kwargs);

kclvm_value_ref_t* kclvm_builtin_str_rfind(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* _kwargs);

kclvm_value_ref_t* kclvm_builtin_str_rindex(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* _kwargs);

kclvm_value_ref_t* kclvm_builtin_str_rsplit(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* kwargs);

kclvm_value_ref_t* kclvm_builtin_str_rstrip(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* _kwargs);

kclvm_value_ref_t* kclvm_builtin_str_split(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* kwargs);

kclvm_value_ref_t* kclvm_builtin_str_splitlines(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* kwargs);

kclvm_value_ref_t* kclvm_builtin_str_startswith(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* _kwargs);

kclvm_value_ref_t* kclvm_builtin_str_strip(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* _kwargs);

kclvm_value_ref_t* kclvm_builtin_str_title(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* _kwargs);

kclvm_value_ref_t* kclvm_builtin_str_upper(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* _kwargs);

kclvm_value_ref_t* kclvm_builtin_sum(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* _kwargs);

kclvm_value_ref_t* kclvm_builtin_typeof(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* kwargs);

kclvm_value_ref_t* kclvm_builtin_zip(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* _kwargs);

void kclvm_config_attr_map(kclvm_value_ref_t* value, kclvm_char_t* name, kclvm_char_t* type_str);

void kclvm_context_delete(kclvm_context_t* p);

char* kclvm_context_invoke(kclvm_context_t* p, char* method, char* args, char* kwargs);

kclvm_context_t* kclvm_context_new();

kclvm_bool_t kclvm_context_pkgpath_is_imported(kclvm_context_t* ctx, kclvm_char_t* pkgpath);

void kclvm_context_set_debug_mode(kclvm_context_t* p, kclvm_bool_t v);

void kclvm_context_set_disable_none(kclvm_context_t* p, kclvm_bool_t v);

void kclvm_context_set_disable_schema_check(kclvm_context_t* p, kclvm_bool_t v);

void kclvm_context_set_import_names(kclvm_context_t* p, kclvm_value_ref_t* import_names);

void kclvm_context_set_kcl_filename(kclvm_context_t* ctx, char* filename);

void kclvm_context_set_kcl_line_col(kclvm_context_t* ctx, int32_t line, int32_t col);

void kclvm_context_set_kcl_location(kclvm_context_t* p, char* filename, int32_t line, int32_t col);

void kclvm_context_set_kcl_modpath(kclvm_context_t* p, char* module_path);

void kclvm_context_set_kcl_pkgpath(kclvm_context_t* p, char* pkgpath);

void kclvm_context_set_kcl_workdir(kclvm_context_t* p, char* workdir);

void kclvm_context_set_strict_range_check(kclvm_context_t* p, kclvm_bool_t v);

kclvm_value_ref_t* kclvm_convert_collection_value(kclvm_context_t* ctx, kclvm_value_ref_t* value, kclvm_char_t* tpe, kclvm_value_ref_t* is_in_schema);

kclvm_value_ref_t* kclvm_crypto_filesha256(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* _kwargs);

kclvm_value_ref_t* kclvm_crypto_md5(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* _kwargs);

kclvm_value_ref_t* kclvm_crypto_sha1(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* _kwargs);

kclvm_value_ref_t* kclvm_crypto_sha224(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* _kwargs);

kclvm_value_ref_t* kclvm_crypto_sha256(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* _kwargs);

kclvm_value_ref_t* kclvm_crypto_sha384(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* _kwargs);

kclvm_value_ref_t* kclvm_crypto_sha512(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* _kwargs);

kclvm_value_ref_t* kclvm_crypto_uuid(kclvm_context_t* ctx, kclvm_value_ref_t* _args, kclvm_value_ref_t* _kwargs);

kclvm_value_ref_t* kclvm_datetime_date(kclvm_context_t* ctx, kclvm_value_ref_t* _args, kclvm_value_ref_t* _kwargs);

kclvm_value_ref_t* kclvm_datetime_now(kclvm_context_t* ctx, kclvm_value_ref_t* _args, kclvm_value_ref_t* _kwargs);

kclvm_value_ref_t* kclvm_datetime_ticks(kclvm_context_t* ctx, kclvm_value_ref_t* _args, kclvm_value_ref_t* _kwargs);

kclvm_value_ref_t* kclvm_datetime_today(kclvm_context_t* ctx, kclvm_value_ref_t* _args, kclvm_value_ref_t* _kwargs);

void kclvm_default_collection_insert_int_pointer(kclvm_value_ref_t* p, kclvm_char_t* key, uint64_t* ptr);

void kclvm_default_collection_insert_value(kclvm_value_ref_t* p, kclvm_char_t* key, kclvm_value_ref_t* value);

void kclvm_dict_clear(kclvm_value_ref_t* p);

kclvm_value_ref_t* kclvm_dict_get(kclvm_context_t* ctx, kclvm_value_ref_t* p, kclvm_value_ref_t* key);

kclvm_value_ref_t* kclvm_dict_get_entry(kclvm_context_t* ctx, kclvm_value_ref_t* p, kclvm_char_t* key);

kclvm_value_ref_t* kclvm_dict_get_value(kclvm_context_t* ctx, kclvm_value_ref_t* p, kclvm_char_t* key);

kclvm_value_ref_t* kclvm_dict_get_value_by_path(kclvm_context_t* ctx, kclvm_value_ref_t* p, kclvm_char_t* path);

kclvm_bool_t kclvm_dict_has_value(kclvm_value_ref_t* p, kclvm_char_t* key);

void kclvm_dict_insert(kclvm_context_t* ctx, kclvm_value_ref_t* p, kclvm_char_t* key, kclvm_value_ref_t* v, kclvm_size_t op, kclvm_size_t insert_index);

void kclvm_dict_insert_unpack(kclvm_context_t* ctx, kclvm_value_ref_t* p, kclvm_value_ref_t* v);

void kclvm_dict_insert_value(kclvm_context_t* ctx, kclvm_value_ref_t* p, kclvm_value_ref_t* key, kclvm_value_ref_t* v, kclvm_size_t op, kclvm_size_t insert_index);

kclvm_bool_t kclvm_dict_is_override_attr(kclvm_value_ref_t* p, kclvm_char_t* key);

kclvm_value_ref_t* kclvm_dict_keys(kclvm_context_t* ctx, kclvm_value_ref_t* p);

kclvm_size_t kclvm_dict_len(kclvm_value_ref_t* p);

void kclvm_dict_merge(kclvm_context_t* ctx, kclvm_value_ref_t* p, kclvm_char_t* key, kclvm_value_ref_t* v, kclvm_size_t op, kclvm_size_t insert_index);

void kclvm_dict_remove(kclvm_value_ref_t* p, kclvm_char_t* key);

void kclvm_dict_safe_insert(kclvm_context_t* ctx, kclvm_value_ref_t* p, kclvm_char_t* key, kclvm_value_ref_t* v, kclvm_size_t op, kclvm_size_t insert_index);

void kclvm_dict_set_value(kclvm_context_t* ctx, kclvm_value_ref_t* p, kclvm_char_t* key, kclvm_value_ref_t* val);

void kclvm_dict_update(kclvm_value_ref_t* p, kclvm_value_ref_t* v);

void kclvm_dict_update_key_value(kclvm_value_ref_t* p, kclvm_value_ref_t* key, kclvm_value_ref_t* v);

kclvm_value_ref_t* kclvm_dict_values(kclvm_context_t* ctx, kclvm_value_ref_t* p);

kclvm_value_ref_t* kclvm_file_abs(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* kwargs);

kclvm_value_ref_t* kclvm_file_exists(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* kwargs);

kclvm_value_ref_t* kclvm_file_glob(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* kwargs);

kclvm_value_ref_t* kclvm_file_modpath(kclvm_context_t* ctx, kclvm_value_ref_t* _args, kclvm_value_ref_t* _kwargs);

kclvm_value_ref_t* kclvm_file_read(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* kwargs);

kclvm_value_ref_t* kclvm_file_workdir(kclvm_context_t* ctx, kclvm_value_ref_t* _args, kclvm_value_ref_t* _kwargs);

kclvm_value_ref_t* kclvm_iterator_cur_key(kclvm_iterator_t* p);

kclvm_value_ref_t* kclvm_iterator_cur_value(kclvm_iterator_t* p);

void kclvm_iterator_delete(kclvm_iterator_t* p);

kclvm_bool_t kclvm_iterator_is_end(kclvm_iterator_t* p);

kclvm_value_ref_t* kclvm_iterator_next_value(kclvm_iterator_t* p, kclvm_value_ref_t* host);

kclvm_value_ref_t* kclvm_json_decode(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* _kwargs);

kclvm_value_ref_t* kclvm_json_dump_to_file(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* kwargs);

kclvm_value_ref_t* kclvm_json_encode(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* kwargs);

kclvm_value_ref_t* kclvm_json_validate(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* _kwargs);

void kclvm_list_append(kclvm_value_ref_t* p, kclvm_value_ref_t* v);

void kclvm_list_append_bool(kclvm_value_ref_t* p, kclvm_bool_t v);

void kclvm_list_append_float(kclvm_value_ref_t* p, kclvm_float_t v);

void kclvm_list_append_int(kclvm_value_ref_t* p, kclvm_int_t v);

void kclvm_list_append_str(kclvm_value_ref_t* p, kclvm_char_t* v);

void kclvm_list_append_unpack(kclvm_value_ref_t* p, kclvm_value_ref_t* v);

void kclvm_list_clear(kclvm_value_ref_t* p);

kclvm_value_ref_t* kclvm_list_count(kclvm_context_t* ctx, kclvm_value_ref_t* p, kclvm_value_ref_t* item);

kclvm_value_ref_t* kclvm_list_find(kclvm_context_t* ctx, kclvm_value_ref_t* p, kclvm_value_ref_t* item);

kclvm_value_ref_t* kclvm_list_get(kclvm_context_t* ctx, kclvm_value_ref_t* p, kclvm_size_t i);

kclvm_value_ref_t* kclvm_list_get_option(kclvm_context_t* ctx, kclvm_value_ref_t* p, kclvm_size_t i);

void kclvm_list_insert(kclvm_value_ref_t* p, kclvm_value_ref_t* index, kclvm_value_ref_t* value);

kclvm_size_t kclvm_list_len(kclvm_value_ref_t* p);

kclvm_value_ref_t* kclvm_list_pop(kclvm_context_t* ctx, kclvm_value_ref_t* p);

kclvm_value_ref_t* kclvm_list_pop_first(kclvm_context_t* ctx, kclvm_value_ref_t* p);

void kclvm_list_remove_at(kclvm_value_ref_t* p, kclvm_size_t i);

void kclvm_list_resize(kclvm_value_ref_t* p, kclvm_size_t newsize);

void kclvm_list_set(kclvm_value_ref_t* p, kclvm_size_t i, kclvm_value_ref_t* v);

void kclvm_manifests_yaml_stream(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* kwargs);

kclvm_value_ref_t* kclvm_math_ceil(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* _kwargs);

kclvm_value_ref_t* kclvm_math_exp(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* _kwargs);

kclvm_value_ref_t* kclvm_math_expm1(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* _kwargs);

kclvm_value_ref_t* kclvm_math_factorial(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* _kwargs);

kclvm_value_ref_t* kclvm_math_floor(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* _kwargs);

kclvm_value_ref_t* kclvm_math_gcd(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* _kwargs);

kclvm_value_ref_t* kclvm_math_isfinite(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* _kwargs);

kclvm_value_ref_t* kclvm_math_isinf(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* _kwargs);

kclvm_value_ref_t* kclvm_math_isnan(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* _kwargs);

kclvm_value_ref_t* kclvm_math_log(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* _kwargs);

kclvm_value_ref_t* kclvm_math_log10(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* _kwargs);

kclvm_value_ref_t* kclvm_math_log1p(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* _kwargs);

kclvm_value_ref_t* kclvm_math_log2(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* _kwargs);

kclvm_value_ref_t* kclvm_math_modf(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* _kwargs);

kclvm_value_ref_t* kclvm_math_pow(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* _kwargs);

kclvm_value_ref_t* kclvm_math_sqrt(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* _kwargs);

kclvm_value_ref_t* kclvm_net_IP_string(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* _kwargs);

kclvm_value_ref_t* kclvm_net_fqdn(kclvm_context_t* _ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* _kwargs);

kclvm_value_ref_t* kclvm_net_is_IP(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* _kwargs);

kclvm_value_ref_t* kclvm_net_is_IPv4(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* _kwargs);

kclvm_value_ref_t* kclvm_net_is_global_unicast_IP(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* _kwargs);

kclvm_value_ref_t* kclvm_net_is_interface_local_multicast_IP(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* _kwargs);

kclvm_value_ref_t* kclvm_net_is_link_local_multicast_IP(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* _kwargs);

kclvm_value_ref_t* kclvm_net_is_link_local_unicast_IP(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* kwargs);

kclvm_value_ref_t* kclvm_net_is_loopback_IP(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* _kwargs);

kclvm_value_ref_t* kclvm_net_is_multicast_IP(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* _kwargs);

kclvm_value_ref_t* kclvm_net_is_unspecified_IP(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* _kwargs);

kclvm_value_ref_t* kclvm_net_join_host_port(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* _kwargs);

kclvm_value_ref_t* kclvm_net_parse_IP(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* kwargs);

kclvm_value_ref_t* kclvm_net_split_host_port(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* _kwargs);

kclvm_value_ref_t* kclvm_net_to_IP16(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* kwargs);

kclvm_value_ref_t* kclvm_net_to_IP4(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* kwargs);

void kclvm_plugin_init(void* fn_ptr);

kclvm_value_ref_t* kclvm_plugin_invoke(kclvm_context_t* ctx, char* method, kclvm_value_ref_t* args, kclvm_value_ref_t* kwargs);

char* kclvm_plugin_invoke_json(char* method, char* args, char* kwargs);

kclvm_value_ref_t* kclvm_regex_compile(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* _kwargs);

kclvm_value_ref_t* kclvm_regex_findall(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* _kwargs);

kclvm_value_ref_t* kclvm_regex_match(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* _kwargs);

kclvm_value_ref_t* kclvm_regex_replace(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* _kwargs);

kclvm_value_ref_t* kclvm_regex_search(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* _kwargs);

kclvm_value_ref_t* kclvm_regex_split(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* _kwargs);

void kclvm_schema_assert(kclvm_context_t* ctx, kclvm_value_ref_t* value, kclvm_value_ref_t* msg, kclvm_value_ref_t* config_meta);

void kclvm_schema_backtrack_cache(kclvm_context_t* ctx, kclvm_value_ref_t* schema, kclvm_value_ref_t* cache, kclvm_value_ref_t* cal_map, kclvm_char_t* name, kclvm_value_ref_t* runtime_type);

void kclvm_schema_default_settings(kclvm_value_ref_t* schema_value, kclvm_value_ref_t* _config_value, kclvm_value_ref_t* args, kclvm_value_ref_t* kwargs, kclvm_char_t* runtime_type);

void kclvm_schema_do_check_with_index_sign_attr(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* kwargs, uint64_t* check_fn_ptr, kclvm_char_t* attr_name);

kclvm_value_ref_t* kclvm_schema_get_value(kclvm_context_t* ctx, kclvm_value_ref_t* p, kclvm_char_t* key, kclvm_value_ref_t* config, kclvm_value_ref_t* config_meta, kclvm_value_ref_t* cal_map, kclvm_char_t* target_attr, kclvm_value_ref_t* backtrack_level_map, kclvm_value_ref_t* backtrack_cache, kclvm_value_ref_t* args, kclvm_value_ref_t* kwargs);

kclvm_value_ref_t* kclvm_schema_instances(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* kwargs);

void kclvm_schema_optional_check(kclvm_context_t* ctx, kclvm_value_ref_t* p);

void kclvm_schema_value_check(kclvm_context_t* ctx, kclvm_value_ref_t* schema_value, kclvm_value_ref_t* schema_config, kclvm_value_ref_t* _config_meta, kclvm_char_t* schema_name, kclvm_value_ref_t* index_sign_value, kclvm_char_t* key_name, kclvm_char_t* key_type, kclvm_char_t* value_type, kclvm_bool_t _any_other);

kclvm_value_ref_t* kclvm_schema_value_new(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* kwargs, kclvm_value_ref_t* schema_value_or_func, kclvm_value_ref_t* config, kclvm_value_ref_t* config_meta, kclvm_char_t* pkgpath);

void kclvm_scope_add_setter(kclvm_context_t* _ctx, kclvm_eval_scope_t* scope, char* pkg, char* name, uint64_t* setter);

void kclvm_scope_free(kclvm_eval_scope_t* scope);

kclvm_value_ref_t* kclvm_scope_get(kclvm_context_t* ctx, kclvm_eval_scope_t* scope, char* pkg, char* name, char* target, kclvm_value_ref_t* default);

kclvm_eval_scope_t* kclvm_scope_new();

void kclvm_scope_set(kclvm_context_t* _ctx, kclvm_eval_scope_t* scope, char* pkg, char* name, kclvm_value_ref_t* value);

kclvm_value_ref_t* kclvm_units_to_G(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* _kwargs);

kclvm_value_ref_t* kclvm_units_to_Gi(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* _kwargs);

kclvm_value_ref_t* kclvm_units_to_K(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* _kwargs);

kclvm_value_ref_t* kclvm_units_to_Ki(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* _kwargs);

kclvm_value_ref_t* kclvm_units_to_M(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* kwargs);

kclvm_value_ref_t* kclvm_units_to_Mi(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* _kwargs);

kclvm_value_ref_t* kclvm_units_to_P(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* _kwargs);

kclvm_value_ref_t* kclvm_units_to_Pi(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* _kwargs);

kclvm_value_ref_t* kclvm_units_to_T(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* _kwargs);

kclvm_value_ref_t* kclvm_units_to_Ti(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* _kwargs);

kclvm_value_ref_t* kclvm_units_to_m(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* _kwargs);

kclvm_value_ref_t* kclvm_units_to_n(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* _kwargs);

kclvm_value_ref_t* kclvm_units_to_u(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* _kwargs);

kclvm_value_ref_t* kclvm_value_Bool(kclvm_context_t* ctx, kclvm_bool_t v);

kclvm_decorator_value_t* kclvm_value_Decorator(kclvm_context_t* ctx, kclvm_char_t* name, kclvm_value_ref_t* args, kclvm_value_ref_t* kwargs, kclvm_value_ref_t* config_meta, kclvm_char_t* attr_name, kclvm_value_ref_t* config_value, kclvm_value_ref_t* is_schema_target);

kclvm_value_ref_t* kclvm_value_Dict(kclvm_context_t* ctx);

kclvm_value_ref_t* kclvm_value_False(kclvm_context_t* ctx);

kclvm_value_ref_t* kclvm_value_Float(kclvm_context_t* ctx, kclvm_float_t v);

kclvm_value_ref_t* kclvm_value_Function(kclvm_context_t* ctx, uint64_t* fn_ptr, kclvm_value_ref_t* closure, kclvm_char_t* name, kclvm_bool_t is_external);

kclvm_value_ref_t* kclvm_value_Function_using_ptr(kclvm_context_t* ctx, uint64_t* fn_ptr, kclvm_char_t* name);

kclvm_value_ref_t* kclvm_value_Int(kclvm_context_t* ctx, kclvm_int_t v);

kclvm_value_ref_t* kclvm_value_List(kclvm_context_t* ctx);

kclvm_value_ref_t* kclvm_value_List10(kclvm_context_t* ctx, kclvm_value_ref_t* v1, kclvm_value_ref_t* v2, kclvm_value_ref_t* v3, kclvm_value_ref_t* v4, kclvm_value_ref_t* v5, kclvm_value_ref_t* v6, kclvm_value_ref_t* v7, kclvm_value_ref_t* v8, kclvm_value_ref_t* v9, kclvm_value_ref_t* v10);

kclvm_value_ref_t* kclvm_value_List6(kclvm_context_t* ctx, kclvm_value_ref_t* v1, kclvm_value_ref_t* v2, kclvm_value_ref_t* v3, kclvm_value_ref_t* v4, kclvm_value_ref_t* v5, kclvm_value_ref_t* v6);

kclvm_value_ref_t* kclvm_value_None(kclvm_context_t* ctx);

kclvm_value_ref_t* kclvm_value_Schema(kclvm_context_t* ctx);

kclvm_value_ref_t* kclvm_value_Str(kclvm_context_t* ctx, kclvm_char_t* v);

kclvm_char_t* kclvm_value_Str_ptr(kclvm_value_ref_t* p);

kclvm_value_ref_t* kclvm_value_True(kclvm_context_t* ctx);

kclvm_value_ref_t* kclvm_value_Undefined(kclvm_context_t* ctx);

kclvm_value_ref_t* kclvm_value_Unit(kclvm_context_t* ctx, kclvm_float_t v, kclvm_int_t raw, kclvm_char_t* unit);

kclvm_value_ref_t* kclvm_value_as(kclvm_context_t* ctx, kclvm_value_ref_t* a, kclvm_value_ref_t* b);

uint64_t* kclvm_value_check_function_ptr(kclvm_value_ref_t* p);

kclvm_value_ref_t* kclvm_value_cmp_equal_to(kclvm_context_t* ctx, kclvm_value_ref_t* a, kclvm_value_ref_t* b);

kclvm_value_ref_t* kclvm_value_cmp_greater_than(kclvm_context_t* ctx, kclvm_value_ref_t* a, kclvm_value_ref_t* b);

kclvm_value_ref_t* kclvm_value_cmp_greater_than_or_equal(kclvm_context_t* ctx, kclvm_value_ref_t* a, kclvm_value_ref_t* b);

kclvm_value_ref_t* kclvm_value_cmp_less_than(kclvm_context_t* ctx, kclvm_value_ref_t* a, kclvm_value_ref_t* b);

kclvm_value_ref_t* kclvm_value_cmp_less_than_or_equal(kclvm_context_t* ctx, kclvm_value_ref_t* a, kclvm_value_ref_t* b);

kclvm_value_ref_t* kclvm_value_cmp_not_equal_to(kclvm_context_t* ctx, kclvm_value_ref_t* a, kclvm_value_ref_t* b);

kclvm_value_ref_t* kclvm_value_deep_copy(kclvm_context_t* ctx, kclvm_value_ref_t* p);

void kclvm_value_delete(kclvm_value_ref_t* p);

kclvm_value_ref_t* kclvm_value_from_json(kclvm_context_t* ctx, kclvm_char_t* s);

kclvm_value_ref_t* kclvm_value_function_invoke(kclvm_value_ref_t* p, kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* kwargs, kclvm_char_t* pkgpath, kclvm_value_ref_t* is_in_schema);

uint64_t* kclvm_value_function_ptr(kclvm_value_ref_t* p);

kclvm_value_ref_t* kclvm_value_in(kclvm_context_t* ctx, kclvm_value_ref_t* a, kclvm_value_ref_t* b);

kclvm_value_ref_t* kclvm_value_is(kclvm_context_t* ctx, kclvm_value_ref_t* a, kclvm_value_ref_t* b);

kclvm_value_ref_t* kclvm_value_is_not(kclvm_context_t* ctx, kclvm_value_ref_t* a, kclvm_value_ref_t* b);

kclvm_bool_t kclvm_value_is_truthy(kclvm_value_ref_t* p);

kclvm_iterator_t* kclvm_value_iter(kclvm_value_ref_t* p);

kclvm_size_t kclvm_value_len(kclvm_value_ref_t* p);

kclvm_value_ref_t* kclvm_value_load_attr(kclvm_context_t* ctx, kclvm_value_ref_t* obj, kclvm_char_t* key);

kclvm_value_ref_t* kclvm_value_load_attr_option(kclvm_context_t* ctx, kclvm_value_ref_t* p, kclvm_char_t* key);

kclvm_value_ref_t* kclvm_value_logic_and(kclvm_context_t* ctx, kclvm_value_ref_t* a, kclvm_value_ref_t* b);

kclvm_value_ref_t* kclvm_value_logic_or(kclvm_context_t* ctx, kclvm_value_ref_t* a, kclvm_value_ref_t* b);

kclvm_value_ref_t* kclvm_value_not_in(kclvm_context_t* ctx, kclvm_value_ref_t* a, kclvm_value_ref_t* b);

kclvm_value_ref_t* kclvm_value_op_add(kclvm_context_t* ctx, kclvm_value_ref_t* a, kclvm_value_ref_t* b);

kclvm_value_ref_t* kclvm_value_op_aug_add(kclvm_context_t* ctx, kclvm_value_ref_t* a, kclvm_value_ref_t* b);

kclvm_value_ref_t* kclvm_value_op_aug_bit_and(kclvm_context_t* _ctx, kclvm_value_ref_t* a, kclvm_value_ref_t* b);

kclvm_value_ref_t* kclvm_value_op_aug_bit_lshift(kclvm_context_t* ctx, kclvm_value_ref_t* a, kclvm_value_ref_t* b);

kclvm_value_ref_t* kclvm_value_op_aug_bit_or(kclvm_context_t* ctx, kclvm_value_ref_t* a, kclvm_value_ref_t* b);

kclvm_value_ref_t* kclvm_value_op_aug_bit_rshift(kclvm_context_t* ctx, kclvm_value_ref_t* a, kclvm_value_ref_t* b);

kclvm_value_ref_t* kclvm_value_op_aug_bit_xor(kclvm_context_t* _ctx, kclvm_value_ref_t* a, kclvm_value_ref_t* b);

kclvm_value_ref_t* kclvm_value_op_aug_div(kclvm_context_t* _ctx, kclvm_value_ref_t* a, kclvm_value_ref_t* b);

kclvm_value_ref_t* kclvm_value_op_aug_floor_div(kclvm_context_t* _ctx, kclvm_value_ref_t* a, kclvm_value_ref_t* b);

kclvm_value_ref_t* kclvm_value_op_aug_mod(kclvm_context_t* _ctx, kclvm_value_ref_t* a, kclvm_value_ref_t* b);

kclvm_value_ref_t* kclvm_value_op_aug_mul(kclvm_context_t* ctx, kclvm_value_ref_t* a, kclvm_value_ref_t* b);

kclvm_value_ref_t* kclvm_value_op_aug_pow(kclvm_context_t* ctx, kclvm_value_ref_t* a, kclvm_value_ref_t* b);

kclvm_value_ref_t* kclvm_value_op_aug_sub(kclvm_context_t* ctx, kclvm_value_ref_t* a, kclvm_value_ref_t* b);

kclvm_value_ref_t* kclvm_value_op_bit_and(kclvm_context_t* ctx, kclvm_value_ref_t* a, kclvm_value_ref_t* b);

kclvm_value_ref_t* kclvm_value_op_bit_lshift(kclvm_context_t* ctx, kclvm_value_ref_t* a, kclvm_value_ref_t* b);

kclvm_value_ref_t* kclvm_value_op_bit_or(kclvm_context_t* ctx, kclvm_value_ref_t* a, kclvm_value_ref_t* b);

kclvm_value_ref_t* kclvm_value_op_bit_rshift(kclvm_context_t* ctx, kclvm_value_ref_t* a, kclvm_value_ref_t* b);

kclvm_value_ref_t* kclvm_value_op_bit_xor(kclvm_context_t* ctx, kclvm_value_ref_t* a, kclvm_value_ref_t* b);

kclvm_value_ref_t* kclvm_value_op_div(kclvm_context_t* ctx, kclvm_value_ref_t* a, kclvm_value_ref_t* b);

kclvm_value_ref_t* kclvm_value_op_floor_div(kclvm_context_t* ctx, kclvm_value_ref_t* a, kclvm_value_ref_t* b);

kclvm_value_ref_t* kclvm_value_op_mod(kclvm_context_t* ctx, kclvm_value_ref_t* a, kclvm_value_ref_t* b);

kclvm_value_ref_t* kclvm_value_op_mul(kclvm_context_t* ctx, kclvm_value_ref_t* a, kclvm_value_ref_t* b);

kclvm_value_ref_t* kclvm_value_op_pow(kclvm_context_t* ctx, kclvm_value_ref_t* a, kclvm_value_ref_t* b);

kclvm_value_ref_t* kclvm_value_op_sub(kclvm_context_t* ctx, kclvm_value_ref_t* a, kclvm_value_ref_t* b);

kclvm_value_ref_t* kclvm_value_plan_to_json(kclvm_context_t* ctx, kclvm_value_ref_t* p);

kclvm_value_ref_t* kclvm_value_plan_to_yaml(kclvm_context_t* ctx, kclvm_value_ref_t* p);

void kclvm_value_remove_item(kclvm_value_ref_t* a, kclvm_value_ref_t* b);

kclvm_value_ref_t* kclvm_value_schema_function(kclvm_context_t* ctx, uint64_t* fn_ptr, uint64_t* check_fn_ptr, kclvm_value_ref_t* attr_map, kclvm_char_t* tpe);

kclvm_value_ref_t* kclvm_value_schema_with_config(kclvm_context_t* ctx, kclvm_value_ref_t* schema_dict, kclvm_value_ref_t* config, kclvm_value_ref_t* config_meta, kclvm_char_t* name, kclvm_char_t* pkgpath, kclvm_value_ref_t* is_sub_schema, kclvm_value_ref_t* record_instance, kclvm_value_ref_t* instance_pkgpath, kclvm_value_ref_t* optional_mapping, kclvm_value_ref_t* args, kclvm_value_ref_t* kwargs);

kclvm_value_ref_t* kclvm_value_slice(kclvm_context_t* ctx, kclvm_value_ref_t* x, kclvm_value_ref_t* a, kclvm_value_ref_t* b, kclvm_value_ref_t* step);

kclvm_value_ref_t* kclvm_value_slice_option(kclvm_context_t* ctx, kclvm_value_ref_t* x, kclvm_value_ref_t* a, kclvm_value_ref_t* b, kclvm_value_ref_t* step);

kclvm_value_ref_t* kclvm_value_subscr(kclvm_context_t* ctx, kclvm_value_ref_t* a, kclvm_value_ref_t* b);

kclvm_value_ref_t* kclvm_value_subscr_option(kclvm_context_t* ctx, kclvm_value_ref_t* a, kclvm_value_ref_t* b);

kclvm_value_ref_t* kclvm_value_to_json_value(kclvm_context_t* ctx, kclvm_value_ref_t* p);

kclvm_value_ref_t* kclvm_value_to_json_value_with_null(kclvm_context_t* ctx, kclvm_value_ref_t* p);

kclvm_value_ref_t* kclvm_value_to_str_value(kclvm_context_t* ctx, kclvm_value_ref_t* p);

kclvm_value_ref_t* kclvm_value_to_yaml_value(kclvm_context_t* ctx, kclvm_value_ref_t* p);

kclvm_value_ref_t* kclvm_value_unary_l_not(kclvm_context_t* ctx, kclvm_value_ref_t* a);

kclvm_value_ref_t* kclvm_value_unary_minus(kclvm_context_t* ctx, kclvm_value_ref_t* a);

kclvm_value_ref_t* kclvm_value_unary_not(kclvm_context_t* ctx, kclvm_value_ref_t* a);

kclvm_value_ref_t* kclvm_value_unary_plus(kclvm_context_t* ctx, kclvm_value_ref_t* a);

kclvm_value_ref_t* kclvm_value_union(kclvm_context_t* ctx, kclvm_value_ref_t* schema, kclvm_value_ref_t* b);

kclvm_value_ref_t* kclvm_value_union_all(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* _kwargs);

kclvm_value_ref_t* kclvm_yaml_decode(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* _kwargs);

kclvm_value_ref_t* kclvm_yaml_decode_all(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* _kwargs);

kclvm_value_ref_t* kclvm_yaml_dump_all_to_file(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* kwargs);

kclvm_value_ref_t* kclvm_yaml_dump_to_file(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* kwargs);

kclvm_value_ref_t* kclvm_yaml_encode(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* kwargs);

kclvm_value_ref_t* kclvm_yaml_encode_all(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* kwargs);

kclvm_value_ref_t* kclvm_yaml_validate(kclvm_context_t* ctx, kclvm_value_ref_t* args, kclvm_value_ref_t* _kwargs);

#ifdef __cplusplus
} // extern "C"
#endif

#endif // _kclvm_h_
