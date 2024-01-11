use std::{
    io::Write,
    path::{Path, PathBuf},
};

fn main() {
    kill_discord().unwrap();
    println!("Discord killed!");
    let data = get_asar().unwrap();
    println!("Downloaded {}kb", data.len() / 1024);
    update(data).unwrap();
    println!("Discord updated!");
    start_discord().unwrap();
    println!("Discord started!");
}

fn get_asar() -> Result<Vec<u8>, String> {
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

fn update(data: Vec<u8>) -> Result<(), String> {
    let localappdata = std::env::var("LOCALAPPDATA")
        .map_err(|err| format!("Couldn't get %LOCALAPPDATA% {}", err))?;
    let appdata =
        std::env::var("APPDATA").map_err(|err| format!("Couldn't get %APPDATA% {}", err))?;

    let asar_path = Path::new(&appdata)
        .join("BetterDiscord")
        .join("data")
        .join("betterdiscord.asar");
    fn write_data(path: &PathBuf, data: &Vec<u8>) -> Result<(), String> {
        std::fs::OpenOptions::new()
            .write(true)
            .open(path)
            .map_err(|err| format!("Error opening {:?}! {}", path, err))?
            .write_all(&data)
            .map_err(|err| format!("Error writing to {:?}! {}", path, err))
    }
    write_data(&asar_path, &data)?;

    fn shims(asar_path: &Path, localappdata: &String) -> Result<(), String> {
        let shim_data_path = asar_path
            .to_str()
            .ok_or(format!("Error converting path to str! {:?}", asar_path))?
            .replace('\\', "\\\\");
        let shim_data = format!(
            "require(\"{}\");\nmodule.exports = require(\"./core.asar\");",
            shim_data_path
        );
        let shims_path = Path::new(localappdata).join("Discord");
        let app_dir = shims_path
            .read_dir()
            .map_err(|err| format!("Failed to read Discord dirs! {}", err))?
            .filter_map(Result::ok)
            .filter(|entry| entry.path().is_dir())
            .filter_map(|entry| entry.file_name().into_string().ok())
            .find(|name| name.starts_with("app"))
            .ok_or("Failed to find app dir!")?;
        let shims_path = shims_path
            .join(app_dir)
            .join("modules")
            .join("discord_desktop_core-1")
            .join("discord_desktop_core")
            .join("index.js");
        write_data(&shims_path, &shim_data.as_bytes().to_owned())?;
        Ok(())
    }
    shims(&asar_path, &localappdata)?;
    Ok(())
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
    let discord = Path::new(&localappdata).join("Discord");
    let app_dir = discord
        .read_dir()
        .map_err(|err| format!("Failed to read Discord dirs! {}", err))?
        .filter_map(Result::ok)
        .filter(|entry| entry.path().is_dir())
        .filter_map(|entry| entry.file_name().into_string().ok())
        .find(|name| name.starts_with("app"))
        .ok_or("Failed to find app dir!")?;
    let discord = discord.join(app_dir).join("Discord.exe");
    std::process::Command::new(discord).spawn().map_err(|err| format!("Error starting Discord! {}", err))?;
    Ok(())
}
