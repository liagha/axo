pub fn indent(string: &String) -> String {
    string.lines()
        .map(|line| format!("    {}", line))
        .collect::<Vec<_>>()
        .join("\n")
}
