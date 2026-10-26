pub type Error = anyhow::Error;
pub type Result<T> = anyhow::Result<T>;

// derive more crate - allows you to derive your own errors from other errors
// - derive "from"
// - derive "display" values
//
// #[derive(Debug, From, Display)]
// pub enum ExampleErrorEnum {
//     #[display("Limit ")]
//     ErrorA { value: usize, limit: usize },
//
//     #[from]
//     ErrorB(serde_yml::Error),
// }

// use derive_more::{Display, From};
//
// pub type Result<T> = core::result::Result<T, Error>;
//
// #[derive(Debug, Display, From)]
// #[display("{self:?}")]
// pub enum Error {
//     #[from(String, &String, &str)]
//     Custom(String),
//
//     // -- Externals
//     #[from]
//     Io(std::io::Error),
// }
//
// impl Error {
//     pub fn custom_from_err(err: impl std::error::Error) -> Self {
//         Self::Custom(err.to_string())
//     }
//
//     pub fn custom(val: impl Into<String>) -> Self {
//         Self::Custom(val.into())
//     }
// }
//
// impl std::error::Error for Error {}
