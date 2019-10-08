use backtrace::Backtrace;
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct KhError {
    msg: String,
    backtrace: Backtrace,
    cause: Option<Box<dyn Error>>,
}

impl KhError {
    pub fn new(msg: &str, cause: Option<Box<dyn Error>>) -> KhError {
        KhError {
            msg: msg.to_string(),
            #[cfg(debug_assertions)]
            backtrace: backtrace_strip_foreign(Backtrace::new()),
            #[cfg(not(debug_assertions))]
            backtrace: Backtrace::new_unresolved(),
            cause,
        }
    }
}

#[cfg(debug_assertions)]
fn backtrace_strip_foreign(backtrace: Backtrace) -> Backtrace {
    use backtrace::BacktraceFrame;
    let backtrace: Vec<BacktraceFrame> = backtrace.into();
    backtrace
        .into_iter()
        .filter(|frame| {
            frame
                .symbols()
                .iter()
                .map(|symbol| {
                    symbol
                        .name()
                        .and_then(|name| name.as_str())
                        .map_or(false, |name| name.contains("khaleesi"))
                })
                .any(|x| x)
        })
        .collect::<Vec<BacktraceFrame>>()
        .into()
}

impl fmt::Display for KhError {
    #[cfg(debug_assertions)]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}\n{:?}", self.msg, self.backtrace)
    }
    #[cfg(not(debug_assertions))]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl Error for KhError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        // lol idunno
        self.cause.as_ref().map(|cause| &**cause)
    }
}

impl From<&str> for KhError {
    fn from(e: &str) -> KhError {
        KhError::new(e, None)
    }
}

impl From<String> for KhError {
    fn from(e: String) -> KhError {
        KhError::new(&e, None)
    }
}

impl From<std::path::StripPrefixError> for KhError {
    fn from(e: std::path::StripPrefixError) -> KhError {
        let description = e.to_string();
        KhError::new(&description, Some(Box::new(e)))
    }
}

impl From<::std::io::Error> for KhError {
    fn from(e: ::std::io::Error) -> KhError {
        let description = e.to_string();
        KhError::new(&description, Some(Box::new(e)))
    }
}
