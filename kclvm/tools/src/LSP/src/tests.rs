use std::path::PathBuf;

use kclvm_error::Position;
use kclvm_tools::util::lsp::get_project_stack;

use crate::semantic_token::{
    get_imcomplete_semantic_tokens,
    imcomplete_semantic_tokens_to_semantic_tokens,
};

#[test]
fn test_resolve_program() {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("./src/test_file/semantic_tokens.k");
    let file = path.to_str().unwrap();

    let (files, opt) = get_project_stack(file);
    let mut tokens = get_imcomplete_semantic_tokens(&files, opt, file);

    // ((strat_line, strat_col), (enc_line, enc_col), kind, len)
    let expect_tokens = vec![
        ((1, 7), (5, 0), 3, 6), // Person kind:3 -> SemanticTokenType::STRUCT,
        ((5, 0), (5, 6), 1, 6), // person kind:1 -> SemanticTokenType::VARIABLE,
        ((8, 0), (8, 1), 1, 1), // a kind:1 -> SemanticTokenType::VARIABLE,
        ((9, 0), (9, 1), 1, 1), // b kind:1 -> SemanticTokenType::VARIABLE,
        ((2, 4), (2, 8), 1, 4), // name kind:4 -> SemanticTokenType::PROPERTY,
        ((6, 4), (6, 8), 9, 4), // name kind:4 -> SemanticTokenType::PROPERTY,
    ];

    for (i, token) in tokens.iter().enumerate() {
        let ((start_line, start_col), (end_line, end_col), kind, len) = expect_tokens[i];
        assert_eq!(
            token.start,
            Position {
                filename: file.to_string(),
                line: start_line,
                column: Some(start_col),
            }
        );
        assert_eq!(
            token.end,
            Position {
                filename: file.to_string(),
                line: end_line,
                column: Some(end_col),
            }
        );
        assert_eq!(token.kind, kind,);
        assert_eq!(token.length, len,);
    }

    let semantic_tokens = imcomplete_semantic_tokens_to_semantic_tokens(&mut tokens);

    let semantic_tokens_pos = vec![(0, 7), (1, 4), (3, 0), (1, 4), (2, 0), (1, 0)];
    for (i, token) in semantic_tokens.iter().enumerate() {
        assert_eq!(token.delta_line, semantic_tokens_pos[i].0);
        assert_eq!(token.delta_start, semantic_tokens_pos[i].1);
    }
}
