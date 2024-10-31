use std::fmt::Debug;

pub fn eval(Token: &str, width: i32, height: i32) -> Result<i32> {
    let mut yard = Yard::new(Token);
    while !yard.source.is_empty() {
        yard.shunt()?;
    }
    let mut ints = vec![];
    for token in yard.expel()? {
        match token {
            Token::Int(int) => ints.push(int),
            Token::Un(un) => {
                let int = ints.pop().unwrap();
                ints.push((Into::<UnFn>::into(un))(int));
            },
            Token::Bin(bin) => {
                let right = ints.pop().unwrap();
                let left = ints.pop().unwrap();
                ints.push((Into::<BinFn>::into(bin))(left, right));
            },
            Token::Width => ints.push(width),
            Token::Height => ints.push(height),
            _ => panic!()
        }
    }
    Ok(*ints.first().unwrap())
}

struct Yard<'a> {
    source: &'a str,
    target: Vec<Token>,
    detour: Vec<Token>,
    edicts: [Edict; 2],
    mode: Mode,
}

impl<'a> Yard<'a> {
    fn new(source: &'a str) -> Self {
        Self { source, detour: Vec::new(), target: Vec::new(), edicts: [default_placing, default_binding], mode: Mode::Place }
    }

    fn shunt(&mut self) -> Result<()> {
        let (token, source) = Token::claim(self.source, self)?;
        (self.edicts[self.mode as usize])(self, token)?;
        Ok(self.source = source.trim_start())
    }

    fn expel(mut self) -> Result<Vec<Token>> {
        while !self.detour.is_empty() {
            self.target.push(self.detour.pop().unwrap());
        }
        Ok(self.target)
    }

    fn insert(&mut self, token: Token) -> Result<()> {
        while let Some(last_token) = self.detour.pop() {
            match (last_token, token.clone()) {
                (operator, Token::Paren(false)) => {
                    self.detour.push(operator);
                    break;
                },
                (Token::Paren(false), Token::Paren(true)) => return Ok(()),
                (Token::Paren(false), operator) => {
                    self.detour.push(Token::Paren(false).clone());
                    self.detour.push(operator);
                    return Ok(());
                },
                (operator, Token::Paren(true)) => self.target.push(operator),
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
        Ok(self.detour.push(token))
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
    Paren(bool),
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
                '0' ..= '9' => {
                    let digit_count = chars.take_while(|c| c.is_digit(10)).count() + 1;
                    return Ok((Token::Int(source[0 .. digit_count].parse().unwrap()), &source[digit_count ..]));
                },
                '+' => Token::affirm,
                '-' => Token::negate,
                'w' => Token::Width,
                'h' => Token::Height,
                '(' => Token::Paren(false),
                _ => panic!(),
            },
            Mode::Bind => match chars.next().unwrap() {
                '+' => Token::add,
                '-' => Token::sub,
                '*' => Token::mul,
                '/' => Token::div,
                ')' => Token::Paren(true),
                _ => panic!(),
            }
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
enum Unary { Affirm, Negate }

impl From<Unary> for UnFn {
    fn from(un: Unary) -> Self {
        match un {
            Unary::Affirm => i32::into,
            Unary::Negate => i32::neg,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
enum Binary { Add, Sub, Mul, Div }

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

#[derive(Clone, Copy)]
enum Mode {
    Place, Bind
}

type Edict = fn (&mut Yard, token: Token) -> Result<()>;

fn default_placing(yard: &mut Yard, token: Token) -> Result<()> {
    match token {
        Token::Int(_) | Token::Width | Token::Height => {
            yard.target.push(token);
            yard.mode = Mode::Bind;
        },
        Token::Un(_) => {
            yard.insert(token)?;
        },
        Token::Paren(false) => {
            yard.edicts[Mode::Bind as usize] = paren_binding;
            yard.insert(token)?
        },
        _ => panic!(),
    }
    Ok(())
}

fn default_binding(yard: &mut Yard, token: Token) -> Result<()> {
    match token {
        Token::Bin(_) => {
            yard.insert(token)?;
            yard.mode = Mode::Place;
        },
        _ => panic!()
    }
    Ok(())
}

fn paren_binding(yard: &mut Yard, token: Token) -> Result<()> {
    if token == Token::Paren(true) {
        yard.insert(token)
    } else {
        default_binding(yard, token)
    }
}

type Result<T> = std::result::Result<T, CalcError>;

#[derive(PartialEq)]
pub enum CalcError {
    Unknown,
}

impl Debug for CalcError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            CalcError::Unknown => write!(f, "invalid syntax in arithmetic argument!"),
        }
    }
}