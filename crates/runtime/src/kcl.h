// Copyright The KCL Authors. All rights reserved.

// Auto generated, DONOT EDIT!!!

#pragma once

#ifndef _kcl_h_
#define _kcl_h_

#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

// please keep same as 'kcl/runtime/src/kind/mod.rs#Kind'

enum kcl_kind_t {
    Invalid = 0,
    Undefined = 1,
    None = 2,
    Bool = 3,
    Int = 4,
    Float = 5,
    Str = 6,
    List = 7,
    Dict = 8,
    Schema = 9,
    Error = 10,
    Any = 11,
    Union = 12,
    BoolLit = 13,
    IntLit = 14,
    FloatLit = 15,
    StrLit = 16,
    Func = 17,
    Max = 18,
};

typedef int8_t kcl_bool_t;

typedef struct kcl_buffer_t kcl_buffer_t;

typedef char kcl_char_t;

typedef struct kcl_context_t kcl_context_t;

typedef struct kcl_decorator_value_t kcl_decorator_value_t;

typedef struct kcl_eval_scope_t kcl_eval_scope_t;

typedef double kcl_float_t;

typedef int64_t kcl_int_t;

typedef struct kcl_iterator_t kcl_iterator_t;

typedef enum kcl_kind_t kcl_kind_t;

typedef int32_t kcl_size_t;

typedef struct kcl_type_t kcl_type_t;

typedef struct kcl_value_ref_t kcl_value_ref_t;

typedef struct kcl_value_t kcl_value_t;

void kcl_assert(kcl_context_t* ctx, kcl_value_ref_t* value, kcl_value_ref_t* msg);

kcl_value_ref_t* kcl_base32_decode(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* kwargs);

kcl_value_ref_t* kcl_base32_encode(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* kwargs);

kcl_value_ref_t* kcl_base64_decode(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* kwargs);

kcl_value_ref_t* kcl_base64_encode(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* kwargs);

kcl_value_ref_t* kcl_builtin_abs(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* kwargs);

kcl_value_ref_t* kcl_builtin_all_true(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* kwargs);

kcl_value_ref_t* kcl_builtin_any_true(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* kwargs);

kcl_value_ref_t* kcl_builtin_bin(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* kwargs);

kcl_value_ref_t* kcl_builtin_bool(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* kwargs);

kcl_value_ref_t* kcl_builtin_dict(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* kwargs);

kcl_value_ref_t* kcl_builtin_float(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* kwargs);

kcl_value_ref_t* kcl_builtin_hex(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* kwargs);

kcl_value_ref_t* kcl_builtin_int(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* kwargs);

kcl_value_ref_t* kcl_builtin_isnullish(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* kwargs);

kcl_value_ref_t* kcl_builtin_isunique(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* kwargs);

kcl_value_ref_t* kcl_builtin_len(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* kwargs);

kcl_value_ref_t* kcl_builtin_list(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* kwargs);

kcl_value_ref_t* kcl_builtin_max(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* _kwargs);

kcl_value_ref_t* kcl_builtin_min(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* _kwargs);

kcl_value_ref_t* kcl_builtin_multiplyof(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* kwargs);

kcl_value_ref_t* kcl_builtin_oct(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* kwargs);

kcl_value_ref_t* kcl_builtin_option(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* kwargs);

void kcl_builtin_option_init(kcl_context_t* ctx, char* key, char* value);

kcl_value_ref_t* kcl_builtin_option_reset(kcl_context_t* ctx, kcl_value_ref_t* _args, kcl_value_ref_t* _kwargs);

kcl_value_ref_t* kcl_builtin_ord(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* kwargs);

kcl_value_ref_t* kcl_builtin_pow(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* kwargs);

kcl_value_ref_t* kcl_builtin_print(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* kwargs);

kcl_value_ref_t* kcl_builtin_range(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* kwargs);

kcl_value_ref_t* kcl_builtin_round(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* kwargs);

kcl_value_ref_t* kcl_builtin_sorted(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* kwargs);

kcl_value_ref_t* kcl_builtin_str(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* kwargs);

kcl_value_ref_t* kcl_builtin_str_capitalize(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* _kwargs);

kcl_value_ref_t* kcl_builtin_str_chars(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* _kwargs);

kcl_value_ref_t* kcl_builtin_str_count(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* _kwargs);

kcl_value_ref_t* kcl_builtin_str_endswith(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* _kwargs);

kcl_value_ref_t* kcl_builtin_str_find(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* _kwargs);

kcl_value_ref_t* kcl_builtin_str_format(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* kwargs);

kcl_value_ref_t* kcl_builtin_str_index(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* _kwargs);

kcl_value_ref_t* kcl_builtin_str_isalnum(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* _kwargs);

kcl_value_ref_t* kcl_builtin_str_isalpha(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* _kwargs);

kcl_value_ref_t* kcl_builtin_str_isdigit(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* _kwargs);

kcl_value_ref_t* kcl_builtin_str_islower(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* _kwargs);

kcl_value_ref_t* kcl_builtin_str_isspace(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* _kwargs);

kcl_value_ref_t* kcl_builtin_str_istitle(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* _kwargs);

kcl_value_ref_t* kcl_builtin_str_isupper(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* _kwargs);

kcl_value_ref_t* kcl_builtin_str_join(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* _kwargs);

kcl_value_ref_t* kcl_builtin_str_lower(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* _kwargs);

kcl_value_ref_t* kcl_builtin_str_lstrip(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* _kwargs);

kcl_value_ref_t* kcl_builtin_str_removeprefix(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* _kwargs);

kcl_value_ref_t* kcl_builtin_str_removesuffix(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* _kwargs);

kcl_value_ref_t* kcl_builtin_str_replace(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* _kwargs);

kcl_value_ref_t* kcl_builtin_str_rfind(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* _kwargs);

kcl_value_ref_t* kcl_builtin_str_rindex(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* _kwargs);

kcl_value_ref_t* kcl_builtin_str_rsplit(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* kwargs);

kcl_value_ref_t* kcl_builtin_str_rstrip(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* _kwargs);

kcl_value_ref_t* kcl_builtin_str_split(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* kwargs);

kcl_value_ref_t* kcl_builtin_str_splitlines(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* kwargs);

kcl_value_ref_t* kcl_builtin_str_startswith(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* _kwargs);

kcl_value_ref_t* kcl_builtin_str_strip(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* _kwargs);

kcl_value_ref_t* kcl_builtin_str_title(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* _kwargs);

kcl_value_ref_t* kcl_builtin_str_upper(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* _kwargs);

kcl_value_ref_t* kcl_builtin_sum(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* kwargs);

kcl_value_ref_t* kcl_builtin_typeof(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* kwargs);

kcl_value_ref_t* kcl_builtin_zip(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* _kwargs);

void kcl_config_attr_map(kcl_value_ref_t* value, kcl_char_t* name, kcl_char_t* type_str);

void kcl_context_delete(kcl_context_t* p);

char* kcl_context_invoke(kcl_context_t* p, char* method, char* args, char* kwargs);

kcl_context_t* kcl_context_new();

kcl_bool_t kcl_context_pkgpath_is_imported(kcl_context_t* ctx, kcl_char_t* pkgpath);

void kcl_context_set_debug_mode(kcl_context_t* p, kcl_bool_t v);

void kcl_context_set_disable_none(kcl_context_t* p, kcl_bool_t v);

void kcl_context_set_disable_schema_check(kcl_context_t* p, kcl_bool_t v);

void kcl_context_set_import_names(kcl_context_t* p, kcl_value_ref_t* import_names);

void kcl_context_set_kcl_filename(kcl_context_t* ctx, char* filename);

void kcl_context_set_kcl_line_col(kcl_context_t* ctx, int32_t line, int32_t col);

void kcl_context_set_kcl_location(kcl_context_t* p, char* filename, int32_t line, int32_t col);

void kcl_context_set_kcl_modpath(kcl_context_t* p, char* module_path);

void kcl_context_set_kcl_pkgpath(kcl_context_t* p, char* pkgpath);

void kcl_context_set_kcl_workdir(kcl_context_t* p, char* workdir);

void kcl_context_set_strict_range_check(kcl_context_t* p, kcl_bool_t v);

kcl_value_ref_t* kcl_convert_collection_value(kcl_context_t* ctx, kcl_value_ref_t* value, kcl_char_t* tpe, kcl_value_ref_t* is_in_schema);

kcl_value_ref_t* kcl_crypto_blake3(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* kwargs);

kcl_value_ref_t* kcl_crypto_fileblake3(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* kwargs);

kcl_value_ref_t* kcl_crypto_filesha256(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* kwargs);

kcl_value_ref_t* kcl_crypto_filesha512(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* kwargs);

kcl_value_ref_t* kcl_crypto_md5(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* kwargs);

kcl_value_ref_t* kcl_crypto_sha1(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* kwargs);

kcl_value_ref_t* kcl_crypto_sha224(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* kwargs);

kcl_value_ref_t* kcl_crypto_sha256(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* kwargs);

kcl_value_ref_t* kcl_crypto_sha384(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* kwargs);

kcl_value_ref_t* kcl_crypto_sha512(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* kwargs);

kcl_value_ref_t* kcl_crypto_uuid(kcl_context_t* ctx, kcl_value_ref_t* _args, kcl_value_ref_t* _kwargs);

kcl_value_ref_t* kcl_datetime_date(kcl_context_t* ctx, kcl_value_ref_t* _args, kcl_value_ref_t* _kwargs);

kcl_value_ref_t* kcl_datetime_now(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* kwargs);

kcl_value_ref_t* kcl_datetime_ticks(kcl_context_t* ctx, kcl_value_ref_t* _args, kcl_value_ref_t* _kwargs);

kcl_value_ref_t* kcl_datetime_today(kcl_context_t* ctx, kcl_value_ref_t* _args, kcl_value_ref_t* _kwargs);

kcl_value_ref_t* kcl_datetime_validate(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* kwargs);

void kcl_default_collection_insert_int_pointer(kcl_value_ref_t* p, kcl_char_t* key, uint64_t* ptr);

void kcl_default_collection_insert_value(kcl_value_ref_t* p, kcl_char_t* key, kcl_value_ref_t* value);

void kcl_dict_clear(kcl_value_ref_t* p);

kcl_value_ref_t* kcl_dict_get(kcl_context_t* ctx, kcl_value_ref_t* p, kcl_value_ref_t* key);

kcl_value_ref_t* kcl_dict_get_entry(kcl_context_t* ctx, kcl_value_ref_t* p, kcl_char_t* key);

kcl_value_ref_t* kcl_dict_get_value(kcl_context_t* ctx, kcl_value_ref_t* p, kcl_char_t* key);

kcl_value_ref_t* kcl_dict_get_value_by_path(kcl_context_t* ctx, kcl_value_ref_t* p, kcl_char_t* path);

kcl_bool_t kcl_dict_has_value(kcl_value_ref_t* p, kcl_char_t* key);

void kcl_dict_insert(kcl_context_t* ctx, kcl_value_ref_t* p, kcl_char_t* key, kcl_value_ref_t* v, kcl_size_t op, kcl_size_t insert_index, kcl_bool_t has_insert_index);

void kcl_dict_insert_unpack(kcl_context_t* ctx, kcl_value_ref_t* p, kcl_value_ref_t* v);

void kcl_dict_insert_value(kcl_context_t* ctx, kcl_value_ref_t* p, kcl_value_ref_t* key, kcl_value_ref_t* v, kcl_size_t op, kcl_size_t insert_index, kcl_bool_t has_insert_index);

kcl_bool_t kcl_dict_is_override_attr(kcl_value_ref_t* p, kcl_char_t* key);

kcl_value_ref_t* kcl_dict_keys(kcl_context_t* ctx, kcl_value_ref_t* p);

kcl_size_t kcl_dict_len(kcl_value_ref_t* p);

void kcl_dict_merge(kcl_context_t* ctx, kcl_value_ref_t* p, kcl_char_t* key, kcl_value_ref_t* v, kcl_size_t op, kcl_size_t insert_index, kcl_bool_t has_insert_index);

void kcl_dict_remove(kcl_value_ref_t* p, kcl_char_t* key);

void kcl_dict_safe_insert(kcl_context_t* ctx, kcl_value_ref_t* p, kcl_char_t* key, kcl_value_ref_t* v, kcl_size_t op, kcl_size_t insert_index, kcl_bool_t has_insert_index);

void kcl_dict_set_value(kcl_context_t* ctx, kcl_value_ref_t* p, kcl_char_t* key, kcl_value_ref_t* val);

void kcl_dict_update(kcl_value_ref_t* p, kcl_value_ref_t* v);

void kcl_dict_update_key_value(kcl_value_ref_t* p, kcl_value_ref_t* key, kcl_value_ref_t* v);

kcl_value_ref_t* kcl_dict_values(kcl_context_t* ctx, kcl_value_ref_t* p);

kcl_value_ref_t* kcl_file_abs(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* kwargs);

kcl_value_ref_t* kcl_file_append(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* kwargs);

kcl_value_ref_t* kcl_file_cp(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* kwargs);

kcl_value_ref_t* kcl_file_current(kcl_context_t* ctx, kcl_value_ref_t* _args, kcl_value_ref_t* _kwargs);

kcl_value_ref_t* kcl_file_delete(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* kwargs);

kcl_value_ref_t* kcl_file_exists(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* kwargs);

kcl_value_ref_t* kcl_file_glob(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* kwargs);

kcl_value_ref_t* kcl_file_mkdir(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* kwargs);

kcl_value_ref_t* kcl_file_modpath(kcl_context_t* ctx, kcl_value_ref_t* _args, kcl_value_ref_t* _kwargs);

kcl_value_ref_t* kcl_file_mv(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* kwargs);

kcl_value_ref_t* kcl_file_read(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* kwargs);

kcl_value_ref_t* kcl_file_read_env(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* kwargs);

kcl_value_ref_t* kcl_file_size(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* kwargs);

kcl_value_ref_t* kcl_file_workdir(kcl_context_t* ctx, kcl_value_ref_t* _args, kcl_value_ref_t* _kwargs);

kcl_value_ref_t* kcl_file_write(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* kwargs);

kcl_value_ref_t* kcl_iterator_cur_key(kcl_iterator_t* p);

kcl_value_ref_t* kcl_iterator_cur_value(kcl_iterator_t* p);

void kcl_iterator_delete(kcl_iterator_t* p);

kcl_bool_t kcl_iterator_is_end(kcl_iterator_t* p);

kcl_value_ref_t* kcl_iterator_next_value(kcl_iterator_t* p, kcl_value_ref_t* host);

kcl_value_ref_t* kcl_json_decode(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* kwargs);

kcl_value_ref_t* kcl_json_dump_to_file(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* kwargs);

kcl_value_ref_t* kcl_json_encode(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* kwargs);

kcl_value_ref_t* kcl_json_validate(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* kwargs);

void kcl_list_append(kcl_value_ref_t* p, kcl_value_ref_t* v);

void kcl_list_append_bool(kcl_value_ref_t* p, kcl_bool_t v);

void kcl_list_append_float(kcl_value_ref_t* p, kcl_float_t v);

void kcl_list_append_int(kcl_value_ref_t* p, kcl_int_t v);

void kcl_list_append_str(kcl_value_ref_t* p, kcl_char_t* v);

void kcl_list_append_unpack(kcl_value_ref_t* p, kcl_value_ref_t* v);

void kcl_list_clear(kcl_value_ref_t* p);

kcl_value_ref_t* kcl_list_count(kcl_context_t* ctx, kcl_value_ref_t* p, kcl_value_ref_t* item);

kcl_value_ref_t* kcl_list_find(kcl_context_t* ctx, kcl_value_ref_t* p, kcl_value_ref_t* item);

kcl_value_ref_t* kcl_list_get(kcl_context_t* ctx, kcl_value_ref_t* p, kcl_size_t i);

kcl_value_ref_t* kcl_list_get_option(kcl_context_t* ctx, kcl_value_ref_t* p, kcl_size_t i);

void kcl_list_insert(kcl_value_ref_t* p, kcl_value_ref_t* index, kcl_value_ref_t* value);

kcl_size_t kcl_list_len(kcl_value_ref_t* p);

kcl_value_ref_t* kcl_list_pop(kcl_context_t* ctx, kcl_value_ref_t* p);

kcl_value_ref_t* kcl_list_pop_first(kcl_context_t* ctx, kcl_value_ref_t* p);

void kcl_list_remove_at(kcl_value_ref_t* p, kcl_size_t i);

void kcl_list_resize(kcl_value_ref_t* p, kcl_size_t newsize);

void kcl_list_set(kcl_value_ref_t* p, kcl_size_t i, kcl_value_ref_t* v);

kcl_value_ref_t* kcl_manifests_yaml_stream(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* kwargs);

kcl_value_ref_t* kcl_math_ceil(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* kwargs);

kcl_value_ref_t* kcl_math_exp(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* kwargs);

kcl_value_ref_t* kcl_math_expm1(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* kwargs);

kcl_value_ref_t* kcl_math_factorial(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* kwargs);

kcl_value_ref_t* kcl_math_floor(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* kwargs);

kcl_value_ref_t* kcl_math_gcd(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* kwargs);

kcl_value_ref_t* kcl_math_isfinite(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* kwargs);

kcl_value_ref_t* kcl_math_isinf(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* kwargs);

kcl_value_ref_t* kcl_math_isnan(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* kwargs);

kcl_value_ref_t* kcl_math_log(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* kwargs);

kcl_value_ref_t* kcl_math_log10(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* kwargs);

kcl_value_ref_t* kcl_math_log1p(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* kwargs);

kcl_value_ref_t* kcl_math_log2(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* kwargs);

kcl_value_ref_t* kcl_math_modf(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* kwargs);

kcl_value_ref_t* kcl_math_pow(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* kwargs);

kcl_value_ref_t* kcl_math_sqrt(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* kwargs);

kcl_value_ref_t* kcl_net_CIDR_host(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* kwargs);

kcl_value_ref_t* kcl_net_CIDR_netmask(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* kwargs);

kcl_value_ref_t* kcl_net_CIDR_subnet(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* kwargs);

kcl_value_ref_t* kcl_net_CIDR_subnets(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* kwargs);

kcl_value_ref_t* kcl_net_IP_string(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* kwargs);

kcl_value_ref_t* kcl_net_fqdn(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* kwargs);

kcl_value_ref_t* kcl_net_is_IP(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* kwargs);

kcl_value_ref_t* kcl_net_is_IP_in_CIDR(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* kwargs);

kcl_value_ref_t* kcl_net_is_IPv4(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* kwargs);

kcl_value_ref_t* kcl_net_is_global_unicast_IP(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* kwargs);

kcl_value_ref_t* kcl_net_is_interface_local_multicast_IP(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* kwargs);

kcl_value_ref_t* kcl_net_is_link_local_multicast_IP(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* kwargs);

kcl_value_ref_t* kcl_net_is_link_local_unicast_IP(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* kwargs);

kcl_value_ref_t* kcl_net_is_loopback_IP(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* kwargs);

kcl_value_ref_t* kcl_net_is_multicast_IP(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* kwargs);

kcl_value_ref_t* kcl_net_is_unspecified_IP(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* kwargs);

kcl_value_ref_t* kcl_net_join_host_port(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* kwargs);

kcl_value_ref_t* kcl_net_parse_CIDR(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* kwargs);

kcl_value_ref_t* kcl_net_parse_IP(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* kwargs);

kcl_value_ref_t* kcl_net_split_host_port(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* kwargs);

kcl_value_ref_t* kcl_net_to_IP4(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* kwargs);

kcl_value_ref_t* kcl_net_to_IP6(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* kwargs);

void kcl_plugin_init(void* fn_ptr);

kcl_value_ref_t* kcl_plugin_invoke(kcl_context_t* ctx, char* method, kcl_value_ref_t* args, kcl_value_ref_t* kwargs);

char* kcl_plugin_invoke_json(char* method, char* args, char* kwargs);

kcl_value_ref_t* kcl_regex_compile(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* kwargs);

kcl_value_ref_t* kcl_regex_findall(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* kwargs);

kcl_value_ref_t* kcl_regex_match(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* kwargs);

kcl_value_ref_t* kcl_regex_replace(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* kwargs);

kcl_value_ref_t* kcl_regex_search(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* kwargs);

kcl_value_ref_t* kcl_regex_split(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* kwargs);

kcl_value_ref_t* kcl_runtime_catch(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* kwargs);

void kcl_schema_assert(kcl_context_t* ctx, kcl_value_ref_t* value, kcl_value_ref_t* msg, kcl_value_ref_t* config_meta);

void kcl_schema_backtrack_cache(kcl_context_t* ctx, kcl_value_ref_t* schema, kcl_value_ref_t* cache, kcl_value_ref_t* cal_map, kcl_char_t* name, kcl_value_ref_t* runtime_type);

void kcl_schema_default_settings(kcl_value_ref_t* schema_value, kcl_value_ref_t* _config_value, kcl_value_ref_t* args, kcl_value_ref_t* kwargs, kcl_char_t* runtime_type);

void kcl_schema_do_check_with_index_sign_attr(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* kwargs, uint64_t* check_fn_ptr, kcl_char_t* attr_name);

kcl_value_ref_t* kcl_schema_get_value(kcl_context_t* ctx, kcl_value_ref_t* p, kcl_char_t* key, kcl_value_ref_t* config, kcl_value_ref_t* config_meta, kcl_value_ref_t* cal_map, kcl_char_t* target_attr, kcl_value_ref_t* backtrack_level_map, kcl_value_ref_t* backtrack_cache, kcl_value_ref_t* args, kcl_value_ref_t* kwargs);

kcl_value_ref_t* kcl_schema_instances(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* kwargs);

void kcl_schema_optional_check(kcl_context_t* ctx, kcl_value_ref_t* p);

void kcl_schema_value_check(kcl_context_t* ctx, kcl_value_ref_t* schema_value, kcl_value_ref_t* schema_config, kcl_value_ref_t* _config_meta, kcl_char_t* schema_name, kcl_value_ref_t* index_sign_value, kcl_char_t* key_name, kcl_char_t* key_type, kcl_char_t* value_type, kcl_bool_t _any_other);

kcl_value_ref_t* kcl_schema_value_new(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* kwargs, kcl_value_ref_t* schema_value_or_func, kcl_value_ref_t* config, kcl_value_ref_t* config_meta, kcl_char_t* pkgpath);

void kcl_scope_add_setter(kcl_context_t* _ctx, kcl_eval_scope_t* scope, char* pkg, char* name, uint64_t* setter);

void kcl_scope_delete(kcl_eval_scope_t* scope);

kcl_value_ref_t* kcl_scope_get(kcl_context_t* ctx, kcl_eval_scope_t* scope, char* pkg, char* name, char* target, kcl_value_ref_t* default);

kcl_eval_scope_t* kcl_scope_new();

void kcl_scope_set(kcl_context_t* _ctx, kcl_eval_scope_t* scope, char* pkg, char* name, kcl_value_ref_t* value);

kcl_value_ref_t* kcl_template_execute(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* kwargs);

kcl_value_ref_t* kcl_template_html_escape(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* kwargs);

kcl_value_ref_t* kcl_units_to_G(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* kwargs);

kcl_value_ref_t* kcl_units_to_Gi(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* kwargs);

kcl_value_ref_t* kcl_units_to_K(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* kwargs);

kcl_value_ref_t* kcl_units_to_Ki(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* kwargs);

kcl_value_ref_t* kcl_units_to_M(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* kwargs);

kcl_value_ref_t* kcl_units_to_Mi(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* kwargs);

kcl_value_ref_t* kcl_units_to_P(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* kwargs);

kcl_value_ref_t* kcl_units_to_Pi(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* kwargs);

kcl_value_ref_t* kcl_units_to_T(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* kwargs);

kcl_value_ref_t* kcl_units_to_Ti(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* kwargs);

kcl_value_ref_t* kcl_units_to_m(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* kwargs);

kcl_value_ref_t* kcl_units_to_n(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* kwargs);

kcl_value_ref_t* kcl_units_to_u(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* kwargs);

kcl_value_ref_t* kcl_value_Bool(kcl_context_t* ctx, kcl_bool_t v);

kcl_decorator_value_t* kcl_value_Decorator(kcl_context_t* ctx, kcl_char_t* name, kcl_value_ref_t* args, kcl_value_ref_t* kwargs, kcl_value_ref_t* config_meta, kcl_char_t* attr_name, kcl_value_ref_t* config_value, kcl_value_ref_t* is_schema_target);

kcl_value_ref_t* kcl_value_Dict(kcl_context_t* ctx);

kcl_value_ref_t* kcl_value_False(kcl_context_t* ctx);

kcl_value_ref_t* kcl_value_Float(kcl_context_t* ctx, kcl_float_t v);

kcl_value_ref_t* kcl_value_Function(kcl_context_t* ctx, uint64_t* fn_ptr, kcl_value_ref_t* closure, kcl_char_t* name, kcl_bool_t is_external);

kcl_value_ref_t* kcl_value_Function_using_ptr(kcl_context_t* ctx, uint64_t* fn_ptr, kcl_char_t* name);

kcl_value_ref_t* kcl_value_Int(kcl_context_t* ctx, kcl_int_t v);

kcl_value_ref_t* kcl_value_List(kcl_context_t* ctx);

kcl_value_ref_t* kcl_value_List10(kcl_context_t* ctx, kcl_value_ref_t* v1, kcl_value_ref_t* v2, kcl_value_ref_t* v3, kcl_value_ref_t* v4, kcl_value_ref_t* v5, kcl_value_ref_t* v6, kcl_value_ref_t* v7, kcl_value_ref_t* v8, kcl_value_ref_t* v9, kcl_value_ref_t* v10);

kcl_value_ref_t* kcl_value_List6(kcl_context_t* ctx, kcl_value_ref_t* v1, kcl_value_ref_t* v2, kcl_value_ref_t* v3, kcl_value_ref_t* v4, kcl_value_ref_t* v5, kcl_value_ref_t* v6);

kcl_value_ref_t* kcl_value_None(kcl_context_t* ctx);

kcl_value_ref_t* kcl_value_Schema(kcl_context_t* ctx);

kcl_value_ref_t* kcl_value_Str(kcl_context_t* ctx, kcl_char_t* v);

kcl_char_t* kcl_value_Str_ptr(kcl_value_ref_t* p);

kcl_value_ref_t* kcl_value_True(kcl_context_t* ctx);

kcl_value_ref_t* kcl_value_Undefined(kcl_context_t* ctx);

kcl_value_ref_t* kcl_value_Unit(kcl_context_t* ctx, kcl_float_t v, kcl_int_t raw, kcl_char_t* unit);

kcl_value_ref_t* kcl_value_as(kcl_context_t* ctx, kcl_value_ref_t* a, kcl_value_ref_t* b);

uint64_t* kcl_value_check_function_ptr(kcl_value_ref_t* p);

kcl_value_ref_t* kcl_value_cmp_equal_to(kcl_context_t* ctx, kcl_value_ref_t* a, kcl_value_ref_t* b);

kcl_value_ref_t* kcl_value_cmp_greater_than(kcl_context_t* ctx, kcl_value_ref_t* a, kcl_value_ref_t* b);

kcl_value_ref_t* kcl_value_cmp_greater_than_or_equal(kcl_context_t* ctx, kcl_value_ref_t* a, kcl_value_ref_t* b);

kcl_value_ref_t* kcl_value_cmp_less_than(kcl_context_t* ctx, kcl_value_ref_t* a, kcl_value_ref_t* b);

kcl_value_ref_t* kcl_value_cmp_less_than_or_equal(kcl_context_t* ctx, kcl_value_ref_t* a, kcl_value_ref_t* b);

kcl_value_ref_t* kcl_value_cmp_not_equal_to(kcl_context_t* ctx, kcl_value_ref_t* a, kcl_value_ref_t* b);

kcl_value_ref_t* kcl_value_deep_copy(kcl_context_t* ctx, kcl_value_ref_t* p);

void kcl_value_delete(kcl_value_ref_t* p);

kcl_value_ref_t* kcl_value_from_json(kcl_context_t* ctx, kcl_char_t* s);

kcl_value_ref_t* kcl_value_function_invoke(kcl_value_ref_t* p, kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* kwargs, kcl_char_t* pkgpath, kcl_value_ref_t* is_in_schema);

uint64_t* kcl_value_function_ptr(kcl_value_ref_t* p);

kcl_value_ref_t* kcl_value_in(kcl_context_t* ctx, kcl_value_ref_t* a, kcl_value_ref_t* b);

kcl_value_ref_t* kcl_value_is(kcl_context_t* ctx, kcl_value_ref_t* a, kcl_value_ref_t* b);

kcl_value_ref_t* kcl_value_is_not(kcl_context_t* ctx, kcl_value_ref_t* a, kcl_value_ref_t* b);

kcl_bool_t kcl_value_is_truthy(kcl_value_ref_t* p);

kcl_iterator_t* kcl_value_iter(kcl_value_ref_t* p);

kcl_size_t kcl_value_len(kcl_value_ref_t* p);

kcl_value_ref_t* kcl_value_load_attr(kcl_context_t* ctx, kcl_value_ref_t* obj, kcl_char_t* key);

kcl_value_ref_t* kcl_value_load_attr_option(kcl_context_t* ctx, kcl_value_ref_t* p, kcl_char_t* key);

kcl_value_ref_t* kcl_value_logic_and(kcl_context_t* ctx, kcl_value_ref_t* a, kcl_value_ref_t* b);

kcl_value_ref_t* kcl_value_logic_or(kcl_context_t* ctx, kcl_value_ref_t* a, kcl_value_ref_t* b);

kcl_value_ref_t* kcl_value_not_in(kcl_context_t* ctx, kcl_value_ref_t* a, kcl_value_ref_t* b);

kcl_value_ref_t* kcl_value_op_add(kcl_context_t* ctx, kcl_value_ref_t* a, kcl_value_ref_t* b);

kcl_value_ref_t* kcl_value_op_aug_add(kcl_context_t* ctx, kcl_value_ref_t* a, kcl_value_ref_t* b);

kcl_value_ref_t* kcl_value_op_aug_bit_and(kcl_context_t* _ctx, kcl_value_ref_t* a, kcl_value_ref_t* b);

kcl_value_ref_t* kcl_value_op_aug_bit_lshift(kcl_context_t* ctx, kcl_value_ref_t* a, kcl_value_ref_t* b);

kcl_value_ref_t* kcl_value_op_aug_bit_or(kcl_context_t* ctx, kcl_value_ref_t* a, kcl_value_ref_t* b);

kcl_value_ref_t* kcl_value_op_aug_bit_rshift(kcl_context_t* ctx, kcl_value_ref_t* a, kcl_value_ref_t* b);

kcl_value_ref_t* kcl_value_op_aug_bit_xor(kcl_context_t* _ctx, kcl_value_ref_t* a, kcl_value_ref_t* b);

kcl_value_ref_t* kcl_value_op_aug_div(kcl_context_t* _ctx, kcl_value_ref_t* a, kcl_value_ref_t* b);

kcl_value_ref_t* kcl_value_op_aug_floor_div(kcl_context_t* _ctx, kcl_value_ref_t* a, kcl_value_ref_t* b);

kcl_value_ref_t* kcl_value_op_aug_mod(kcl_context_t* _ctx, kcl_value_ref_t* a, kcl_value_ref_t* b);

kcl_value_ref_t* kcl_value_op_aug_mul(kcl_context_t* ctx, kcl_value_ref_t* a, kcl_value_ref_t* b);

kcl_value_ref_t* kcl_value_op_aug_pow(kcl_context_t* ctx, kcl_value_ref_t* a, kcl_value_ref_t* b);

kcl_value_ref_t* kcl_value_op_aug_sub(kcl_context_t* ctx, kcl_value_ref_t* a, kcl_value_ref_t* b);

kcl_value_ref_t* kcl_value_op_bit_and(kcl_context_t* ctx, kcl_value_ref_t* a, kcl_value_ref_t* b);

kcl_value_ref_t* kcl_value_op_bit_lshift(kcl_context_t* ctx, kcl_value_ref_t* a, kcl_value_ref_t* b);

kcl_value_ref_t* kcl_value_op_bit_or(kcl_context_t* ctx, kcl_value_ref_t* a, kcl_value_ref_t* b);

kcl_value_ref_t* kcl_value_op_bit_rshift(kcl_context_t* ctx, kcl_value_ref_t* a, kcl_value_ref_t* b);

kcl_value_ref_t* kcl_value_op_bit_xor(kcl_context_t* ctx, kcl_value_ref_t* a, kcl_value_ref_t* b);

kcl_value_ref_t* kcl_value_op_div(kcl_context_t* ctx, kcl_value_ref_t* a, kcl_value_ref_t* b);

kcl_value_ref_t* kcl_value_op_floor_div(kcl_context_t* ctx, kcl_value_ref_t* a, kcl_value_ref_t* b);

kcl_value_ref_t* kcl_value_op_mod(kcl_context_t* ctx, kcl_value_ref_t* a, kcl_value_ref_t* b);

kcl_value_ref_t* kcl_value_op_mul(kcl_context_t* ctx, kcl_value_ref_t* a, kcl_value_ref_t* b);

kcl_value_ref_t* kcl_value_op_pow(kcl_context_t* ctx, kcl_value_ref_t* a, kcl_value_ref_t* b);

kcl_value_ref_t* kcl_value_op_sub(kcl_context_t* ctx, kcl_value_ref_t* a, kcl_value_ref_t* b);

kcl_value_ref_t* kcl_value_plan_to_json(kcl_context_t* ctx, kcl_value_ref_t* p);

kcl_value_ref_t* kcl_value_plan_to_yaml(kcl_context_t* ctx, kcl_value_ref_t* p);

void kcl_value_remove_item(kcl_value_ref_t* a, kcl_value_ref_t* b);

kcl_value_ref_t* kcl_value_schema_function(kcl_context_t* ctx, uint64_t* fn_ptr, uint64_t* check_fn_ptr, kcl_value_ref_t* attr_map, kcl_char_t* tpe);

kcl_value_ref_t* kcl_value_schema_with_config(kcl_context_t* ctx, kcl_value_ref_t* schema_dict, kcl_value_ref_t* config, kcl_value_ref_t* config_meta, kcl_char_t* name, kcl_char_t* pkgpath, kcl_value_ref_t* is_sub_schema, kcl_value_ref_t* record_instance, kcl_value_ref_t* instance_pkgpath, kcl_value_ref_t* optional_mapping, kcl_value_ref_t* args, kcl_value_ref_t* kwargs);

kcl_value_ref_t* kcl_value_slice(kcl_context_t* ctx, kcl_value_ref_t* x, kcl_value_ref_t* a, kcl_value_ref_t* b, kcl_value_ref_t* step);

kcl_value_ref_t* kcl_value_slice_option(kcl_context_t* ctx, kcl_value_ref_t* x, kcl_value_ref_t* a, kcl_value_ref_t* b, kcl_value_ref_t* step);

kcl_value_ref_t* kcl_value_subscr(kcl_context_t* ctx, kcl_value_ref_t* a, kcl_value_ref_t* b);

kcl_value_ref_t* kcl_value_subscr_option(kcl_context_t* ctx, kcl_value_ref_t* a, kcl_value_ref_t* b);

void kcl_value_subscr_set(kcl_context_t* ctx, kcl_value_ref_t* p, kcl_value_ref_t* index, kcl_value_ref_t* val);

kcl_value_ref_t* kcl_value_to_json_value(kcl_context_t* ctx, kcl_value_ref_t* p);

kcl_value_ref_t* kcl_value_to_json_value_with_null(kcl_context_t* ctx, kcl_value_ref_t* p);

kcl_value_ref_t* kcl_value_to_str_value(kcl_context_t* ctx, kcl_value_ref_t* p);

kcl_value_ref_t* kcl_value_to_yaml_value(kcl_context_t* ctx, kcl_value_ref_t* p);

kcl_value_ref_t* kcl_value_unary_l_not(kcl_context_t* ctx, kcl_value_ref_t* a);

kcl_value_ref_t* kcl_value_unary_minus(kcl_context_t* ctx, kcl_value_ref_t* a);

kcl_value_ref_t* kcl_value_unary_not(kcl_context_t* ctx, kcl_value_ref_t* a);

kcl_value_ref_t* kcl_value_unary_plus(kcl_context_t* ctx, kcl_value_ref_t* a);

kcl_value_ref_t* kcl_value_union(kcl_context_t* ctx, kcl_value_ref_t* schema, kcl_value_ref_t* b);

kcl_value_ref_t* kcl_value_union_all(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* _kwargs);

kcl_value_ref_t* kcl_yaml_decode(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* kwargs);

kcl_value_ref_t* kcl_yaml_decode_all(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* kwargs);

kcl_value_ref_t* kcl_yaml_dump_all_to_file(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* kwargs);

kcl_value_ref_t* kcl_yaml_dump_to_file(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* kwargs);

kcl_value_ref_t* kcl_yaml_encode(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* kwargs);

kcl_value_ref_t* kcl_yaml_encode_all(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* kwargs);

kcl_value_ref_t* kcl_yaml_validate(kcl_context_t* ctx, kcl_value_ref_t* args, kcl_value_ref_t* kwargs);

#ifdef __cplusplus
} // extern "C"
#endif

#endif // _kcl_h_
