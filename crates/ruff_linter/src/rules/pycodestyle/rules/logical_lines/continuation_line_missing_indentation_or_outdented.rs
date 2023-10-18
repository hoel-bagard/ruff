use std::collections::HashMap;

use super::{LogicalLine, LogicalLineToken};
use crate::checkers::logical_lines::LogicalLinesContext;
use ruff_diagnostics::Violation;
use ruff_macros::{derive_message_formats, violation};
use ruff_python_parser::{Tok, TokenKind};
use ruff_source_file::Locator;
use ruff_text_size::{TextRange, TextSize};
use rustc_hash::FxHashMap;

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

fn get_continuation_indices(logical_line: &LogicalLine, locator: &Locator) -> Vec<usize> {
    let mut non_logical_newlines_indices = Vec::new();
    let mut prev_end = TextSize::default();
    let mut prev_token: Option<&TokenKind> = None;
    for token in logical_line.tokens() {
        if matches!(prev_token, Some(TokenKind::NonLogicalNewline)) {
            non_logical_newlines_indices.push(token.range.start().into());
        }

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

/// Because there is no Indent token for continuation lines.
fn line_indent(
    locator: &Locator,
    indent_char: char,
    indent_size: usize,
    physical_line_start: TextSize,
    first_token_start: TextSize,
) -> usize {
    let line_text = locator.slice(TextRange::new(physical_line_start, first_token_start));

    // To remove any trailing 'indent'.
    match line_text.lines().last() {
        None => 0,
        Some(line_text) => {
            let nb_indent_char = line_text.chars().filter(|ch| ch == &indent_char).count();

            if indent_char == '\t' {
                nb_indent_char
            } else {
                nb_indent_char / indent_size
            }
        }
    }
}

/// Return the amount of indentation.
/// Tabs are expanded to the next multiple of 8.
fn expand_indent(line: &str) -> usize {
    line.strip_suffix('\n');
    // Remove trailing newline and carriage return characters. TODO: Why ?
    let line = line.trim_end_matches(&['\n', '\r']);

    if !line.contains('\t') {
        // If there are no tabs in the line, return the leading space count
        return line.len() - line.trim_start().len();
    }
    let mut indent = 0;

    for ch in line.chars() {
        if ch == '\t' {
            indent = indent / 8 * 8 + 8;
        } else if ch == ' ' {
            indent += 1;
        } else {
            break;
        }
    }

    indent
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
    let continuation_indices = get_continuation_indices(logical_line, locator);
    let nb_physical_lines = continuation_indices.len() + 1; // Plus 1 to count the last newline token / empty lines.
    dbg!(&logical_line.text());
    dbg!(&logical_line.tokens());
    dbg!(nb_physical_lines);
    dbg!(&continuation_indices);
    if nb_physical_lines == 1 {
        return;
    }

    // Indent of the first physical line.
    let start_indent_level = line_indent(
        locator,
        indent_char,
        indent_size,
        logical_line.first_token().unwrap().range.start(),
        logical_line.first_token().unwrap().range.end(),
    );

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
    let mut open_rows = vec![vec![0]];
    // # for each depth, memorize the hanging indentation
    let mut hangs: Vec<Option<usize>> = Vec::new();
    // # visual indents
    // let mut indent_chances: FxHashMap<usize, usize> = FxHashMap::default();
    let mut indent_chances: Vec<usize> = Vec::new();
    let mut last_indent = start_indent_level;
    // visual_indent = None
    let last_token_multiline = false;
    // # for each depth, memorize the visual indent column
    // indent = [last_indent[1]]
    let mut indent = vec![last_indent];

    // To be able to compute the line relative start of a token.
    let mut physical_line_start = logical_line.tokens().first().unwrap().range.start();
    let mut prev_end = TextSize::default();
    for token in logical_line.tokens() {
        // TODO: instead of creating continuation indices, create the start and end of pycodestyle, and then zip it
        //       to the tokens iterator to get the start_physical_line_idx, end_physical_line_idx,
        //       token_start_within_physical_line , token_end_within_physical_line.

        // this is the beginning of a continuation line.
        if continuation_indices.contains(&token.range.start().into()) {
            // record the initial indent.
            let physical_line_start_text =
                locator.slice(TextRange::new(prev_end, token.range.start()));
            let indent_level = expand_indent(physical_line_start_text);
            rel_indent[row] = indent_level - start_indent_level;
            physical_line_start = token.range.start() - TextSize::try_from(indent_level).unwrap();
            // The start[1] from pycodestyle.
            let token_start_within_physical_line = token.range.start() - physical_line_start;
            dbg!(&token);
            dbg!(token_start_within_physical_line);

            let is_closing_bracket = matches!(
                token.kind,
                TokenKind::Rpar | TokenKind::Rsqb | TokenKind::Rbrace
            );

            let is_opening_bracket = matches!(
                token.kind,
                TokenKind::Lpar | TokenKind::Lsqb | TokenKind::Lbrace
            );

            if matches!(token.kind, TokenKind::Colon)
                && matches!(
                    logical_line.tokens().last().unwrap().kind,
                    TokenKind::NonLogicalNewline
                )
            {
                open_rows[depth].push(row);
            }

            // Is there any chance of visual indent?
            // let visual_indent = !is_closing_bracket
            //     && hang > 0
            //     && indent_chances.contains(&token_start_within_physical_line);

            if is_opening_bracket {
                depth += 1;
                indent.push(0);
                hangs.push(None);
                if open_rows.len() == depth {
                    open_rows.push(Vec::new());
                }
                open_rows[depth].push(row);
                parens[row] += 1;
            } else if is_closing_bracket && depth > 0 {
                // Parent indents should not be more than this one.
                let prev_indent = if let Some(i) = indent.pop() {
                    i
                } else {
                    last_indent
                };
                hangs.pop();
                for d in 0..depth {
                    if indent[d] > prev_indent {
                        indent[d] = 0
                    }
                }
                // for ind in indent
            }

            // # keep track of bracket depth
            // if token_type == tokenize.OP:
            //     if text in '([{':
            //         depth += 1
            //         indent.append(0)
            //         hangs.append(None)
            //         if len(open_rows) == depth:
            //             open_rows.append([])
            //         open_rows[depth].append(row)
            //         parens[row] += 1
            //         if verbose >= 4:
            //             print("bracket depth %s seen, col %s, visual min = %s" %
            //                   (depth, start[1], indent[depth]))
            //     elif text in ')]}' and depth > 0:
            //         # parent indents should not be more than this one
            //         prev_indent = indent.pop() or last_indent[1]
            //         hangs.pop()
            //         for d in range(depth):
            //             if indent[d] > prev_indent:
            //                 indent[d] = 0
            //         for ind in list(indent_chances):
            //             if ind >= prev_indent:
            //                 del indent_chances[ind]
            //         del open_rows[depth + 1:]
            //         depth -= 1
            //         if depth:
            //             indent_chances[indent[depth]] = True
            //         for idx in range(row, -1, -1):
            //             if parens[idx]:
            //                 parens[idx] -= 1
            //                 break
            //     assert len(indent) == depth + 1
            //     if start[1] not in indent_chances:
            //         # allow lining up tokens
            //         indent_chances[start[1]] = text
            // last_indent = line_indent(
            //     locator,
            //     indent_char,
            //     indent_size,
            //     prev_end,
            //     token.range.start(),
            // );
            // dbg!(&last_indent);

            // last_token_multiline = (start[0] != end[0])
            // if last_token_multiline:
            //     rel_indent[end[0] - first_row] = rel_indent[row]
        }
        prev_end = token.range.end();
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
