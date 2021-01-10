pub fn is_test() -> bool {
    #[cfg(test)]
    return true;
    #[cfg(not(test))]
    return false;
}

pub fn should_conio() -> bool {
    #[cfg(feature = "conio")]
    return true;
    #[cfg(not(feature = "conio"))]
    return false;
}

pub fn should_displayio() -> bool {
    #[cfg(feature = "displayio")]
    return true;
    #[cfg(not(feature = "displayio"))]
    return false;
}

pub fn should_fini_exit() -> bool {
    #[cfg(feature = "fini_exit")]
    return true;
    #[cfg(not(feature = "fini_exit"))]
    return false;
}

pub fn should_fini_wait() -> bool {
    #[cfg(feature = "fini_wait")]
    return true;
    #[cfg(not(feature = "fini_wait"))]
    return false;
}
pub fn should_debug_log() -> bool {
    #[cfg(feature = "debug_logs")]
    return true;
    #[cfg(not(feature = "debug_logs"))]
    return false;
}

pub fn check_const_correct() {
    assert_eq!(
        should_fini_exit() || should_fini_wait(),
        true,
        "fini_exit or fini_wait must be set"
    );
}
