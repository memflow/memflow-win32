/*!
This example shows how to use a dynamically loaded connector in conjunction
with memflow-win32 to read the key-states of a windows computer.
This example uses the `Inventory` feature of memflow but hard-wires the
connector instance into the memflow-win32 OS layer.

The example is an adaption of the memflow core process list example:
https://github.com/memflow/memflow/blob/next/memflow/examples/process_list.rs

# Usage:
```bash
cargo run --release --example keyboard_listen -- -vv -c kvm
```
*/
use clap::*;
use log::Level;

use memflow::prelude::v1::*;
use memflow_win32::{prelude::v1::*, win32::vkey::*};

pub fn main() -> Result<()> {
    let matches = parse_args();
    let (chain, conn_args) = extract_args(&matches)?;

    // create inventory + connector
    let inventory = Inventory::scan();
    let connector = inventory
        .builder()
        .connector_chain(chain)
        .args(conn_args.parse()?)
        .build()?;

    let os = Win32Kernel::builder(connector)
        .build_default_caches()
        .build()
        .unwrap();

    let mut kb = os.into_keyboard()?;
    println!("Running keyboard example...\n...............................");

    println!("Checking all keys...");

    for k in vkey_range(VK_LBUTTON, VK_NONE) {
        let down = if kb.is_down(k.into()) { "down" } else { "up" };
        println!("Key {k} is {down}");
    }

    println!("Starting keyboard listener...");

    println!("Press ESC to exit");
    println!("Press any key to see if it reads out here");
    // listen for keyboard events until escape is pressed
    'listener: loop {
        for k in vkey_range(VK_LBUTTON, VK_NONE) {
            // check escape first each time for responsiveness
            if kb.is_down(vkey::VK_ESCAPE.into()) {
                println!("Escape pressed, exiting...");
                break 'listener;
            }
            if kb.is_down(k.into()) {
                println!("Key {k} is down");
                // sleep for a bit to avoid spamming
                // note: this is for example purposes only, in a real application
                // you would probably not want to block the thread like this
                std::thread::sleep(std::time::Duration::from_millis(200));
            }
        }
    }

    Ok(())
}

fn parse_args() -> ArgMatches {
    Command::new("process_list example")
        .version(crate_version!())
        .author(crate_authors!())
        .arg(Arg::new("verbose").short('v').action(ArgAction::Count))
        .arg(
            Arg::new("connector")
                .short('c')
                .action(ArgAction::Append)
                .required(true),
        )
        .arg(
            Arg::new("connector_args")
                .short('a')
                .action(ArgAction::Append)
                .required(false),
        )
        .arg(Arg::new("os").short('o').action(ArgAction::Append))
        .get_matches()
}

fn extract_args(matches: &ArgMatches) -> Result<(ConnectorChain<'_>, String)> {
    let log_level = match matches.get_count("verbose") {
        0 => Level::Error,
        1 => Level::Warn,
        2 => Level::Info,
        3 => Level::Debug,
        4 => Level::Trace,
        _ => Level::Trace,
    };
    simplelog::TermLogger::init(
        log_level.to_level_filter(),
        simplelog::Config::default(),
        simplelog::TerminalMode::Stdout,
        simplelog::ColorChoice::Auto,
    )
    .unwrap();

    let conn_iter = matches
        .indices_of("connector")
        .zip(matches.get_many::<String>("connector"))
        .map(|(a, b)| a.zip(b.map(String::as_str)))
        .into_iter()
        .flatten();

    let os_iter = matches
        .indices_of("os")
        .zip(matches.get_many::<String>("os"))
        .map(|(a, b)| a.zip(b.map(String::as_str)))
        .into_iter()
        .flatten();

    let conn_args = matches
        .get_one::<String>("connector_args")
        .map(String::to_owned)
        .unwrap_or(String::new());

    Ok((ConnectorChain::new(conn_iter, os_iter)?, conn_args))
}
