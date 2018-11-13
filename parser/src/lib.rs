
mod parse;
mod error;
mod load;

pub mod optimize;

pub use parse::*;
pub use error::*;

use load::load_file_text;


use std::path::Path;

/// Load a file from a file and parse it's contents into an abstract syntax tree
pub fn parse_file<P: AsRef<Path>>(path: P) -> Result<Vec<parse::Sequence>> {
    let source = load_file_text(path)?;

    parse_str(&source)
}



#[cfg(test)]
mod tests {
    use super::*;
    use parse::{
        Instruction::*,
        ValueSource::*,
        Direction::*,
        Start::*,
        QueueEnd::*
    };

    #[test]
    fn optimize_constant_arithmetic() {
        let sequences = parse_str("155+55+55+**-").unwrap();
        let sequences = optimize::compute_constants(sequences);

        assert_eq!(
            sequences,
            vec![
                vec![vec![Exit]],

                vec![vec![Push(Constant(999)), Exit]],

                vec![vec![Exit]],
            ]
        );
    }

    #[test]
    fn optimize_insert() {
        let sequences = parse_str("77+}").unwrap();
        let sequences = optimize::compute_constants(sequences);

        assert_eq!(
            sequences,
            vec![
                vec![vec![Exit]],

                vec![vec![Insert(Constant(14), Front), Exit]],

                vec![vec![Exit]],
            ]
        );
    }

    #[test]
    fn optimize_constant_clone() {
        let sequences = parse_str("79+\\+").unwrap();
        let sequences = optimize::compute_constants(sequences);

        assert_eq!(
            sequences,
            vec![
                vec![vec![Exit]],

                vec![vec![Push(Constant(32)), Exit]],

                vec![vec![Exit]],
            ]
        );
    }
}
