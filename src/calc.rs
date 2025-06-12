use std::fmt::Debug;

pub fn eval(source: &str, width: i32, height: i32) -> Result<i32> {
    let mut yard = Yard::new(source)?;
    while !yard.source.is_empty() {
        yard.shunt()?;
    }
    let mut ints = vec![];
    for token in yard.expel()? {
        match token {
            Token::Int(int) => ints.push(int),
            Token::Un(un) => match ints.pop() {
                Some(int) => ints.push(UnFn::from(un)(int)),
                _ => panic!(),
            },
            Token::Bin(bin) => match (ints.pop(), ints.pop()) {
                (Some(0), Some(_)) if bin == Binary::Div => return Err(Error::DivideByZero),
                (Some(right), Some(left)) => ints.push(BinFn::from(bin)(left, right)),
                _ => panic!(),
            },
            Token::Width => ints.push(width),
            Token::Height => ints.push(height),
            Token::OpenParen => return Err(Error::DanglingOpenParen),
            Token::CloseParen => return Err(Error::DanglingCloseParen),
        }
    }
    Ok(*ints.first().unwrap())
}

struct Yard<'a> {
    source: &'a str,
    detour: Vec<Token>,
    target: Vec<Token>,
    edicts: [Edict; 2],
    mode: Mode,
}

impl<'a> Yard<'a> {
    fn new(source: &'a str) -> Result<Self> {
        let source = source.trim_start();
        if source.is_empty() {
            Err(Error::EmptyExpression)
        } else {
            Ok(Self {
                source,
                detour: vec![],
                target: vec![],
                edicts: [default_placing, default_binding],
                mode: Mode::Place,
            })
        }
    }

    fn shunt(&mut self) -> Result<()> {
        let (token, source) = Token::claim(self.source, self)?;
        self.edicts[self.mode as usize](self, token)?;
        Ok(self.source = source.trim_start())
    }

    fn expel(mut self) -> Result<Vec<Token>> {
        match (self.detour.pop(), self.mode) {
            (Some(Token::Bin(bin)), Mode::Place) => {
                return Err(Error::DanglingOperator(char::from(bin)));
            }
            (Some(Token::Un(un)), Mode::Place) => {
                return Err(Error::DanglingOperator(char::from(un)));
            }
            (Some(operator), _) => self.target.push(operator),
            _ => (),
        }
        while !self.detour.is_empty() {
            self.target.push(self.detour.pop().unwrap());
        }
        Ok(self.target)
    }

    fn insert(&mut self, new_token: Token) -> Result<()> {
        while let Some(old_token) = self.detour.pop() {
            match (old_token.clone(), new_token.clone()) {
                (_, Token::OpenParen) | (Token::Un(_), Token::Un(_)) => {
                    self.detour.push(old_token);
                    break;
                }
                (Token::OpenParen, Token::CloseParen) => return Ok(()),
                (Token::OpenParen, new_operator) => {
                    self.detour.push(Token::OpenParen);
                    self.detour.push(new_operator);
                    return Ok(());
                }
                (old_operator, Token::CloseParen) => self.target.push(old_operator),
                (old_operator, new_operator) => {
                    if old_operator.precedence() >= new_operator.precedence() {
                        self.target.push(old_operator);
                    } else {
                        self.detour.push(old_operator);
                        break;
                    }
                }
            }
        }
        Ok(self.detour.push(new_token))
    }
}

type UnFn = fn(i32) -> i32;
type BinFn = fn(i32, i32) -> i32;
use std::ops::*;

#[derive(Clone, Debug, PartialEq)]
enum Token {
    Int(i32),
    Un(Unary),
    Bin(Binary),
    OpenParen,
    CloseParen,
    Width,
    Height,
}

impl Token {
    const affirm: Token = Token::Un(Unary::Affirm);
    const negate: Token = Token::Un(Unary::Negate);
    const add: Token = Token::Bin(Binary::Add);
    const sub: Token = Token::Bin(Binary::Sub);
    const mul: Token = Token::Bin(Binary::Mul);
    const div: Token = Token::Bin(Binary::Div);

    fn claim<'a>(source: &'a str, yard: &Yard) -> Result<(Token, &'a str)> {
        let mut chars = source.chars();
        let token = match yard.mode {
            Mode::Place => match chars.next().unwrap() {
                '0'..='9' => {
                    let digit_count = chars.take_while(|c| c.is_digit(10)).count() + 1;
                    return Ok((
                        Token::Int(source[0..digit_count].parse().unwrap()),
                        &source[digit_count..],
                    ));
                }
                'w' => Token::Width,
                'h' => Token::Height,
                '+' => Token::affirm,
                '-' => Token::negate,
                '*' => return Err(Error::MisplacedOperator('*')),
                '/' => return Err(Error::MisplacedOperator('/')),
                '(' => Token::OpenParen,
                ')' => return Err(Error::MisplacedCloseParen),
                c => return Err(Error::UnknownCharacter(c)),
            },
            Mode::Bind => match chars.next().unwrap() {
                '0'..='9' => {
                    let digit_count = chars.take_while(|c| c.is_digit(10)).count() + 1;
                    return Err(Error::MisplacedInteger(source[0..digit_count].to_string()));
                }
                'w' => return Err(Error::MisplacedInteger("w".into())),
                'h' => return Err(Error::MisplacedInteger("h".into())),
                '+' => Token::add,
                '-' => Token::sub,
                '*' => Token::mul,
                '/' => Token::div,
                '(' => return Err(Error::MisplacedOpenParen),
                ')' => Token::CloseParen,
                c => return Err(Error::UnknownCharacter(c)),
            },
        };
        Ok((token, &source[1..]))
    }

    fn precedence(&self) -> i32 {
        match self {
            &Token::affirm | &Token::negate => 3,
            &Token::add | &Token::sub => 1,
            &Token::mul | &Token::div => 2,
            _ => panic!(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
enum Unary {
    Affirm,
    Negate,
}

impl From<Unary> for UnFn {
    fn from(un: Unary) -> Self {
        match un {
            Unary::Affirm => i32::into,
            Unary::Negate => i32::neg,
        }
    }
}

impl From<Unary> for char {
    fn from(un: Unary) -> Self {
        match un {
            Unary::Affirm => '+',
            Unary::Negate => '-',
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
enum Binary {
    Add,
    Sub,
    Mul,
    Div,
}

impl From<Binary> for BinFn {
    fn from(bin: Binary) -> Self {
        match bin {
            Binary::Add => i32::add,
            Binary::Sub => i32::sub,
            Binary::Mul => i32::mul,
            Binary::Div => i32::div,
        }
    }
}

impl From<Binary> for char {
    fn from(bin: Binary) -> Self {
        match bin {
            Binary::Add => '+',
            Binary::Sub => '-',
            Binary::Mul => '*',
            Binary::Div => '/',
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    Place,
    Bind,
}

type Edict = fn(&mut Yard, token: Token) -> Result<()>;

fn default_placing(yard: &mut Yard, token: Token) -> Result<()> {
    match token {
        Token::Int(_) | Token::Width | Token::Height => {
            yard.target.push(token);
            yard.mode = Mode::Bind;
        }
        Token::Un(_) => {
            yard.insert(token)?;
        }
        Token::OpenParen => {
            yard.edicts[Mode::Bind as usize] = paren_binding;
            yard.insert(token)?
        }
        _ => panic!(),
    }
    Ok(())
}

fn default_binding(yard: &mut Yard, token: Token) -> Result<()> {
    match token {
        Token::Bin(_) => {
            yard.insert(token)?;
            yard.mode = Mode::Place;
        }
        Token::CloseParen => return Err(Error::DanglingCloseParen),
        _ => panic!(),
    }
    Ok(())
}

fn paren_binding(yard: &mut Yard, token: Token) -> Result<()> {
    if token == Token::CloseParen {
        yard.insert(token)
    } else {
        default_binding(yard, token)
    }
}

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum Error {
    #[error("empty expression")]
    EmptyExpression,

    #[error("unknown character: '{0}'")]
    UnknownCharacter(char),

    #[error("misplaced integer '{0}'")]
    MisplacedInteger(String),

    #[error("misplaced operator '{0}'")]
    MisplacedOperator(char),

    #[error("dangling operator '{0}'")]
    DanglingOperator(char),

    #[error("misplaced '('")]
    MisplacedOpenParen,

    #[error("misplaced ')'")]
    MisplacedCloseParen,

    #[error("dangling '('")]
    DanglingOpenParen,

    #[error("dangling ')'")]
    DanglingCloseParen,

    #[error("divide by zero")]
    DivideByZero,
}
