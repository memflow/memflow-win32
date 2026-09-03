/*!
This example shows how to use a dynamically loaded connector in conjunction
with memflow-win32. This example uses the `Inventory` feature of memflow
but hard-wires the connector instance into the memflow-win32 OS layer.

The example is an adaption of the memflow core process list example:
https://github.com/memflow/memflow/blob/next/memflow/examples/process_list.rs

# Usage:
```bash
cargo run --release --example envar_list -- -vv -c memraw:/path/to/memory.raw
```
*/
use clap::*;
use log::Level;

use memflow::prelude::v1::*;
use memflow_win32::prelude::v1::*;

pub fn main() -> Result<()> {
    let matches = parse_args();
    let chain = extract_args(&matches)?;

    let mut inventory = Inventory::scan();
    let connector = inventory.builder().connector_chain(chain).build()?;

    let os = Win32Kernel::builder(connector)
        .build_default_caches()
        .build()
        .unwrap();

    let mut process = os
        .into_process_by_name("explorer.exe")
        .expect("unable to find process");
    println!("found process: {:?}", process.proc_info.base_info.name);

    println!(
        "\nPID {:>5} | {:<} | sys={:?} proc={:?}",
        process.proc_info.base_info.pid,
        process.proc_info.base_info.name,
        process.proc_info.base_info.sys_arch,
        process.proc_info.base_info.proc_arch
    );
    let envar_list = process
        .envar_list()
        .expect("unable to retrieve environment variables list");

    println!("   VARIABLE | VALUE");

    for ev in envar_list {
        println!("    {}={}", ev.name.as_ref(), ev.value.as_ref());
    }

    Ok(())
}

fn parse_args() -> ArgMatches {
    Command::new("envar_list example")
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
