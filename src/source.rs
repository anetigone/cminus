use std::path::Path;

/// 源文件后缀
const SOURCE_EXTENSION: &str = "cm";

/// 从文件路径读取源代码内容
///
/// 校验文件后缀为 `.cm`，然后读取文件内容返回。
pub fn read_source_file(path: &Path) -> Result<String, SourceError> {
    let extension = path
        .extension()
        .ok_or(SourceError::InvalidExtension(path.to_path_buf()))?;

    if extension != SOURCE_EXTENSION {
        return Err(SourceError::InvalidExtension(path.to_path_buf()));
    }

    std::fs::read_to_string(path).map_err(SourceError::Io)
}

/// 源文件读取错误
#[derive(Debug)]
pub enum SourceError {
    /// 文件后缀不是 `.cm`
    InvalidExtension(std::path::PathBuf),
    /// IO 错误
    Io(std::io::Error),
}

impl std::fmt::Display for SourceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SourceError::InvalidExtension(path) => {
                write!(
                    f,
                    "invalid source file extension: {:?}, expected `.cm`",
                    path
                )
            }
            SourceError::Io(err) => write!(f, "failed to read source file: {}", err),
        }
    }
}

impl std::error::Error for SourceError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            SourceError::Io(err) => Some(err),
            _ => None,
        }
    }
}
