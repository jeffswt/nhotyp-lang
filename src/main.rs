use std::collections::HashMap;
use std::env;
use std::error::Error as StdError;
use std::fmt;
use std::fs;
use std::ops;

///////////////////////////////////////////////////////////////////////////////
/// Error handling

#[derive(PartialEq, Eq)]
enum Error {
    IllegalChar { line: usize, value: char },
    TokenTooLong { line: usize, value: usize },
    UnknownExpr { line: usize, value: String },
    MalformedAssign { line: usize },
    MalformedCond { line: usize },
    MalformedLoop { line: usize },
    MalformedRet { line: usize },
    MalformedFunc { line: usize },
    MalformedEnd { line: usize },
    DuplicateToken { line: usize, value: String },
    WildStatement { line: usize },
    WildFunction { line: usize },
    MissingMain,
    MisplacedRet { line: usize },
    UndeclaredVar { line: usize, value: String },
}

impl Error {
    pub fn debug(&self) -> String {
        match self {
            Self::IllegalChar { line, value } => format!("IllegalChar({}, {:?})", line, value),
            Self::TokenTooLong { line, value } => format!("TokenTooLong({}, {})", line, value),
            Self::UnknownExpr { line, value } => format!("UnknownExpr({}, {:?})", line, value),
            Self::MalformedAssign { line } => format!("MalformedAssign({})", line),
            Self::MalformedCond { line } => format!("MalformedCond({})", line),
            Self::MalformedLoop { line } => format!("MalformedLoop({})", line),
            Self::MalformedRet { line } => format!("MalformedRet({})", line),
            Self::MalformedFunc { line } => format!("MalformedFunc({})", line),
            Self::MalformedEnd { line } => format!("MalformedEnd({})", line),
            Self::DuplicateToken { line, value } => {
                format!("DuplicateToken({}, {:?})", line, value)
            }
            Self::WildStatement { line } => format!("WildStatement({})", line),
            Self::WildFunction { line } => format!("WildFunction({})", line),
            Self::MissingMain => format!("MissingMain"),
            Self::MisplacedRet { line } => format!("MisplacedRet({})", line),
            Self::UndeclaredVar { line, value } => format!("UndeclaredVar({}, {})", line, value),
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
            Self::UnknownExpr { value, .. } => {
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
            Self::DuplicateToken { value, .. } => {
                format!("conflict token {:?}", value)
            }
            Self::WildStatement { .. } => {
                format!("statements should appear in functions")
            }
            Self::WildFunction { .. } => {
                format!("function should not appear in functions")
            }
            Self::MissingMain { .. } => format!("missing main function"),
            Self::MisplacedRet { .. } => {
                format!("always return at end of function")
            }
            Self::UndeclaredVar { value, .. } => {
                format!("variable {:?} undeclared", value)
            }
        }
    }

    pub fn line(&self) -> usize {
        match self {
            Self::IllegalChar { line, .. } => *line,
            Self::TokenTooLong { line, .. } => *line,
            Self::UnknownExpr { line, .. } => *line,
            Self::MalformedAssign { line, .. } => *line,
            Self::MalformedCond { line, .. } => *line,
            Self::MalformedLoop { line, .. } => *line,
            Self::MalformedRet { line, .. } => *line,
            Self::MalformedFunc { line, .. } => *line,
            Self::MalformedEnd { line, .. } => *line,
            Self::DuplicateToken { line, .. } => *line,
            Self::WildStatement { line, .. } => *line,
            Self::WildFunction { line, .. } => *line,
            Self::MissingMain => 0,
            Self::MisplacedRet { line, .. } => *line,
            Self::UndeclaredVar { line, .. } => *line,
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

    pub fn from_any(state: &mut State, s: &str) -> Result<Self, Error> {
        Self::from(s, state.ptr, true)
    }

    pub fn from_var(state: &mut State, s: &str) -> Result<Self, Error> {
        Self::from(s, state.ptr, false)
    }
}

impl Clone for Token {
    fn clone(&self) -> Self {
        Self {
            value: String::from(&self.value),
        }
    }
}

struct Expr {
    tokens: Vec<Token>,
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

struct Node {
    stmts: Vec<Statement>,
}

struct State<'a> {
    lines: Vec<&'a str>,
    ptr: usize,
}

type StmtParseResult = Result<Statement, Error>;

fn parse_stmt_assign(state: &mut State, words: &Vec<&str>) -> StmtParseResult {
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
    Ok(Statement::Assign {
        var,
        expr: Expr { tokens },
        line: state.ptr,
    })
}

fn parse_stmt_cond(state: &mut State, words: &Vec<&str>) -> StmtParseResult {
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
    Ok(Statement::Cond {
        expr: Expr { tokens },
        child: parse_node(state, "if")?,
        line: state.ptr,
    })
}

fn parse_stmt_loop(state: &mut State, words: &Vec<&str>) -> StmtParseResult {
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
    Ok(Statement::Loop {
        expr: Expr { tokens },
        child: parse_node(state, "while")?,
        line: state.ptr,
    })
}

fn parse_stmt_print(state: &mut State, words: &Vec<&str>) -> StmtParseResult {
    // print <var1> <var2> ... <varn>
    // allows 0 variables
    let mut vars = vec![];
    for i in 1..words.len() {
        vars.push(Token::from_var(state, words[i])?);
    }
    Ok(Statement::Print {
        vars,
        line: state.ptr,
    })
}

fn parse_stmt_ret(state: &mut State, words: &Vec<&str>) -> StmtParseResult {
    // return <expression>
    let len = words.len();
    if len < 2 {
        return Err(Error::MalformedRet { line: state.ptr });
    }
    let mut tokens = vec![];
    for i in 1..len {
        tokens.push(Token::from_any(state, words[i])?);
    }
    Ok(Statement::Ret {
        expr: Expr { tokens },
        line: state.ptr,
    })
}

fn parse_stmt_func(state: &mut State, words: &Vec<&str>) -> StmtParseResult {
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
    Ok(Statement::Func {
        name,
        params,
        child: parse_node(state, "function")?,
        line: state.ptr,
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
        _ => Err(Error::UnknownExpr {
            line: state.ptr,
            value: String::from(words[0].to_string()),
        }),
    }
}

fn parse_node(state: &mut State, term: &str) -> Result<Node, Error> {
    let mut stmts = vec![];
    // splitting words here to check for terminations
    while state.ptr < state.lines.len() {
        // eradicate comments
        let mut line = String::from(state.lines[state.ptr]);
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
                break;
            }
            return Err(Error::MalformedEnd { line: state.ptr });
        }
        // send statement to corresponding parser
        stmts.push(parse_stmt(state, &words)?);
    }
    // done node parsing
    Ok(Node { stmts })
}

///////////////////////////////////////////////////////////////////////////////
/// Variables

const VARIABLE_LIMIT: i128 = 0x1_0000_0000_0000;

#[derive(Eq, PartialEq, Ord, PartialOrd, Clone)]
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
        Self::from((self % other).data / b)
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

///////////////////////////////////////////////////////////////////////////////
/// Program execution

struct Function {
    name: Token,
    params: Vec<Token>,
    root: Node,
    line: usize,
}

struct Program {
    funcs: HashMap<Token, Function>,
}

struct RunInstance<'a> {
    prog: &'a Program,
    func: &'a Function,
    scope: HashMap<Token, Variable>,
}

fn eval_expr(instance: &mut RunInstance, expr: &Expr) -> Result<Variable, Error> {
    Ok(Variable::from(0))
}

fn exec_statement(instance: &mut RunInstance, stmt: &Statement) -> Result<(), Error> {
    println!("on statement line:{}", stmt.line());
    match &stmt {
        &Statement::Assign { var, expr, .. } => {
            println!("assigning");
            let res = eval_expr(instance, &expr)?;
            instance.scope.insert(var.clone(), res);
        }
        &Statement::Cond { expr, child, .. } => {
            let cond = eval_expr(instance, &expr)?;
            if cond.data != 0 {
                exec_node(instance, &child)?;
            }
        }
        &Statement::Loop { expr, child, .. } => loop {
            let cond = eval_expr(instance, &expr)?;
            if cond.data == 0 {
                break;
            }
            exec_node(instance, &child)?;
        },
        &Statement::Print { vars, line } => {
            for var in vars {
                if !instance.scope.contains_key(var) {
                    return Err(Error::UndeclaredVar {
                        line: *line,
                        value: String::from(&var.value),
                    });
                }
                let val = &instance.scope[&var];
                println!("> {}", val.data);
            }
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
    func: &Function,
    params: Vec<&Variable>,
) -> Result<Variable, Error> {
    // generate instance
    let scope = HashMap::new();
    let mut instance = RunInstance { prog, func, scope };
    // put parameters into scope
    for i in 0..params.len() {
        let key = func.params[i].clone();
        let value = params[i].clone();
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
        Statement::Ret { expr, .. } => eval_expr(&mut instance, &expr),
        _ => Err(Error::MisplacedRet { line: func.line }),
    }
}

fn run_program(content: &str) -> Result<i64, Error> {
    // parse file for functions
    let mut state = State {
        lines: content.split('\n').collect(),
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
            if prog.funcs.contains_key(&name) {
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
                    name,
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
    let main_func = if let Some(v) = prog.funcs.get(&main_token) {
        v
    } else {
        return Err(Error::MissingMain);
    };
    Ok(call_function(&prog, main_func, vec![])?.data as i64)
}

fn main() {
    // read program from file
    let args: Vec<_> = env::args().collect();
    if args.len() < 2 {
        eprintln!("nhotyp: fatal error: no input files");
        eprintln!("intepretation terminated.");
        return;
    }
    let filename = &args[1];
    let content;
    match fs::read_to_string(&filename) {
        Ok(v) => content = v,
        Err(_) => {
            eprintln!("nhotyp: fatal error: {}: cannot read file", &filename);
            eprintln!("nhotyp: fatal error: no input files");
            eprintln!("interpretation terminated.");
            return;
        }
    }
    // parse and execute
    match run_program(&content) {
        Ok(v) => std::process::exit((v & 0xffffffffi64) as i32),
        Err(err) => eprintln!("{}:{}: error: {}", &args[1], err.line(), err.format()),
    }
}
