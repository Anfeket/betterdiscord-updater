use std::{
    io::Write, path::{Path, PathBuf}, process::exit
};

fn main() {
    kill_discord().unwrap();
    println!("Discord killed!");
    let data = get_betterdiscord_asar().unwrap();
    println!("Downloaded betterdiscord.asar: {}kb", data.len() / 1024);
    update_betterdiscord(data).unwrap();
    println!("Betterdiscord updated!");
    let data = get_openasar().unwrap();
    println!("Downloaded openasar: {}kb", data.len() / 1024);
    update_openasar(data).unwrap();
    println!("Updated openasar!");
    start_discord().unwrap();
    println!("Discord started!");
    exit(0);
}

fn get_betterdiscord_asar() -> Result<Vec<u8>, String> {
    use reqwest::{blocking::Client, header};
    let app = "anfeket/betterdiscord-updater";
    let url = "https://betterdiscord.app/Download/betterdiscord.asar";
    let data = Client::new()
        .get(url)
        .header(header::USER_AGENT, app)
        .send()
        .map_err(|err| format!("Error sending request! {}", err))?;
    if let Some(version) = data.headers().get("x-bd-version") {
        match version.to_str() {
            Ok(version) => {
                println!("Downloading version {}", version);
            }
            Err(err) => println!("Error scanning version header! {}", err),
        }
    } else {
        println!("Version not found! Continuing...");
    }
    data.bytes()
        .map_err(|err| format!("Couldn't convert to bytes! {}", err))
        .map(|data| data.to_vec())
}

fn get_openasar() -> Result<Vec<u8>, String> {
    use reqwest::{blocking::Client, header};
    let app = "anfeket/betterdiscord-updater";
    let url = "https://github.com/GooseMod/OpenAsar/releases/download/nightly/app.asar";
    let data = Client::new()
        .get(url)
        .header(header::USER_AGENT, app)
        .send()
        .map_err(|err| format!("Error sending request! {}", err))?;
    println!("Downloading latest openasar release...");
    data.bytes()
        .map_err(|err| format!("Couldn't convert to bytes! {}", err))
        .map(|data| data.to_vec())
}

fn write_data_to_path(path: &PathBuf, data: &Vec<u8>) -> Result<(), String> {
    std::fs::OpenOptions::new()
        .write(true)
        .open(path)
        .map_err(|err| format!("Error opening {:?}! {}", path, err))?
        .write_all(&data)
        .map_err(|err| format!("Error writing to {:?}! {}", path, err))
}

fn find_latest_app_version(discord_path: &Path) -> Result<PathBuf, String> {
    let mut app_dir: Vec<std::fs::DirEntry> = discord_path
        .read_dir()
        .map_err(|err| format!("Failed to read Discord dirs! {}", err))?
        .filter_map(Result::ok)
        .filter(|entry| entry.path().is_dir())
        .filter(|entry| entry.file_name().to_str().unwrap().starts_with("app"))
        .collect();
    app_dir.sort_by_key(|entry| entry.file_name());
    let latest = app_dir.last().unwrap().path();
    Ok(latest)
}

fn update_betterdiscord(data: Vec<u8>) -> Result<(), String> {
    let localappdata = std::env::var("LOCALAPPDATA")
        .map_err(|err| format!("Couldn't get %LOCALAPPDATA% {}", err))?;
    let appdata =
        std::env::var("APPDATA").map_err(|err| format!("Couldn't get %APPDATA% {}", err))?;

    let asar_path = Path::new(&appdata)
        .join("BetterDiscord")
        .join("data")
        .join("betterdiscord.asar");
    write_data_to_path(&asar_path, &data)?;

    fn shims(asar_path: &Path, localappdata: &String) -> Result<(), String> {
        let shim_data_path = asar_path
            .to_str()
            .ok_or(format!("Error converting path to str! {:?}", asar_path))?
            .replace('\\', "\\\\");
        let shim_data = format!(
            "require(\"{}\");\nmodule.exports = require(\"./core.asar\");",
            shim_data_path
        );
        let appdata_path = Path::new(localappdata).join("Discord");
        let appdir = find_latest_app_version(&appdata_path);
        let shims_path = appdata_path
            .join(appdir.unwrap())
            .join("modules")
            .join("discord_desktop_core-1")
            .join("discord_desktop_core")
            .join("index.js");
        write_data_to_path(&shims_path, &shim_data.as_bytes().to_owned())?;
        Ok(())
    }
    shims(&asar_path, &localappdata)?;
    Ok(())
}

fn update_openasar(data: Vec<u8>) -> Result<(), String> {
    let localappdata = std::env::var("LOCALAPPDATA")
        .map_err(|err| format!("Couldn't get %LOCALAPPDATA% {}", err))?;
    let appdata_path = Path::new(&localappdata).join("Discord");
    let discord_path = find_latest_app_version(&appdata_path)?;
    let asar_path = discord_path.join("resources").join("app.asar");
    let backup = discord_path.join("resources").join("app.asar.orig");
    if std::fs::copy(&asar_path, backup).is_err() {
        println!("Failed to backup, continuing...")
    };
    write_data_to_path(&asar_path, &data)
}

fn kill_discord() -> Result<(), String> {
    use std::process::Command;
    Command::new("taskkill")
        .args(["/F", "/IM", "discord.exe"])
        .output()
        .map(|_| ())
        .map_err(|err| format!("Couldn't kill discord! {}", err))
}

fn start_discord() -> Result<(), String> {
    let localappdata = std::env::var("LOCALAPPDATA")
        .map_err(|err| format!("Couldn't get %LOCALAPPDATA% {}", err))?;
    let discord = Path::new(&localappdata).join("Discord").join("Update.exe");
    std::process::Command::new(discord)
        .args(["--processStart", "Discord.exe"])
        // .stdin(Stdio::null())
        // .stdout(Stdio::null())
        // .stderr(Stdio::null())
        .spawn()
        .map_err(|err| format!("Error starting Discord! {}", err))?;
    Ok(())
}
