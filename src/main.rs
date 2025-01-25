use clap::Parser;
use core::panic;
use directories::ProjectDirs;
use std::{
    fs::{self, File},
    io::{Read, Write},
    process,
    time::{SystemTime, UNIX_EPOCH},
};
use tree_sitter::Node;

#[derive(Parser)]
struct Cli {
    #[arg(long, help = "Flag to disable file backup")]
    disable_backup: bool,
}

fn main() {
    let project_dirs = ProjectDirs::from("fr", "kensa", "y86fmt").unwrap();
    let settings = Cli::parse();
    let mut stdin = std::io::stdin();

    let mut data = Vec::new();
    stdin
        .read_to_end(&mut data)
        .expect("failed to read from stdin");

    if !settings.disable_backup {
        let mut backup_folder = project_dirs.cache_dir().to_path_buf();
        if !backup_folder.exists() {
            fs::create_dir(&backup_folder).expect(&format!(
                "failed to create cache folder at {}",
                backup_folder.to_str().unwrap()
            ));
        }
        backup_folder.push("backup");
        if !backup_folder.exists() {
            fs::create_dir(&backup_folder).expect(&format!(
                "failed to create backup folder at {}",
                backup_folder.to_str().unwrap()
            ));
        }

        let time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let backup_file = backup_folder.join(format!("{}.ys", time));
        let mut backup_file = File::create(backup_file).expect("failed to create backup file");
        backup_file
            .write_all(&data)
            .expect("failed to write to backup file");
    }

    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&tree_sitter_y86::LANGUAGE.into())
        .expect("failed to set parser language");

    let tree = parser.parse(&data, None).expect("failed to parse input");
    let mut cursor = tree.root_node().walk();
    if !cursor.goto_first_child() {
        println!("exit");
        process::exit(0);
    }

    // Find all empty line separated blocks
    let mut blocks: Vec<Vec<Node>> = Vec::new();
    let mut current_block: Vec<Node> = Vec::new();
    loop {
        let node = cursor.node();

        if node.child_count() == 0 {
            // line is empty ==> end of block
            blocks.push(current_block);
            current_block = Vec::new();
        } else {
            // if the line is a label, end the block
            let mut cursor = node.walk();
            cursor.goto_first_child();
            if cursor.node().kind() == "label" {
                blocks.push(current_block);
                current_block = Vec::new();
            }
        }
        current_block.push(node);

        if !cursor.goto_next_sibling() {
            break;
        }
    }
    blocks.push(current_block);

    let mut output: Vec<String> = Vec::new();

    // unpack line into arrays of children and split line that should be
    for block in blocks {
        let mut separated_block = Vec::with_capacity(block.len());
        for node in block {
            match node.child_count() {
                0 => separated_block.push(Vec::new()),
                1 => separated_block.push(Vec::from([node.child(0).unwrap()])),
                _ => {
                    let mut children = Vec::new();
                    let mut cursor = node.walk();
                    cursor.goto_first_child();
                    loop {
                        let child = cursor.node();
                        children.push(child);

                        if !cursor.goto_next_sibling() {
                            break;
                        }
                    }

                    // if a line has more than one token, it must be splited into multiple lines, except:
                    // - label + directive
                    // - anything + comment
                    let mut end_comment = None;
                    if children.last().unwrap().kind() == "comment" {
                        // if the last token of the line is a comment, remove it, process the rest then re-add it at the end
                        end_comment = Some(children.pop().unwrap());
                    }

                    let children_count = children.len();
                    let mut child_iter = children.into_iter().enumerate();

                    while let Some((child_index, child)) = child_iter.next() {
                        let is_last = child_index == children_count - 1;
                        if is_last {
                            // the current child is the last, add the comment we maybe removed before
                            let mut line = Vec::from([child]);
                            if end_comment.is_some() {
                                line.push(end_comment.unwrap());
                            }
                            separated_block.push(line);
                        } else {
                            if child.kind() == "label" {
                                let (next_index, next) = child_iter.next().unwrap();
                                let next_is_last = next_index == children_count - 1;
                                if next.kind() == "directive" {
                                    // label + directive
                                    let mut line = Vec::from([child, next]);
                                    if next_is_last {
                                        if end_comment.is_some() {
                                            line.push(end_comment.unwrap());
                                        }
                                    }
                                    separated_block.push(line);
                                } else {
                                    separated_block.push(Vec::from([child]));

                                    let mut line = Vec::from([next]);
                                    if next_is_last {
                                        if end_comment.is_some() {
                                            line.push(end_comment.unwrap());
                                        }
                                    }
                                    separated_block.push(line);
                                }
                            }
                        }
                    }
                }
            }
        }

        let mut instr_length = 0;
        let mut first_arg_length = 0;
        let mut second_arg_length = 0;
        for line in separated_block.iter() {
            for token in line {
                if token.kind() == "instruction" {
                    let (indentifier, arg1, arg2) = get_instruction_args(&data, token);
                    instr_length = instr_length.max(indentifier.len());
                    first_arg_length = first_arg_length.max(arg1.len());
                    second_arg_length = second_arg_length.max(arg2.len());
                }
            }
        }

        for (line_index, line) in separated_block.iter().enumerate() {
            if line.is_empty() {
                // empty line (not changed for now)
                output.push(String::new());
            } else {
                let mut output_parts = Vec::new();
                for (token_index, token) in line.iter().enumerate() {
                    match token.kind() {
                        "label" | "directive" => {
                            // label and directive are never indented
                            output_parts.push(get_string(&data, token));
                        }
                        "comment" => {
                            let comment = format!(
                                "# {}",
                                get_string(
                                    &data,
                                    &token.child(1).expect("failed to get comment content")
                                )
                                .trim()
                            );
                            if token_index == 0 {
                                // comment is the first token in a line ==> comment alone on a line (because of previous loop)
                                if line_index == 0 {
                                    // first line of the block => not indented
                                    output_parts.push(comment);
                                } else {
                                    let prev_line = &separated_block[line_index - 1];
                                    if prev_line.iter().any(|node| node.kind() == "directive") {
                                        // previous line contains a directive, so don't indent
                                        // this part i'm not certain of, may need change
                                        output_parts.push(comment);
                                    } else {
                                        // otherwise comment is always formatt
                                        output_parts.push(format!("\t{}", comment));
                                    }
                                }
                            } else {
                                // comment is after another token ==> not indented
                                output_parts.push(comment);
                            }
                        }
                        "instruction" => {
                            // instruction are always indented
                            output_parts.push(format!(
                                "\t{}",
                                format_instruction(
                                    &data,
                                    &token,
                                    instr_length,
                                    first_arg_length,
                                    second_arg_length
                                )
                            ))
                        }
                        kind => panic!("invalid kind {}", kind),
                    }
                }
                let output_line = output_parts.join(" ");
                output.push(output_line);
            }
        }
    }

    let output = output.join("\n");
    println!("{}", output);
}

fn get_string(source_code: &Vec<u8>, node: &Node) -> String {
    node.utf8_text(&source_code)
        .expect("failed to get node as string")
        .to_string()
}

/// Returns the 3 parts of the instruction (instruction, arg1, arg2)
fn get_instruction_args(source_code: &Vec<u8>, node: &Node) -> (String, String, String) {
    let identifier = get_string(
        source_code,
        &node.child(0).expect("failed to get instruction identifier"),
    );

    if node.child_count() == 1 {
        return (identifier, "".to_string(), "".to_string());
    }
    let args_node = node.child(1).expect("failed to get instruction args");
    let mut args = Vec::new();
    let mut cursor = args_node.walk();

    if cursor.goto_first_child() {
        loop {
            let node = cursor.node();

            if node.kind() == "arg" {
                args.push(get_string(source_code, &node));
            }
            if !cursor.goto_next_sibling() {
                break;
            }
        }
    }

    let empty = "".to_string();
    let arg1 = args.get(0).unwrap_or(&empty);
    let arg2 = args.get(1).unwrap_or(&empty);
    return (identifier, arg1.clone(), arg2.clone());
}

/// instr_length: The length of the instruction part (contains the whitespace before the 1st arg)
/// first_arg_length: The length of the first arg part (the whitespace before the 2nd arg)
/// second_arg_length: The length of the second arg part (contains the whitespace before the end of the line)
fn format_instruction(
    source_code: &Vec<u8>,
    node: &Node,
    instr_length: usize,
    first_arg_length: usize,
    second_arg_length: usize,
) -> String {
    let (mut identifier, mut arg1, mut arg2) = get_instruction_args(source_code, node);

    identifier = pad(&identifier, instr_length);
    if arg1 != "" {
        if arg2 != "" {
            arg2 = pad(&arg2, second_arg_length);
            arg1.push(',');
            arg1 = pad(&arg1, first_arg_length + 1);
            return format!("{} {} {}", identifier, arg1, arg2);
        } else {
            arg1 = pad(&arg1, first_arg_length);
            return format!("{} {}", identifier, arg1);
        }
    }
    return format!("{}", identifier);
}

fn pad(s: &String, len: usize) -> String {
    let to_add = len - s.len();
    let mut new_str = s.clone();
    for _ in 0..to_add {
        new_str.push(' ');
    }
    return new_str;
}
