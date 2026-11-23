#[cfg(feature = "tracing")]
macro_rules! sdebug {
    ($($arg:tt)*) => {
        ::tracing::debug!(target: "suci", $($arg)*)
    };
}

#[cfg(feature = "tracing")]
macro_rules! strace {
    ($($arg:tt)*) => {
        ::tracing::trace!(target: "suci", $($arg)*)
    };
}

#[cfg(not(feature = "tracing"))]
macro_rules! sdebug {
    ($($arg:tt)*) => {};
}

#[cfg(not(feature = "tracing"))]
macro_rules! strace {
    ($($arg:tt)*) => {};
}
