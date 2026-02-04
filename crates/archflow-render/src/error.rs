// ═════════════════════════════════════════════════════════════════
// ArchFlow Render - Renderer Error Types
//
// This module defines error types for rendering operations.
// ═════════════════════════════════════════════════════════════

use alloc::string::String;

/// Renderer error kind
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderErrorKind {
    /// WebGPU-specific error
    WebGPU,
    /// WebGL2-specific error
    WebGL2,
    /// Canvas 2D-specific error
    Canvas2D,
    /// Rendering context was lost
    ContextLost,
    /// Shader compilation failed
    ShaderCompilation,
    /// Backend not available (e.g., WebGPU not supported by browser)
    BackendNotAvailable,
    /// Generic rendering error
    Generic,
    /// Invalid texture data (wrong size, format, or alignment)
    InvalidTextureData,
}

/// Renderer errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RenderError {
    /// WebGPU-specific error
    WebGPU(String),

    /// WebGL2-specific error
    WebGL2(String),

    /// Canvas 2D-specific error
    Canvas2D(String),

    /// Rendering context was lost
    ContextLost,

    /// Shader compilation failed
    ShaderCompilation(String),

    /// Backend not available (e.g., WebGPU not supported by browser)
    BackendNotAvailable(String),

    /// Generic rendering error
    Generic(String),
}

impl RenderError {
    /// Create a new RenderError with given kind and message
    pub fn new(kind: RenderErrorKind, message: &str) -> Self {
        match kind {
            RenderErrorKind::WebGPU => RenderError::WebGPU(String::from(message)),
            RenderErrorKind::WebGL2 => RenderError::WebGL2(String::from(message)),
            RenderErrorKind::Canvas2D => RenderError::Canvas2D(String::from(message)),
            RenderErrorKind::ContextLost => RenderError::ContextLost,
            RenderErrorKind::ShaderCompilation => {
                RenderError::ShaderCompilation(String::from(message))
            }
            RenderErrorKind::BackendNotAvailable => {
                RenderError::BackendNotAvailable(String::from(message))
            }
            RenderErrorKind::Generic => RenderError::Generic(String::from(message)),
            RenderErrorKind::InvalidTextureData => RenderError::WebGL2(String::from(message)),
        }
    }
}

impl core::fmt::Display for RenderError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            RenderError::WebGPU(msg) => write!(f, "WebGPU error: {}", msg),
            RenderError::WebGL2(msg) => write!(f, "WebGL2 error: {}", msg),
            RenderError::Canvas2D(msg) => write!(f, "Canvas 2D error: {}", msg),
            RenderError::ContextLost => write!(f, "Rendering context lost"),
            RenderError::ShaderCompilation(msg) => {
                write!(f, "Shader compilation failed: {}", msg)
            }
            RenderError::BackendNotAvailable(msg) => {
                write!(f, "Backend not available: {}", msg)
            }
            RenderError::Generic(msg) => write!(f, "Rendering error: {}", msg),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for RenderError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            RenderError::WebGPU(_) => None,
            RenderError::WebGL2(_) => None,
            RenderError::Canvas2D(_) => None,
            RenderError::ContextLost => None,
            RenderError::ShaderCompilation(_) => None,
            RenderError::BackendNotAvailable(_) => None,
            RenderError::Generic(_) => None,
        }
    }

    fn description(&self) -> Option<&str> {
        Some("Rendering error occurred")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::format;

    #[test]
    fn test_render_error_display() {
        assert_eq!(
            format!("{}", RenderError::WebGPU(String::from("test"))),
            "WebGPU error: test"
        );
        assert_eq!(
            format!("{}", RenderError::WebGL2(String::from("test"))),
            "WebGL2 error: test"
        );
        assert_eq!(
            format!("{}", RenderError::Canvas2D(String::from("test"))),
            "Canvas 2D error: test"
        );
        assert_eq!(
            format!("{}", RenderError::ContextLost),
            "Rendering context lost"
        );
        assert_eq!(
            format!("{}", RenderError::ShaderCompilation(String::from("test"))),
            "Shader compilation failed: test"
        );
        assert_eq!(
            format!(
                "{}",
                RenderError::BackendNotAvailable(String::from("WebGPU not supported"))
            ),
            "Backend not available: WebGPU not supported"
        );
        assert_eq!(
            format!("{}", RenderError::Generic(String::from("test"))),
            "Rendering error: test"
        );
    }

    #[test]
    fn test_render_error_equality() {
        let err1 = RenderError::WebGPU(String::from("test"));
        let err2 = RenderError::WebGPU(String::from("test"));
        assert_eq!(err1, err2);

        let err3 = RenderError::WebGL2(String::from("test"));
        assert_ne!(err1, err3);

        let err4 = RenderError::ContextLost;
        assert_eq!(err4, RenderError::ContextLost);
    }

    #[test]
    fn test_context_lost_error() {
        let err = RenderError::ContextLost;
        assert!(matches!(err, RenderError::ContextLost));
    }
}
