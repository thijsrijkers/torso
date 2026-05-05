pub struct Flag {
    pub name: &'static str,
    pub description: &'static str,
}

pub const AVAILABLE_FLAGS: &[Flag] = &[
    Flag {
        name: "--help",
        description: "Shows all available flag options",
    },
    Flag {
        name: "--serve",
        description: "Launches Torso engine listerner"
    },
    Flag {
        name: "--init",
        description: "Loads configuration from torso file in folder",
    },
    Flag {
        name: "--keep-lifecycle",
        description: "An addition flag to preserve already running lifecycles",
    },
];
