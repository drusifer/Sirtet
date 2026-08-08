/// Which renderer backend the player chose (or will be prompted to choose).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RendererChoice {
    Terminal,
    Gfx3d,
}

const FLAG_PREFIX: &str = "--renderer=";

/// Parses the `--renderer=terminal|3d` flag out of the process arguments (excluding argv[0]).
///
/// Returns `Ok(None)` if no `--renderer` flag was given (caller should prompt via the
/// picker). Returns `Err` with a human-readable message listing valid values if the flag
/// was given with an unrecognized value.
pub fn parse_renderer_arg(args: &[String]) -> Result<Option<RendererChoice>, String> {
    for arg in args {
        if let Some(value) = arg.strip_prefix(FLAG_PREFIX) {
            return match value {
                "terminal" => Ok(Some(RendererChoice::Terminal)),
                "3d" => Ok(Some(RendererChoice::Gfx3d)),
                other => Err(format!(
                    "Unrecognized --renderer value '{other}'. Valid options: terminal, 3d"
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
    fn invalid_value_errors_with_valid_options_listed() {
        let err = parse_renderer_arg(&args(&["--renderer=bogus"])).unwrap_err();
        assert!(err.contains("terminal"));
        assert!(err.contains("3d"));
        assert!(err.contains("bogus"));
    }
}
