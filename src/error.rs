use crate::ast::parse::Rule;

pub fn print_error(filename: &str, source: &str, error: pest::error::Error<Rule>) {
    use codespan_reporting::diagnostic::{Diagnostic, Label};
    use codespan_reporting::files::SimpleFile;
    use codespan_reporting::term::{self, Config};
    use pest::error::LineColLocation;

    let file = SimpleFile::new(filename, source);

    // Get proper span information from the error
    let (start_offset, end_offset) = match &error.line_col {
        LineColLocation::Pos((line, col)) => {
            // For single position errors, convert line/col to offset
            let line_offsets: Vec<usize> = source
                .char_indices()
                .filter_map(|(i, c)| if c == '\n' { Some(i) } else { None })
                .collect();

            let line_idx = line - 1; // convert to 0-based index
            let line_start = if line_idx == 0 {
                0
            } else {
                line_offsets[line_idx - 1] + 1
            };
            let offset = line_start + col - 1;

            (offset, offset + 1) // Make a single character span
        }
        LineColLocation::Span((start_line, start_col), (end_line, end_col)) => {
            // For span errors, calculate offsets for both ends
            let line_offsets: Vec<usize> = source
                .char_indices()
                .filter_map(|(i, c)| if c == '\n' { Some(i) } else { None })
                .collect();

            // Calculate start offset
            let start_line_idx = start_line - 1; // convert to 0-based index
            let start_line_offset = if start_line_idx == 0 {
                0
            } else {
                line_offsets[start_line_idx - 1] + 1
            };
            let start_pos = start_line_offset + start_col - 1;

            // Calculate end offset
            let end_line_idx = end_line - 1; // convert to 0-based index
            let end_line_offset = if end_line_idx == 0 {
                0
            } else {
                line_offsets[end_line_idx - 1] + 1
            };
            let end_pos = end_line_offset + end_col;

            (start_pos, end_pos)
        }
    };

    // Get the problematic part of the input for better error messages
    let error_text = if end_offset > start_offset && end_offset <= source.len() {
        source[start_offset..end_offset].to_string()
    } else {
        "".to_string()
    };

    // Create a more descriptive message based on the error type
    let message = match &error.variant {
        pest::error::ErrorVariant::ParsingError {
            positives,
            negatives,
        } => {
            if !positives.is_empty() {
                format!("Expected {}", format_rules(positives))
            } else if !negatives.is_empty() {
                format!("Unexpected {}", format_rules(negatives))
            } else {
                "Parsing error".to_string()
            }
        }
        pest::error::ErrorVariant::CustomError { message } => message.clone(),
    };

    // Create notes with additional context
    let mut notes = Vec::new();

    match &error.variant {
        pest::error::ErrorVariant::ParsingError {
            positives,
            negatives,
        } => {
            if !positives.is_empty() && !negatives.is_empty() {
                notes.push(format!(
                    "Found `{}`, but expected {}",
                    if error_text.is_empty() {
                        "???"
                    } else {
                        &error_text
                    },
                    format_rules(positives)
                ));
            }
        }
        _ => {}
    }

    // Create the diagnostic
    let mut diagnostic = Diagnostic::error()
        .with_message("Syntax error")
        .with_labels(vec![
            Label::primary((), start_offset..end_offset).with_message(message),
        ]);

    // Add notes if there are any
    if !notes.is_empty() {
        diagnostic = diagnostic.with_notes(notes);
    }

    // Emit the diagnostic
    let writer = term::termcolor::StandardStream::stderr(term::termcolor::ColorChoice::Auto);
    let config = Config::default();
    term::emit(&mut writer.lock(), &config, &file, &diagnostic).unwrap();
}

// Helper function to format rules in a readable way
fn format_rules(rules: &[Rule]) -> String {
    if rules.is_empty() {
        return "nothing".to_string();
    }

    let rule_strings: Vec<String> = rules.iter().map(|rule| format!("`{:?}`", rule)).collect();

    if rule_strings.len() == 1 {
        rule_strings[0].clone()
    } else {
        let last = rule_strings.last().unwrap();
        let rest = &rule_strings[..rule_strings.len() - 1];
        format!("{} or {}", rest.join(", "), last)
    }
}
