use std::fs;
use std::io::{self, Error as IoError, Write};
use std::process;

use clap::{App, Arg, ArgAction};
use tempfile::Builder;

use monolith::cache::Cache;
use monolith::cookies::parse_cookie_file_contents;
use monolith::core::{create_monolithic_document, print_error_message, Options};

enum Output {
    Stdout(io::Stdout),
    File(fs::File),
}

impl Output {
    fn new(destination: &str) -> Result<Output, IoError> {
        if destination.is_empty() || destination.eq("-") {
            Ok(Output::Stdout(io::stdout()))
        } else {
            Ok(Output::File(fs::File::create(destination)?))
        }
    }

    fn write(&mut self, bytes: &Vec<u8>) -> Result<(), IoError> {
        match self {
            Output::Stdout(stdout) => {
                stdout.write_all(bytes)?;
                // Ensure newline at end of output
                if bytes.last() != Some(&b"\n"[0]) {
                    stdout.write_all(b"\n")?;
                }
                stdout.flush()
            }
            Output::File(file) => {
                file.write_all(bytes)?;
                // Ensure newline at end of output
                if bytes.last() != Some(&b"\n"[0]) {
                    file.write_all(b"\n")?;
                }
                file.flush()
            }
        }
    }
}

const ASCII: &str = " \
 _____     ______________    __________      ___________________    ___
|     \\   /              \\  |          |    |                   |  |   |
|      \\_/       __       \\_|    __    |    |    ___     ___    |__|   |
|               |  |            |  |   |    |   |   |   |   |          |
|   |\\     /|   |__|    _       |__|   |____|   |   |   |   |    __    |
|   | \\___/ |          | \\                      |   |   |   |   |  |   |
|___|       |__________|  \\_____________________|   |___|   |___|  |___|
";
const CACHE_ASSET_FILE_SIZE_THRESHOLD: usize = 1024 * 10; // Minimum file size for on-disk caching (in bytes)
const DEFAULT_NETWORK_TIMEOUT: u64 = 120;
const DEFAULT_USER_AGENT: &str =
    "Mozilla/5.0 (X11; Ubuntu; Linux x86_64; rv:135.0) Gecko/20100101 Firefox/135.0";

fn main() {
    // Process CLI flags and options
    let mut cookie_file_path: Option<String> = None;
    let mut exit_code = 0;
    let mut options: Options = Options::default();
    let source;
    let destination;
    {
        let app = App::new(env!("CARGO_PKG_NAME"))
            .version(env!("CARGO_PKG_VERSION"))
            .author(format!("\n{}\n\n", env!("CARGO_PKG_AUTHORS").replace(':', "\n")).as_str())
            .about(format!("{}\n{}", ASCII, env!("CARGO_PKG_DESCRIPTION")).as_str())
            .args_from_usage("-a, --no-audio 'Remove audio sources'")
            .args_from_usage("-b, --base-url=[http://localhost/] 'Set custom base URL'")
            .args_from_usage(
                "-B, --blacklist-domains 'Treat list of specified domains as blacklist'",
            )
            .args_from_usage("-c, --no-css 'Remove CSS'")
            .args_from_usage("-C, --cookie-file=[cookies.txt] 'Specify cookie file'")
            .arg(
                Arg::with_name("domains")
                    .short('d')
                    .long("domain")
                    .takes_value(true)
                    .value_name("example.com")
                    .action(ArgAction::Append)
                    .help("Specify domains to use for white/black-listing"),
            )
            .args_from_usage("-e, --ignore-errors 'Ignore network errors'")
            .args_from_usage("-E, --encoding=[UTF-8] 'Enforce custom charset'")
            .args_from_usage("-f, --no-frames 'Remove frames and iframes'")
            .args_from_usage("-F, --no-fonts 'Remove fonts'")
            .args_from_usage("-i, --no-images 'Remove images'")
            .args_from_usage("-I, --isolate 'Cut off document from the Internet'")
            .args_from_usage("-j, --no-js 'Remove JavaScript'")
            .args_from_usage("-k, --insecure 'Allow invalid X.509 (TLS) certificates'")
            .args_from_usage("-M, --no-metadata 'Exclude timestamp and source information'")
            .args_from_usage(
                "-n, --unwrap-noscript 'Replace NOSCRIPT elements with their contents'",
            )
            .args_from_usage(
                "-o, --output=[document.html] 'Write output to <file>, use - for STDOUT'",
            )
            .args_from_usage("-q, --quiet 'Suppress verbosity'")
            .args_from_usage("-t, --timeout=[60] 'Adjust network request timeout'")
            .args_from_usage("-u, --user-agent=[Firefox] 'Set custom User-Agent string'")
            .args_from_usage("-v, --no-video 'Remove video sources'")
            .arg(
                Arg::with_name("target")
                    .required(true)
                    .takes_value(true)
                    .index(1)
                    .help("URL or file path, use - for STDIN"),
            )
            .get_matches();

        // Process the command
        source = app
            .value_of("target")
            .expect("please set target")
            .to_string();
        options.no_audio = app.is_present("no-audio");
        if let Some(base_url) = app.value_of("base-url") {
            options.base_url = Some(base_url.to_string());
        }
        options.blacklist_domains = app.is_present("blacklist-domains");
        options.no_css = app.is_present("no-css");
        if let Some(cookie_file) = app.value_of("cookie-file") {
            cookie_file_path = Some(cookie_file.to_string());
        }
        if let Some(encoding) = app.value_of("encoding") {
            options.encoding = Some(encoding.to_string());
        }
        if let Some(domains) = app.get_many::<String>("domains") {
            let list_of_domains: Vec<String> = domains.cloned().collect::<Vec<_>>();
            options.domains = Some(list_of_domains);
        }
        options.ignore_errors = app.is_present("ignore-errors");
        options.no_frames = app.is_present("no-frames");
        options.no_fonts = app.is_present("no-fonts");
        options.no_images = app.is_present("no-images");
        options.isolate = app.is_present("isolate");
        options.no_js = app.is_present("no-js");
        options.insecure = app.is_present("insecure");
        options.no_metadata = app.is_present("no-metadata");
        destination = app.value_of("output").unwrap_or("").to_string();
        options.silent = app.is_present("quiet");
        options.timeout = app
            .value_of("timeout")
            .unwrap_or(&DEFAULT_NETWORK_TIMEOUT.to_string())
            .parse::<u64>()
            .unwrap();
        if let Some(user_agent) = app.value_of("user-agent") {
            options.user_agent = Some(user_agent.to_string());
        } else {
            options.user_agent = Some(DEFAULT_USER_AGENT.to_string());
        }
        options.unwrap_noscript = app.is_present("unwrap-noscript");
        options.no_video = app.is_present("no-video");
    }

    // Set up cache (attempt to create temporary file)
    let temp_cache_file = match Builder::new().prefix("monolith-").tempfile() {
        Ok(tempfile) => Some(tempfile),
        Err(_) => None,
    };
    let mut cache = Some(Cache::new(
        CACHE_ASSET_FILE_SIZE_THRESHOLD,
        if temp_cache_file.is_some() {
            Some(
                temp_cache_file
                    .as_ref()
                    .unwrap()
                    .path()
                    .display()
                    .to_string(),
            )
        } else {
            None
        },
    ));

    // Read and parse cookie file
    if let Some(opt_cookie_file) = cookie_file_path.clone() {
        match fs::read_to_string(&opt_cookie_file) {
            Ok(str) => match parse_cookie_file_contents(&str) {
                Ok(parsed_cookies_from_file) => {
                    options.cookies = parsed_cookies_from_file;
                }
                Err(_) => {
                    print_error_message(
                        &format!(
                            "could not parse specified cookie file \"{}\"",
                            opt_cookie_file
                        ),
                        &options,
                    );
                    process::exit(1);
                }
            },
            Err(_) => {
                print_error_message(
                    &format!(
                        "could not read specified cookie file \"{}\"",
                        opt_cookie_file
                    ),
                    &options,
                );
                process::exit(1);
            }
        }
    }

    match create_monolithic_document(source, &options, &mut cache) {
        Ok(result) => {
            // Define output
            let mut output = Output::new(&destination).expect("could not prepare output");

            // Write result into STDOUT or file
            output.write(&result).expect("could not write output");
        }
        Err(error) => {
            print_error_message(&format!("Error: {}", error), &options);

            exit_code = 1;
        }
    }

    // Clean up (shred database file)
    cache.unwrap().destroy_database_file();

    if exit_code > 0 {
        process::exit(exit_code);
    }
}
