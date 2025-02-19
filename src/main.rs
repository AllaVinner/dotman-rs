mod arg_parser;
mod config;

fn main() {
    arg_parser::parse_args();
    config::test();
}
