#[derive(Fail, Debug)]
pub enum PdbError {
    #[fail(display = "can't fetch {:?}", _0)]
    DownloadFailed(String),
    #[fail(display = "can't fetch {:?}", _0)]
    StatusError(String),
    #[fail(display = "parsing file: {}", _0)]
    ParseError(String),
}

