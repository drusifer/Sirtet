/// Which renderer backend the player chose (or will be prompted to choose).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RendererChoice {
    Terminal,
    Gfx3d,
    Terminal3d,
    Gfx3dBox,
}

const FLAG_PREFIX: &str = "--renderer=";

/// Parses the `--renderer=terminal|3d|terminal_3d|3d_box` flag out of the process arguments.
pub fn parse_renderer_arg(args: &[String]) -> Result<Option<RendererChoice>, String> {
    for arg in args {
        if let Some(value) = arg.strip_prefix(FLAG_PREFIX) {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn args(values: &[&str]) -> Vec<String> {
        values.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn no_flag_returns_none() {
        assert_eq!(parse_renderer_arg(&args(&[])), Ok(None));
    }

    #[test]
    fn terminal_flag_parses() {
        assert_eq!(
            parse_renderer_arg(&args(&["--renderer=terminal"])),
            Ok(Some(RendererChoice::Terminal))
        );
    }

    #[test]
    fn three_d_flag_parses() {
        assert_eq!(
            parse_renderer_arg(&args(&["--renderer=3d"])),
            Ok(Some(RendererChoice::Gfx3d))
        );
    }

    #[test]
    fn terminal_3d_flag_parses() {
        assert_eq!(
            parse_renderer_arg(&args(&["--renderer=terminal_3d"])),
            Ok(Some(RendererChoice::Terminal3d))
        );
    }

    #[test]
    fn three_d_box_flag_parses() {
        assert_eq!(
            parse_renderer_arg(&args(&["--renderer=3d_box"])),
            Ok(Some(RendererChoice::Gfx3dBox))
        );
    }

    #[test]
    fn invalid_value_errors_with_valid_options_listed() {
        let err = parse_renderer_arg(&args(&["--renderer=bogus"])).unwrap_err();
        assert!(err.contains("terminal"));
        assert!(err.contains("3d_box"));
        assert!(err.contains("bogus"));
    }
}
