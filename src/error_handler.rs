use std::fs;

pub fn get_line_and_column(file_path: &str, index: usize) -> Option<String> {
    let content = fs::read_to_string(file_path).ok()?;
    let line_num: usize;
    let col_num: usize;
    let mut current_index = 0;

    for (line_idx, line) in content.lines().enumerate() {
        let line_length = line.len() + 1;
        if current_index + line_length > index {
            line_num = line_idx + 1;
            col_num = index - current_index + 1;
            return Some(format!(
                "file {} , line {}, column {}",
                file_path, line_num, col_num
            ));
        }
        current_index += line_length;
    }
    None
}
