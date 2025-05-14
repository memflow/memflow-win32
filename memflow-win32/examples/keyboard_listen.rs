/*!
This example shows how to use a dynamically loaded connector in conjunction
with memflow-win32. This example uses the `Inventory` feature of memflow
but hard-wires the connector instance into the memflow-win32 OS layer.

The example is an adaption of the memflow core process list example:
https://github.com/memflow/memflow/blob/next/memflow/examples/process_list.rs

# Usage:
```bash
cargo run --release --example process_list -- -vv -c kvm
```
*/
use clap::*;
use log::{info, Level};

use memflow::prelude::v1::*;
use memflow_win32::prelude::v1::*;

pub fn main() -> Result<()> {
    let matches = parse_args();
    let chain = extract_args(&matches)?;

    // create inventory + connector
    let inventory = Inventory::scan();
    let connector = inventory.builder().connector_chain(chain).build()?;

    let os = Win32Kernel::builder(connector)
        .build_default_caches()
        .build()
        .unwrap();

    let mut kb = os.into_keyboard()?;

    // listen for keyboard events until escape is pressed
    while !kb.is_down(vkey::VK_ESCAPE.into()) {
        if kb.is_down(vkey::VK_LSHIFT.into())
        {
            info!("Left Shift is down");
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
        .arg(Arg::new("os").short('o').action(ArgAction::Append))
        .get_matches()
}

fn extract_args(matches: &ArgMatches) -> Result<ConnectorChain<'_>> {
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

    ConnectorChain::new(conn_iter, os_iter)
}
