// 模拟全局 exec（获取所有匹配结果）
fn regex_exec_all(re: &Regex, input: &str) -> Vec<ExecResult> {
    let mut results = Vec::new();
    for mat in re.find_iter(input) {
        // 对每个匹配项重新获取捕获组
        let caps = re.captures(input).unwrap();
        // 复用之前的逻辑构建 ExecResult
        let mut groups = Vec::new();
        for i in 0..caps.len() {
            groups.push(caps.get(i).map(|m| m.as_str().to_string()));
        }
        let mut named_groups = HashMap::new();
        for name in re.capture_names().flatten() {
            if let Some(val) = caps.name(name) {
                named_groups.insert(name.to_string(), val.as_str().to_string());
            }
        }
        results.push(ExecResult {
            match_str: mat.as_str().to_string(),
            groups,
            named_groups,
            start: mat.start(),
            end: mat.end(),
            input: input.to_string(),
        });
    }
    results
}
