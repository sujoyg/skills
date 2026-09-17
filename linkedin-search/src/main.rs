//! Opens a LinkedIn people search in the default browser.
//!
//! The browser carries the user's existing LinkedIn session, so the results
//! page loads signed in.

use std::process::{Command, ExitCode};

const USAGE: &str = "usage: linkedin-search <name> [company]";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (name, company) = match parse_args(&args) {
        Ok(parsed) => parsed,
        Err(message) => {
            eprintln!("{message}\n{USAGE}");
            return ExitCode::FAILURE;
        }
    };

    let url = search_url(name, company);
    let (program, args) = browser_command(&url);
    match Command::new(program).args(args).status() {
        Ok(status) if status.success() => ExitCode::SUCCESS,
        Ok(_) => {
            eprintln!("{program} could not open {url}");
            ExitCode::FAILURE
        }
        Err(error) => {
            eprintln!("could not run {program}: {error}");
            ExitCode::FAILURE
        }
    }
}

/// Splits the command line into the person's name and an optional company.
fn parse_args(args: &[String]) -> Result<(&str, Option<&str>), String> {
    match args {
        [name] => Ok((name, None)),
        [name, company] => Ok((name, Some(company))),
        [] => Err("missing name".to_string()),
        _ => Err("too many arguments".to_string()),
    }
}

/// Builds the people-search URL.
///
/// The company goes into the keywords rather than LinkedIn's company filter,
/// which only accepts numeric company IDs. Keywords match the whole profile,
/// so both current and former employers count.
fn search_url(name: &str, company: Option<&str>) -> String {
    let keywords = match company {
        Some(company) => format!("{name} {company}"),
        None => name.to_string(),
    };
    format!(
        "https://www.linkedin.com/search/results/people/?keywords={}",
        encode(keywords.trim())
    )
}

/// Percent-encodes everything outside the URL unreserved set.
fn encode(text: &str) -> String {
    let mut encoded = String::new();
    for byte in text.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                encoded.push(byte as char)
            }
            _ => encoded.push_str(&format!("%{byte:02X}")),
        }
    }
    encoded
}

/// The platform command that hands a URL to the default browser.
fn browser_command(url: &str) -> (&'static str, Vec<String>) {
    if cfg!(target_os = "macos") {
        ("open", vec![url.to_string()])
    } else if cfg!(target_os = "windows") {
        // `start` treats its first quoted argument as a window title, so pass an empty one.
        (
            "cmd",
            vec!["/C".into(), "start".into(), "".into(), url.to_string()],
        )
    } else {
        ("xdg-open", vec![url.to_string()])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(values: &[&str]) -> Vec<String> {
        values.iter().map(|v| v.to_string()).collect()
    }

    #[test]
    fn parses_name_only() {
        assert_eq!(parse_args(&args(&["Jane Doe"])), Ok(("Jane Doe", None)));
    }

    #[test]
    fn parses_name_and_company() {
        assert_eq!(
            parse_args(&args(&["Jane Doe", "Acme"])),
            Ok(("Jane Doe", Some("Acme")))
        );
    }

    #[test]
    fn rejects_missing_name() {
        assert!(parse_args(&args(&[])).is_err());
    }

    #[test]
    fn rejects_extra_arguments() {
        assert!(parse_args(&args(&["Jane Doe", "Acme", "extra"])).is_err());
    }

    #[test]
    fn builds_url_from_name() {
        assert_eq!(
            search_url("Jane Doe", None),
            "https://www.linkedin.com/search/results/people/?keywords=Jane%20Doe"
        );
    }

    #[test]
    fn appends_company_to_keywords() {
        assert_eq!(
            search_url("Jane Doe", Some("Acme Corp")),
            "https://www.linkedin.com/search/results/people/?keywords=Jane%20Doe%20Acme%20Corp"
        );
    }

    #[test]
    fn encodes_reserved_characters() {
        assert_eq!(encode("O'Neill & Sons"), "O%27Neill%20%26%20Sons");
    }

    #[test]
    fn encodes_non_ascii_characters() {
        assert_eq!(encode("Ana Muñoz"), "Ana%20Mu%C3%B1oz");
    }

    #[test]
    fn leaves_unreserved_characters_alone() {
        assert_eq!(encode("Jane-Doe_1.0~x"), "Jane-Doe_1.0~x");
    }

    #[test]
    fn browser_command_passes_url_last() {
        let (program, args) = browser_command("https://example.com");
        assert!(!program.is_empty());
        assert_eq!(args.last().unwrap(), "https://example.com");
    }
}
