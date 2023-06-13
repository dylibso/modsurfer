fn main() {
    let mut buffer = String::new();
    std::io::stdin().read_line(&mut buffer).unwrap();
    let name = modsurfer_demangle::demangle_function_name(buffer);
    print!("{}", name);
}
