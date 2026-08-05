use chrono::{Date, DateTime, Duration, NaiveDate, NaiveDateTime, Utc};
use std::fmt::{self, write};
use std::io::{self, Read, Write};
enum StatusTodolist {
    Ongoing,
    Done,
}

impl fmt::Display for StatusTodolist {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            StatusTodolist::Ongoing => write!(f, "Ongoing"),
            StatusTodolist::Done => write!(f, "Done"),
        }
    }
}
struct TodoList {
    name: String,
    status: StatusTodolist,
    date_started: Option<DateTime<Utc>>,
    due_date: Option<DateTime<Utc>>,
}

impl fmt::Display for TodoList {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "===Todo====")?;
        writeln!(f, "Nama = {}", self.name)?;
        writeln!(f, "Status = {}", self.status)?;

        match self.date_started {
            Some(Hasil) => writeln!(f, "Started at = {}", Hasil)?,
            None => writeln!(f, "Started at = -")?,
        };

        match self.due_date {
            Some(Hasil2) => write!(f, "Due date = {}", Hasil2),
            None => write!(f, "Due date = -"),
        }
    }
}

impl TodoList {
    fn NewTodo(NameTodo: String) -> TodoList {
        TodoList {
            name: NameTodo,
            status: StatusTodolist::Ongoing,
            date_started: Some(Utc::now()),
            due_date: None,
        }
    }

    fn EditNameTodo(&mut self, NameTodo: String) {
        self.name = NameTodo;
    }

    fn FilDue_Date(&mut self, due_date: DateTime<Utc>) {
        self.due_date = Some(due_date);
    }

    fn setTodoOngoing(&mut self) {
        self.status = StatusTodolist::Ongoing;
    }

    fn setTodoDone(&mut self) {
        self.status = StatusTodolist::Done;
    }

    fn printTodo(&self) {
        println!("{}", self);
    }
}

struct V {
    todos: Vec<TodoList>,
}

impl V {
    fn CreateTodoVec() -> Self {
        Self { todos: Vec::new() }
    }

    fn NewTodo(&mut self, nama: String) {
        if nama.trim().is_empty() {
            println!("Nama Kosong");
            return;
        }

        let mut Todo = TodoList::NewTodo(nama);
        self.todos.push(Todo);
    }

    fn NewTodo_withDuedate(&mut self, nama: String, due_date: DateTime<Utc>) {
        if nama.trim().is_empty() {
            println!("Nama Kosong");
            return;
        }

        let now = Utc::now();
        if due_date < now {
            return;
        }

        let mut Todo = TodoList::NewTodo(nama);
        Todo.due_date = Some(due_date);
        self.todos.push(Todo);
    }

    fn Print_allTodo(&self) {
        for i in 0..self.todos.len() {
            println!("");
            let temp: String = self.todos[i].to_string();
            println!("Hasil {}", temp);
            println!("{}", self.todos[i]);
        }
    }
}

fn main() {
    let date = Utc::now();

    let mut ve: V = V::CreateTodoVec();
    loop {
        print!("Masukan nama Todo : ");
        io::stdout().flush().unwrap();

        let mut nameTodo = String::new();
        io::stdin().read_line(&mut nameTodo).expect("Gagal");

        print!("Masukan due date? :");
        io::stdout().flush().unwrap();
        let mut handler_duedate = String::new();

        io::stdin().read_line(&mut handler_duedate).expect("Gagal");
        match handler_duedate.trim() {
            "y" | "Y" => {
                print!("Masukan format (YYYY-MM-DD HH:MM): ");
                io::stdout().flush().unwrap();

                let mut due_date = String::new();
                io::stdin().read_line(&mut due_date).expect("Gagal");

                match NaiveDateTime::parse_from_str(due_date.trim(), "%Y-%m-%d %H:%M") {
                    Ok(hasil) => {
                        let fix = DateTime::<Utc>::from_naive_utc_and_offset(hasil, Utc);
                        ve.NewTodo_withDuedate(nameTodo.trim().to_string(), fix);
                    }
                    Err(_) => {
                        println!("Gagal Parsing");
                        break;
                    }
                }
            }
            "n" | "N" => {
                ve.NewTodo(nameTodo.trim().to_string());
            }
            _ => {
                println!("Err program berhenti");
                break;
            }
        }

        print!("Lanjut ? :");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Gagal");

        match input.trim() {
            "y" | "Y" => continue,
            "n" | "N" => break,
            _ => {
                println!("Err program berhenti");
                break;
            }
        }
    }

    ve.Print_allTodo();
}
