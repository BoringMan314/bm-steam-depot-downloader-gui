// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod depotdownloader;
mod steam;
mod terminal;

use crate::depotdownloader::{get_depotdownloader_url, DEPOTDOWNLOADER_VERSION};
use crate::terminal::{async_read_from_pty, async_resize_pty, async_write_to_pty};
use portable_pty::{native_pty_system, ChildKiller, PtyPair, PtySize};
use std::io::ErrorKind::AlreadyExists;
use std::io::{BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc};
use std::time::Duration;
use std::{env, thread};
use tauri::async_runtime::Mutex;
use tauri::{AppHandle, Emitter, Manager, State};

struct AppState {
    pty_pair: Arc<Mutex<PtyPair>>,
    writer: Arc<Mutex<Box<dyn Write + Send>>>,
    reader: Arc<Mutex<BufReader<Box<dyn Read + Send>>>>,
    /// Kept separately from the child so it stays usable while a thread is blocked in `wait`.
    killer: Arc<Mutex<Option<Box<dyn ChildKiller + Send + Sync>>>>,
}

#[tauri::command]
async fn start_download(steam_download: steam::SteamDownload, app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    let working_dir: PathBuf = get_working_dir(&app);

    // std::env::set_current_dir(&WORKING_DIR.get().unwrap()).unwrap();
    dbg!(&steam_download);

    println!("\n-------------------------DEBUG INFO------------------------");
    println!("received these values from frontend:");
    println!("\t- Username: {}", steam_download.username().as_ref().unwrap_or(&String::from("Not provided")));
    // println!("\t- Password: {}", steam_download.password().as_ref().unwrap_or(&String::from("Not provided"))); Don't log in prod lol
    println!("\t- App ID: {}", steam_download.app_id());
    println!("\t- Depot ID: {}", steam_download.depot_id());
    println!("\t- Manifest ID: {}", steam_download.manifest_id());
    println!("\t- Output Path: {}", steam_download.output_path());
    println!("\t- Working directory: {}", &working_dir.display());
    println!("----------------------------------------------------------\n");

    /* Build the command and spawn it in our terminal */
    let mut cmd = terminal::create_depotdownloader_command(steam_download, &working_dir);

    // add the $TERM env variable so we can use clear and other commands
    #[cfg(target_os = "windows")]
    cmd.env("TERM", "cygwin");
    #[cfg(not(target_os = "windows"))]
    cmd.env("TERM", "xterm-256color");

    let mut child = state
        .pty_pair
        .lock()
        .await
        .slave
        .spawn_command(cmd)
        .map_err(|err| err.to_string())?;

    *state.killer.lock().await = Some(child.clone_killer());

    thread::spawn(move || {
        let status = child.wait().unwrap();
        println!("Command exited with status: {status}");
        app.emit("command-exited", {}).unwrap();
        // exit(status.exit_code() as i32)
    });
    Ok(())
}

/// Downloads the DepotDownloader zip file from the internet based on the OS.
#[tauri::command]
async fn download_depotdownloader(app: AppHandle) -> Result<(), String> {
    let working_dir: PathBuf = get_working_dir(&app);
    let url = get_depotdownloader_url();

    // Where we store the DepotDownloader zip.
    let zip_filename = format!("DepotDownloader-v{}-{}.zip", DEPOTDOWNLOADER_VERSION, env::consts::OS);
    let depotdownloader_zip = Path::join(&working_dir, Path::new(&zip_filename));
    let binary = working_dir.join(depotdownloader::BINARY_NAME);

    match depotdownloader::download_file(url.as_str(), depotdownloader_zip.as_path()).await {
        Ok(()) => println!("Downloaded DepotDownloader for {} to {}", env::consts::OS, depotdownloader_zip.display()),
        Err(e) if e.kind() == AlreadyExists => {
            println!("DepotDownloader already exists. Skipping download.");
            if binary.is_file() {
                return Ok(());
            }
            println!("Binary is missing, extracting the existing archive again.");
        }
        Err(e) => return Err(format!("無法下載 DepotDownloader：{e}\n{url}")),
    }

    if let Err(e) = depotdownloader::unzip(depotdownloader_zip.as_path(), &working_dir) {
        // A damaged archive would otherwise fail forever, since it counts as already downloaded.
        let _ = std::fs::remove_file(&depotdownloader_zip);
        return Err(format!("無法解壓 DepotDownloader，已移除損壞的檔案，請再試一次：{e}"));
    }

    if !binary.is_file() {
        return Err(format!("解壓完成但找不到 {}", binary.display()));
    }

    println!("Succesfully extracted DepotDownloader zip.");
    Ok(())
}

/// Terminates the running DepotDownloader process, if there is one.
/// The `command-exited` event is emitted by the waiting thread once the process is gone.
#[tauri::command]
async fn cancel_download(state: State<'_, AppState>) -> Result<(), String> {
    let mut killer = state.killer.lock().await;

    let Some(killer) = killer.as_mut() else {
        return Ok(());
    };

    let result = killer.kill();

    // portable-pty 0.9.0 inverts the TerminateProcess return value, so a kill that worked comes
    // back as an error holding whatever `last_os_error` happened to be. Only the `command-exited`
    // event can tell us whether the process is really gone.
    if cfg!(windows) {
        if let Err(err) = &result {
            println!("DEBUG: ignoring kill result from portable-pty: {err}");
        }
        return Ok(());
    }

    result.map_err(|err| err.to_string())
}

/// Checks internet connectivity using Google
#[tauri::command]
async fn internet_connection() -> bool {
    let client = reqwest::Client::builder().timeout(Duration::from_secs(5)).build().unwrap();

    client.get("https://connectivitycheck.android.com/generate_204").send().await.is_ok()
}


pub fn get_os() -> &'static str {
    match env::consts::OS {
        "linux" => "linux",
        "macos" => "macos",
        "windows" => "windows",
        _ => "unknown",
    }
}

pub fn get_working_dir(app: &AppHandle) -> PathBuf {
    Path::join(&app.path().local_data_dir().unwrap(), "SteamDepotDownloaderGUI")
}

fn main() {
    // macOS: change dir to documents because upon opening, our current dir by default is "/".
    // todo: Is this still needed ??
/*    if get_os() == "macos" {
        let _ = fix_path_env::fix();
        // let documents_dir = format!(
        //     "{}/Documents/SteamDepotDownloaderGUI",
        //     std::env::var_os("HOME").unwrap().to_str().unwrap()
        // );
        // let documents_dir = Path::new(&documents_dir);
        // // println!("{}", documents_dir.display());

        // std::fs::create_dir_all(documents_dir).unwrap();
        // env::set_current_dir(documents_dir).unwrap();
    }*/

    /* Initialize the pty system */
    let pty_system = native_pty_system();

    let pty_pair = pty_system
        .openpty(PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        })
        .unwrap();

    let reader = pty_pair.master.try_clone_reader().unwrap();
    let writer = pty_pair.master.take_writer().unwrap();

    println!();
    tauri::Builder::default()
        .manage(AppState {
            pty_pair: Arc::new(Mutex::new(pty_pair)),
            writer: Arc::new(Mutex::new(writer)),
            reader: Arc::new(Mutex::new(BufReader::new(reader))),
            killer: Arc::new(Mutex::new(None)),
        })
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .invoke_handler(tauri::generate_handler![
            start_download,
            cancel_download,
            download_depotdownloader,
            internet_connection,
            async_write_to_pty,
            async_read_from_pty,
            async_resize_pty,
        ]).run(tauri::generate_context!())
        .expect("error while running tauri application");
}