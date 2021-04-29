use std::env;
use std::fs;

use std::error::Error as StdError;
use std::fmt;

#[derive(PartialEq, Eq)]
enum Error {
    IllegalChar { line: usize, value: char },
    UnknownExpr { line: usize, value: String },
    MalformedAssign { line: usize },
    MalformedCond { line: usize },
    MalformedLoop { line: usize },
    MalformedRet { line: usize },
    MalformedFunc { line: usize },
}

impl Error {
    pub fn debug(&self) -> &str {
        match self {
            Self::IllegalChar { line, value } => format!("IllegalChar({}, {:?})", line, value),
            Self::UnknownExpr { line, value } => format!("UnknownExpr({}, {:?})", line, value),
            Self::MalformedAssign { line } => format!("MalformedAssign({})", line),
            Self::MalformedCond { line } => format!("MalformedCond({})", line),
            Self::MalformedLoop { line } => format!("MalformedLoop({})", line),
            Self::MalformedRet { line } => format!("MalformedRet({})", line),
            Self::MalformedFunc { line } => format!("MalformedFunc({})", line),
        }
    }

    pub fn format(&self) -> &str {
        match self {
            Self::IllegalChar { line, value } => {
                format!("unexpected character {:?} at line {}", value, line)
            }
            Self::UnknownExpr { line, value } => {
                format!("unexpected statement token {:?} at line {})", value, line)
            }
            Self::MalformedAssign { line } => {
                format!("malformed assignment statement at line {})", line)
            }
            Self::MalformedCond { line } => {
                format!("malformed conditional statement at line {})", line)
            }
            Self::MalformedLoop { line } => {
                format!("malformed loop statement at line {})", line)
            }
            Self::MalformedRet { line } => {
                format!("malformed return statement at line {})", line)
            }
            Self::MalformedFunc { line } => {
                format!("bad function definition at line {})", line)
            }
        }
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.debug())
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.format())
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        self.debug()
    }

    fn cause(&self) -> Option<&dyn StdError> {
        None
    }
}

struct Token {
    value: String,
}

impl Token {
    fn from(s: &str, ptr: usize, allow_const: bool) -> Result<Self, Error> {
        let x: Vec<_> = s
            .chars()
            .filter(|c| match c {
                '0'..='9' => !allow_const,
                '-' => !allow_const,
                'a'..='z' => false,
                '_' => false,
                _ => true,
            })
            .collect();
        match x.len() {
            0 => Ok(Self {
                value: String::from(s),
            }),
            _ => Err(Error::IllegalChar {
                line: ptr,
                value: x[0],
            }),
        }
    }

    pub fn from_any(state: &mut State, s: &str) -> Result<Self, Error> {
        Self::from(s, state.ptr, true)
    }

    pub fn from_var(state: &mut State, s: &str) -> Result<Self, Error> {
        Self::from(s, state.ptr, false)
    }

    pub fn to_string(&self) -> &str {
        &self.value
    }
}

struct Expr {
    tokens: Vec<Token>,
}

enum Statement {
    Assign {
        var: Token,
        expr: Expr,
        line: usize,
    },
    Cond {
        expr: Expr,
        child: Node,
        line: usize,
    },
    Loop {
        expr: Expr,
        child: Node,
        line: usize,
    },
    Print {
        vars: Vec<Token>,
        line: usize,
    },
    Ret {
        expr: Expr,
        line: usize,
    },
    Func {
        name: Token,
        params: Vec<Token>,
        child: Node,
        line: usize,
    },
}

struct Node {
    nodes: Vec<Statement>,
}

struct State<'a> {
    lines: Vec<&'a str>,
    ptr: usize,
}

type StmtParseResult = Result<Option<Statement>, Error>;

pub fn parse_stmt_assign(state: &mut State, words: &Vec<&str>) -> StmtParseResult {
    // let <variable> = <expression>
    let len = words.len();
    if len < 4 {
        return Err(Error::MalformedAssign { line: state.ptr });
    }
    let var = Token::from_var(state, words[1])?;
    let mut tokens = vec![];
    for i in 3..len {
        tokens.push(Token::from_any(state, words[i])?);
    }
    Ok(Some(Statement::Assign {
        var,
        expr: Expr { tokens },
        line: state.ptr,
    }))
}

pub fn parse_stmt_cond(state: &mut State, words: &Vec<&str>) -> StmtParseResult {
    // if <expression> then
    //     <code block>
    // end if
    let len = words.len();
    if words.len() < 3 || words[len - 1] != "then" {
        return Err(Error::MalformedCond { line: state.ptr });
    }
    // generate expression
    let mut tokens = vec![];
    for i in 1..len - 1 {
        tokens.push(Token::from_any(state, words[i])?);
    }
    // get child node
    state.ptr += 1;
    Ok(Some(Statement::Cond {
        expr: Expr { tokens },
        child: parse_node(state, "if")?,
        line: state.ptr,
    }))
}

pub fn parse_stmt_loop(state: &mut State, words: &Vec<&str>) -> StmtParseResult {
    // while <expression> do
    //     <code block>
    // end while
    let len = words.len();
    if words.len() < 3 || words[len - 1] != "do" {
        return Err(Error::MalformedLoop { line: state.ptr });
    }
    // generate expression
    let mut tokens = vec![];
    for i in 1..len - 1 {
        tokens.push(Token::from_any(state, words[i])?);
    }
    // get child node
    state.ptr += 1;
    Ok(Some(Statement::Loop {
        expr: Expr { tokens },
        child: parse_node(state, "while")?,
        line: state.ptr,
    }))
}

pub fn parse_stmt_print(state: &mut State, words: &Vec<&str>) -> StmtParseResult {
    // print <var1> <var2> ... <varn>
    // allows 0 variables
    let mut vars = vec![];
    for i in 1..words.len() {
        vars.push(Token::from_var(state, words[i])?);
    }
    Ok(Some(Statement::Print {
        vars,
        line: state.ptr,
    }))
}

pub fn parse_stmt_ret(state: &mut State, words: &Vec<&str>) -> StmtParseResult {
    // return <expression>
    let len = words.len();
    if len < 2 {
        return Err(Error::MalformedRet { line: state.ptr });
    }
    let mut tokens = vec![];
    for i in 1..len {
        tokens.push(Token::from_any(state, words[i])?);
    }
    Ok(Some(Statement::Ret {
        expr: Expr { tokens },
        line: state.ptr,
    }))
}

pub fn parse_stmt_func(state: &mut State, words: &Vec<&str>) -> StmtParseResult {
    // function <name> <param1> <param2> ... <paramn> as
    //     <code block>
    // end function
    let len = words.len();
    if words.len() < 3 || words[len - 1] != "as" {
        return Err(Error::MalformedFunc { line: state.ptr });
    }
    // parse parameters
    let name = Token::from_var(state, words[1])?;
    let mut params = vec![];
    for i in 2..len - 1 {
        params.push(Token::from_var(state, words[i])?);
    }
    // get child node
    state.ptr += 1;
    Ok(Some(Statement::Func {
        name,
        params,
        child: parse_node(&mut state, "function")?,
        line: state.ptr,
    }))
}

pub fn parse_stmt(state: &mut State) -> StmtParseResult {
    // eradicate comments
    let mut line = String::from(state.lines[state.ptr]);
    if line.contains('#') {
        let splits: Vec<_> = line.split('#').collect();
        line = String::from(splits[0]);
    }
    // filter into singular words and check if is empty line
    let words: Vec<_> = line.split(' ').filter(|w| w.len() > 0).collect();
    if words.len() == 0 {
        return Ok(None);
    }
    // filter by expression type
    match words[0] {
        "let" => parse_stmt_assign(&mut state, &words),
        "if" => parse_stmt_cond(&mut state, &words),
        "while" => parse_stmt_loop(&mut state, &words),
        "print" => parse_stmt_print(&mut state, &words),
        "return" => parse_stmt_ret(&mut state, &words),
        "function" => parse_stmt_func(&mut state, &words),
        _ => Err(Error::UnknownExpr {
            line: state.ptr,
            value: String::from(words[0].to_string()),
        }),
    }
}

pub fn parse_node(state: &mut State, term: &str) -> Result<Node, Error> {
    // eradicate comments
    Err(Error::IllegalChar {
        line: 0,
        value: '2',
    })
}

fn main() {
    // read program from file
    let args: Vec<_> = env::args().collect();
    if args.len() < 2 {
        eprintln!("nhotyp: fatal error: no input files");
        eprintln!("intepretation terminated.");
        return;
    }
    let contents;
    match fs::read_to_string(&args[1]) {
        Ok(v) => contents = v,
        Err(_) => {
            eprintln!("nhotyp: fatal error: {}: cannot read file", &args[1]);
            eprintln!("nhotyp: fatal error: no input files");
            eprintln!("interpretation terminated.");
            return;
        }
    }
    // parse program
    let state = State {
        lines: contents.split('\n').collect(),
        ptr: 0,
    };
    let node = parse_node(&mut state, "");
    println!("read ok");
}
