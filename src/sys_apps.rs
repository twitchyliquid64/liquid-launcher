use freedesktop_entry_parser::{parse_entry, Entry};
use std::env;
use std::path::PathBuf;

/// In addition to entries in $XDG_DATA_DIRS
const APPLICATION_PATHS: [&str; 4] = [
    "/usr/share/applications",
    "/usr/local/share/applications",
    "$HOME/.local/share/applications",
    "/var/lib/flatpak/exports/share/applications",
];

#[derive(Debug, Clone)]
pub struct App {
    pub name: String,
    cmd: String,
    pub icon_name: Option<String>,
}

impl TryFrom<Entry> for App {
    type Error = ();

    fn try_from(e: Entry) -> Result<Self, Self::Error> {
        let name: Option<&str> = e.section("Desktop Entry").attr("Name");
        let cmd: Option<&str> = e.section("Desktop Entry").attr("Exec");
        let icon_name: Option<String> = e
            .section("Desktop Entry")
            .attr("Icon")
            .map(|icon| icon.to_owned());

        match (name, cmd) {
            (Some(name), Some(cmd)) => Ok(Self {
                name: name.into(),
                cmd: cmd.into(),
                icon_name,
            }),
            _ => Err(()),
        }
    }
}

impl App {
    // This whole method is terrible, im sorry (not sorry)
    pub fn find_icon(&self, size: u16) -> Option<PathBuf> {
        match &self.icon_name {
            Some(ic) => {
                use freedesktop_icons::lookup;
                if let Some(icon) = lookup(ic).with_size(size).find() {
                    return Some(icon);
                }

                for path in AppList::xdg_app_dirs().into_iter() {
                    let mut hicolor_png = path.clone();
                    hicolor_png.pop(); // remove 'applications/'
                    hicolor_png.push("icons");
                    hicolor_png.push("hicolor");
                    hicolor_png.push(size.to_string() + "x" + &size.to_string());
                    hicolor_png.push("apps");
                    hicolor_png.push(ic.to_owned() + ".png");
                    if hicolor_png.exists() {
                        return Some(hicolor_png);
                    }

                    let mut pixmaps_png = path.clone();
                    pixmaps_png.pop(); // remove 'applications/'
                    pixmaps_png.push("pixmaps");
                    pixmaps_png.push(ic.to_owned() + ".png");
                    if pixmaps_png.exists() {
                        return Some(pixmaps_png);
                    }

                    let mut hicolor_jpg = path.clone();
                    hicolor_jpg.pop(); // remove 'applications/'
                    hicolor_jpg.push("icons");
                    hicolor_jpg.push("hicolor");
                    hicolor_jpg.push(size.to_string() + "x" + &size.to_string());
                    hicolor_jpg.push("apps");
                    hicolor_jpg.push(ic.to_owned() + ".jpg");
                    if hicolor_jpg.exists() {
                        return Some(hicolor_jpg);
                    }

                    let mut hicolor_svg = path.clone();
                    hicolor_svg.pop(); // remove 'applications/'
                    hicolor_svg.push("icons");
                    hicolor_svg.push("hicolor");
                    hicolor_svg.push("scalable");
                    hicolor_svg.push("apps");
                    hicolor_svg.push(ic.to_owned() + ".svg");
                    if hicolor_svg.exists() {
                        return Some(hicolor_svg);
                    }

                    let mut gnome_apps_png = path.clone();
                    gnome_apps_png.pop(); // remove 'applications/'
                    gnome_apps_png.push("icons");
                    gnome_apps_png.push("gnome");
                    gnome_apps_png.push(size.to_string() + "x" + &size.to_string());
                    gnome_apps_png.push("apps");
                    gnome_apps_png.push(ic.to_owned() + ".png");
                    if gnome_apps_png.exists() {
                        return Some(gnome_apps_png);
                    }
                    let mut gnome_devices_png = path.clone();
                    gnome_devices_png.pop(); // remove 'applications/'
                    gnome_devices_png.push("icons");
                    gnome_devices_png.push("gnome");
                    gnome_devices_png.push(size.to_string() + "x" + &size.to_string());
                    gnome_devices_png.push("devices");
                    gnome_devices_png.push(ic.to_owned() + ".png");
                    if gnome_devices_png.exists() {
                        return Some(gnome_devices_png);
                    }

                    let mut adwaita_actions_png = path.clone();
                    adwaita_actions_png.pop(); // remove 'applications/'
                    adwaita_actions_png.push("icons");
                    adwaita_actions_png.push("Adwaita");
                    adwaita_actions_png.push(size.to_string() + "x" + &size.to_string());
                    adwaita_actions_png.push("actions");
                    adwaita_actions_png.push(ic.to_owned() + "-symbolic.symbolic.png");
                    if adwaita_actions_png.exists() {
                        return Some(adwaita_actions_png);
                    }
                    let mut adwaita_actions_svg = path.clone();
                    adwaita_actions_svg.pop(); // remove 'applications/'
                    adwaita_actions_svg.push("icons");
                    adwaita_actions_svg.push("Adwaita");
                    adwaita_actions_svg.push("symbolic");
                    adwaita_actions_svg.push("actions");
                    adwaita_actions_svg.push(ic.to_owned() + "-symbolic.svg");
                    if adwaita_actions_svg.exists() {
                        return Some(adwaita_actions_svg);
                    }
                }
                None
            }
            None => None,
        }
    }

    pub fn run(&self, quit: bool) {
        use std::process::{exit, Command};

        let split_command = shell_words::split(&self.cmd).unwrap();
        let args: Vec<String> = split_command[1..]
            .iter()
            .filter(|a| !a.starts_with("%"))
            .map(|a| a.clone())
            .collect();

        Command::new(split_command[0].trim_matches('\"'))
            .args(args)
            .spawn()
            .unwrap();
        if quit {
            exit(0);
        }
    }
}

#[derive(Default, Debug, Clone)]
pub struct AppList {
    pub apps: Vec<App>,
}

impl AppList {
    #[cfg(not(target_os = "linux"))]
    pub fn new() -> std::io::Result<Self> {
        Ok(AppList {
            apps: vec![],
            ..Self::default()
        })
    }

    #[cfg(target_os = "linux")]
    pub fn new() -> std::io::Result<Self> {
        let mut entries = Vec::new();
        let xdg = AppList::xdg_app_dirs();
        if xdg.len() == 0 {
            for path in APPLICATION_PATHS.iter() {
                AppList::read_desktop_entries(&PathBuf::from(path), &mut entries)
            }
        }
        for path in xdg.into_iter() {
            AppList::read_desktop_entries(&path, &mut entries)
        }

        // panic!("{:?}", entries);
        Ok(AppList {
            apps: entries,
            ..Self::default()
        })
    }

    fn read_desktop_entries(path: &PathBuf, entries: &mut Vec<App>) {
        match std::fs::read_dir(path) {
            Ok(files) => {
                let mut path_applications: Vec<App> = files
                    .collect::<Vec<Result<std::fs::DirEntry, std::io::Error>>>()
                    .iter()
                    .map(|file_res| match file_res {
                        Ok(file) => {
                            if !file.file_name().to_string_lossy().ends_with(".desktop") {
                                return None;
                            }
                            match parse_entry(file.path()) {
                                Ok(e) => {
                                    if let Some(nodisplay) =
                                        e.section("Desktop Entry").attr("NoDisplay")
                                    {
                                        if nodisplay == "true" {
                                            None
                                        } else {
                                            Some(e)
                                        }
                                    } else {
                                        Some(e)
                                    }
                                }
                                Err(e) => {
                                    println!("parse err: {:?}", e);
                                    None
                                }
                            }
                        }

                        Err(e) => {
                            println!("err: {:?}", e);
                            None
                        }
                    })
                    .filter(|e| e.is_some())
                    .map(|e| e.unwrap())
                    .map(|e| App::try_from(e).ok())
                    .filter(|e| e.is_some())
                    .map(|e| e.unwrap())
                    .collect();

                entries.append(&mut path_applications);
            }
            Err(_) => {}
        }
    }

    fn xdg_app_dirs() -> Vec<PathBuf> {
        let mut out = Vec::with_capacity(24); // arbitrarily chosen
        for (key, value) in env::vars() {
            if key == "XDG_DATA_DIRS" {
                for dir in value.split(":") {
                    let mut entry: PathBuf = dir.into();
                    entry.push("applications");
                    out.push(entry);
                }
            }
        }

        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new() {
        let _ = AppList::new();
    }

    #[test]
    fn xdg_app_dirs() {
        #[cfg(os = "linux")]
        assert!(AppList::xdg_app_dirs().len() > 0);

        // if you want to see em
        // assert_eq!(AppList::xdg_app_dirs(), Vec::<PathBuf>::new());
    }
}
