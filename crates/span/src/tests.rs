use super::session_globals::*;
use super::*;

#[test]
fn interner_tests() {
    let i = Interner::default();
    // first one is zero:
    assert_eq!(i.intern("dog"), Symbol::new(0));
    // re-use gets the same entry:
    assert_eq!(i.intern("dog"), Symbol::new(0));
    // different string gets a different #:
    assert_eq!(i.intern("cat"), Symbol::new(1));
    assert_eq!(i.intern("cat"), Symbol::new(1));
    // dog is still at zero
    assert_eq!(i.intern("dog"), Symbol::new(0));
}

#[test]
fn interner_symbols() {
    create_session_globals_then(|| {
        let symbol1 = Symbol::intern("test_str_1");
        let symbol2 = Symbol::intern("test_str_2");
        assert_eq!(symbol1.as_str(), "test_str_1");
        assert_eq!(symbol2.as_str(), "test_str_2");
        assert_eq!(symbol2.as_u32(), symbol1.as_u32() + 1);
    });
}
