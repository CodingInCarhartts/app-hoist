use crate::models::OptionInfo;
use indicatif::ProgressBar;
use inquire::{MultiSelect, Text};
use std::process::{Command, Stdio};
use tokio::process::Command as AsyncCommand;

pub fn select_options(options: &[OptionInfo]) -> anyhow::Result<Vec<(String, Option<String>)>> {
    // Create a list of option descriptions for selection
    let option_texts: Vec<String> = options
        .iter()
        .enumerate()
        .map(|(i, opt)| format!("[{}] {}: {}", i, opt.flags.join(", "), opt.description))
        .collect();

    // Use MultiSelect to let user choose options
    let selected_texts = MultiSelect::new("Select options to include:", option_texts).prompt()?;

    let mut selected = Vec::new();

    for text in selected_texts {
        // Extract the index from [idx]
        if let Some(start) = text.find('[')
            && let Some(end) = text.find(']')
            && let Ok(idx) = text[start + 1..end].parse::<usize>()
            && let Some(opt) = options.get(idx)
        {
            let flag = opt.flags[0].clone(); // Use the first flag

            let value = if opt.requires_value {
                // Ask for value
                Some(Text::new(&format!("Enter value for {}:", flag)).prompt()?)
            } else {
                None
            };

            selected.push((flag, value));
        }
    }

    Ok(selected)
}

pub fn build_command(selected: &[(String, Option<String>)]) -> anyhow::Result<Vec<String>> {
    let mut args = Vec::new();

    for (flag, value) in selected {
        args.push(flag.clone());
        if let Some(val) = value {
            args.push(val.clone());
        }
    }

    Ok(args)
}

pub fn execute_command(executable: &str, args: &[String]) -> anyhow::Result<()> {
    println!("Executing: {} {}", executable, args.join(" "));

    let mut command = Command::new(executable);
    command.args(args);

    let status = command.status()?;

    if status.success() {
        println!("Command executed successfully");
    } else {
        println!("Command failed with exit code: {:?}", status.code());
    }

    Ok(())
}

pub fn execute_project_command(
    executable: &str,
    args: &[String],
    path: &str,
) -> anyhow::Result<()> {
    println!("Executing: {} {}", executable, args.join(" "));

    let mut command = Command::new(executable);
    command.args(args);
    command.current_dir(path);

    let status = command.status()?;

    if status.success() {
        println!("Command executed successfully");
    } else {
        println!("Command failed with exit code: {:?}", status.code());
    }

    Ok(())
}

pub async fn execute_project_command_async(
    executable: &str,
    args: &[String],
    path: &str,
    pb: &ProgressBar,
) -> anyhow::Result<()> {
    pb.set_message(format!("Running: {} {}", executable, args.join(" ")));

    let mut command = AsyncCommand::new(executable);
    command
        .args(args)
        .current_dir(path)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    let status = command.status().await?;

    if status.success() {
        pb.set_message(format!("✅ Completed: {} {}", executable, args.join(" ")));
        Ok(())
    } else {
        pb.set_message(format!(
            "❌ Failed: {} {} (exit code: {:?})",
            executable,
            args.join(" "),
            status.code()
        ));
        anyhow::bail!("Command failed with exit code: {:?}", status.code());
    }
}
