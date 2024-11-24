use std::{error::Error, io, path::Path};

pub trait FromIoError: Error {
    fn io_err<P: AsRef<Path>>(path: P, err: io::Error) -> Self;
}

pub fn io_err_mapper<P: AsRef<Path>, E: FromIoError>(path: P) -> impl Fn(io::Error) -> E {
    move |err| E::io_err(path.as_ref(), err)
}
