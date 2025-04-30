use std::{
  error::Error,
  fmt::{Debug, Display},
  io::Error as IoError,
  path::PathBuf
};
use crate::parser::FIPSParserError;

pub enum ASPRError{
  Io(IoError),
  Parse(FIPSParserError),
  EmptyFile(PathBuf)
}

impl Display for ASPRError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      ASPRError::Io(e)    => write!(f, "ASPR IO error: {}", e),
      ASPRError::Parse(e) => write!(f, "ASPR Parse error: {}", e),
      ASPRError::EmptyFile(path) => write!(f, "ASPR data file is empty: {}", path.display()),
    }
  }
}

impl Debug for ASPRError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    Display::fmt(self, f)
  }
}

impl Error for ASPRError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ASPRError::Io(e)    => Some(e),
            ASPRError::Parse(e) => Some(e),
            ASPRError::EmptyFile(_) => None,
        }
    }
}
