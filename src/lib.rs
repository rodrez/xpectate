use pyo3::prelude::*;
use pyo3::{wrap_pyfunction, PyResult};
use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher, EventKind, Event};
use std::path::Path;
use std::process::{Child, Command};
use std::sync::mpsc::channel;
use std::time::{Duration, Instant};


#[doc = r"
    Convert an `Event` to a tuple containing the event kind and the path that changed.

    Arguments:
        event (Event): The event to convert.

    Returns:
        Tuple[str, str]: A tuple containing the event kind and the path that changed.
"]
fn event_to_tuple(event: &Event) -> (String, String) {
    // We get a string representation of the event kind
    let kind = match &event.kind {
        EventKind::Access(_) => "Access",
        EventKind::Create(_) => "Create",
        EventKind::Modify(_) => "Modify",
        EventKind::Remove(_) => "Remove",
        EventKind::Other => "Other",
        _ => "Unknown"
    }.to_string();

    // Get the first path for the event, if any
    let path = event.paths.get(0).map(|p| p.display().to_string());

    (kind, path.unwrap())
}

#[doc = r"
    Call a command in the shell.

    Arguments:
        arg_str (str): The command to call.

    Returns:
        Child: The child process.
"]
fn call_command(arg_str: String) -> Child {
    let args: Vec<String> = arg_str.split_whitespace().map(|s| s.to_string()).collect();

    // println!("Running command: {:?}", args);
    let mut echo = Command::new("pwsh");
    echo.arg("-Command");
    echo.args(&args).spawn().unwrap()
}

#[pyfunction]
#[doc = r"
    watch(path: str, extensions: Optional[list[str]], cb: Optional[Callable]) -> None

    Monitor the specified `path` for changes, filtering by file extension if
    `extensions` is provided. If `cb` is provided, it will be called with a
    tuple containing the event kind and the path that changed. This should be a command
    that would be run in the shell, e.g. `npx tailwind -i /path -o /path` or `python -m http.server`.

    Arguments:
        path (str): The path to monitor for changes.
        extensions (Optional[List[str]]): A list of file extensions to filter by.
        Only changes to files with these extensions will be reported.
        command (Optional[Callable]): A callback to call when a change is detected.


    Returns:
        None
"]
fn watch(path: &str, extensions: Option<Vec<String>>, command: Option<&str>) -> PyResult<()> {
    println!("Watching {:?} for changes...", path);
    let (tx, rx) = channel();

    let mut watcher = RecommendedWatcher::new(tx, Config::default()).unwrap();

    watcher.watch(Path::new(path), RecursiveMode::Recursive).unwrap();

    let mut has_changes = false;
    // Initialize to 1 second ago
    let mut last_call_time = Instant::now() - Duration::new(1, 0);

    for res in rx {
        match res {
            Ok(event) => {
                // If extensions are provided, filter out events that don't match
                if let Some(exts) = &extensions {
                    let should_process = event.paths.iter().any(|p| {
                        p.extension()
                            .map(|ext| exts.contains(&ext.to_string_lossy().into_owned()))
                            .unwrap_or(false)
                    });

                    if !should_process {
                        continue;
                    }
                }

                // Convert the event to a tuple
                let event = event_to_tuple(&event);

                if !has_changes {
                    has_changes = true;
                    println!("Change detected!");
                    println!("Change: {:?}", event);
                }
            }
            Err(error) => println!("Error: {:?}", error),
        }
        // If a cmd is provided, call it and we have changes
        if let Some(cmd) = command {
            if has_changes {
                let now = Instant::now();
                if now.duration_since(last_call_time) >= Duration::new(1, 0) {
                    println!("Running command: {:?}", cmd);
                    call_command(cmd.to_string());
                    // Update the last call time
                    last_call_time = now;
                }
            }
        }
    }
    Ok(())
}


#[pymodule]
fn xpectate(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(watch, m)?)?;
    Ok(())
}
