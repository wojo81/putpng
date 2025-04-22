#![allow(nonstandard_style)]
pub mod grab;
pub mod crop;
pub mod calc;

#[cfg(test)]
mod calc_tests {
    use crate::calc::*;

    #[test]
    fn add() {
        assert_eq!(eval("1 + 2", 0, 0), Ok(3));
    }

    #[test]
    fn subtract() {
        assert_eq!(eval("2 - 1", 0, 0), Ok(1));
    }

    #[test]
    fn multiply() {
        assert_eq!(eval("2 * 3", 0, 0), Ok(6));
    }

    #[test]
    fn divide() {
        assert_eq!(eval("6 / 2", 0, 0), Ok(3));
    }

    #[test]
    fn divide_floor() {
        assert_eq!(eval("3 / 4", 0, 0), Ok(0));
    }

    #[test]
    fn width_and_height() {
        assert_eq!(eval("w + h", 10, 20), Ok(30));
    }

    #[test]
    fn operator_precedence() {
        assert_eq!(eval("1 + 2 * 3", 0, 0), Ok(7));
        assert_eq!(eval("1 * 2 + 3", 0, 0), Ok(5));
    }

    #[test]
    fn unary_operator() {
        assert_eq!(eval("-1 + 2", 0, 0), Ok(1));
    }

    #[test]
    fn unary_operator_chain() {
        assert_eq!(eval("1 + - 2", 0, 0), Ok(-1));
        assert_eq!(eval("1 + -- 2", 0, 0), Ok(3));
    }

    #[test]
    fn paren_expressions() {
        assert_eq!(eval("(1 + 2) * 3", 0, 0), Ok(9));
        assert_eq!(eval("1 + (2 * 3)", 0, 0), Ok(7));
        assert_eq!(eval("1 + (2 + 3) * 4", 0, 0), Ok(21));
    }

    #[test]
    fn long_expressions() {
        assert_eq!(eval("1 + 2 * 3 - 4 / 5", 0, 0), Ok(7));
        assert_eq!(eval("1 + 2 * (3 - 4) / 5", 0, 0), Ok(1));
        assert_eq!(eval("1 + (2 * 3 - 4) / 5", 0, 0), Ok(1));
        assert_eq!(eval("(1 + 2) * (3 - 4) / (5 + 6)", 0, 0), Ok(0));
    }

    #[test]
    fn empty_parens() {
        assert_eq!(eval("()", 0, 0), Err(Error::MisplacedCloseParen));
    }

    #[test]
    fn unknown_character() {
        assert_eq!(eval("1 + @", 0, 0), Err(Error::UnknownCharacter('@')));
    }

    #[test]
    fn misplaced_integer() {
        assert_eq!(eval("1 + 2 3", 0, 0), Err(Error::MisplacedInteger("3".into())));
    }

    #[test]
    fn misplaced_operator() {
        assert_eq!(eval("1 * * 2", 0, 0), Err(Error::MisplacedOperator('*')));
    }

    #[test]
    fn dangling_operator() {
        assert_eq!(eval("1 +", 0, 0), Err(Error::DanglingOperator('+')));
    }

    #[test]
    fn misplaced_open_paren() {
        assert_eq!(eval("1 (", 0, 0), Err(Error::MisplacedOpenParen));
    }

    #[test]
    fn misplaced_close_paren() {
        assert_eq!(eval(")", 0, 0), Err(Error::MisplacedCloseParen));
    }

    #[test]
    fn dangling_open_paren() {
        assert_eq!(eval("(", 0, 0), Err(Error::DanglingOpenParen));
    }

    #[test]
    fn dangling_close_paren() {
        assert_eq!(eval("2 )", 0, 0), Err(Error::DanglingCloseParen));
    }

    #[test]
    fn empty_expression() {
        assert_eq!(eval("", 0, 0), Err(Error::EmptyExpression));
    }

    #[test]
    fn divide_by_zero() {
        assert_eq!(eval("1 / 0", 0, 0), Err(Error::DivideByZero));
    }
}