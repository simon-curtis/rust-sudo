use std::env;
use std::io::Write;
use std::process;

mod win;

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut iter = args.iter().peekable();
    iter.next();

    let process_id = process::id().to_string();

    let mut file_writer = std::fs::OpenOptions::new()
        .append(true)
        .open("log.txt")
        .unwrap();

    file_writer
        .write(format!("{} args: {:?}\n", process_id, args).as_bytes())
        .unwrap();

    while let Some(current) = iter.next() {
        match current.as_str() {
            "sudo" => match unsafe { win::is_admin() } {
                Ok(true) => println!("You are already admin"),
                Ok(false) => unsafe {
                    let file_name = env::current_exe().unwrap().display().to_string();
                    let working_dir = env::current_dir().unwrap().display().to_string();

                    let mut new_args = iter.map(|x| x.clone()).collect::<Vec<String>>();
                    new_args.insert(0, "--bind-console".to_string());
                    new_args.insert(1, process_id.to_string());

                    println!("Here, we will start the new process as admin using UAC prompt");
                    println!(" > exe: {0}", file_name);
                    println!(" > args: {0}", new_args.join(" "));
                    println!(" > in: {0}", working_dir);

                    // close file_writer
                    drop(file_writer);

                    match win::start_admin_instance(file_name, new_args, working_dir) {
                        Ok(()) => process::exit(0),
                        Err(err) => {
                            println!("Failed to start process: {}", err);
                            process::exit(1);
                        }
                    }
                },
                Err(err) => {
                    println!("Failed to check if admin: {}", err);
                    process::exit(1);
                }
            },
            "--bind-console" => match iter.peek() {
                None => {
                    println!("'process_id' is required for --bind-console");
                    std::io::stdin().read_line(&mut String::new()).unwrap();
                }
                Some(next) if next.starts_with('-') => {
                    println!("'process_id' is required for --bind-console");
                    std::io::stdin().read_line(&mut String::new()).unwrap();
                }
                Some(next) => match next.parse::<u32>() {
                    Ok(process_id) => match unsafe { win::bind_console(process_id) } {
                        Ok(()) => {
                            iter.next();
                            println!("Console bound to process {}", process_id);
                            if iter.len() > 0 {
                                println!(
                                    "Remaining args: {:?}",
                                    iter.clone().collect::<Vec<&String>>()
                                );
                            }
                        }
                        Err(err) => {
                            println!("Failed to bind console: {}", err);
                            std::io::stdin().read_line(&mut String::new()).unwrap();
                        }
                    },
                    Err(err) => {
                        println!("couldn't parse 'process_id' '{next}' to int: {err}");
                        std::io::stdin().read_line(&mut String::new()).unwrap();
                    }
                },
            },
            _ => continue,
        };
    }

    println!("End process");
}
