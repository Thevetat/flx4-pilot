mod bridge;
mod mapping;
mod profile;

use anyhow::{anyhow, bail, Context, Result};
use bridge::Bridge;
use clap::Parser;
use midir::{Ignore, MidiInput, MidiOutput};
use profile::Profile;
use std::{
    io::{ErrorKind, Write},
    net::{SocketAddr, UdpSocket},
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Arc,
    },
    time::{Duration, Instant},
};

#[derive(Parser, Debug)]
#[command(version, about = "DDJ-FLX4 → AUTO/PILOT OSC bridge")]
struct Options {
    /// List MIDI ports without opening them.
    #[arg(long)]
    list: bool,
    /// Read-only MIDI log. No MIDI output, keepalive or OSC traffic.
    #[arg(long, conflicts_with = "list")]
    monitor: bool,
    /// Input name substring or #index from --list. Must match exactly one port.
    #[arg(long, default_value = "DDJ-FLX4")]
    input: String,
    /// Output name substring or #index from --list. Must match exactly one port.
    #[arg(long, default_value = "DDJ-FLX4")]
    output: String,
    #[arg(long, default_value = "127.0.0.1:9090")]
    osc: SocketAddr,
    #[arg(long, default_value = "127.0.0.1:9096")]
    feedback: SocketAddr,
    /// JSON pad profile. Defaults are embedded in the executable.
    #[arg(long)]
    config: Option<PathBuf>,
    /// Export the default profile to a NEW file, without opening MIDI.
    #[arg(long)]
    export_config: Option<PathBuf>,
    /// Bypass tempo pickup (still -1..+1 OSC). Fader movement can jump tempo.
    #[arg(long)]
    direct_tempo: bool,
    /// Compatibility flag: mixer controls are enabled by default, with pickup.
    #[arg(long, hide = true)]
    mixer: bool,
    /// Log incoming MIDI and outgoing OSC.
    #[arg(long)]
    verbose: bool,
}

fn choose(names: &[String], selector: &str) -> Result<usize> {
    if let Some(index) = selector.strip_prefix('#') {
        let index: usize = index
            .parse()
            .context("Port index must be # followed by a number")?;
        if index < names.len() {
            return Ok(index);
        }
        bail!("Port {selector} does not exist. Use --list");
    }
    let matches: Vec<_> = names
        .iter()
        .enumerate()
        .filter(|(_, name)| name.to_lowercase().contains(&selector.to_lowercase()))
        .map(|(index, _)| index)
        .collect();
    match matches.as_slice() {
        [index] => Ok(*index),
        [] => bail!("No MIDI port matches {selector:?}. Use --list; close apps holding the port"),
        _ => bail!("Multiple MIDI ports match {selector:?}. Choose an explicit #index from --list"),
    }
}

fn main() -> Result<()> {
    let options = Options::parse();
    if let Some(path) = &options.export_config {
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
            .context("Export requires a new file")?;
        writeln!(
            file,
            "{}",
            serde_json::to_string_pretty(&Profile::default())?
        )?;
        println!("Wrote {}", path.display());
        return Ok(());
    }
    let mut input = MidiInput::new("FLX4 Pilot input")?;
    input.ignore(Ignore::None);
    let inputs = input.ports();
    let input_names = inputs
        .iter()
        .map(|p| input.port_name(p))
        .collect::<std::result::Result<Vec<_>, _>>()?;
    let output = MidiOutput::new("FLX4 Pilot output")?;
    let outputs = output.ports();
    let output_names = outputs
        .iter()
        .map(|p| output.port_name(p))
        .collect::<std::result::Result<Vec<_>, _>>()?;
    if options.list {
        for (kind, names) in [("IN", &input_names), ("OUT", &output_names)] {
            for (i, name) in names.iter().enumerate() {
                println!("{kind} #{i}: {name}");
            }
        }
        return Ok(());
    }
    let profile = Profile::load(options.config.as_deref())?;
    let index = choose(&input_names, &options.input)?;
    let running = Arc::new(AtomicBool::new(true));
    let signal = running.clone();
    ctrlc::set_handler(move || signal.store(false, Ordering::Relaxed))?;
    let (sender, receiver) = mpsc::sync_channel(4096);
    let overflow = Arc::new(AtomicBool::new(false));
    let callback_overflow = overflow.clone();
    let connection = input
        .connect(
            &inputs[index],
            "FLX4 Pilot",
            move |_, bytes, _| {
                if sender.try_send(bytes.to_vec()).is_err() {
                    callback_overflow.store(true, Ordering::Relaxed);
                }
            },
            (),
        )
        .map_err(|e| anyhow!("Open MIDI input: {e}"))?;
    println!("FLX4 Pilot {}", env!("CARGO_PKG_VERSION"));
    println!("Input: {}", input_names[index]);

    if options.monitor {
        println!("Read-only monitor. Ctrl+C exits. No output ports opened.");
        while running.load(Ordering::Relaxed) {
            if overflow.load(Ordering::Relaxed) {
                bail!("MIDI queue overflow; capture incomplete");
            }
            match receiver.recv_timeout(Duration::from_millis(20)) {
                Ok(bytes) => println!("{:02X?}", bytes),
                Err(mpsc::RecvTimeoutError::Timeout) => {}
                Err(mpsc::RecvTimeoutError::Disconnected) => bail!("MIDI input closed"),
            }
        }
        return Ok(());
    }
    // Bind before sending any MIDI so a second bridge cannot change modes on a busy socket.
    let osc = UdpSocket::bind(options.feedback)
        .context("Feedback port busy; stop the old bridge first")?;
    osc.set_nonblocking(true)?;
    let index = choose(&output_names, &options.output)?;
    let midi = output
        .connect(&outputs[index], "FLX4 Pilot feedback")
        .map_err(|e| anyhow!("Open MIDI output: {e}"))?;
    let mut bridge = Bridge::new(
        profile,
        midi,
        osc,
        options.osc,
        options.direct_tempo,
        options.verbose,
    );
    println!(
        "Output: {}\nOSC → {} | feedback ← {}",
        output_names[index], options.osc, options.feedback
    );
    println!("Disable FLX4 MIDI input in AUTO/PILOT. OSC Feedback: Toggle All on (including TempoSlider).");
    println!(
        "Shift+LOAD selects 1/3 or 2/4. Mixer/tempo pickup is active by default. Ctrl+C exits."
    );
    println!(
        "Tempo requires AP's -1..+1 input and position-feedback API; legacy 0..1 is not supported."
    );
    if options.direct_tempo {
        eprintln!("Direct tempo enabled: no pickup; fader movement can jump tempo.");
    } else {
        println!("Tempo waits for feedback and pickup. Leave --direct-tempo off to avoid jumps on deck selection.");
    }
    let result = run(&mut bridge, &receiver, &running, &overflow);
    drop(connection);
    bridge.cleanup();
    result
}

fn run(
    bridge: &mut Bridge,
    receiver: &mpsc::Receiver<Vec<u8>>,
    running: &AtomicBool,
    overflow: &AtomicBool,
) -> Result<()> {
    bridge.start()?;
    let mut keepalive = Instant::now() - Duration::from_secs(1);
    let mut osc_seen = false;
    let mut feedback_seen = false;
    let mut feedback_warning = false;
    let started = Instant::now();
    let mut buffer = [0u8; 65535];
    while running.load(Ordering::Relaxed) {
        if overflow.load(Ordering::Relaxed) {
            bail!("MIDI queue overflow; stopping to avoid stuck controls");
        }
        if keepalive.elapsed() >= Duration::from_millis(200) {
            bridge.keepalive()?;
            keepalive = Instant::now();
        }
        match receiver.recv_timeout(Duration::from_millis(5)) {
            Ok(bytes) => bridge.input(&bytes)?,
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => bail!("MIDI input closed"),
        }
        for bytes in receiver.try_iter().take(255) {
            bridge.input(&bytes)?;
        }
        for _ in 0..32 {
            match bridge.osc.recv_from(&mut buffer) {
                Ok((length, source)) if source.ip() == bridge.destination.ip() => {
                    if let Ok((remaining, packet)) = rosc::decoder::decode_udp(&buffer[..length]) {
                        if remaining.is_empty() {
                            osc_seen = true;
                            feedback_seen |= bridge.feedback(packet, 0)?;
                        }
                    }
                }
                Ok(_) => {}
                Err(e) if e.kind() == ErrorKind::WouldBlock => break,
                Err(e)
                    if matches!(
                        e.kind(),
                        ErrorKind::ConnectionReset | ErrorKind::ConnectionRefused
                    ) =>
                {
                    break
                }
                Err(e) => return Err(e.into()),
            }
        }
        if !feedback_seen && !feedback_warning && started.elapsed() >= Duration::from_secs(5) {
            if osc_seen {
                eprintln!("OSC packets received, but no usable deck/mixer feedback. Enable OSC Feedback subscriptions (Toggle All).");
            } else {
                eprintln!("No OSC packets received from AP. Check the feedback destination IP/port and enable subscriptions (Toggle All).");
            }
            feedback_warning = true;
        }
    }
    Ok(())
}
