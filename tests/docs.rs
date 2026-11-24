use yahucode::{emit, parser, runtime, types};

struct Block {
    info: String,
    text: String,
    line: usize,
}

/// Collects the fenced code blocks of a Markdown file.
fn blocks(path: &str) -> Vec<Block> {
    let text = std::fs::read_to_string(path).unwrap();
    let mut out = Vec::new();
    let mut open: Option<Block> = None;
    for (i, line) in text.lines().enumerate() {
        match (&mut open, line.strip_prefix("```")) {
            (None, Some(info)) => {
                open = Some(Block {
                    info: info.trim().to_string(),
                    text: String::new(),
                    line: i + 1,
                });
            }
            (Some(_), Some(_)) => out.extend(open.take()),
            (Some(block), None) => {
                block.text.push_str(line);
                block.text.push('\n');
            }
            (None, None) => {}
        }
    }
    assert!(open.is_none(), "{path} ends inside a code block");
    out
}

fn language_blocks() -> Vec<Block> {
    blocks("LANGUAGE.md")
}

#[test]
fn every_yahu_block_in_language_md_is_a_program_that_runs() {
    let all = language_blocks();
    let programs: Vec<&Block> = all.iter().filter(|b| b.info == "yahu").collect();
    assert!(
        programs.len() >= 10,
        "found only {} yahu blocks",
        programs.len()
    );
    for block in programs {
        let at = format!("LANGUAGE.md line {}", block.line);
        let program = parser::parse(&block.text).unwrap_or_else(|e| panic!("{at}: {e}"));
        let diags = types::check(&program);
        assert!(diags.is_empty(), "{at}: {diags:?}");
        let st = runtime::run(&program);
        assert!(st.runtime_error.is_none(), "{at}: {:?}", st.runtime_error);
    }
}

#[test]
fn language_md_code_blocks_use_hash_comments() {
    for block in language_blocks() {
        if block.info == "yahu" {
            assert!(
                !block.text.contains("//"),
                "LANGUAGE.md line {} uses `//` in a yahu block",
                block.line
            );
        }
        assert!(
            ["yahu", "sh", "output", "press-output"].contains(&block.info.as_str()),
            "LANGUAGE.md line {} has an unknown block type {:?}",
            block.line,
            block.info
        );
    }
}

#[test]
fn output_blocks_match_a_real_run_of_the_block_before_them() {
    let mut previous: Option<Block> = None;
    let mut checked = 0;
    for block in language_blocks() {
        match block.info.as_str() {
            "yahu" => previous = Some(block),
            "output" | "press-output" => {
                let program_block = previous.as_ref().expect("output without a program");
                let program = parser::parse(&program_block.text).unwrap();
                let st = runtime::run(&program);
                let real = if block.info == "output" {
                    emit::emit(&st)
                } else {
                    emit::press(&st, &program.comments)
                };
                assert_eq!(
                    block.text.trim_end(),
                    real,
                    "LANGUAGE.md line {} does not match a real run",
                    block.line
                );
                checked += 1;
            }
            _ => {}
        }
    }
    assert!(checked >= 1, "expected at least one output block");
}

#[test]
fn worked_examples_in_language_md_match_the_example_files() {
    let all = language_blocks();
    let programs: Vec<&Block> = all.iter().filter(|b| b.info == "yahu").collect();
    for (file, name) in [
        ("examples/02_protective_edge.yahu", "Protective Edge"),
        ("examples/15_rule_of_law.yahu", "Iron Law"),
    ] {
        let src = std::fs::read_to_string(file).unwrap();
        let code: Vec<&str> = src.lines().filter(|l| !l.starts_with('#')).collect();
        let found = programs.iter().any(|b| {
            b.text.contains(&format!("@operation(\"{name}\")"))
                && code.iter().all(|l| b.text.contains(l))
        });
        assert!(found, "no block in LANGUAGE.md covers {file}");
    }
}

#[test]
fn docs_avoid_em_dashes_and_section_signs() {
    for path in ["README.md", "LANGUAGE.md", "docs/sources.md"] {
        let text = std::fs::read_to_string(path).unwrap();
        assert!(!text.contains('\u{2014}'), "{path} has an em dash");
        assert!(!text.contains('\u{00a7}'), "{path} has a section sign");
    }
}
