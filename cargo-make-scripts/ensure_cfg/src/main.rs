
const TRIPLE_SINGLE_QUOTES: &str = "\u{27}\u{27}\u{27}";
const CFG_EXAMPLE_PATH: &str = "Secrets.example.toml";
const CFG_PATH: &str = "Secrets.toml";

#[derive(Default)]
struct TidyStringReadBuffer(String);

#[derive(Debug, thiserror::Error)]
#[error("Unexpected input")]
struct UnexpectedInput;

impl TidyStringReadBuffer {
    fn read_line_and_then<F, O>(&mut self, f: F) -> O
    where
        F: FnOnce(&str) -> O,
    {
        std::io::stdin()
            .read_line(&mut self.0)
            .unwrap_or_else(|e| panic!("Failed to read line: {e:?}"));
        let ret = f(self.0.trim());
        self.0.clear();
        ret
    }
}

fn repeat_until_succeeds<F, O, E, H>(mut f: F, h: H) -> O
where
    F: FnMut() -> Result<O, E>,
    H: Fn(E) -> (),
{
    loop {
        match f() {
            Ok(o) => return o,
            Err(e) => h(e),
        };
    }
}

fn read_cfg_example(crate_root: impl AsRef<std::path::Path>) -> toml::Table {
    let cfg_example: std::path::PathBuf = crate_root.as_ref().join(CFG_EXAMPLE_PATH);
    let cfg_example = std::fs::read_to_string(&cfg_example)
        .unwrap_or_else(|e| panic!("Failed to read {cfg_example:?}: {e:?}"));
    toml::from_str(&cfg_example)
        .unwrap_or_else(|e| panic!("Failed to parse {cfg_example:?}: {e:?}"))
}

pub fn output_cfg(cfg: &toml::Table, path: impl AsRef<std::path::Path>) {
    let path = path.as_ref();
    let pretty_toml = toml::to_string_pretty(&cfg).unwrap();

    println!();
    println!("Writing {path:?}...");
    println!("{TRIPLE_SINGLE_QUOTES}");
    println!("{pretty_toml}");
    println!("{TRIPLE_SINGLE_QUOTES}");

    std::fs::write(path, &pretty_toml)
        .unwrap_or_else(|e| panic!("Failed to write {path:?}: {e:?}"));

    let full_path = dunce::canonicalize(path).unwrap_or(path.to_path_buf());
    println!("The configuration file has been successfully crated at {full_path:?}.");
}

pub fn main() {
    println!("\tENSURE CFG");
    let crate_root = std::env::current_dir().unwrap();
    println!("The bot directory: {}", crate_root.display());

    let cfg_path = std::path::Path::new(CFG_PATH);
    if cfg_path.exists() {
        println!("{CFG_PATH} found");
        return;
    };
    print!("{CFG_PATH} not found.\n\n");

    let mut input = TidyStringReadBuffer::default();

    // TODO: consider using a good prompting crate, e.g. `inquire`
    repeat_until_succeeds(
        || {
            println!("Do you want to create a new Discord bot token and invite your bot to the server? (y/n)");
            println!();
            println!("If you press 'y' and hit enter, your browser will open two links:");
            println!("1. The \"Applications\" tab of the Discord Developer Portal (https://discord.com/developers/applications)");
            println!("2. A tutorial on how to create a Discord bot token and invite the bot to the server (https://www.writebots.com/discord-bot-token/)");
            println!();
            println!("Please ensure that the bot is given admin permissions.");

            input.read_line_and_then(|s| match s.trim().to_lowercase().as_str() {
            "n" => Ok(()),
            "y" => {
                open::that("https://discord.com/developers/applications")
                    .unwrap_or_else(|e| panic!("Failed to open https://discord.com/developers/applications: {e:?}"));
                open::that("https://www.writebots.com/discord-bot-token/")
                    .unwrap_or_else(|e| panic!("Failed to open https://www.writebots.com/discord-bot-token/: {e:?}"));
                Ok(())
            },
            _ => Err(UnexpectedInput),
        })
        },
        |UnexpectedInput| println!("Invalid input. Please input 'y' or 'n'."),
    );

    let mut cfg: toml::Table = read_cfg_example(&crate_root);

    {
        let token = cfg
            .get_mut("DISCORD_TOKEN")
            .unwrap_or_else(|| panic!("Failed to get token from {CFG_EXAMPLE_PATH}"));
        let toml::Value::String(token) = token else {
            panic!("Token in {CFG_EXAMPLE_PATH} is not a string");
        };

        print!("\nPlease enter your bot token:\n");
        input.read_line_and_then(|s| {
            *token = s.trim().to_string();
            print!("\n");
        });
    }

    println!("Running a Discord bot requires...");
    println!("* A server where the bot would be invited.");
    println!("* Permissions to run on the server.");
    println!("* A channel where the bot would be able to send messages (aka bot channel).");
    println!("* A channel where the bot would track reaction for assigning roles (aka self-role channel).");

    println!();
    println!("The default {CFG_EXAMPLE_PATH} uses the values for Cherry's server.");
    println!("Most likely, you'll want to use the values for your test server.");
    println!("Would you like to overwrite the default configuration values? [Recommended] (y/n)");

    let overwrite_default: bool = repeat_until_succeeds(
        || {
            input.read_line_and_then(|s| match s.trim().to_lowercase().as_str() {
                "y" => Ok(true),
                "n" => Ok(false),
                _ => Err(UnexpectedInput),
            })
        },
        |UnexpectedInput| println!("Invalid input. Please input 'y' or 'n'."),
    );

    if !overwrite_default {
        output_cfg(&cfg, cfg_path);
        return;
    };

    println!();
    println!(
        "For overwriting the default values, you'll need to have \
        Discord Developer Mode enabled."
    );

    let is_dev_mode_on: Option<bool> = repeat_until_succeeds(
        || {
            println!("Do you have Discord Developer Mode enabled? (y/n/idk)");

            input.read_line_and_then(|s| match s.trim().to_lowercase().as_str() {
                "y" => Ok(Some(true)),
                "n" => Ok(Some(false)),
                "idk" => Ok(None),
                _ => Err(UnexpectedInput),
            })
        },
        |UnexpectedInput| println!("Invalid input. Please input 'y', 'n' or 'idk'."),
    );

    if !matches!(is_dev_mode_on, Some(true)) {
        println!();
        println!("To enable the Discord Developer Mode, you'll need to follow the instructions described here:");
        println!("https://beebom.com/how-enable-disable-developer-mode-discord/");

        repeat_until_succeeds(
            || {
                println!();
                println!("Input the number to choose one of the following options:");
                println!("1. Continue. I've enabled the Discord Developer Mode.");
                println!("2. Open the link.");

                input.read_line_and_then(|s| {
                match s.trim() {
                    "1" => Ok(()),
                    "2" => {
                        open::that("https://beebom.com/how-enable-disable-developer-mode-discord/")
                            .unwrap_or_else(|e| panic!("Failed to open https://beebom.com/how-enable-disable-developer-mode-discord/: {e:?}"));
                        Ok(())
                    },
                    _ => Err(UnexpectedInput),
                }
            })
            },
            |UnexpectedInput| println!("Invalid input. Please input '1' or '2'."),
        );
        println!();
    }

    {
        let server_id: &mut toml::Value = cfg
            .get_mut("DISCORD_SERVER_ID")
            .unwrap_or_else(|| panic!("Failed to get the server ID from {CFG_EXAMPLE_PATH}"));
        let toml::Value::String(server_id) = server_id else {
            panic!("Server ID in {CFG_EXAMPLE_PATH} is not a string");
        };

        *server_id = repeat_until_succeeds(
            || {
                println!("Input the ID of the Discord server where you want to invite the bot.");
                println!("You can get the ID by right-clicking on the server icon and selecting 'Copy ID'.");

                input.read_line_and_then(|s| {
                    let s = s.trim();
                    if s.chars().all(char::is_numeric) {
                        Ok(s.to_string())
                    } else {
                        Err(UnexpectedInput)
                    }
                })
            },
            |UnexpectedInput| println!("Invalid input. Please input the server ID."),
        );
    }

    {
        let bot_channel: &mut toml::Value =
            cfg.get_mut("DISCORD_BOT_CHANNEL").unwrap_or_else(|| {
                panic!("Failed to get the bot channel ID from {CFG_EXAMPLE_PATH}")
            });
        let toml::Value::String(bot_channel) = bot_channel else {
            panic!("Bot channel ID in {CFG_EXAMPLE_PATH} is not a string");
        };

        *bot_channel = repeat_until_succeeds(
            || {
                println!();
                println!(
                    "Input the ID of the channel where the bot is going to respond to commands. [Bot channel]"
                );
                println!(
                    "You can get the ID by right-clicking on the channel and selecting 'Copy ID'."
                );

                input.read_line_and_then(|s| {
                    let s = s.trim();
                    if s.chars().all(char::is_numeric) {
                        Ok(s.to_string())
                    } else {
                        Err(UnexpectedInput)
                    }
                })
            },
            |UnexpectedInput| println!("Invalid input. Please input the channel ID."),
        );
    }

    {
        let self_role: &mut toml::Value =
            cfg.get_mut("DISCORD_SELF_ROLE_CHANNEL").unwrap_or_else(|| {
                panic!("Failed to get the self-role channel ID from {CFG_EXAMPLE_PATH}")
            });
        let toml::Value::String(self_role) = self_role else {
            panic!("Self-role channel ID in {CFG_EXAMPLE_PATH} is not a string");
        };

        *self_role = repeat_until_succeeds(
            || {
                println!();
                println!("Input the ID of the channel for self-assigning roles. [Self-role channel]");
                println!(
                    "You can get the ID by right-clicking on the channel and selecting 'Copy ID'."
                );

                input.read_line_and_then(|s| {
                    let s = s.trim();
                    if s.chars().all(char::is_numeric) {
                        Ok(s.to_string())
                    } else {
                        Err(UnexpectedInput)
                    }
                })
            },
            |UnexpectedInput| println!("Invalid input. Please input the channel ID."),
        );
    }

    output_cfg(&cfg, CFG_PATH);
}
