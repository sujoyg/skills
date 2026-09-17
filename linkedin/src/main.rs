//! Opens a LinkedIn people search in the default browser.
//!
//! The browser carries the user's existing LinkedIn session, so the results
//! page loads signed in.

use std::process::{Command, ExitCode};

const USAGE: &str = "usage: linkedin find person --name <name> [--company <name>]";

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

/// Parses `find person --name <name> [--company <name>]`.
///
/// `--company` narrows the search rather than standing on its own, so
/// `--name` is required.
fn parse_args(args: &[String]) -> Result<(&str, Option<&str>), String> {
    let (command, rest) = args.split_first().ok_or("missing command")?;
    if command != "find" {
        return Err(format!("unknown command: {command}"));
    }

    let (subject, options) = rest.split_first().ok_or("missing subject")?;
    if subject != "person" {
        return Err(format!("unknown subject: {subject}"));
    }

    let mut name = None;
    let mut company = None;
    let mut options = options.iter();
    while let Some(flag) = options.next() {
        let slot = match flag.as_str() {
            "--name" => &mut name,
            "--company" => &mut company,
            _ => return Err(format!("unknown option: {flag}")),
        };
        if slot.is_some() {
            return Err(format!("repeated option: {flag}"));
        }
        let value = options.next().ok_or(format!("{flag} needs a value"))?;
        *slot = Some(value.as_str());
    }

    Ok((name.ok_or("--name is required")?, company))
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
    fn parses_a_find_person_command() {
        assert_eq!(
            parse_args(&args(&["find", "person", "--name", "Jane Doe"])),
            Ok(("Jane Doe", None))
        );
        assert_eq!(
            parse_args(&args(&[
                "find",
                "person",
                "--name",
                "Jane Doe",
                "--company",
                "Acme"
            ])),
            Ok(("Jane Doe", Some("Acme")))
        );
        // Options may come in either order.
        assert_eq!(
            parse_args(&args(&[
                "find",
                "person",
                "--company",
                "Acme",
                "--name",
                "Jane Doe"
            ])),
            Ok(("Jane Doe", Some("Acme")))
        );
    }

    #[test]
    fn rejects_malformed_commands() {
        let malformed = [
            vec![],                                                    // no command
            vec!["search", "person", "--name", "Jane"],                // wrong command
            vec!["find"],                                              // no subject
            vec!["find", "company", "--name", "Acme"],                 // wrong subject
            vec!["find", "person", "--company", "Acme"],               // no --name
            vec!["find", "person", "--name", "Jane", "--person", "J"], // unknown option
            vec!["find", "person", "--name"],                          // option without a value
            vec!["find", "person", "--name", "A", "--name", "B"],      // repeated option
        ];
        for command in malformed {
            assert!(
                parse_args(&args(&command)).is_err(),
                "expected an error for {command:?}"
            );
        }
    }

    #[test]
    fn builds_an_encoded_search_url() {
        assert_eq!(
            search_url("Jane Doe", None),
            "https://www.linkedin.com/search/results/people/?keywords=Jane%20Doe"
        );
        // The company joins the keywords; reserved and non-ASCII bytes are escaped.
        assert_eq!(
            search_url("O'Neill Muñoz", Some("Acme & Co")),
            "https://www.linkedin.com/search/results/people/?keywords=O%27Neill%20Mu%C3%B1oz%20Acme%20%26%20Co"
        );
    }

    #[test]
    fn browser_command_passes_url_last() {
        let (program, args) = browser_command("https://example.com");
        assert!(!program.is_empty());
        assert_eq!(args.last().unwrap(), "https://example.com");
    }
}
