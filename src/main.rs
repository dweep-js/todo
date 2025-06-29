use serde::{Deserialize, Serialize}; // For serializing/deserializing data
use serde_json::{from_reader, to_writer_pretty}; // For JSON operations
use std::fs::{self, File}; // For file system operations
use std::io::{self, BufReader, BufWriter, Write}; // For buffered I/O, and `Write` trait for stdout flush
use std::path::PathBuf; // For constructing file paths
use colored::Colorize; // For colored terminal output
use std::process; // To explicitly exit the process

// --- Constants ---
const DATA_DIR: &str = "data";
const FILE_NAME: &str = "todos.json";

// We'll use a consistent padding for "centering" the content visually.
// True terminal centering is complex and varies by terminal.
const PADDING: &str = "    "; // 4 spaces for left padding

// ASCII Art Banner for a better CLI UI with colors and emojis
const BANNER: &str = r#"
         .--------'         .-.                    
(_)   /  .--.    .-(_) )-.      .--.    .- 
     /  /    )`-'     /   \    /    )`-'   
    /  /    /        /     \  /    /       
 .-/._(    /      .-/.      )(    /        
(_/  `-`-.'      (_/  `----'  `-.'         

      📝  Rust CLI Todo App ✅
"#;

// --- Structs ---

// Todo struct represents a single to-do item.
// `Deserialize` and `Serialize` are derived for JSON conversion.
// `Debug` allows easy printing with `{:?}`.
// `Clone` allows copying Todo instances, which is useful for filtering/modifying lists.
#[derive(Deserialize, Serialize, Debug, Clone)]
struct Todo {
    id: u32,
    // Use serde rename to map PascalCase JSON keys to snake_case Rust fields
    #[serde(rename = "Task")]
    task: String,
    #[serde(rename = "Completed")]
    completed: bool,
}

// TodoList struct manages a collection of Todo items.
#[derive(Deserialize, Serialize, Debug)]
struct TodoList {
    todos: Vec<Todo>, // Vector to store all todo items
    next_id: u32,     // Counter to assign unique IDs to new tasks
}

// --- Implementations for TodoList ---

impl TodoList {
    // `new` attempts to load todos from the JSON file.
    // If the file doesn't exist or is invalid, it initializes an empty list.
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // Construct the full path to the data file.
        let mut path = PathBuf::from(DATA_DIR);
        path.push(FILE_NAME);

        // Check if the file exists.
        if path.exists() {
            let file = File::open(&path)?; // Open the file
            let reader = BufReader::new(file); // Create a buffered reader

            // Attempt to deserialize the JSON content into a TodoList.
            // Handle potential JSON parsing errors.
            match from_reader(reader) {
                Ok(list) => Ok(list),
                Err(e) => {
                    eprintln!(
                        "{}{} {}",
                        PADDING,
                        format!("Warning: Could not parse existing data file. Starting fresh. Error: {}", e)
                            .yellow(),
                        "🚨".yellow()
                    );
                    Ok(TodoList {
                        todos: Vec::new(),
                        next_id: 1,
                    })
                }
            }
        } else {
            // If the file doesn't exist, create the data directory (if it doesn't exist)
            // and return an empty TodoList.
            fs::create_dir_all(DATA_DIR)?;
            Ok(TodoList {
                todos: Vec::new(),
                next_id: 1,
            })
        }
    }

    // `save` writes the current state of the TodoList to the JSON file.
    fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut path = PathBuf::from(DATA_DIR);
        path.push(FILE_NAME);

        // Create the parent directory if it doesn't exist.
        fs::create_dir_all(DATA_DIR)?;

        // Create/truncate the file.
        let file = File::create(&path)?;
        let writer = BufWriter::new(file);

        // Serialize the TodoList to JSON with pretty printing.
        to_writer_pretty(writer, &self)?;
        Ok(())
    }

    // `add_todo` creates a new Todo item and adds it to the list.
    fn add_todo(&mut self, task: String) {
        let new_todo = Todo {
            id: self.next_id,
            task: task.trim().to_string(), // Trim whitespace from task input
            completed: false,
        };
        self.todos.push(new_todo);
        self.next_id += 1; // Increment the next ID for future tasks
        println!(
            "{}{} {}",
            PADDING,
            format!("Added task (ID: {}): \"{}\"", self.next_id - 1, self.todos.last().unwrap().task)
                .green(),
            "➕"
        );
    }

    // `remove_todo` removes a task by its ID.
    fn remove_todo(&mut self, id_to_remove: u32) {
        // Find the index of the todo with the given ID.
        let initial_len = self.todos.len();
        self.todos.retain(|todo| todo.id != id_to_remove);

        // Check if a task was actually removed.
        if self.todos.len() < initial_len {
            println!("{}{} {}", PADDING, format!("Removed task with ID: {}", id_to_remove).red(), "🗑️");
        } else {
            eprintln!("{}{} {}", PADDING, format!("Error: Task with ID {} not found.", id_to_remove).red(), "❌");
        }
    }

    // `mark_completed` marks a task as completed by its ID.
    fn mark_completed(&mut self, id_to_complete: u32) {
        let mut found = false;
        // Iterate through todos and find the one to update.
        for todo in &mut self.todos {
            if todo.id == id_to_complete {
                if todo.completed {
                    println!(
                        "{}{} {}",
                        PADDING,
                        format!("Task with ID {} is already completed.", id_to_complete)
                            .yellow(),
                        "⚠️"
                    );
                } else {
                    todo.completed = true;
                    println!(
                        "{}{} {}",
                        PADDING,
                        format!("Marked task (ID: {}) as completed: \"{}\"", id_to_complete, todo.task)
                            .blue(),
                        "✔️"
                    );
                }
                found = true;
                break;
            }
        }
        if !found {
            eprintln!("{}{} {}", PADDING, format!("Error: Task with ID {} not found.", id_to_complete).red(), "❌");
        }
    }

    // `list_todos` prints all tasks in the list.
    fn list_todos(&self) {
        if self.todos.is_empty() {
            println!("{}{} {}", PADDING, "Your todo list is empty. Add some tasks!".cyan(), "✨");
            return;
        }

        println!("\n{}{}", PADDING, "--- Your To-Do List ---".purple().bold());
        println!("{}{}"," ", "-----------------------".purple().bold()); // Visual separator
        for todo in &self.todos {
            let status = if todo.completed {
                "[x]".green()
            } else {
                "[ ]".white()
            };
            println!(
                "{} {} {}. {}",
                PADDING,
                status,
                todo.id.to_string().cyan(), // Colorize ID
                todo.task
            );
        }
        println!("{}{}"," ", "-----------------------".purple().bold()); // Visual separator
    }

    // `count_stats` calculates and prints the number of completed and pending tasks.
    fn count_stats(&self) {
        let completed_count = self.todos.iter().filter(|todo| todo.completed).count();
        let total_count = self.todos.len();
        let pending_count = total_count - completed_count;

        println!("\n{}{}", PADDING, "--- To-Do Statistics ---".purple().bold());
        println!("{}{}"," ", "------------------------".purple().bold()); // Visual separator
        println!(
            "{}{}: {}",
            PADDING,
            "Total tasks".yellow(),
            total_count.to_string().cyan()
        );
        println!(
            "{}{}: {}",
            PADDING,
            "Completed tasks".green(),
            completed_count.to_string().cyan()
        );
        println!(
            "{}{}: {}",
            PADDING,
            "Pending tasks".red(),
            pending_count.to_string().cyan()
        );
        println!("{}{}"," ", "------------------------".purple().bold()); // Visual separator
    }
}

// --- Main Function (Interactive Loop) ---
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Print the banner once at the start, making it more vibrant
    println!("{}", BANNER.bright_magenta().bold());

    let mut todo_list = TodoList::new()?;

    // Main interactive loop
    loop {
        println!("\n{}{}", PADDING, "--- Menu ---".purple().bold());
        println!("{}{} {}", PADDING, "1. Add a new task".yellow(), "➕");
        println!("{}{} {}", PADDING, "2. List all tasks".cyan(), "📋");
        println!("{}{} {}", PADDING, "3. Mark task as completed".blue(), "✔️");
        println!("{}{} {}", PADDING, "4. Remove a task".red(), "🗑️");
        println!("{}{} {}", PADDING, "5. Show statistics".green(), "📊");
        println!("{}{} {}", PADDING, "6. Exit".magenta(), "👋");
        println!("{}{}"," ", "------------".purple().bold());

        print!("{}{} ", PADDING, "Enter your choice: ".white().bold());
        // Flush stdout to ensure the prompt is displayed immediately
        io::stdout().flush()?;

        let mut choice = String::new();
        io::stdin().read_line(&mut choice)?; // Read user input
        let choice = choice.trim().to_lowercase(); // Trim whitespace and convert to lowercase for easier matching

        match choice.as_str() {
            "1" | "add" => {
                print!("{}{} ", PADDING, "Enter task description: ".white().bold());
                io::stdout().flush()?;
                let mut task = String::new();
                io::stdin().read_line(&mut task)?;
                todo_list.add_todo(task);
            }
            "2" | "list" => {
                todo_list.list_todos();
            }
            "3" | "complete" => {
                print!("{}{} ", PADDING, "Enter ID of task to complete: ".white().bold());
                io::stdout().flush()?;
                let mut id_str = String::new();
                io::stdin().read_line(&mut id_str)?;
                match id_str.trim().parse::<u32>() {
                    Ok(id) => todo_list.mark_completed(id),
                    Err(_) => eprintln!("{}{} {}", PADDING, "Invalid ID. Please enter a number.".red(), "❌"),
                }
            }
            "4" | "remove" => {
                print!("{}{} ", PADDING, "Enter ID of task to remove: ".white().bold());
                io::stdout().flush()?;
                let mut id_str = String::new();
                io::stdin().read_line(&mut id_str)?;
                match id_str.trim().parse::<u32>() {
                    Ok(id) => todo_list.remove_todo(id),
                    Err(_) => eprintln!("{}{} {}", PADDING, "Invalid ID. Please enter a number.".red(), "❌"),
                }
            }
            "5" | "stats" => {
                todo_list.count_stats();
            }
            "6" | "exit" => {
                println!("{}{} {}", PADDING, "Exiting To-Do App. Goodbye!".magenta(), "👋");
                todo_list.save()?; // Save before exiting
                process::exit(0); // Explicitly exit the process
            }
            _ => {
                eprintln!("{}{} {}", PADDING, "Invalid choice. Please try again.".red(), "❓");
            }
        }

        // Save after each successful operation (or attempted operation)
        // This ensures persistence even if the app crashes before explicit exit.
        todo_list.save()?;
    }
}
