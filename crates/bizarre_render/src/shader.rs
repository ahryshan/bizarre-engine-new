use std::{
    fs::File,
    io::{self, Cursor, SeekFrom, Write},
    path::Path,
};

use ash::vk;
use bizarre_core::utils::{io_err_mapper, FromIoError};
use bizarre_log::core_info;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ShaderError {
    #[error("IO error on `{path}`: {source}")]
    Io { path: String, source: io::Error },
    #[error("Could not create a Spir-V compiler")]
    CouldNotCreateCompiler,
    #[error(transparent)]
    CompilationError(#[from] shaderc::Error),
    #[error(transparent)]
    VkError(#[from] vk::Result),
    #[error("Invalid Spir-V in `{path}`: {source}")]
    SpirvError { path: String, source: SpirvError },
}

impl FromIoError for ShaderError {
    fn io_err<P: AsRef<Path>>(path: P, err: io::Error) -> Self {
        Self::Io {
            path: path.as_ref().to_string_lossy().into(),
            source: err,
        }
    }
}

pub type ShaderResult<T> = Result<T, ShaderError>;

const SRC_SHADER_PREFIX: &'static str = "assets/shaders/";
const CACHE_SHADER_PREFIX: &'static str = "cache/shaders";

#[derive(Clone, Copy, Debug)]
pub enum ShaderKind {
    Vertex,
    Fragment,
    Compute,
}

impl From<ShaderKind> for shaderc::ShaderKind {
    fn from(value: ShaderKind) -> Self {
        match value {
            ShaderKind::Vertex => shaderc::ShaderKind::Vertex,
            ShaderKind::Fragment => shaderc::ShaderKind::Fragment,
            ShaderKind::Compute => shaderc::ShaderKind::Compute,
        }
    }
}

impl From<ShaderKind> for vk::ShaderStageFlags {
    fn from(value: ShaderKind) -> Self {
        match value {
            ShaderKind::Vertex => vk::ShaderStageFlags::VERTEX,
            ShaderKind::Fragment => vk::ShaderStageFlags::FRAGMENT,
            ShaderKind::Compute => vk::ShaderStageFlags::COMPUTE,
        }
    }
}

pub fn load_shader(path: &Path, shader_type: ShaderKind) -> ShaderResult<Vec<u32>> {
    let filename = path.file_name().unwrap().to_str().unwrap();
    let asset_dir = path
        .parent()
        .unwrap()
        .strip_prefix(SRC_SHADER_PREFIX)
        .unwrap();

    let io_err = io_err_mapper::<_, ShaderError>(path);

    let cached_path = Path::new(CACHE_SHADER_PREFIX)
        .join(asset_dir)
        .join(format!("{filename}.spv"));

    let invalid_cache = if cached_path.is_file() {
        let source_metadata = std::fs::metadata(path).map_err(&io_err)?;
        let cached_metadata = std::fs::metadata(&cached_path).map_err(&io_err)?;

        let source_modified = source_metadata.modified().map_err(&io_err)?;
        let cache_modified = cached_metadata.modified().map_err(&io_err)?;

        source_modified > cache_modified
    } else {
        true
    };

    let spv = if invalid_cache {
        core_info!("Compiling shader '{}'", path.to_str().unwrap());

        let mut file = File::open(path).map_err(|err| ShaderError::Io {
            path: path.to_string_lossy().to_string(),
            source: err,
        })?;

        let artifact = compile_shader(&mut file, shader_type, path)?;

        validate_spv(&mut Cursor::new(&artifact.as_binary_u8())).map_err(|err| {
            ShaderError::SpirvError {
                path: format!("[compiled from {path:?}]"),
                source: err,
            }
        })?;

        let prefix = cached_path.parent().unwrap();
        if !prefix.is_dir() {
            std::fs::create_dir_all(prefix).map_err(|err| ShaderError::Io {
                path: prefix.to_string_lossy().into(),
                source: err,
            })?;
        }

        let mut cached_file = File::create(&cached_path).map_err(|err| ShaderError::Io {
            path: cached_path.to_string_lossy().into(),
            source: err,
        })?;
        cached_file
            .write_all(artifact.as_binary_u8())
            .map_err(io_err);

        artifact.as_binary().to_vec()
    } else {
        let mut file = File::open(&cached_path).map_err(|err| ShaderError::Io {
            path: cached_path.to_string_lossy().into(),
            source: err,
        })?;

        validate_spv(&mut file).map_err(|err| ShaderError::SpirvError {
            path: cached_path.to_string_lossy().into(),
            source: err,
        })?;

        read_spv(&mut file).map_err(io_err)?
    };

    Ok(spv)
}

pub fn compile_shader<S>(
    stream: &mut S,
    shader_type: ShaderKind,
    path: &Path,
) -> ShaderResult<shaderc::CompilationArtifact>
where
    S: std::io::Read + std::io::Seek,
{
    let io_err = io_err_mapper::<_, ShaderError>(path);

    let source_len = stream.seek(SeekFrom::End(0)).map_err(&io_err)? as usize;

    stream.rewind().map_err(&io_err)?;

    let mut source = String::with_capacity(source_len);
    stream.read_to_string(&mut source).map_err(io_err)?;

    let compiler = shaderc::Compiler::new().ok_or(ShaderError::CouldNotCreateCompiler)?;
    let options = shaderc::CompileOptions::new().unwrap();

    let filename = path.file_name().unwrap().to_str().unwrap();

    let result = compiler.compile_into_spirv(
        &source,
        shaderc::ShaderKind::from(shader_type),
        filename,
        "main",
        Some(&options),
    )?;

    Ok(result)
}

#[derive(Error, Debug)]
pub enum SpirvError {
    #[error("Length is not a multiple of 4")]
    InvalidLength,
    #[error("Invalid magic number")]
    InvalidMagic,
    #[error(transparent)]
    Io(#[from] io::Error),
}

pub fn validate_spv<S>(stream: &mut S) -> Result<(), SpirvError>
where
    S: std::io::Seek + std::io::Read,
{
    let buf_len = stream.seek(SeekFrom::End(0))? as usize;

    if buf_len % 4 != 0 {
        return Err(SpirvError::InvalidLength);
    }
    stream.rewind()?;

    let mut magic_number = [0u8; 4];
    stream.read_exact(&mut magic_number)?;

    if magic_number != [0x03, 0x02, 0x23, 0x07] {
        return Err(SpirvError::InvalidMagic);
    }

    Ok(())
}

pub fn read_spv<S>(stream: &mut S) -> Result<Vec<u32>, io::Error>
where
    S: std::io::Seek + std::io::Read,
{
    let buf_len = stream.seek(SeekFrom::End(0))? as usize;
    stream.rewind()?;

    let mut buf = vec![0u32; buf_len / 4];
    unsafe {
        stream.read_exact(std::slice::from_raw_parts_mut(
            buf.as_mut_ptr().cast::<u8>(),
            buf_len,
        ))?;
    }

    Ok(buf)
}
