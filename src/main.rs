mod image_downloader;

use anyhow::{Context, Result};
use std::{
    fs::{copy, create_dir_all, File},
    os::unix::fs::chroot,
};
use tempfile::TempDir;

#[derive(Debug)]
#[allow(dead_code)]
struct Command {
    image: String,
    command: String,
    args: Vec<String>,
}

// Usage: your_docker.sh run <image> <command> <arg1> <arg2> ...
#[tokio::main]
async fn main() -> Result<()> {
    let command = parse_arguments();
    let tmp_dir = TempDir::new()?;
    setup_container(&command, &tmp_dir).await;

    let output = execute_command(&command.command, &command.args)?;

    handle_output(&output)?;
    std::process::exit(get_status_code(&output));
}

fn execute_command(
    command: &String,
    command_args: &[String],
) -> Result<std::process::Output, anyhow::Error> {
    let output = std::process::Command::new(command)
        .args(command_args)
        .output()
        .with_context(|| {
            format!(
                "Tried to run '{}' with arguments {:?}",
                command, command_args
            )
        })?;
    Ok(output)
}

async fn setup_container(command: &Command, tmp_dir: &TempDir) {
    image_downloader::download_image(&command.image, tmp_dir).await;
    let _ = setup_chroot(command, tmp_dir);
    setup_pid_jail();
}

fn setup_chroot(command: &Command, tmp_dir: &TempDir) -> Result<()> {
    let final_path = tmp_dir
        .path()
        .join(command.command.strip_prefix('/').unwrap());
    create_dir_all(final_path.parent().unwrap()).expect("Failed to create temporary directory");
    copy(&command.command, final_path).expect("Failed to copy");
    let dev_null = tmp_dir.path().join("dev/null");
    create_dir_all(dev_null.parent().unwrap()).expect("Failed to create /dev");
    File::create(dev_null).expect("Failed to create /dev/null");
    chroot(tmp_dir.path()).expect("Failed to chroot");
    Ok(())
}

fn setup_pid_jail() {
    // Namespaces are not a thing outside Linux
    #[cfg(target_os = "linux")]
    unsafe {
        libc::unshare(libc::CLONE_NEWPID);
    };
}

fn parse_arguments() -> Command {
    let args: Vec<_> = std::env::args().collect();
    let image = &args[2];
    let command = &args[3];
    let command_args = &args[4..];
    Command {
        image: image.clone(),
        command: command.clone(),
        args: command_args.to_vec(),
    }
}

fn handle_output(output: &std::process::Output) -> Result<(), anyhow::Error> {
    let std_out = std::str::from_utf8(&output.stdout)?;
    let std_err = std::str::from_utf8(&output.stderr)?;
    print!("{}", std_out);
    eprint!("{}", std_err);
    Ok(())
}

fn get_status_code(output: &std::process::Output) -> i32 {
    output.status.code().unwrap_or(1)
}
