use std::collections::HashMap;
use std::env;
use std::error::Error as StdError;
use std::fmt;
use std::fs;
use std::io::Write;
use std::ops;

///////////////////////////////////////////////////////////////////////////////
/// Error handling

#[derive(PartialEq, Eq)]
enum Error {
    IllegalChar { line: usize, value: char },
    TokenTooLong { line: usize, value: usize },
    UnknownToken { line: usize, value: String },
    MalformedAssign { line: usize },
    MalformedCond { line: usize },
    MalformedLoop { line: usize },
    MalformedRet { line: usize },
    MalformedFunc { line: usize },
    MalformedEnd { line: usize },
    UnclosedBlock,
    DuplicateToken { line: usize, value: String },
    WildStatement { line: usize },
    WildFunction { line: usize },
    MisplacedRet { line: usize },
    UndeclaredToken { line: usize, value: String },
    BadExpression { line: usize },
    InputError { line: usize, value: String },
}

impl Error {
    pub fn debug(&self) -> String {
        match self {
            Self::IllegalChar { line, value } => format!("IllegalChar({}, {:?})", line, value),
            Self::TokenTooLong { line, value } => format!("TokenTooLong({}, {})", line, value),
            Self::UnknownToken { line, value } => format!("UnknownToken({}, {:?})", line, value),
            Self::MalformedAssign { line } => format!("MalformedAssign({})", line),
            Self::MalformedCond { line } => format!("MalformedCond({})", line),
            Self::MalformedLoop { line } => format!("MalformedLoop({})", line),
            Self::MalformedRet { line } => format!("MalformedRet({})", line),
            Self::MalformedFunc { line } => format!("MalformedFunc({})", line),
            Self::MalformedEnd { line } => format!("MalformedEnd({})", line),
            Self::UnclosedBlock => format!("UnclosedBlock"),
            Self::DuplicateToken { line, value } => {
                format!("DuplicateToken({}, {:?})", line, value)
            }
            Self::WildStatement { line } => format!("WildStatement({})", line),
            Self::WildFunction { line } => format!("WildFunction({})", line),
            Self::MisplacedRet { line } => format!("MisplacedRet({})", line),
            Self::UndeclaredToken { line, value } => {
                format!("UndeclaredToken({}, {})", line, value)
            }
            Self::BadExpression { line } => format!("BadExpression({})", line),
            Self::InputError { line, value } => format!("InputError({}, {:?})", line, value),
        }
    }

    pub fn format(&self) -> String {
        match self {
            Self::IllegalChar { value, .. } => {
                format!("unexpected character {:?}", value)
            }
            Self::TokenTooLong { value, .. } => {
                format!("token length exceeded ({} of 63)", value)
            }
            Self::UnknownToken { value, .. } => {
                format!("unexpected statement token {:?}", value)
            }
            Self::MalformedAssign { .. } => {
                format!("malformed assignment statement")
            }
            Self::MalformedCond { .. } => {
                format!("malformed conditional statement")
            }
            Self::MalformedLoop { .. } => {
                format!("malformed loop statement")
            }
            Self::MalformedRet { .. } => {
                format!("malformed return statement")
            }
            Self::MalformedFunc { .. } => {
                format!("bad function definition")
            }
            Self::MalformedEnd { .. } => {
                format!("illegal code block end")
            }
            Self::UnclosedBlock => format!("code block unclosed"),
            Self::DuplicateToken { value, .. } => {
                format!("conflict token {:?}", value)
            }
            Self::WildStatement { .. } => {
                format!("statements should appear in functions")
            }
            Self::WildFunction { .. } => {
                format!("function should not appear in functions")
            }
            Self::MisplacedRet { .. } => {
                format!("always return at end of function")
            }
            Self::UndeclaredToken { value, .. } => {
                format!("token {:?} undeclared", value)
            }
            Self::BadExpression { .. } => {
                format!("expression having misplaced tokens")
            }
            Self::InputError { value, .. } => {
                format!("invalid input {:?}", value)
            }
        }
    }

    pub fn line(&self) -> usize {
        match self {
            Self::IllegalChar { line, .. } => *line,
            Self::TokenTooLong { line, .. } => *line,
            Self::UnknownToken { line, .. } => *line,
            Self::MalformedAssign { line, .. } => *line,
            Self::MalformedCond { line, .. } => *line,
            Self::MalformedLoop { line, .. } => *line,
            Self::MalformedRet { line, .. } => *line,
            Self::MalformedFunc { line, .. } => *line,
            Self::MalformedEnd { line, .. } => *line,
            Self::UnclosedBlock => 0,
            Self::DuplicateToken { line, .. } => *line,
            Self::WildStatement { line, .. } => *line,
            Self::WildFunction { line, .. } => *line,
            Self::MisplacedRet { line, .. } => *line,
            Self::UndeclaredToken { line, .. } => *line,
            Self::BadExpression { line, .. } => *line,
            Self::InputError { line, .. } => *line,
        }
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.debug())
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.format())
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        "ParserError"
    }

    fn cause(&self) -> Option<&dyn StdError> {
        None
    }
}

///////////////////////////////////////////////////////////////////////////////
/// Tokens and Expressions

#[derive(Eq, PartialEq, Ord, PartialOrd, Hash)]
struct Token {
    value: String,
}

impl Token {
    fn from(s: &str, ptr: usize, allow_const: bool) -> Result<Self, Error> {
        if s.len() > 63 {
            return Err(Error::TokenTooLong {
                line: ptr,
                value: s.len(),
            });
        }
        let x: Vec<_> = s
            .chars()
            .filter(|c| match c {
                '0'..='9' => !allow_const,
                '<' | '=' | '>' => !allow_const,
                '+' | '-' | '*' | '%' | '/' => !allow_const,
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

    pub fn from_any(ptr: usize, s: &str) -> Result<Self, Error> {
        Self::from(s, ptr, true)
    }

    pub fn from_var(ptr: usize, s: &str) -> Result<Self, Error> {
        Self::from(s, ptr, false)
    }
}

impl Clone for Token {
    fn clone(&self) -> Self {
        Self {
            value: String::from(&self.value),
        }
    }
}

impl fmt::Debug for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.value)
    }
}

struct Expr {
    tokens: Vec<Token>,
}

impl fmt::Debug for Expr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_fmt(format_args!("{:?}", self.tokens))
    }
}

///////////////////////////////////////////////////////////////////////////////
/// Statements and Nodes

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

impl Statement {
    pub fn line(&self) -> usize {
        *match self {
            Self::Assign { line, .. } => line,
            Self::Cond { line, .. } => line,
            Self::Loop { line, .. } => line,
            Self::Print { line, .. } => line,
            Self::Ret { line, .. } => line,
            Self::Func { line, .. } => line,
        }
    }
}

impl fmt::Debug for Statement {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Assign { var, expr, line } => {
                f.write_fmt(format_args!("let({:?} <- {:?} @ {})", var, expr, line))
            }
            Self::Cond { expr, child, line } => {
                f.write_fmt(format_args!("if({:?} => {:?} @ {})", expr, child, line))
            }
            Self::Loop { expr, child, line } => {
                f.write_fmt(format_args!("while({:?} => {:?} @ {})", expr, child, line))
            }
            Self::Print { vars, line } => f.write_fmt(format_args!("print({:?} @ {})", vars, line)),
            Self::Ret { expr, line } => f.write_fmt(format_args!("ret({:?} @ {})", expr, line)),
            Self::Func {
                name,
                params,
                child,
                line,
            } => f.write_fmt(format_args!(
                "def({:?} -> {:?} => {:?} @ {})",
                name, params, child, line
            )),
        }
    }
}

struct Node {
    stmts: Vec<Statement>,
}

impl fmt::Debug for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_fmt(format_args!("{:?}", self.stmts))
    }
}

struct State<'a> {
    lines: &'a mut Vec<String>,
    ptr: usize,
}

type StmtParseResult = Result<Statement, Error>;

fn parse_stmt_assign(state: &mut State, words: &Vec<&str>) -> StmtParseResult {
    // let <variable> = <expression>
    let len = words.len();
    if len < 4 {
        return Err(Error::MalformedAssign { line: state.ptr - 1 });
    }
    let var = Token::from_var(state.ptr, words[1])?;
    let mut tokens = vec![];
    for i in 3..len {
        tokens.push(Token::from_any(state.ptr, words[i])?);
    }
    Ok(Statement::Assign {
        var,
        expr: Expr { tokens },
        line: state.ptr - 1,
    })
}

fn parse_stmt_cond(state: &mut State, words: &Vec<&str>) -> StmtParseResult {
    // if <expression> then
    //     <code block>
    // end if
    let len = words.len();
    if words.len() < 3 || words[len - 1] != "then" {
        return Err(Error::MalformedCond { line: state.ptr - 1 });
    }
    // generate expression
    let mut tokens = vec![];
    for i in 1..len - 1 {
        tokens.push(Token::from_any(state.ptr, words[i])?);
    }
    // get child node
    Ok(Statement::Cond {
        expr: Expr { tokens },
        child: parse_node(state, "if")?,
        line: state.ptr - 1,
    })
}

fn parse_stmt_loop(state: &mut State, words: &Vec<&str>) -> StmtParseResult {
    // while <expression> do
    //     <code block>
    // end while
    let len = words.len();
    if words.len() < 3 || words[len - 1] != "do" {
        return Err(Error::MalformedLoop { line: state.ptr - 1 });
    }
    // generate expression
    let mut tokens = vec![];
    for i in 1..len - 1 {
        tokens.push(Token::from_any(state.ptr, words[i])?);
    }
    // get child node
    Ok(Statement::Loop {
        expr: Expr { tokens },
        child: parse_node(state, "while")?,
        line: state.ptr - 1,
    })
}

fn parse_stmt_print(state: &mut State, words: &Vec<&str>) -> StmtParseResult {
    // print <var1> <var2> ... <varn>
    // allows 0 variables
    let mut vars = vec![];
    for i in 1..words.len() {
        vars.push(Token::from_var(state.ptr, words[i])?);
    }
    Ok(Statement::Print {
        vars,
        line: state.ptr - 1,
    })
}

fn parse_stmt_ret(state: &mut State, words: &Vec<&str>) -> StmtParseResult {
    // return <expression>
    let len = words.len();
    if len < 2 {
        return Err(Error::MalformedRet { line: state.ptr - 1 });
    }
    let mut tokens = vec![];
    for i in 1..len {
        tokens.push(Token::from_any(state.ptr, words[i])?);
    }
    Ok(Statement::Ret {
        expr: Expr { tokens },
        line: state.ptr - 1,
    })
}

fn parse_stmt_func(state: &mut State, words: &Vec<&str>) -> StmtParseResult {
    // function <name> <param1> <param2> ... <paramn> as
    //     <code block>
    // end function
    let len = words.len();
    if words.len() < 3 || words[len - 1] != "as" {
        return Err(Error::MalformedFunc { line: state.ptr - 1 });
    }
    // parse parameters
    let name = Token::from_var(state.ptr, words[1])?;
    let mut params = vec![];
    for i in 2..len - 1 {
        let token = Token::from_var(state.ptr, words[i])?;
        if is_reserved_kw(&token.value) {
            return Err(Error::DuplicateToken {
                line: state.ptr - 1,
                value: token.value,
            });
        }
        params.push(token);
    }
    // too many parameters
    if params.len() > 16 {
        return Err(Error::MalformedFunc { line: state.ptr - 1 });
    }
    // get child node
    Ok(Statement::Func {
        name,
        params,
        child: parse_node(state, "function")?,
        line: state.ptr - 1,
    })
}

fn parse_stmt(state: &mut State, words: &Vec<&str>) -> StmtParseResult {
    match words[0] {
        "let" => parse_stmt_assign(state, &words),
        "if" => parse_stmt_cond(state, &words),
        "while" => parse_stmt_loop(state, &words),
        "print" => parse_stmt_print(state, &words),
        "return" => parse_stmt_ret(state, &words),
        "function" => parse_stmt_func(state, &words),
        _ => Err(Error::UnknownToken {
            line: state.ptr - 1,
            value: String::from(words[0].to_string()),
        }),
    }
}

fn parse_node(state: &mut State, term: &str) -> Result<Node, Error> {
    let mut stmts = vec![];
    let mut gracefully_ended = term.len() == 0;
    // splitting words here to check for terminations
    while state.ptr < state.lines.len() {
        // eradicate comments
        let mut line = state.lines[state.ptr].clone();
        state.ptr += 1;
        if line.contains('#') {
            let splits: Vec<_> = line.split('#').collect();
            line = String::from(splits[0]);
        }
        // filter into singular words and check if is empty line
        let words: Vec<_> = line.split(' ').filter(|w| w.len() > 0).collect();
        if words.len() == 0 {
            continue;
        }
        // 'end' statement triggers code block close
        if words[0] == "end" {
            if words.len() == 2 && words[1] == term {
                gracefully_ended = true;
                break;
            }
            return Err(Error::MalformedEnd { line: state.ptr - 1 });
        }
        // send statement to corresponding parser
        stmts.push(parse_stmt(state, &words)?);
    }
    // check if block is unterminated
    if !gracefully_ended {
        return Err(Error::UnclosedBlock);
    }
    // done node parsing
    Ok(Node { stmts })
}

///////////////////////////////////////////////////////////////////////////////
/// Variables

const VARIABLE_LIMIT: i128 = 0x1_0000_0000_0000;

#[derive(Eq, PartialEq, Ord, PartialOrd, Copy, Clone)]
struct Variable {
    data: i128,
}

impl Variable {
    fn from(val: i128) -> Self {
        let mut data = val;
        if data > 0 {
            data = data & (VARIABLE_LIMIT - 1);
        } else if data < 0 {
        }
        Self { data }
    }
}

impl ops::Add for Variable {
    type Output = Self;
    fn add(self, other: Self) -> Self::Output {
        Self::from(self.data + other.data)
    }
}

impl ops::Sub for Variable {
    type Output = Self;
    fn sub(self, other: Self) -> Self::Output {
        Self::from(self.data - other.data)
    }
}

impl ops::Mul for Variable {
    type Output = Self;
    fn mul(self, other: Self) -> Self::Output {
        Self::from(self.data * other.data)
    }
}

impl ops::Rem for Variable {
    type Output = Self;
    fn rem(self, other: Self) -> Self::Output {
        let a = self.data;
        let b = other.data.abs();
        if b == 0 {
            return Self::from(0);
        }
        Self::from(match a > 0 {
            true => a % b,
            false => (b - (-a) % b) % b,
        })
    }
}

impl ops::Div for Variable {
    type Output = Self;
    fn div(self, other: Self) -> Self::Output {
        let b = other.data.abs();
        if b == 0 {
            return Self::from(0);
        }
        Self::from((self - self % other).data / b)
    }
}

impl ops::BitAnd for Variable {
    type Output = bool;
    fn bitand(self, other: Self) -> bool {
        self.data != 0 && other.data != 0
    }
}

impl ops::BitOr for Variable {
    type Output = bool;
    fn bitor(self, other: Self) -> bool {
        self.data != 0 || other.data != 0
    }
}

impl ops::BitXor for Variable {
    type Output = bool;
    fn bitxor(self, other: Self) -> bool {
        (self.data != 0) ^ (other.data != 0)
    }
}

impl ops::Not for Variable {
    type Output = bool;
    fn not(self) -> bool {
        self.data == 0
    }
}

impl fmt::Debug for Variable {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_fmt(format_args!("{:?}", self.data))
    }
}

///////////////////////////////////////////////////////////////////////////////
/// Program execution

struct Function {
    params: Vec<Token>,
    root: Node,
    line: usize,
}

struct Program {
    funcs: HashMap<Token, Function>,
}

struct RunInstance<'a> {
    prog: &'a Program,
    scope: HashMap<Token, Variable>,
}

fn eval_expr_func(
    instance: &mut RunInstance,
    expr: &Expr,
    ptr: &mut usize,
    line: usize,
) -> Result<Variable, Error> {
    // detect out-of-bounds error
    if *ptr >= expr.tokens.len() {
        return Err(Error::BadExpression { line: line });
    }
    // retrieve function parameter count
    let op_token: &str = &expr.tokens[*ptr].value;
    let op_cnt = match op_token {
        "scan" => 0,
        "+" | "-" | "*" | "%" | "/" => 2,
        "==" | "<" | ">" | "<=" | ">=" | "!=" => 2,
        "and" | "or" | "xor" => 2,
        "not" => 1,
        _ => {
            // parse constant first
            if let Ok(v) = op_token.parse() {
                return Ok(Variable::from(v));
            }
            let token = Token::from_var(line, op_token)?;
            if instance.scope.contains_key(&token) {
                // variable takes precedence
                return Ok(instance.scope[&token].clone());
            } else if instance.prog.funcs.contains_key(&token) {
                // then attempt to call function
                instance.prog.funcs[&token].params.len()
            } else {
                // and nothing else
                return Err(Error::UndeclaredToken {
                    line: line,
                    value: String::from(&token.value),
                });
            }
        }
    };
    // parse parameters
    let mut params = vec![];
    for _ in 0..op_cnt {
        *ptr += 1;
        params.push(eval_expr_func(instance, expr, ptr, line)?);
    }
    // evaluate result
    let v = &params;
    let is = |i: usize| -> bool { v[i].data != 0 };
    Ok(match op_token {
        "scan" => {
            let mut inp = String::new();
            print!("  > ");
            std::io::stdout().flush().expect("unable to flush stdout");
            if let Err(_) = std::io::stdin().read_line(&mut inp) {
                return Err(Error::InputError {
                    line,
                    value: String::from("null"),
                });
            }
            inp = String::from(inp.trim());
            match inp.parse() {
                Ok(v) => Variable::from(v),
                Err(_) => return Err(Error::InputError { line, value: inp }),
            }
        }
        "+" => v[0] + v[1],
        "-" => v[0] - v[1],
        "*" => v[0] * v[1],
        "%" => v[0] % v[1],
        "/" => v[0] / v[1],
        "==" => Variable::from(if v[0] == v[1] { 1 } else { 0 }),
        "<" => Variable::from(if v[0] < v[1] { 1 } else { 0 }),
        ">" => Variable::from(if v[0] > v[1] { 1 } else { 0 }),
        "<=" => Variable::from(if v[0] <= v[1] { 1 } else { 0 }),
        ">=" => Variable::from(if v[0] >= v[1] { 1 } else { 0 }),
        "!=" => Variable::from(if v[0] != v[1] { 1 } else { 0 }),
        "and" => Variable::from(if is(0) && is(1) { 1 } else { 0 }),
        "or" => Variable::from(if is(0) || is(1) { 1 } else { 0 }),
        "xor" => Variable::from(if is(0) != is(1) { 1 } else { 0 }),
        "not" => Variable::from(if is(0) { 0 } else { 1 }),
        _ => {
            let token = Token::from_var(line, op_token)?;
            call_function(instance.prog, &token, params, line)?
        }
    })
}

fn eval_expr(instance: &mut RunInstance, expr: &Expr, from_line: usize) -> Result<Variable, Error> {
    let mut ptr = 0;
    let res = eval_expr_func(instance, expr, &mut ptr, from_line)?;
    if ptr + 1 < expr.tokens.len() {
        return Err(Error::BadExpression { line: from_line });
    }
    Ok(res)
}

fn is_reserved_kw(token: &str) -> bool {
    match token {
        "and" | "or" | "xor" | "not" | "scan" => true,
        "let" => true,
        "if" | "then" => true,
        "while" | "do" => true,
        "function" | "as" | "return" => true,
        "end" => true,
        "print" => true,
        _ => false,
    }
}

fn exec_statement(instance: &mut RunInstance, stmt: &Statement) -> Result<(), Error> {
    match &stmt {
        &Statement::Assign { var, expr, line } => {
            if is_reserved_kw(&var.value) || instance.prog.funcs.contains_key(&var) {
                return Err(Error::DuplicateToken {
                    line: *line,
                    value: String::from(&var.value),
                });
            }
            let res = eval_expr(instance, &expr, *line)?;
            instance.scope.insert(var.clone(), res);
        }
        &Statement::Cond { expr, child, line } => {
            let cond = eval_expr(instance, &expr, *line)?;
            if cond.data != 0 {
                exec_node(instance, &child)?;
            }
        }
        &Statement::Loop { expr, child, line } => loop {
            let cond = eval_expr(instance, &expr, *line)?;
            if cond.data == 0 {
                break;
            }
            exec_node(instance, &child)?;
        },
        &Statement::Print { vars, line } => {
            // collect values
            let mut vals = vec![];
            for var in vars {
                if !instance.scope.contains_key(var) {
                    return Err(Error::UndeclaredToken {
                        line: *line,
                        value: String::from(&var.value),
                    });
                }
                vals.push(instance.scope[&var].data);
            }
            // flush into stdout in one go
            print!("  .");
            for val in vals {
                print!(" {}", val);
            }
            println!("");
            std::io::stdout().flush().expect("unable to flush stdout");
        }
        &Statement::Ret { line, .. } => return Err(Error::MisplacedRet { line: *line }),
        &Statement::Func { line, .. } => return Err(Error::WildFunction { line: *line }),
    }
    Ok(())
}

fn exec_node(instance: &mut RunInstance, node: &Node) -> Result<(), Error> {
    for stmt in &node.stmts {
        match stmt {
            _ => exec_statement(instance, &stmt)?,
        }
    }
    Ok(())
}

fn call_function(
    prog: &Program,
    token: &Token,
    params: Vec<Variable>,
    from_line: usize,
) -> Result<Variable, Error> {
    // lookup function
    if !prog.funcs.contains_key(&token) {
        return Err(Error::UndeclaredToken {
            line: from_line,
            value: String::from(&token.value),
        });
    }
    let func = &prog.funcs[&token];
    // generate instance
    let scope = HashMap::new();
    let mut instance = RunInstance { prog, scope };
    // put parameters into scope
    for i in 0..params.len() {
        let key = func.params[i].clone();
        let value = params[i].clone();
        if prog.funcs.contains_key(&key) {
            return Err(Error::DuplicateToken {
                line: func.line,
                value: key.value,
            });
        }
        instance.scope.insert(key, value);
    }
    // iterate function statements
    let stmts = &func.root.stmts;
    if stmts.len() < 1 {
        return Err(Error::MisplacedRet { line: func.line });
    }
    for i in 0..stmts.len() - 1 {
        exec_statement(&mut instance, &stmts[i])?;
    }
    // last statement must return value
    match &stmts[stmts.len() - 1] {
        Statement::Ret { expr, .. } => eval_expr(&mut instance, &expr, from_line),
        _ => Err(Error::MisplacedRet { line: func.line }),
    }
}

fn format_runtime_err(
    filename: Option<&str>,
    lines: &Vec<String>,
    err: &Error,
    line_offset: usize,
) -> String {
    let line = err.line();
    let header = match filename {
        Some(v) => format!("{}:{}: error: ", v, line + line_offset),
        None => format!("stdin:{}: error: ", line + line_offset),
    };
    let padding: String = (2..header.len()).map(|_| ' ').collect();
    let line = if let Some(v) = lines.get(line) { v } else { "" };
    format!("{}{}\n{}> {}\n", header, err, padding, line.trim())
}

fn execute_program(content: &mut Vec<String>) -> Result<i64, Error> {
    // parse file for functions
    let mut state = State {
        lines: content,
        ptr: 0,
    };
    let node = parse_node(&mut state, "")?;
    // check for wild statements at global scope and construct program
    let mut prog = Program {
        funcs: HashMap::new(),
    };
    for stmt in node.stmts {
        if let Statement::Func {
            name,
            params,
            child,
            line,
        } = stmt
        {
            if is_reserved_kw(&name.value) || prog.funcs.contains_key(&name) {
                return Err(Error::DuplicateToken {
                    line,
                    value: name.value,
                });
            }
            prog.funcs.insert(
                Token {
                    value: String::from(&name.value),
                },
                Function {
                    params,
                    root: child,
                    line,
                },
            );
        } else {
            return Err(Error::WildStatement { line: stmt.line() });
        }
    }
    // check if main function exists and call
    let main_token = Token {
        value: String::from("main"),
    };
    Ok(call_function(&mut prog, &main_token, vec![], 0)?.data as i64)
}

fn main_run_file(filename: &str) -> i32 {
    let content;
    match fs::read_to_string(&filename) {
        Ok(v) => content = v,
        Err(_) => {
            eprintln!("nhotyp: fatal error: {}: cannot read file", &filename);
            eprintln!("nhotyp: fatal error: no input files");
            eprintln!("interpretation terminated.");
            return 1;
        }
    }
    let mut lines = content.split('\n').map(|s| String::from(s)).collect();
    // catch return value or errors
    match execute_program(&mut lines) {
        Ok(v) => (v & 0xffffffffi64) as i32,
        Err(err) => {
            eprint!("{}", format_runtime_err(Some(filename), &lines, &err, 1));
            1
        }
    }
}

fn execute_block(
    state: &mut State,
    prog: &mut Program,
    scope: &mut HashMap<Token, Variable>,
    main_stmts: &mut Vec<Statement>,
    last_ptr: &mut usize,
    exec_ptr: &mut usize,
) -> Result<(), Error> {
    // try to parse node into statements
    let node = parse_node(state, "")?;
    // validate all statements, adding function, denying return
    for stmt in node.stmts {
        if let Statement::Func {
            name,
            params,
            child,
            line,
        } = stmt
        {
            if is_reserved_kw(&name.value) || prog.funcs.contains_key(&name) {
                return Err(Error::DuplicateToken {
                    line,
                    value: name.value,
                });
            }
            prog.funcs.insert(
                Token {
                    value: String::from(&name.value),
                },
                Function {
                    params,
                    root: child,
                    line,
                },
            );
        } else if let Statement::Ret { line, .. } = stmt {
            return Err(Error::WildStatement { line });
        } else {
            main_stmts.push(stmt);
        }
    }
    // create instance
    let mut new_scope = HashMap::new();
    for key in scope.keys() {
        new_scope.insert(key.clone(), scope[key].clone());
    }
    let mut instance = RunInstance {
        prog: prog,
        scope: new_scope,
    };
    // attempt execution
    let mut new_exec_ptr = *exec_ptr;
    while new_exec_ptr < main_stmts.len() {
        let stmt = &main_stmts[new_exec_ptr];
        exec_statement(&mut instance, stmt)?;
        new_exec_ptr += 1;
    }
    // writeback state
    for key in instance.scope.keys() {
        scope.insert(key.clone(), instance.scope[key].clone());
    }
    *last_ptr = state.ptr;
    *exec_ptr = new_exec_ptr;
    Ok(())
}

fn main_ii_show_copyright() -> () {
    println!("Copyright (c) 2021 Geoffrey Tang");
    println!("All lefts reversed.");
    println!("");
}

fn main_ii_show_license() -> () {
    println!("MIT License");
    println!("");
    println!("Copyright (c) 2021 Geoffrey Tang");
    println!("");
    println!("Permission is hereby granted, free of charge, to any person obtaining a copy");
    println!("of this software and associated documentation files (the \"Software\"), to deal");
    println!("in the Software without restriction, including without limitation the rights");
    println!("to use, copy, modify, merge, publish, distribute, sublicense, and/or sell");
    println!("copies of the Software, and to permit persons to whom the Software is");
    println!("furnished to do so, subject to the following conditions:");
    println!("");
    println!("The above copyright notice and this permission notice shall be included in all");
    println!("copies or substantial portions of the Software.");
    println!("");
    println!("THE SOFTWARE IS PROVIDED \"AS IS\", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR");
    println!("IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,");
    println!("FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE");
    println!("AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER");
    println!("LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,");
    println!("OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE");
    println!("SOFTWARE.");
    println!("");
}

fn main_interactive_interpreter() -> () {
    // prepare interactive parsing
    // first empty line is magic, used to avoid -1 pointers
    // output debug messages need to be checked for sanity (line + 1)
    let mut lines: Vec<String> = vec![String::default()];
    let mut state = State {
        lines: &mut lines,
        ptr: 0,
    };
    // prepare execution unit (this is modifed on interaction)
    let mut prog = Program {
        funcs: HashMap::new(),
    };
    let mut main_stmts = vec![];
    let mut scope = HashMap::new();
    // the next statement to execute exec_ptr[..]
    let mut exec_ptr = 0;
    // the last validated lines[..]
    let mut last_ptr = 0; 
    // start parsing
    let mut in_block = false;
    loop {
        // read input if possible
        let mut inp_line = String::new();
        print!("{}", if !in_block { ">>> " } else { "... " });
        std::io::stdout().flush().expect("unable to flush stdout");
        // reached EOF, gracefully exit
        if let Err(_) = std::io::stdin().read_line(&mut inp_line) {
            break;
        }
        inp_line = String::from(inp_line.trim());
        // additional information
        if inp_line == "copyright" {
            main_ii_show_copyright();
            continue;
        } else if inp_line == "license" {
            main_ii_show_license();
            continue;
        }
        // push and attempt to parse, check for errors
        while state.ptr > 0 && state.ptr > state.lines.len() {
            state.ptr -= 1;
        }
        state.lines.push(inp_line);
        match execute_block(
            &mut state,
            &mut prog,
            &mut scope,
            &mut main_stmts,
            &mut last_ptr,
            &mut exec_ptr,
        ) {
            Ok(()) => {
                in_block = false;
            }
            Err(Error::UnclosedBlock) => {
                in_block = true;
                state.ptr = last_ptr;
                continue;
            }
            Err(err) => {
                in_block = false;
                print!("{}", format_runtime_err(None, state.lines, &err, 0));
                while state.lines.len() > last_ptr + 1 {
                    state.lines.pop();
                }
                while main_stmts.len() > exec_ptr {
                    main_stmts.pop();
                }
                state.ptr = last_ptr;
                continue;
            }
        };
    }
    println!("\n");
    return;
}

fn main() {
    // read program from file
    let args: Vec<_> = env::args().collect();
    if args.len() <= 1 {
        println!("Nhotyp 0.1.0 (default, May 5 2021, 01:52:38)");
        println!("[rustc 1.50.0 (cb75ad5db 2021-02-10)] on linux");
        println!("Type \"copyright\" or \"license\" for more information.");
        main_interactive_interpreter();
    } else if args.len() == 2 {
        std::process::exit(main_run_file(&args[1]));
    } else {
        eprintln!("nhotyp: fatal error: too many arguments");
        eprintln!("intepretation terminated.");
        return;
    }
}
