pub mod init;

#[macro_export]
macro_rules! trace_err {
    (Err($err:expr)) => {{
        let err = $err;
        ::tracing::error!(error = %err, "Returning error");
        Err(err)
    }};
}
