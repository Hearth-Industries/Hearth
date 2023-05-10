use std::{env, fs};
use log::{warn};



fn is_program_in_path(program: &str) -> bool {
    if let Ok(path) = env::var("PATH") {
        for p in path.split(":") {
            let p_str = format!("{}/{}", p, program);
            if fs::metadata(p_str).is_ok() {
                return true;
            }
        }
    }
    false
}

pub fn print_warnings() {
    if is_program_in_path("yt-dlp") == false {
        warn!("yt-dlp is not installed! This Lantern instance will not be able to play tracks from YouTube until it is installed!")
    }
}

pub fn over_servers_warning() {
    warn!("Guild count is over 2500. Hearth is only free up to 2500 discord servers. You must contact Hearth Industries within 14 Days of passing 1000 servers or you may owe damages. For more details see the license in the github repo: LICENSE.md. If you have already acquired a paid license no further action is needed.")
}

pub fn print_intro() {
    println!(r"
    ██╗░░██╗███████╗░█████╗░██████╗░████████╗██╗░░██╗
    ██║░░██║██╔════╝██╔══██╗██╔══██╗╚══██╔══╝██║░░██║
    ███████║█████╗░░███████║██████╔╝░░░██║░░░███████║
    ██╔══██║██╔══╝░░██╔══██║██╔══██╗░░░██║░░░██╔══██║
    ██║░░██║███████╗██║░░██║██║░░██║░░░██║░░░██║░░██║
    ╚═╝░░╚═╝╚══════╝╚═╝░░╚═╝╚═╝░░╚═╝░░░╚═╝░░░╚═╝░░╚═╝")
}