use super::{LogicalLine, LogicalLineToken};
use crate::checkers::logical_lines::LogicalLinesContext;
use ruff_diagnostics::Violation;
use ruff_macros::{derive_message_formats, violation};
use ruff_python_parser::{Tok, TokenKind};
use ruff_source_file::Locator;
use ruff_text_size::{TextRange, TextSize};

/// ## What it does
/// Checks for continuation lines not indented as far as they should be or indented too far.
///
/// ## Why is this bad?
/// This makes reading code harder.
///
/// ## Example
/// ```python
/// print("Python", (
/// "Rules"))
/// ```
///
/// Use instead:
/// ```python
/// print("Python", (
///     "Rules"))
/// ```
#[violation]
pub struct MissingOrOutdentedIndentation;

impl Violation for MissingOrOutdentedIndentation {
    #[derive_message_formats]
    fn message(&self) -> String {
        format!("Continuation line missing indentation or outdented.")
    }
}

fn is_new_physical_line(token: &LogicalLineToken, locator: &Locator) -> bool {
    let token_text = locator.slice(token.range);

    dbg!(&token_text);
    matches!(token.kind, TokenKind::NonLogicalNewline) || token_text.ends_with("\\\n")
}

fn get_physical_lines(logical_line: &LogicalLine, locator: &Locator) -> Vec<usize> {
    let mut non_logical_newlines_indices = Vec::new();
    let mut prev_end = TextSize::default();
    let mut prev_token: Option<&TokenKind> = None;
    for token in logical_line.tokens() {
        let trivia = locator.slice(TextRange::new(prev_end, token.range.start()));

        // Get the trivia between the previous and the current token and detect any newlines.
        // This is necessary because `RustPython` doesn't emit `[Tok::Newline]` tokens
        // between any two tokens that form a continuation. That's why we have to extract the
        // newlines "manually".
        for (index, text) in trivia.match_indices(['\n', '\r']) {
            if text == "\r" && trivia.as_bytes().get(index + 1) == Some(&b'\n') {
                continue;
            }
            // Newlines after a newline never form a continuation.
            if !matches!(
                prev_token,
                Some(TokenKind::Newline | TokenKind::NonLogicalNewline)
            ) && prev_token.is_some()
            {
                non_logical_newlines_indices.push(token.range.start().into())
            }
        }

        prev_token = Some(&token.kind);
        prev_end = token.range.end();
    }

    non_logical_newlines_indices
}

fn line_indent(logical_line: &LogicalLine) -> u32 {
    let mut nb_indents = 0;
    for token in logical_line.tokens() {
        if matches!(token.kind, TokenKind::Indent) {
            nb_indents += 1;
        }
    }

    nb_indents
}

/// E122
pub(crate) fn continuation_line_missing_indentation_or_outdented(
    context: &mut LogicalLinesContext,
    logical_line: &LogicalLine,
    locator: &Locator,
    indent_char: char,
    indent_size: usize,
) {
    // dbg!(&logical_line);
    let non_logical_newlines_indices = get_physical_lines(logical_line, locator);
    let nb_physical_lines = non_logical_newlines_indices.len();
    dbg!(&logical_line.text());
    dbg!(nb_physical_lines);
    dbg!(&non_logical_newlines_indices);
    if nb_physical_lines == 1 {
        return;
    }

    // indent_next tells us whether the next block is indented.
    // Assuming that it is indented by 4 spaces, then we should not allow 4-space
    // indents on the final continuation line.
    // In turn, some other indents are allowed to have an extra 4 spaces.
    let indent_next = logical_line.text().ends_with(':');

    let mut row = 0;
    let mut depth = 0;
    // Remember how many brackets were opened on each line
    let mut parens = vec![0; nb_physical_lines];
    // Relative indents of physical lines
    let mut rel_indent = vec![0; nb_physical_lines];
    // For each depth, collect a list of opening rows.
    // open_rows = [[0]]
    // # for each depth, memorize the hanging indentation
    // hangs = [None]
    // # visual indents
    // indent_chances = {}
    // last_indent = tokens[0][2]
    // visual_indent = None
    // last_token_multiline = False
    // # for each depth, memorize the visual indent column
    // indent = [last_indent[1]]

    for token in logical_line.tokens() {
        // this is the beginning of a continuation line.
        dbg!(&token);
        if is_new_physical_line(token, locator) {
            dbg!(&token);
            // Need to use only the physical line's token here.
            // last_indent = line_indent(logical_line);
        }
    }

    // let mut diagnostic = Diagnostic::new(
    //                         WhitespaceAfterOpenBracket { symbol },
    //                         TextRange::at(token.end(), trailing_len),
    //                     );
    //                     if autofix_after_open_bracket {
    //                         diagnostic
    //                             .set_fix(Fix::automatic(Edit::range_deletion(diagnostic.range())));
    //                     }
    //                     context.push_diagnostic(diagnostic);
}
