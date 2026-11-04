//! Copyright The KCL Authors. All rights reserved.

extern crate fancy_regex;

use std::collections::HashMap;
use std::rc::Rc;

use crate::*;

/// The maximum number of compiled patterns a [`RegexCache`] keeps.
const REGEX_CACHE_CAPACITY: usize = 128;

/// Compiled patterns of the `regex` system module, kept in the runtime
/// [`Context`] so that calling `regex.match`, `regex.replace` and the other
/// functions in a loop compiles each pattern once rather than on every call.
///
/// The cache holds at most 128 patterns and evicts the least recently used
/// one when it is full, so a program that builds many distinct patterns does
/// not keep all of them alive. A pattern that fails to compile is never
/// cached: every call with it compiles it again and returns the same error.
#[derive(Clone, Default)]
pub struct RegexCache {
    entries: HashMap<String, RegexCacheEntry>,
    clock: u64,
}

#[derive(Clone)]
struct RegexCacheEntry {
    regex: Rc<fancy_regex::Regex>,
    last_used: u64,
}

impl RegexCache {
    /// Returns the compiled `pattern`, compiling and caching it if it is not
    /// cached yet. The error is the one `fancy_regex::Regex::new` returns.
    pub fn get_or_compile(
        &mut self,
        pattern: &str,
    ) -> Result<Rc<fancy_regex::Regex>, fancy_regex::Error> {
        self.clock += 1;
        if let Some(entry) = self.entries.get_mut(pattern) {
            entry.last_used = self.clock;
            return Ok(Rc::clone(&entry.regex));
        }
        let regex = Rc::new(fancy_regex::Regex::new(pattern)?);
        if self.entries.len() >= REGEX_CACHE_CAPACITY {
            self.evict_least_recently_used();
        }
        self.entries.insert(
            pattern.to_string(),
            RegexCacheEntry {
                regex: Rc::clone(&regex),
                last_used: self.clock,
            },
        );
        Ok(regex)
    }

    fn evict_least_recently_used(&mut self) {
        let oldest = self
            .entries
            .iter()
            .min_by_key(|(_, entry)| entry.last_used)
            .map(|(pattern, _)| pattern.clone());
        if let Some(pattern) = oldest {
            self.entries.remove(&pattern);
        }
    }
}

// match(string: str, pattern: str) -> bool:

/// # Safety
/// The caller must ensure that `ctx`, `args`, and `kwargs` are valid pointers
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_regex_match(
    ctx: *mut kcl_context_t,
    args: *const kcl_value_ref_t,
    kwargs: *const kcl_value_ref_t,
) -> *const kcl_value_ref_t {
    let args = unsafe { ptr_as_ref(args) };
    let kwargs = unsafe { ptr_as_ref(kwargs) };
    let ctx = unsafe { mut_ptr_as_ref(ctx) };

    if let Some(string) = get_call_arg_str(args, kwargs, 0, Some("string"))
        && let Some(pattern) = get_call_arg_str(args, kwargs, 1, Some("pattern"))
    {
        let re = ctx.regex_cache.get_or_compile(pattern.as_ref()).unwrap();
        match re.is_match(string.as_ref()) {
            Ok(ok) => {
                if ok {
                    return unsafe { kcl_value_Bool(ctx, 1) };
                } else {
                    return unsafe { kcl_value_Bool(ctx, 0) };
                }
            }
            _ => return unsafe { kcl_value_Bool(ctx, 0) },
        }
    }

    panic!("match() missing 2 required positional arguments: 'string' and 'pattern'")
}

// replace(string: str, pattern: str, replace: str, count: int = 0):

/// # Safety
/// The caller must ensure that `ctx`, `args`, and `kwargs` are valid pointers
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_regex_replace(
    ctx: *mut kcl_context_t,
    args: *const kcl_value_ref_t,
    kwargs: *const kcl_value_ref_t,
) -> *mut kcl_value_ref_t {
    let args = unsafe { ptr_as_ref(args) };
    let kwargs = unsafe { ptr_as_ref(kwargs) };
    let ctx = unsafe { mut_ptr_as_ref(ctx) };
    if let Some(string) = get_call_arg_str(args, kwargs, 0, Some("string")) {
        if let Some(pattern) = get_call_arg_str(args, kwargs, 1, Some("pattern")) {
            if let Some(replace) = get_call_arg_str(args, kwargs, 2, Some("replace")) {
                let count = get_call_arg_int(args, kwargs, 3, Some("count")).unwrap_or(0);
                let re = ctx.regex_cache.get_or_compile(pattern.as_ref()).unwrap();
                let s = re.replacen(string.as_ref(), count as usize, replace.as_ref() as &str);
                return ValueRef::str(&s).into_raw(ctx);
            }
            panic!("replace() missing the required positional argument: 'replace'");
        }
        panic!("replace() missing the required positional argument: 'pattern'");
    }
    panic!("replace() missing 3 required positional arguments: 'string', 'pattern', and 'replace");
}

// compile(pattern: str) -> bool:

/// # Safety
/// The caller must ensure that `ctx`, `args`, and `kwargs` are valid pointers
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_regex_compile(
    ctx: *mut kcl_context_t,
    args: *const kcl_value_ref_t,
    kwargs: *const kcl_value_ref_t,
) -> *mut kcl_value_ref_t {
    let args = unsafe { ptr_as_ref(args) };
    let kwargs = unsafe { ptr_as_ref(kwargs) };
    let ctx = unsafe { mut_ptr_as_ref(ctx) };

    if let Some(pattern) = get_call_arg_str(args, kwargs, 0, Some("pattern")) {
        match ctx.regex_cache.get_or_compile(pattern.as_ref()) {
            Ok(_) => return unsafe { kcl_value_Bool(ctx, 1) },
            _ => return unsafe { kcl_value_Bool(ctx, 0) },
        }
    }
    panic!("compile() missing the required positional argument: 'pattern'")
}

// findall(string: str, pattern: str) -> [str]:

/// # Safety
/// The caller must ensure that `ctx`, `args`, and `kwargs` are valid pointers
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_regex_findall(
    ctx: *mut kcl_context_t,
    args: *const kcl_value_ref_t,
    kwargs: *const kcl_value_ref_t,
) -> *mut kcl_value_ref_t {
    let args = unsafe { ptr_as_ref(args) };
    let kwargs = unsafe { ptr_as_ref(kwargs) };
    let ctx = unsafe { mut_ptr_as_ref(ctx) };
    if let Some(string) = get_call_arg_str(args, kwargs, 0, Some("string")) {
        if let Some(pattern) = get_call_arg_str(args, kwargs, 1, Some("pattern")) {
            let mut list = ValueRef::list(None);

            let re = ctx.regex_cache.get_or_compile(pattern.as_ref()).unwrap();
            for x in re.captures_iter(string.as_ref()).flatten() {
                // Like Python's `re.findall`, return the whole match when the pattern
                // has no group, the group when it has exactly one, and a list of the
                // groups otherwise. A group that did not participate in the match
                // yields an empty string.
                let group = |i| ValueRef::str(x.get(i).map_or("", |m| m.as_str()));
                match x.len() {
                    1 => list.list_append(&group(0)),
                    2 => list.list_append(&group(1)),
                    len => {
                        let mut sub_list = ValueRef::list(None);
                        for i in 1..len {
                            sub_list.list_append(&group(i));
                        }
                        list.list_append(&sub_list)
                    }
                }
            }

            return list.into_raw(ctx);
        }
        panic!("findall() missing the required positional argument: 'pattern'")
    }
    panic!("findall() missing 2 required positional arguments: 'string' and 'pattern'")
}

// search(string: str, pattern: str):

/// # Safety
/// The caller must ensure that `ctx`, `args`, and `kwargs` are valid pointers
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_regex_search(
    ctx: *mut kcl_context_t,
    args: *const kcl_value_ref_t,
    kwargs: *const kcl_value_ref_t,
) -> *mut kcl_value_ref_t {
    let args = unsafe { ptr_as_ref(args) };
    let kwargs = unsafe { ptr_as_ref(kwargs) };
    let ctx = unsafe { mut_ptr_as_ref(ctx) };

    if let Some(string) = get_call_arg_str(args, kwargs, 0, Some("string")) {
        if let Some(pattern) = get_call_arg_str(args, kwargs, 1, Some("pattern")) {
            let re = ctx.regex_cache.get_or_compile(pattern.as_ref()).unwrap();

            if let Ok(Some(..)) = re.find(string.as_ref()) {
                return unsafe { kcl_value_Bool(ctx, 1) };
            }
            return unsafe { kcl_value_Bool(ctx, 0) };
        }
        panic!("search() missing the required positional argument: 'pattern'");
    }
    panic!("search() missing 2 required positional arguments: 'string' and 'pattern'");
}

// split(string: str, pattern: str, maxsplit: int = 0):

/// # Safety
/// The caller must ensure that `ctx`, `args`, and `kwargs` are valid pointers
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn kcl_regex_split(
    ctx: *mut kcl_context_t,
    args: *const kcl_value_ref_t,
    kwargs: *const kcl_value_ref_t,
) -> *mut kcl_value_ref_t {
    let args = unsafe { ptr_as_ref(args) };
    let kwargs = unsafe { ptr_as_ref(kwargs) };
    let ctx = unsafe { mut_ptr_as_ref(ctx) };
    if let Some(string) = get_call_arg_str(args, kwargs, 0, Some("string")) {
        if let Some(pattern) = get_call_arg_str(args, kwargs, 1, Some("pattern")) {
            let maxsplit = get_call_arg_int(args, kwargs, 2, Some("maxsplit")).unwrap_or(0);
            let mut list = ValueRef::list(None);

            let re = ctx.regex_cache.get_or_compile(pattern.as_ref()).unwrap();

            let mut fields: Vec<String> = Vec::new();
            let mut current_pos = 0;
            loop {
                let capture = re
                    .captures_from_pos(string.as_ref(), current_pos)
                    .map_or(None, |c| c);
                if let Some(Some(cap)) = capture.map(|c| c.get(0)) {
                    fields.push(string[current_pos..cap.start()].to_string());
                    current_pos = cap.end();
                    if maxsplit > 0 && fields.len() >= (maxsplit as usize) {
                        fields.push(string[current_pos..].to_string());
                        break;
                    }
                } else {
                    fields.push(string[current_pos..].to_string());
                    break;
                }
            }

            for s in fields {
                list.list_append(&ValueRef::str(s.as_ref()));
            }
            return list.into_raw(ctx);
        }
        panic!("split() missing the required positional argument: 'pattern'");
    }
    panic!("split() missing 2 required positional arguments: 'string' and 'pattern'");
}

#[cfg(test)]
mod test_regex {
    use super::*;
    use std::panic::AssertUnwindSafe;

    fn args(values: &[&str]) -> ValueRef {
        let values: Vec<ValueRef> = values.iter().map(|v| ValueRef::str(v)).collect();
        ValueRef::list(Some(&values.iter().collect::<Vec<_>>()))
    }

    fn strs(values: &[&str]) -> ValueRef {
        ValueRef::list_str(&values.iter().map(|v| v.to_string()).collect::<Vec<_>>())
    }

    #[test]
    fn test_regex_cache_reuses_compiled_pattern() {
        let mut cache = RegexCache::default();
        let first = cache.get_or_compile(r"\d+").unwrap();
        let second = cache.get_or_compile(r"\d+").unwrap();
        assert!(Rc::ptr_eq(&first, &second));
        assert_eq!(cache.entries.len(), 1);
    }

    #[test]
    fn test_regex_cache_does_not_cache_invalid_patterns() {
        let mut cache = RegexCache::default();
        for pattern in ["(", "[a-", "a{2,1}", r"(?<=a+)b"] {
            let expected = format!("{:?}", fancy_regex::Regex::new(pattern).unwrap_err());
            for _ in 0..2 {
                let err = cache.get_or_compile(pattern).unwrap_err();
                assert_eq!(format!("{err:?}"), expected, "pattern {pattern}");
            }
        }
        assert!(cache.entries.is_empty());
        assert!(cache.get_or_compile("a+").is_ok());
        assert_eq!(cache.entries.len(), 1);
    }

    #[test]
    fn test_regex_cache_is_bounded() {
        let mut cache = RegexCache::default();
        for i in 0..REGEX_CACHE_CAPACITY + 16 {
            cache.get_or_compile(&format!("p{i}")).unwrap();
            assert!(cache.entries.len() <= REGEX_CACHE_CAPACITY);
        }
        assert_eq!(cache.entries.len(), REGEX_CACHE_CAPACITY);
        for i in 0..16 {
            assert!(!cache.entries.contains_key(&format!("p{i}")));
        }
        for i in 16..REGEX_CACHE_CAPACITY + 16 {
            assert!(cache.entries.contains_key(&format!("p{i}")));
        }
    }

    #[test]
    fn test_regex_cache_evicts_least_recently_used() {
        let mut cache = RegexCache::default();
        let first = cache.get_or_compile("p0").unwrap();
        for i in 1..REGEX_CACHE_CAPACITY {
            cache.get_or_compile(&format!("p{i}")).unwrap();
        }
        // Using p0 again leaves p1 as the least recently used pattern.
        cache.get_or_compile("p0").unwrap();
        cache.get_or_compile("q").unwrap();
        assert_eq!(cache.entries.len(), REGEX_CACHE_CAPACITY);
        assert!(!cache.entries.contains_key("p1"));
        assert!(Rc::ptr_eq(&first, &cache.get_or_compile("p0").unwrap()));
    }

    #[test]
    fn test_regex_cache_handle_survives_eviction() {
        let mut cache = RegexCache::default();
        let first = cache.get_or_compile(r"\d+").unwrap();
        for i in 0..REGEX_CACHE_CAPACITY {
            cache.get_or_compile(&format!("p{i}")).unwrap();
        }
        assert!(!cache.entries.contains_key(r"\d+"));
        assert!(first.is_match("123").unwrap());
        assert!(!Rc::ptr_eq(&first, &cache.get_or_compile(r"\d+").unwrap()));
    }

    #[test]
    fn test_regex_cache_context_isolation_and_clone() {
        let mut ctx = Context::default();
        let first = ctx.regex_cache.get_or_compile("a").unwrap();
        let mut cloned = ctx.clone();
        assert!(Rc::ptr_eq(
            &first,
            &cloned.regex_cache.get_or_compile("a").unwrap()
        ));
        cloned.regex_cache.get_or_compile("b").unwrap();
        assert_eq!(ctx.regex_cache.entries.len(), 1);
        assert_eq!(cloned.regex_cache.entries.len(), 2);

        let mut other = Context::new();
        assert!(other.regex_cache.entries.is_empty());
        assert!(!Rc::ptr_eq(
            &first,
            &other.regex_cache.get_or_compile("a").unwrap()
        ));
    }

    #[test]
    fn test_regex_cache_is_dropped_with_c_api_context() {
        unsafe {
            let ctx = kcl_context_new();
            let compiled = kcl_regex_compile(ctx, &args(&["a"]), &ValueRef::dict(None));
            assert_eq!(*compiled, ValueRef::bool(true));
            let pattern = Rc::downgrade(&(&*ctx).regex_cache.entries["a"].regex);
            kcl_context_delete(ctx);
            assert!(pattern.upgrade().is_none());
        }
    }

    #[test]
    fn test_regex_functions_reuse_cached_pattern() {
        let mut ctx = Context::new();
        let cached = ctx.regex_cache.get_or_compile(",").unwrap();
        let kwargs = ValueRef::dict(None);
        for _ in 0..3 {
            unsafe {
                let replaced = kcl_regex_replace(&mut ctx, &args(&["a,b,c", ",", "|"]), &kwargs);
                assert_eq!(*replaced, ValueRef::str("a|b|c"));
                let split = kcl_regex_split(&mut ctx, &args(&["a,b,c", ","]), &kwargs);
                assert_eq!(*split, strs(&["a", "b", "c"]));
                let found = kcl_regex_findall(&mut ctx, &args(&["a,b,c", ","]), &kwargs);
                assert_eq!(*found, strs(&[",", ","]));
                let matched = kcl_regex_match(&mut ctx, &args(&["a,b", ","]), &kwargs);
                assert_eq!(*matched, ValueRef::bool(true));
                let searched = kcl_regex_search(&mut ctx, &args(&["ab", ","]), &kwargs);
                assert_eq!(*searched, ValueRef::bool(false));
                let compiled = kcl_regex_compile(&mut ctx, &args(&[","]), &kwargs);
                assert_eq!(*compiled, ValueRef::bool(true));
            }
            assert!(Rc::ptr_eq(&cached, &ctx.regex_cache.entries[","].regex));
        }
        assert_eq!(ctx.regex_cache.entries.len(), 1);
    }

    #[test]
    fn test_regex_functions_report_invalid_pattern_on_every_call() {
        let pattern = "(";
        let expected = format!(
            "called `Result::unwrap()` on an `Err` value: {:?}",
            fancy_regex::Regex::new(pattern).unwrap_err()
        );
        let mut ctx = Context::new();
        let ctx: *mut Context = &mut ctx;
        for _ in 0..2 {
            unsafe {
                let compiled = kcl_regex_compile(ctx, &args(&[pattern]), &ValueRef::dict(None));
                assert_eq!(*compiled, ValueRef::bool(false));
            }
            assert_panic(
                &expected,
                AssertUnwindSafe(|| unsafe {
                    kcl_regex_match(ctx, &args(&["a", pattern]), &ValueRef::dict(None));
                }),
            );
            assert_panic(
                &expected,
                AssertUnwindSafe(|| unsafe {
                    kcl_regex_replace(ctx, &args(&["a", pattern, "-"]), &ValueRef::dict(None));
                }),
            );
            assert_panic(
                &expected,
                AssertUnwindSafe(|| unsafe {
                    kcl_regex_findall(ctx, &args(&["a", pattern]), &ValueRef::dict(None));
                }),
            );
            assert_panic(
                &expected,
                AssertUnwindSafe(|| unsafe {
                    kcl_regex_search(ctx, &args(&["a", pattern]), &ValueRef::dict(None));
                }),
            );
            assert_panic(
                &expected,
                AssertUnwindSafe(|| unsafe {
                    kcl_regex_split(ctx, &args(&["a", pattern]), &ValueRef::dict(None));
                }),
            );
        }
        assert!(unsafe { &*ctx }.regex_cache.entries.is_empty());
    }
}
