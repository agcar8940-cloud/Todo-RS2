use dirs;
use rusqlite::{Connection, OptionalExtension, Result};
use std::env;
use std::io::{self, Write};
use std::path::Path;

#[derive(Debug)]
struct Task {
    id: i32,
    task: String,
    is_completed: bool,
}

#[derive(Debug)]
struct TaskStats {
    total: i32,
    completed: i32,
    percent: f64,
}

fn input() -> String {
    let mut x = String::new();
    io::stdin().read_line(&mut x).expect("Failed");
    x.trim().to_string()
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let db_path = if args.len() >= 3 && args[1] == "--db" {
        let raw_path = Path::new(&args[2]);
        if raw_path.is_dir() {
            raw_path.join("todors2.db")
        } else {
            raw_path.to_path_buf()
        }
    } else {
        let home_dir = dirs::home_dir().expect("Could not find home directory");
        home_dir.join(".todors2.db")
    };

    let conn = Connection::open(&db_path).expect("Failed to conect to database");

    conn.execute(
        "CREATE TABLE IF NOT EXISTS tasks (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                task TEXT NOT NULL,
                is_completed BOOLEAN NOT NULL DEFAULT 0
            )",
        [],
    )
    .expect("Failed to create table");

    println!(
        r###"
 _________  ________  ________  ________                 ________  ________    _______
|\___   ___\\   __  \|\   ___ \|\   __  \               |\   __  \|\   ____\  /  ___  \
\|___ \  \_\ \  \|\  \ \  \_|\ \ \  \|\  \  ____________\ \  \|\  \ \  \___|_/__/|_/  /|
     \ \  \ \ \  \\\  \ \  \ \\ \ \  \\\  \|\____________\ \   _  _\ \_____  \__|//  / /
      \ \  \ \ \  \\\  \ \  \_\\ \ \  \\\  \|____________|\ \  \\  \\|____|\  \  /  /_/__
       \ \__\ \ \_______\ \_______\ \_______\              \ \__\\ _\ ____\_\  \|\________\
        \|__|  \|_______|\|_______|\|_______|               \|__|\|__|\_________\\|_______|
                                                                     \|_________|
        "###
    );
    println!("Made by agcar8940-cloud on github");
    println!("V~1.5");
    println!("type <<help>> for commands list");
    'mainLoop: loop {
        print!("todo-rs2>> ");
        io::stdout().flush().expect("Failed");
        let inp = input();

        let inpp: &str = &inp;
        let inp_fs_wrd = inp.split_whitespace().next().unwrap_or("");
        let inp_oths = inp.splitn(2, ' ').nth(1).unwrap_or("").trim_start();

        if !inpp.contains(" ") {
            match inpp {
                "exit" => {
                    break 'mainLoop;
                }

                "stats" => match get_stats(&conn) {
                    Ok(stats) => {
                        println!("--- Statistics ---");
                        println!("Total Tasks:    {}", stats.total);
                        println!("Tasks Finished: {}", stats.completed);
                        println!("Completion:     {:.1}%", stats.percent);
                    }
                    Err(e) => println!("Error getting stats: {}", e),
                },

                "listall" => {
                    if let Err(e) = list_all(&conn) {
                        println!("Error listing tasks: {}", e);
                    }
                }

                "completeall" => match complete_all(&conn) {
                    Ok(_) => println!("Successfully completed all tasks"),
                    Err(e) => println!("Error completing all tasks: {}", e),
                },

                "uncompleteall" => match uncomplete_all(&conn) {
                    Ok(_) => println!("Successfully uncompleted all tasks"),
                    Err(e) => println!("Error uncompleting all tasks: {}", e),
                },

                "deleteall" => match delete_all(&conn) {
                    Ok(count) => println!("Successfully deleted {} tasks", count),
                    Err(e) => println!("Error clearing tasks: {}", e),
                },

                "dbdir" => {
                    println!("Path of Database: {}", &db_path.to_string_lossy());
                }

                "debug" => {
                    debug(&conn);
                }

                "help" => {
                    println!("

                        -- Todo-Rs2 Commands List --

                        use --db <<path>> to enter a custom path for the database

                        To make a task: make <<task>>

                        To list all tasks: listall

                        to delete all tasks: deleteall

                        to list by id: list <<id>>

                        to delete by id: delete <<id>>

                        to search for tasks by a keyword: search <<keyword(s)>>

                        to edit an already existing task: edit <<id>> <<new task (word or sentence)>>

                        to delete all tasks with a common keyword: delete-common <<keyword(s)>>

                        to mark a task as completed: complete <<id>>

                        to mark a task as uncompleted: uncomplete <<id>>

                        to mark all tasks as completed: completeall

                        to mark all tasks as uncompleted: uncompleteall

                        to complete all tasks containing a keyword: complete-common <<keyword(s)>>

                        to uncomplete all tasks containing a keyword: uncomplete-common <<keyword(s)>>

                        to get statistics: stats

                        to get path of the database file: dbdir

                        to exit: exit

                    ");
                }

                _ => {
                    continue 'mainLoop;
                }
            }
        } else {
            match inp_fs_wrd {
                "make" => {
                    make_user(&conn, inp_oths.to_string());
                }

                "list" => match inp_oths.trim().parse::<i32>() {
                    Ok(search_id) => match list_by_id(&conn, search_id) {
                        Ok(Some(found_task)) => {
                            println!("ID:   {}", found_task.id);
                            println!("Task: {}", found_task.task);
                        }
                        Ok(None) => {
                            println!("No task exists with ID: {}", search_id);
                        }
                        Err(e) => {
                            println!("Database error: {}", e);
                        }
                    },
                    Err(_) => {
                        println!(
                            "Error: '{}' is not a valid task ID number.",
                            inp_oths.trim()
                        );
                    }
                },

                "delete" => match inp_oths.trim().parse::<i32>() {
                    Ok(target_id) => match delete_by_id(&conn, target_id) {
                        Ok(0) => {
                            println!("No task exists with the id: {}", target_id);
                        }
                        Ok(_) => {
                            println!("Successfully deleted task with id: {}", target_id);
                        }
                        Err(e) => {
                            println!("Error while deleting task: {}", e);
                        }
                    },
                    Err(_) => {
                        println!("Error: {} is not a valid task id", inp_oths.trim());
                    }
                },

                "delete-common" => {
                    let keyword = inp_oths.trim();

                    if keyword.is_empty() {
                        println!("Error: Please provide a keyword to target for deletion.");
                    } else {
                        match delete_by_keyword(&conn, keyword) {
                            Ok(0) => {
                                println!("No tasks found containing the keyword: '{}'", keyword);
                            }
                            Ok(count) => {
                                println!(
                                    "Success! Permanently deleted {} task(s) containing '{}'.",
                                    count, keyword
                                );
                            }
                            Err(e) => {
                                println!("Database error while deleting: {}", e);
                            }
                        }
                    }
                }

                "complete-common" => {
                    let keyword = inp_oths.trim();

                    if keyword.is_empty() {
                        println!("Error: Please provide a keyword to target for completion.");
                    } else {
                        match complete_by_keyword(&conn, keyword) {
                            Ok(0) => {
                                println!("No tasks found containing the keyword: '{}'", keyword);
                            }
                            Ok(count) => {
                                println!(
                                    "Success! Completed {} task(s) containing '{}'.",
                                    count, keyword
                                );
                            }
                            Err(e) => {
                                println!("Database error while completing: {}", e);
                            }
                        }
                    }
                }

                "uncomplete-common" => {
                    let keyword = inp_oths.trim();

                    if keyword.is_empty() {
                        println!("Error: Please provide a keyword to target for uncompletion.");
                    } else {
                        match uncomplete_by_keyword(&conn, keyword) {
                            Ok(0) => {
                                println!("No tasks found containing the keyword: '{}'", keyword);
                            }
                            Ok(count) => {
                                println!(
                                    "Success! Uncompleted {} task(s) containing '{}'.",
                                    count, keyword
                                );
                            }
                            Err(e) => {
                                println!("Database error while uncompleting: {}", e);
                            }
                        }
                    }
                }

                "complete" => {
                    if let Ok(id) = inp_oths.trim().parse::<i32>() {
                        match complete_task(&conn, id) {
                            Ok(0) => println!("No task found with id: {}", id),
                            Ok(_) => println!("Successfully completed task with id: {}", id),
                            Err(e) => println!("Error while completing task: {}", e),
                        }
                    }
                }

                "uncomplete" => {
                    if let Ok(id) = inp_oths.trim().parse::<i32>() {
                        match uncomplete_task(&conn, id) {
                            Ok(0) => println!("No task found with id: {}", id),
                            Ok(_) => println!("Successfully uncompleted task with id: {}", id),
                            Err(e) => println!("Error while uncompleting task: {}", e),
                        }
                    }
                }

                "search" => {
                    let keyword = inp_oths.trim();

                    if keyword.is_empty() {
                        println!("Error: nothing cant be searched");
                    } else {
                        match list_by_keyword(&conn, keyword.to_string()) {
                            Ok(tasks) => {
                                if tasks.is_empty() {
                                    println!("No tasks found with keyword: {}", keyword);
                                } else {
                                    println!(
                                        "Found {} tasks containing keyword: {}",
                                        tasks.len(),
                                        keyword
                                    );
                                    for item in tasks {
                                        println!("Id: {}, Task: {}", item.id, item.task);
                                    }
                                }
                            }
                            Err(e) => {
                                println!("Error while searching: {}", e);
                            }
                        }
                    }
                }

                "edit" => {
                    let mut parts = inp_oths.splitn(2, ' ');
                    let id_str = parts.next().unwrap_or("");
                    let new_task = parts.next().unwrap_or("").trim();

                    if let Ok(id) = id_str.parse::<i32>() {
                        if !new_task.is_empty() {
                            match edit_task(&conn, id, new_task.to_string()) {
                                Ok(0) => println!("No task found with ID {}", id),
                                Ok(_) => {
                                    println!("Successfully updated task {} to: {}", id, new_task)
                                }
                                Err(e) => println!("Error updating: {}", e),
                            }
                        } else {
                            println!("Usage: edit <id> <new task text>");
                        }
                    } else {
                        println!("Invalid ID format. Use: edit <id> <new task>");
                    }
                }

                _ => (),
            }
        }
    }
}

fn make_user(conn: &Connection, task: String) -> Result<()> {
    conn.execute("INSERT INTO tasks (task) VALUES (?1)", (task.clone(),))?;
    println!("Successfully Added task: {}", task);
    Ok(())
}

fn debug(conn: &Connection) {
    let letters = ["a", "b", "c", "d", "e"];
    for i in 0..5 {
        make_user(conn, letters[i].to_string());
    }
}

fn list_all(conn: &Connection) -> Result<()> {
    let mut stmt = conn.prepare("SELECT id, task, is_completed FROM tasks")?;

    let task_iter = stmt.query_map([], |row| {
        Ok(Task {
            id: row.get(0)?,
            task: row.get(1)?,
            is_completed: row.get(2)?,
        })
    })?;

    println!("\n{:-<35}", "");
    println!("{:<5} | {:<20} | {:<8}", "ID", "Task", "Status");
    println!("{:-<35}", "");

    for task in task_iter {
        let t = task?;
        let status = if t.is_completed { "[X]" } else { "[ ]" };
        // {:<5} means left-align in a space of 5 characters
        println!("{:<5} | {:<20} | {:<8}", t.id, t.task, status);
    }
    println!("{:-<35}\n", "");

    Ok(())
}

fn delete_all(conn: &Connection) -> Result<usize> {
    let deleted_rows = conn.execute("DELETE FROM tasks", [])?;
    conn.execute("DELETE FROM sqlite_sequence WHERE name = 'tasks'", [])?;

    Ok(deleted_rows)
}

fn edit_task(conn: &Connection, id: i32, task: String) -> Result<usize> {
    let rows_updated = conn.execute("UPDATE tasks SET task = ?1 WHERE id = ?2", (&task, id))?;
    Ok(rows_updated)
}

fn list_by_id(conn: &Connection, id: i32) -> Result<Option<Task>> {
    let res = conn
        .query_row(
            "SELECT id, task, is_completed FROM tasks WHERE id = ?1",
            [id],
            |row| {
                Ok(Task {
                    id: row.get(0)?,
                    task: row.get(1)?,
                    is_completed: row.get(2)?,
                })
            },
        )
        .optional()?;

    Ok(res)
}

fn list_by_keyword(conn: &Connection, keyword: String) -> Result<Vec<Task>> {
    let mut stmt = conn.prepare("SELECT id, task, is_completed FROM tasks WHERE task LIKE ?1")?;

    let search_pattern = format!("%{}%", keyword);

    let task_iter = stmt.query_map([search_pattern], |row| {
        Ok(Task {
            id: row.get(0)?,
            task: row.get(1)?,
            is_completed: row.get(2)?,
        })
    })?;
    let mut tasks = Vec::new();

    for task in task_iter {
        tasks.push(task?);
    }

    Ok(tasks)
}

fn delete_by_id(conn: &Connection, id: i32) -> Result<usize> {
    let rows_deleted = conn.execute("DELETE FROM tasks WHERE id = ?1", [id])?;
    Ok(rows_deleted)
}

fn delete_by_keyword(conn: &Connection, keyword: &str) -> Result<usize> {
    let search_pattern = format!("%{}%", keyword);

    let rows_deleted = conn.execute("DELETE FROM tasks WHERE task LIKE ?1", [search_pattern])?;

    Ok(rows_deleted)
}

fn complete_task(conn: &Connection, id: i32) -> Result<usize> {
    conn.execute("UPDATE tasks SET is_completed = 1 WHERE id = ?1", [id])
}
fn uncomplete_task(conn: &Connection, id: i32) -> Result<usize> {
    conn.execute("UPDATE tasks SET is_completed = 0 WHERE id = ?1", [id])
}
fn complete_all(conn: &Connection) -> Result<usize> {
    conn.execute("UPDATE tasks SET is_completed = 1", [])
}

fn uncomplete_all(conn: &Connection) -> Result<usize> {
    conn.execute("UPDATE tasks SET is_completed = 0", [])
}

fn complete_by_keyword(conn: &Connection, keyword: &str) -> Result<usize> {
    let search_pattern = format!("%{}%", keyword);

    conn.execute(
        "UPDATE tasks SET is_completed = 1 WHERE task LIKE ?1",
        [search_pattern],
    )
}

fn uncomplete_by_keyword(conn: &Connection, keyword: &str) -> Result<usize> {
    let search_pattern = format!("%{}%", keyword);

    conn.execute(
        "UPDATE tasks SET is_completed = 0 WHERE task LIKE ?1",
        [search_pattern],
    )
}

fn get_stats(conn: &Connection) -> Result<TaskStats> {
    conn.query_row(
        "SELECT COUNT(*), SUM(is_completed), (CAST(SUM(is_completed) AS FLOAT) / COUNT(*)) * 100 FROM tasks",
        [],
        |row| {
            Ok(TaskStats {
                total: row.get(0).unwrap_or(0),
                completed: row.get(1).unwrap_or(0),
                percent: row.get(2).unwrap_or(0.0),
            })
        }
    )
}
