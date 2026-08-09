use crate::battle::GameMode;



/// Which renderer backend the player chose (or will be prompted to choose).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RendererChoice {
    Terminal,
    Gfx3d,
    Terminal3d,
    Gfx3dBox,
}

const RENDERER_PREFIX: &str = "--renderer=";
const MODE_PREFIX: &str = "--mode=";

/// Parses the `--renderer=terminal|3d|terminal_3d|3d_box` flag out of the process arguments.
pub fn parse_renderer_arg(args: &[String]) -> Result<Option<RendererChoice>, String> {
    for arg in args {
        if let Some(value) = arg.strip_prefix(RENDERER_PREFIX) {
            return match value {
                "terminal" => Ok(Some(RendererChoice::Terminal)),
                "3d" => Ok(Some(RendererChoice::Gfx3d)),
                "terminal_3d" | "tui_3d" => Ok(Some(RendererChoice::Terminal3d)),
                "3d_box" | "3d-box" | "blockout" => Ok(Some(RendererChoice::Gfx3dBox)),
                other => Err(format!(
                    "Unrecognized --renderer value '{other}'. Valid options: terminal, 3d, terminal_3d, 3d_box"
                )),
            };
        }
    }
    Ok(None)
}

/// Parses the `--mode=single|2p_local|vs_cpu` flag out of the process arguments.
pub fn parse_mode_arg(args: &[String]) -> Result<Option<GameMode>, String> {
    for arg in args {
        if let Some(value) = arg.strip_prefix(MODE_PREFIX) {
            return match value {
                "single" | "1p" => Ok(Some(GameMode::Single)),
                "2p_local" | "2p" | "local" => Ok(Some(GameMode::TwoPlayerLocal)),
                "vs_cpu" | "vs-cpu" | "cpu" => Ok(Some(GameMode::VsCpu)),
                other => Err(format!(
                    "Unrecognized --mode value '{other}'. Valid options: single, 2p_local, vs_cpu"
                )),
            };
        }
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(values: &[&str]) -> Vec<String> {
        values.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn no_flag_returns_none() {
        assert_eq!(parse_renderer_arg(&args(&[])), Ok(None));
        assert_eq!(parse_mode_arg(&args(&[])), Ok(None));
    }

    #[test]
    fn terminal_flag_parses() {
        assert_eq!(
            parse_renderer_arg(&args(&["--renderer=terminal"])),
            Ok(Some(RendererChoice::Terminal))
        );
    }

    #[test]
    fn mode_flag_parses() {
        assert_eq!(
            parse_mode_arg(&args(&["--mode=2p_local"])),
            Ok(Some(GameMode::TwoPlayerLocal))
        );
        assert_eq!(
            parse_mode_arg(&args(&["--mode=vs_cpu"])),
            Ok(Some(GameMode::VsCpu))
        );
    }
}
