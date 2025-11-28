#[allow(unused_imports)]
use std::io::{self, Write};
use std::str::FromStr;

#[derive(Debug, PartialEq)]
enum Command {
    Exit,
    Echo,
    Type,
    Pwd,
    Cd,
}

#[derive(Debug, PartialEq)]
enum Redirect {
    OutputRedir,
    ErrRedir,
    OutputAppend,
    ErrAppend,
}

#[derive(Debug, PartialEq, Eq)]
struct ParseRedirectError;

impl FromStr for Redirect {
    type Err = ParseRedirectError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            ">" | "1>" => Ok(Self::OutputRedir),
            "2>" => Ok(Self::ErrRedir),
            ">>" | "1>>" => Ok(Self::OutputAppend),
            "2>>" => Ok(Self::ErrAppend),

            _ => Err(ParseRedirectError),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
struct ParseCommandError;

impl FromStr for Command {
    type Err = ParseCommandError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "exit" => Ok(Self::Exit),
            "echo" => Ok(Self::Echo),
            "type" => Ok(Self::Type),
            "pwd" => Ok(Self::Pwd),
            "cd" => Ok(Self::Cd),

            _ => Err(ParseCommandError),
        }
    }
}

use std::fmt;

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Command::Exit => write!(f, "exit"),
            Command::Echo => write!(f, "echo"),
            Command::Type => write!(f, "type"),
            Command::Pwd => write!(f, "pwd"),
            Command::Cd => write!(f, "cd"),
        }
    }
}

use regex::Regex;
use rustyline::completion::{Completer, Pair};
use rustyline::config::{CompletionType, Config};
use rustyline::error::ReadlineError;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::history::DefaultHistory;
use rustyline::validate::Validator;
use rustyline::Helper;
use rustyline::Result;
use rustyline::{Context, Editor};
use std::collections::VecDeque;
use std::fmt::Display;
use std::fs::File;
use std::fs::OpenOptions;

#[derive(Default)]
pub struct Node {
    pub children: Vec<Node>,
    pub key: Option<char>,
    pub value: Option<String>,
    pub count: usize,
}

impl Node {
    pub fn new() -> Self {
        Node {
            ..Default::default()
        }
    }

    pub fn with_key(c: char) -> Self {
        Node {
            key: Some(c),
            ..Default::default()
        }
    }
}

pub struct Trie {
    pub root: Node,
}

impl Trie {
    pub fn new() -> Self {
        Trie { root: Node::new() }
    }

    pub fn insert(&mut self, s: &str) {
        let mut cur = &mut self.root;
        for c in s.chars() {
            match cur.children.binary_search_by(|f| f.key.cmp(&Some(c))) {
                Ok(i) => {
                    cur = &mut cur.children[i];
                }
                Err(i) => {
                    cur.children.insert(i, Node::with_key(c));
                    cur = &mut cur.children[i];
                }
            }
        }

        cur.count += 1;
        cur.value.replace(s.to_string());
    }
    pub fn exists(&self, s: &str) -> bool {
        let mut cur = &self.root;
        for c in s.chars() {
            match cur.children.binary_search_by(|f| f.key.cmp(&Some(c))) {
                Ok(i) => {
                    cur = &cur.children[i];
                }
                Err(_) => {
                    return false;
                }
            }
        }

        cur.count > 0
    }
    pub fn search(&self, s: &str) -> Vec<String> {
        let mut cur = &self.root;
        for c in s.chars() {
            match cur.children.binary_search_by(|f| f.key.cmp(&Some(c))) {
                Ok(i) => {
                    cur = &cur.children[i];
                }
                Err(_) => {
                    return Vec::new();
                }
            }
        }

        let mut results = Vec::new();
        let mut q = Vec::new();
        q.push(cur);
        while let Some(c) = q.pop() {
            for child in c.children.iter() {
                q.push(&child);
            }

            if c.count > 0 {
                let value = c.value.as_ref().unwrap();
                let count = c.count;
                results.push((count, value));
            }
        }

        results.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(b.1)));
        results.iter().map(|m| m.1.clone()).collect()
    }
}

impl Display for Trie {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut q: VecDeque<&Node> = VecDeque::new();
        let root = &self.root;
        q.push_back(root);

        while !q.is_empty() {
            for _ in 0..q.len() {
                if let Some(node) = q.pop_front() {
                    for c in node.children.iter() {
                        let r = write!(f, "{} ", &c.key.unwrap());
                        if r.is_err() {
                            return r;
                        }

                        if c.children.len() > 0 {
                            q.push_back(&c);
                        }
                    }
                }
            }

            if q.len() > 0 {
                let r = writeln!(f);
                if r.is_err() {
                    return r;
                }
            }

        }
        Ok(())
    }
}

struct MyHelper {
    commands: Trie
}

impl Completer for MyHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> std::result::Result<(usize, Vec<Pair>), ReadlineError> {
        // Find the start of the current word
        let start = line[..pos]
            .rfind(|c: char| c.is_whitespace() || c == '(' || c == ')')
            .map_or(0, |i| i + 1);
        let word = &line[start..pos];

        let candidates: Vec<Pair> = self
            .commands
            .search(word)
            .iter()
            .map(|cmd| Pair {
                display: cmd.clone(),
                replacement: cmd.clone(),
            })
            .collect();

        Ok((start, candidates))
    }
}

impl Helper for MyHelper {}
impl Hinter for MyHelper {
    type Hint = String;

    fn hint(&self, _line: &str, _pos: usize, _ctx: &rustyline::Context<'_>) -> Option<String> {
        // Return Some("suggested text".to_string()) for actual hints, or None if you have no hints
        None
    }
}
impl Highlighter for MyHelper {
    fn highlight<'l>(&self, line: &'l str, _history_index: usize) -> std::borrow::Cow<'l, str> {
        std::borrow::Cow::Borrowed(line)
    }
}

impl Validator for MyHelper {
    fn validate(
        &self,
        _ctx: &mut rustyline::validate::ValidationContext,
    ) -> rustyline::Result<rustyline::validate::ValidationResult> {
        Ok(rustyline::validate::ValidationResult::Valid(None))
    }
}

fn main() -> Result<()> {
    let mut commands = Trie::new();
    commands.insert("exit");
    commands.insert("echo");
    commands.insert("type");
    commands.insert("pwd");
    commands.insert("cd");
    add_executable_files("PATH", &mut commands);
    // println!("{}", commands);
    let helper = MyHelper { commands };

    // `Editor` can use any struct that implements the `Helper` trait.
    // The type parameter <H: Helper> is set to `MyHelper`.
    let config = Config::builder()
        .tab_stop(2)
        .completion_type(CompletionType::List)
        .build();
    let mut rl: Editor<MyHelper, DefaultHistory> = Editor::with_config(config)?;
    rl.set_helper(Some(helper));

    if rl.load_history("history.txt").is_err() {
        println!("No previous history.");
    }

    let re = Regex::new(r#"("[/'\w\s\\:]+"|'[^']+'|[~/\.\-\w\\\d>]+)"#).unwrap();

    loop {
        let readline = rl.readline("$ ");
        match readline {
            Ok(input) => {
                rl.add_history_entry(input.as_str())?;
                let input = input.trim().replace("''", "").replace("\"\"", "");
                if input.is_empty() {
                    continue;
                }

                let mut args = vec![];
                for (_, [arg]) in re.captures_iter(&input).map(|c| c.extract()) {
                    let x: &[_] = &['\'', '"'];
                    let arg = arg.trim_matches(x);
                    args.push(arg);
                }

                // println!("args:{:?}", args);

                if let Err(err) = execution(&args) {
                    println!("Error: {:?}", err);
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
    let _ = rl.save_history("history.txt");
    Ok(())
}

use std::env;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

fn search_executable_file(filename: &str, key: &str) -> Option<PathBuf> {
    match env::var(key) {
        Ok(val) => {
            let dirs: Vec<&str> = val.split(':').collect();
            for dir in dirs {
                if let Ok(entries) = fs::read_dir(dir) {
                    for entry in entries {
                        if let Ok(entry) = entry {
                            if let Ok(file_type) = entry.file_type() {
                                if file_type.is_file() && entry.file_name() == filename {
                                    if let Ok(metadata) = entry.metadata() {
                                        if metadata.permissions().mode() & 0o100 != 0 {
                                            return Some(entry.path());
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            None
        }
        Err(e) => {
            println!("env var: {key} not found: {e}");
            None
        }
    }
}

fn redirection(
    args: &Vec<&str>,
) -> std::result::Result<Option<(Redirect, String, usize)>, ParseRedirectError> {
    for (i, arg) in args.iter().enumerate() {
        let redir = arg.parse::<Redirect>();
        if let Ok(redir) = redir {
            if i != args.len() - 1 {
                return Ok(Some((redir, String::from(args[i + 1]), i)));
            } else {
                return Err::<Option<(Redirect, String, usize)>, ParseRedirectError>(
                    ParseRedirectError,
                );
            }
        }
    }
    Ok(None)
}

fn execution(args: &Vec<&str>) -> Result<()> {
    let cmd = args[0].parse::<Command>();
    match cmd {
        Ok(cmd) => match cmd {
            Command::Exit => std::process::exit(0),
            Command::Echo => match redirection(args) {
                Ok(val) => match val {
                    Some((redir, file_name, i)) => {
                        let mut file;
                        match redir {
                            Redirect::OutputRedir | Redirect::ErrRedir => {
                                file = File::create(file_name)?
                            }
                            Redirect::OutputAppend | Redirect::ErrAppend => {
                                file = OpenOptions::new()
                                    .append(true)
                                    .write(true)
                                    .create(true)
                                    .open(file_name)?
                            }
                        }
                        let mut index = 1;

                        while index < i {
                            if let Redirect::OutputRedir | Redirect::OutputAppend = redir {
                                file.write_all(args[index].as_bytes())?;
                                file.write_all(b" ")?;
                            } else {
                                io::stdout().write_all(args[index].as_bytes())?;
                                io::stdout().write_all(b" ")?;
                            }
                            index += 1;
                        }
                        if let Redirect::OutputRedir | Redirect::OutputAppend = redir {
                            file.write_all(b"\n")?;
                        } else {
                            io::stdout().write_all(b"\n")?;
                        }
                    }
                    None => {
                        let mut index = 1;
                        while index < args.len() {
                            print!("{}", args[index]);
                            if index != args.len() - 1 {
                                print!(" ")
                            }
                            index += 1;
                        }
                        println!();
                    }
                },
                Err(_) => println!("missing filename"),
            },
            Command::Pwd => {
                println!("{}", env::current_dir()?.display());
            }
            Command::Cd => {
                if args.len() > 1 {
                    let parsed_directory = args[1].replace("~", &std::env::var("HOME").unwrap());
                    if let Err(_) = env::set_current_dir(parsed_directory) {
                        println!("cd: {}: No such file or directory", args[1])
                    }
                }
            }
            Command::Type => {
                if args.len() > 1 {
                    for i in 1..args.len() {
                        let cmd = args[i].parse::<Command>();
                        match cmd {
                            Ok(cmd) => println!("{} is a shell builtin", cmd),
                            Err(_) => match search_executable_file(args[i], "PATH") {
                                Some(path) => println!("{} is {}", args[i], path.display()),
                                None => println!("{}: not found", args[i]),
                            },
                        }
                    }
                }
            }
        },
        Err(_) => match search_executable_file(args[0], "PATH") {
            Some(path) => match redirection(args) {
                Ok(val) => match val {
                    Some((redir, file_name, i)) => {
                        let mut cmd = std::process::Command::new(
                            path.file_name().unwrap_or(path.as_os_str()),
                        );
                        let mut file;
                        match redir {
                            Redirect::OutputRedir | Redirect::ErrRedir => {
                                file = File::create(file_name)?
                            }
                            Redirect::OutputAppend | Redirect::ErrAppend => {
                                file = OpenOptions::new()
                                    .append(true)
                                    .write(true)
                                    .create(true)
                                    .open(file_name)?
                            }
                        }
                        let mut index = 1;
                        while index < i {
                            cmd.arg(args[index]);
                            index += 1;
                        }
                        let output = cmd.output()?;
                        if let Redirect::OutputRedir | Redirect::OutputAppend = redir {
                            file.write_all(&output.stdout)?;
                            io::stderr().write_all(&output.stderr)?;
                        } else {
                            io::stdout().write_all(&output.stdout)?;
                            file.write_all(&output.stderr)?;
                        }
                    }
                    None => {
                        let mut cmd = std::process::Command::new(
                            path.file_name().unwrap_or(path.as_os_str()),
                        );
                        let mut index = 1;
                        while index < args.len() {
                            cmd.arg(args[index]);
                            index += 1;
                        }
                        let output = cmd.output()?;
                        io::stdout().write_all(&output.stdout)?;
                        io::stderr().write_all(&output.stderr)?;
                    }
                },
                Err(_) => println!("missing file name"),
            },
            None => println!("{}: not found", args[0]),
        },
    }

    Ok(())
}

fn add_executable_files(key: &str, commands: &mut Trie) {
    match env::var(key) {
        Ok(val) => {
            let dirs: Vec<&str> = val.split(':').collect();
            for dir in dirs {
                if let Ok(entries) = fs::read_dir(dir) {
                    for entry in entries {
                        if let Ok(entry) = entry {
                            if let Ok(file_type) = entry.file_type() {
                                if file_type.is_file() {
                                    if let Ok(metadata) = entry.metadata() {
                                        if metadata.permissions().mode() & 0o100 != 0 {
                                            match entry.file_name().into_string() {
                                                Ok(mut filename) => {
                                                    filename.push(' ');
                                                    commands.insert(filename.as_str());
                                                }
                                                Err(os_string) => println!(
                                                    "couldn't convert {:?} to String",
                                                    os_string
                                                ),
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        Err(e) => {
            println!("{key} not found: {e}");
        }
    }
}
