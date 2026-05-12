use std::{env, fs};
use std::path::PathBuf;
use yaml_rust::YamlLoader;
use yaml_rust::Yaml;

pub fn load_scaffold() {
    let torso_file = read_yaml_from_root();
    check_required_values(&torso_file);
}

fn check_required_values(cfg: &Yaml) {
    let app_name = cfg["app_name"]
        .as_str()
        .expect("Missing or invalid 'app_name' in torso.yaml");

    println!("app_name={app_name}");
}

fn read_yaml_from_root() -> Yaml {
    let current_dir = env::current_dir().expect("Failed to get current dir");
    let torso_file: PathBuf = current_dir.join("torso.yaml");
    let raw_torso_file = fs::read_to_string(&torso_file).expect("Failed to read torso file");
    let configuration = YamlLoader::load_from_str(&raw_torso_file).expect("Failed out to load yaml from string");
    return configuration.into_iter()
        .next()
        .expect("torso.yaml is empty")
}
