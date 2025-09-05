#[allow(unused_variables)]
pub fn info(msg: String) {
    #[cfg(feature = "tracing")]
    tracing::info!(msg)
}

#[allow(unused_variables)]
pub fn error(msg: String) {
    #[cfg(feature = "tracing")]
    tracing::error!(msg)
}
