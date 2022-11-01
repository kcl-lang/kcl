// Copyright 2021 The KCL Authors. All rights reserved.

#[allow(non_camel_case_types)]
type kclvm_value_ref_t = crate::ValueRef;
use crate::{new_mut_ptr, IndexMap};
use indexmap::IndexSet;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::{
    cell::RefCell,
    cmp::Ordering,
    hash::{Hash, Hasher},
    rc::Rc,
};

#[allow(non_upper_case_globals)]
pub const UNDEFINED: Value = Value::undefined;

#[allow(non_upper_case_globals)]
pub const NONE: Value = Value::none;

#[allow(non_upper_case_globals)]
pub const TRUE: Value = Value::bool_value(true);

#[allow(non_upper_case_globals)]
pub const FALSE: Value = Value::bool_value(false);

#[derive(PartialEq, Eq, Clone, Default, Debug)]
pub struct KclError {
    pub err_code: i32,
    pub err_text: String,
    pub filename: String,
    pub source_code: String,
    pub line: i32,
    pub column: i32,
}

#[allow(non_camel_case_types)]
#[derive(Clone, PartialEq, Debug)]
pub enum Type {
    any_type,
    bool_type,
    bool_lit_type(bool),
    int_type,
    int_lit_type(i64),
    float_type,
    float_lit_type(f64),
    str_type,
    str_lit_type(String),
    list_type(ListType),
    dict_type(DictType),
    union_type(UnionType),
    schema_type(SchemaType),
    func_type(FuncType),
}

impl Default for Type {
    fn default() -> Self {
        Type::any_type
    }
}

impl Type {
    #[allow(dead_code)]
    pub fn into_raw(self) -> *mut Self {
        Box::into_raw(Box::new(self))
    }
}

#[derive(PartialEq, Clone, Default, Debug)]
pub struct ListType {
    pub elem_type: Box<Type>,
}

#[derive(PartialEq, Clone, Default, Debug)]
pub struct DictType {
    pub key_type: Box<Type>,
    pub elem_type: Box<Type>,
}

#[derive(PartialEq, Clone, Default, Debug)]
pub struct UnionType {
    pub elem_types: Vec<Type>,
}

#[derive(PartialEq, Clone, Default, Debug)]
pub struct SchemaType {
    pub name: String,
    pub parent_name: String,
    pub field_names: Vec<String>,
    pub field_types: Vec<Type>,
}

#[derive(PartialEq, Clone, Default, Debug)]
pub struct FuncType {
    pub args_types: Vec<Type>,
    pub return_type: Box<Type>,
}

#[repr(C)]
#[allow(non_camel_case_types)]
#[derive(Clone, Debug)]
pub struct ValueRef {
    pub rc: Rc<RefCell<Value>>,
}

impl Eq for ValueRef {}

impl PartialEq for ValueRef {
    fn eq(&self, other: &Self) -> bool {
        self.cmp_equal(other)
    }
}

impl Ord for ValueRef {
    fn cmp(&self, other: &ValueRef) -> Ordering {
        let ord = match *self.rc.borrow() {
            Value::int_value(a) => match *other.rc.borrow() {
                Value::int_value(b) => a.partial_cmp(&b),
                Value::float_value(b) => (a as f64).partial_cmp(&b),
                _ => None,
            },
            Value::float_value(a) => match *other.rc.borrow() {
                Value::int_value(b) => a.partial_cmp(&(b as f64)),
                Value::float_value(b) => a.partial_cmp(&b),
                _ => None,
            },
            Value::str_value(ref a) => match &*other.rc.borrow() {
                Value::str_value(ref b) => a.partial_cmp(b),
                _ => None,
            },
            _ => None,
        };
        match ord {
            Some(ord) => ord,
            _ => panic!(
                "cannot compare {} and {}",
                self.type_str(),
                other.type_str()
            ),
        }
    }
}

impl PartialOrd for ValueRef {
    fn partial_cmp(&self, other: &ValueRef) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Hash for ValueRef {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match &*self.rc.borrow() {
            Value::undefined => panic!("unsupport hash for undefined"),
            Value::none => panic!("unsupport hash for none"),
            Value::int_value(v) => (*v as f64).to_bits().hash(state),
            Value::unit_value(_real, raw, unit) => {
                raw.hash(state);
                unit.hash(state);
            }
            Value::float_value(v) => v.to_bits().hash(state),
            Value::bool_value(v) => v.hash(state),
            Value::str_value(ref v) => (*v).hash(state),
            Value::list_value(ref v) => {
                for i in 0..v.values.len() {
                    v.values[i].hash(state);
                }
            }
            Value::dict_value(ref v) => {
                for (k, v) in v.values.iter() {
                    (*k).hash(state);
                    v.hash(state);
                }
            }
            Value::schema_value(ref v) => {
                for (k, v) in v.config.values.iter() {
                    (*k).hash(state);
                    v.hash(state);
                }
            }
            Value::func_value(ref v) => {
                v.fn_ptr.hash(state);
            }
        }
    }
}

impl Default for ValueRef {
    fn default() -> Self {
        Self {
            rc: Rc::new(RefCell::new(Value::undefined)),
        }
    }
}

impl ValueRef {
    pub fn into_raw(self) -> *mut Self {
        new_mut_ptr(self)
    }

    pub fn from_raw(&self) {
        //if value is a func,clear captured ValueRef to break circular reference
        if let Value::func_value(val) = &mut *self.rc.borrow_mut() {
            val.closure = ValueRef::none();
        }
    }
}

#[allow(non_camel_case_types)]
#[derive(Clone, PartialEq, Debug)]
pub enum Value {
    undefined,
    none,
    bool_value(bool),
    int_value(i64),
    float_value(f64),
    str_value(String),
    list_value(Box<ListValue>),
    dict_value(Box<DictValue>),
    schema_value(Box<SchemaValue>),
    func_value(Box<FuncValue>),
    unit_value(f64, i64, String), // (Real value, raw value, unit string)
}

impl Default for Value {
    fn default() -> Self {
        Self::undefined
    }
}

#[derive(PartialEq, Eq, Clone, Default, Debug)]
pub struct ListValue {
    pub values: Vec<ValueRef>,
}

#[derive(PartialEq, Eq, Clone, Default, Debug)]
pub struct DictValue {
    pub values: IndexMap<String, ValueRef>,
    pub ops: IndexMap<String, ConfigEntryOperationKind>,
    pub insert_indexs: IndexMap<String, i32>,
    pub attr_map: IndexMap<String, String>,
}

#[derive(PartialEq, Clone, Default, Debug)]
pub struct SchemaValue {
    pub name: String,
    pub pkgpath: String,
    pub config: Box<DictValue>,
    pub config_keys: Vec<String>,
}

#[derive(PartialEq, Eq, Clone, Default, Debug)]
pub struct DecoratorValue {
    pub name: String,
    pub args: ValueRef,
    pub kwargs: ValueRef,
}

#[derive(PartialEq, Eq, Clone, Default, Debug)]
pub struct FuncValue {
    // TODO (refactor): SchemaFuncValue
    pub fn_ptr: u64,
    pub check_fn_ptr: u64,
    pub closure: ValueRef,
    pub external_name: String,
    pub runtime_type: String,
}

#[derive(PartialEq, Clone, Default, Debug)]
pub struct ErrorValue {
    pub errors: Vec<KclError>,
}

#[derive(PartialEq, Eq, Clone, Default, Debug)]
pub struct OptionHelp {
    pub name: String,
    pub typ: String,
    pub required: bool,
    pub default_value: Option<String>,
    pub help: String,
}

#[allow(non_snake_case)]
#[derive(PartialEq, Eq, Clone, Default, Debug, Serialize, Deserialize)]
pub struct PanicInfo {
    pub __kcl_PanicInfo__: bool, // "__kcl_PanicInfo__"

    pub rust_file: String,
    pub rust_line: i32,
    pub rust_col: i32,

    pub kcl_pkgpath: String,
    pub kcl_file: String,
    pub kcl_line: i32,
    pub kcl_col: i32,
    pub kcl_arg_msg: String,

    // only for schema check
    pub kcl_config_meta_file: String,
    pub kcl_config_meta_line: i32,
    pub kcl_config_meta_col: i32,
    pub kcl_config_meta_arg_msg: String,

    pub message: String,
    pub err_type_code: i32,
    pub is_warning: bool,
}

#[derive(PartialEq, Eq, Clone, Default, Debug)]
pub struct ContextConfig {
    pub debug_mode: bool,

    pub strict_range_check: bool,
    pub disable_none: bool,
    pub disable_schema_check: bool,

    pub list_option_mode: bool,
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct ContextBuffer {
    pub kclvm_context_invoke_result: String,
}

impl Default for ContextBuffer {
    fn default() -> Self {
        Self {
            kclvm_context_invoke_result: "\0".to_string(),
        }
    }
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct ContextOutput {
    pub stdout: String,
    pub stderr: String,

    pub return_value: *mut kclvm_value_ref_t, // *mut kclvm_value_ref_t
}

impl Default for ContextOutput {
    fn default() -> Self {
        Self {
            stdout: "".to_string(),
            stderr: "".to_string(),
            return_value: std::ptr::null_mut(),
        }
    }
}

#[derive(PartialEq, Clone, Default, Debug)]
pub struct Context {
    pub cfg: ContextConfig,
    pub output: ContextOutput,
    pub panic_info: PanicInfo,

    pub main_pkg_path: String,
    pub main_pkg_files: Vec<String>,

    pub imported_pkgpath: HashSet<String>,
    pub app_args: HashMap<String, u64>,
    pub instances: RefCell<HashMap<String, Vec<ValueRef>>>,
    pub all_types: Vec<Type>,
    pub all_schemas: RefCell<HashMap<String, ValueRef>>,
    pub import_names: IndexMap<String, IndexMap<String, String>>,
    pub symbol_names: Vec<String>,
    pub symbol_values: Vec<Value>,
    pub func_handlers: Vec<FuncHandler>,

    pub option_helps: Vec<OptionHelp>,
    pub buffer: ContextBuffer,
    /// objects is to store all KCL object pointers.
    pub objects: IndexSet<usize>,
}

impl Context {
    pub fn new() -> Self {
        Context {
            instances: RefCell::new(HashMap::new()),
            ..Default::default()
        }
    }
}

#[derive(PartialEq, Eq, Clone, Default, Debug)]
pub struct FuncHandler {
    pub namespace: String,
    pub fn_pointer: u64,
}

#[repr(C)]
#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub enum Kind {
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
    Unit = 17,
    Func = 18,
}

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub enum ConfigEntryOperationKind {
    Union = 0,
    Override = 1,
    Insert = 2,
}

impl Default for ConfigEntryOperationKind {
    fn default() -> Self {
        ConfigEntryOperationKind::Union
    }
}

impl ConfigEntryOperationKind {
    pub fn from_i32(v: i32) -> Self {
        match v {
            x if x == ConfigEntryOperationKind::Union as i32 => ConfigEntryOperationKind::Union,
            x if x == ConfigEntryOperationKind::Override as i32 => {
                ConfigEntryOperationKind::Override
            }
            x if x == ConfigEntryOperationKind::Insert as i32 => ConfigEntryOperationKind::Insert,
            _ => panic!("Invalid AttrOpKind integer {}, expected 0, 1 or 2", v),
        }
    }
}
