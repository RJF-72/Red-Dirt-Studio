#![allow(non_snake_case)]

mod pipeline;
mod audio_player;
mod apprentice_pipeline;

use std::path::PathBuf;
use dioxus::prelude::*;
use serde::{Serialize, Deserialize};
use futures_channel::mpsc::UnboundedReceiver;
use futures::StreamExt;
use pipeline::{SongInput, StemResult};
use audio_player::SharedPlayer;

const CSS_STYLE: &str = include_str!("styles.css");

#[derive(Clone, Copy, PartialEq, Debug)]
enum View {
    Studio,
    Teleprompter,
    Writer,
    Band,
    Voice,
    Settings,
}

#[derive(Clone, PartialEq, Debug)]
enum PipelineStatus {
    Idle,
    Generating(String),
    Rendering(String),
    Done(String),
    Error(String),
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum TransportState {
    Stopped,
    Playing,
    Recording,
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
struct MixerChannel {
    name: String,
    volume: f64,
    pan: f64,
    mute: bool,
    solo: bool,
    armed: bool,
}

impl Default for MixerChannel {
    fn default() -> Self {
        Self {
            name: String::new(),
            volume: 0.8,
            pan: 0.0,
            mute: false,
            solo: false,
            armed: false,
        }
    }
}

#[derive(Clone)]
enum BandCommand {
    Generate {
        input: SongInput,
    },
    Render {
        output_dir: String,
    },
}

#[derive(Clone)]
enum VoiceCommand {
    Process {
        model_path: String,
        input_path: String,
        output_path: String,
        pitch_shift: f64,
        speed: f64,
    },
}

fn App() -> Element {
    let mut current_view = use_signal(|| View::Studio);
    let band_decision = use_signal(|| None::<pipeline::BandDecision>);
    let stems = use_signal(|| None::<Vec<StemResult>>);
    let pipeline_status = use_signal(|| PipelineStatus::Idle);
    let voice_status = use_signal(|| PipelineStatus::Idle);
    let song_title = use_signal(|| "Untitled Song".to_string());
    let song_artist = use_signal(|| "Artist".to_string());
    let song_bpm = use_signal(|| 120u32);
    let song_key = use_signal(|| "C".to_string());
    let song_sections = use_signal(|| Vec::<(String, u32)>::new());
    let song_chords = use_signal(|| String::new());
    let song_lyrics = use_signal(|| String::new());
    let song_description = use_signal(|| String::new());
    let mut zoom = use_signal(|| 100u32);

    let player: SharedPlayer = use_hook(|| {
        audio_player::create_player().expect("Failed to create audio player")
    });

    // Initialize the apprentice-integrated pipeline (owner mode enabled for now)
    let apprentice_pipeline = use_hook(|| {
        std::sync::Arc::new(tokio::sync::Mutex::new(
            apprentice_pipeline::ApprenticelyIntegratedPipeline::new(true)
        ))
    });

    let pipeline_coroutine = use_coroutine(|mut rx: UnboundedReceiver<BandCommand>| {
        let mut band_decision = band_decision.clone();
        let mut stems = stems.clone();
        let mut pipeline_status = pipeline_status.clone();
        let apprentice_pipeline = apprentice_pipeline.clone();
        async move {
            while let Some(cmd) = rx.next().await {
                match cmd {
                    BandCommand::Generate { input } => {
                        pipeline_status.set(PipelineStatus::Generating("Generating arrangement with apprentice learning...".to_string()));
                        let mut pipeline = apprentice_pipeline.lock().await;
                        match pipeline.generate_with_apprentice(&input).await {
                            Ok((decision, stem_results)) => {
                                band_decision.set(Some(decision));
                                stems.set(Some(stem_results));
                                
                                // Check for pending model approvals
                                let pending = pipeline.check_model_readiness();
                                let status_msg = if pending.is_empty() {
                                    "Arrangement generated! Switch to Studio to render stems.".to_string()
                                } else {
                                    format!("Arrangement generated! Pending approvals: {}. Check settings.", pending.join(", "))
                                };
                                pipeline_status.set(PipelineStatus::Done(status_msg));
                            }
                            Err(e) => {
                                pipeline_status.set(PipelineStatus::Error(format!("Generation failed: {}", e)));
                            }
                        }
                    }
                    BandCommand::Render { output_dir } => {
                        pipeline_status.set(PipelineStatus::Rendering("Rendering stems...".to_string()));
                        if let Some(ref s) = *stems.read() {
                            match pipeline::export_stems(s, &PathBuf::from(&output_dir)).await {
                                Ok(_) => {
                                    pipeline_status.set(PipelineStatus::Done(
                                        format!("Stems exported to {}/", output_dir)
                                    ));
                                }
                                Err(e) => {
                                    pipeline_status.set(PipelineStatus::Error(format!("Render failed: {}", e)));
                                }
                            }
                        } else {
                            pipeline_status.set(PipelineStatus::Error("No arrangement to render. Generate one first.".to_string()));
                        }
                    }
                }
            }
        }
    });

    let voice_coroutine = use_coroutine(|mut rx: UnboundedReceiver<VoiceCommand>| {
        let mut voice_status = voice_status.clone();
        async move {
            while let Some(cmd) = rx.next().await {
                match cmd {
                    VoiceCommand::Process { model_path, input_path, output_path, pitch_shift, speed } => {
                        voice_status.set(PipelineStatus::Generating("Processing voice conversion...".to_string()));
                        match pipeline::run_voice_inference(&model_path, &input_path, &output_path, pitch_shift, speed).await {
                            Ok(msg) => {
                                voice_status.set(PipelineStatus::Done(msg));
                            }
                            Err(e) => {
                                voice_status.set(PipelineStatus::Error(format!("Voice conversion failed: {}", e)));
                            }
                        }
                    }
                }
            }
        }
    });

    rsx! {
        style { {CSS_STYLE} }
        div { class: "app-container", style: format!("zoom: {}%;", zoom()),
            NavBar {
                current_view: current_view,
                on_navigate: move |v| current_view.set(v),
            }
            div { class: "view-container",
                match current_view() {
                    View::Studio => rsx! {
                        StudioView {
                            pipeline_status: pipeline_status(),
                            band_decision: band_decision(),
                            on_render: move |dir| pipeline_coroutine.send(BandCommand::Render { output_dir: dir }),
                            stems_signal: stems,
                            player: player.clone(),
                        }
                    },
                    View::Writer => rsx! {
                        WriterView {
                            title: song_title,
                            artist: song_artist,
                            bpm: song_bpm,
                            song_key: song_key,
                            sections: song_sections,
                            chords_text: song_chords,
                            lyrics_text: song_lyrics,
                            song_description: song_description,
                        }
                    },
                    View::Band => rsx! {
                        BandView {
                            song_title: song_title,
                            song_artist: song_artist,
                            song_bpm: song_bpm,
                            song_key: song_key,
                            song_sections: song_sections,
                            song_chords: song_chords,
                            song_lyrics: song_lyrics,
                            song_description: song_description,
                            on_generate: move |input| pipeline_coroutine.send(BandCommand::Generate { input }),
                            pipeline_status: pipeline_status(),
                        }
                    },
                    View::Voice => rsx! {
                        VoiceView {
                            voice_status: voice_status(),
                            on_process: move |cmd| voice_coroutine.send(cmd),
                        }
                    },
                    View::Teleprompter => rsx! { TeleprompterView {} },
                    View::Settings => rsx! {
                        SettingsView {
                            zoom: zoom,
                            on_zoom_in: move |_| { let z = zoom(); if z < 200 { zoom.set(z + 10); } },
                            on_zoom_out: move |_| { let z = zoom(); if z > 50 { zoom.set(z - 10); } },
                            on_zoom_reset: move |_| zoom.set(100),
                        }
                    },
                }
            }
        }
    }
}

#[component]
fn NavBar(current_view: Signal<View>, on_navigate: EventHandler<View>) -> Element {
    let views = [
        (View::Studio, "Studio"),
        (View::Teleprompter, "Teleprompter"),
        (View::Writer, "Writer"),
        (View::Band, "Band"),
        (View::Voice, "Voice"),
        (View::Settings, "Settings"),
    ];

    rsx! {
        nav { class: "navbar",
            div { class: "navbar-brand",
                img { src: "assets/logo.png", class: "navbar-logo" }
                span { "Red Dirt Studio" }
            }
            div { class: "navbar-links",
                for (view, label) in views {
                    button {
                        class: "nav-btn",
                        class: if current_view() == view { "active" },
                        onclick: move |_| on_navigate.call(view),
                        "{label}"
                    }
                }
            }
        }
    }
}

#[component]
fn StudioView(
    pipeline_status: PipelineStatus,
    band_decision: Option<pipeline::BandDecision>,
    on_render: EventHandler<String>,
    stems_signal: Signal<Option<Vec<StemResult>>>,
    player: SharedPlayer,
) -> Element {
    let mut transport = use_signal(|| TransportState::Stopped);
    let bpm = use_signal(|| 120);
    let current_bar = use_signal(|| 1u32);
    let current_beat = use_signal(|| 1u32);
    let mut output_dir = use_signal(|| "output".to_string());
    let channels = use_signal(|| {
        let names = [
            "Kick", "Snare", "Hi-Hat", "Tom 1", "Tom 2", "Ride",
            "Crash", "Perc", "Bass", "Gtr 1", "Gtr 2", "Gtr 3",
            "Keys", "Strings", "Horns", "Pads", "FX 1", "FX 2",
            "Vox 1", "Vox 2", "BGV 1", "BGV 2", "Aux 1", "Aux 2",
        ];
        names.iter().enumerate().map(|(i, name)| MixerChannel {
            name: name.to_string(),
            volume: 0.8,
            pan: 0.0,
            mute: false,
            solo: false,
            armed: i < 8,
        }).collect::<Vec<_>>()
    });

    let handle_stop = {
        let player = player.clone();
        move |_| player.borrow_mut().stop()
    };
    let handle_toggle_play = {
        let player = player.clone();
        move |_| {
            let is_playing = *transport.read() == TransportState::Playing;
            if is_playing {
                player.borrow_mut().stop();
            } else if let Some(ref stems_list) = *stems_signal.read() {
                if let Some(master) = stems_list.iter().find(|s| s.name.ends_with("_master")) {
                    let buf = &master.buffer;
                    player.borrow_mut().play(
                        buf.data.as_slice().unwrap_or(&[]),
                        buf.sample_rate,
                        buf.channels as u16,
                    );
                }
            }
        }
    };
    let has_arrangement = band_decision.is_some();
    let status_msg = match &pipeline_status {
        PipelineStatus::Generating(m) | PipelineStatus::Rendering(m)
        | PipelineStatus::Done(m) | PipelineStatus::Error(m) => Some(m.clone()),
        _ => None,
    };

    let handle_record = move |_| {
        let cur = *transport.read();
        transport.set(match cur {
            TransportState::Recording => TransportState::Stopped,
            _ => TransportState::Recording,
        });
    };
    rsx! {
        div { class: "studio-view",
            TransportBar {
                transport: transport,
                bpm: bpm,
                current_bar: current_bar,
                current_beat: current_beat,
                on_stop: handle_stop,
                on_toggle_play: handle_toggle_play,
                on_record: handle_record,
            }
            div { class: "render-panel",
                h3 { "Export Stems" }
                div { class: "render-controls",
                    label { "Output:" }
                    input {
                        value: "{output_dir}",
                        oninput: move |e| output_dir.set(e.value()),
                    }
                    button {
                        class: "render-btn",
                        disabled: !has_arrangement,
                        onclick: move |_| on_render.call(output_dir()),
                        "Render & Export Stems"
                    }
                }
                {status_msg.as_ref().map(|msg| {
                    let cls = match pipeline_status {
                        PipelineStatus::Error(_) => "pipeline-status error",
                        PipelineStatus::Done(_) => "pipeline-status done",
                        _ => "pipeline-status",
                    };
                    rsx! { div { class: "{cls}", "{msg}" } }
                })}
            }
            MixerPanel { channels: channels }
        }
    }
}

#[component]
fn TransportBar(
    transport: Signal<TransportState>,
    bpm: Signal<u32>,
    current_bar: Signal<u32>,
    current_beat: Signal<u32>,
    on_stop: EventHandler<()>,
    on_toggle_play: EventHandler<()>,
    on_record: EventHandler<()>,
) -> Element {
    rsx! {
        div { class: "transport-bar",
            div { class: "transport-controls",
                button {
                    class: "transport-btn",
                    class: if transport() == TransportState::Stopped { "active" },
                    onclick: move |_| {
                        transport.set(TransportState::Stopped);
                        on_stop.call(());
                    },
                    "⏹"
                }
                button {
                    class: "transport-btn play-btn",
                    class: if transport() == TransportState::Playing { "active" },
                    onclick: move |_| {
                        let cur = transport();
                        transport.set(match cur {
                            TransportState::Playing => TransportState::Stopped,
                            _ => TransportState::Playing,
                        });
                        on_toggle_play.call(());
                    },
                    if transport() == TransportState::Playing { "⏸" } else { "▶" }
                }
                button {
                    class: "transport-btn rec-btn",
                    class: if transport() == TransportState::Recording { "active" },
                    onclick: move |_| {
                        let cur = transport();
                        transport.set(match cur {
                            TransportState::Recording => TransportState::Stopped,
                            _ => TransportState::Recording,
                        });
                        on_record.call(());
                    },
                    "⏺"
                }
            }
            div { class: "transport-info",
                label { "BPM:" }
                input {
                    class: "bpm-input",
                    value: "{bpm}",
                    r#type: "number",
                    min: "20",
                    max: "300",
                    oninput: move |e| {
                        if let Ok(val) = e.value().parse::<u32>() {
                            bpm.set(val);
                        }
                    },
                }
                span { class: "position-display",
                    "Bar: {current_bar} Beat: {current_beat}"
                }
            }
        }
    }
}

#[component]
fn MixerPanel(channels: Signal<Vec<MixerChannel>>) -> Element {
    rsx! {
        div { class: "mixer-panel",
            div { class: "mixer-scroll",
                for (idx, chan) in channels.iter().enumerate() {
                    MixerChannelView {
                        key: "{idx}",
                        index: idx,
                        channel: chan.clone(),
                        on_update: move |updated| {
                            let mut chs = channels();
                            chs[idx] = updated;
                            channels.set(chs);
                        },
                    }
                }
            }
            div { class: "mixer-master",
                h3 { "Master" }
                div { class: "master-meter", "0 dB" }
            }
        }
    }
}

#[component]
fn MixerChannelView(index: usize, channel: MixerChannel, on_update: EventHandler<MixerChannel>) -> Element {
    let color_class = match index {
        0..=7 => "channel-drum",
        8..=14 => "channel-harmony",
        15..=17 => "channel-fx",
        _ => "channel-vocal",
    };

    let c_vol = channel.clone();
    let c_pan = channel.clone();
    let c_mute = channel.clone();
    let c_solo = channel.clone();

    rsx! {
        div { class: "mixer-channel {color_class}",
            div { class: "channel-label", "{channel.name}" }
            div { class: "channel-faders",
                div { class: "fader-row",
                    label { "Vol" }
                    input {
                        class: "fader",
                        r#type: "range",
                        min: "0",
                        max: "1.5",
                        step: "0.01",
                        value: "{channel.volume}",
                        oninput: move |e| {
                            if let Ok(v) = e.value().parse::<f64>() {
                                let mut c = MixerChannel { name: c_vol.name.clone(), ..c_vol };
                                c.volume = v;
                                on_update.call(c);
                            }
                        },
                    }
                }
                div { class: "fader-row",
                    label { "Pan" }
                    input {
                        class: "fader pan-fader",
                        r#type: "range",
                        min: "-1",
                        max: "1",
                        step: "0.01",
                        value: "{channel.pan}",
                        oninput: move |e| {
                            if let Ok(v) = e.value().parse::<f64>() {
                                let mut c = MixerChannel { name: c_pan.name.clone(), ..c_pan };
                                c.pan = v;
                                on_update.call(c);
                            }
                        },
                    }
                }
            }
            div { class: "channel-buttons",
                button {
                    class: "mute-btn",
                    class: if channel.mute { "active" },
                    onclick: move |_| {
                        let mut c = MixerChannel { name: c_mute.name.clone(), ..c_mute };
                        c.mute = !channel.mute;
                        on_update.call(c);
                    },
                    "M"
                }
                button {
                    class: "solo-btn",
                    class: if channel.solo { "active" },
                    onclick: move |_| {
                        let mut c = MixerChannel { name: c_solo.name.clone(), ..c_solo };
                        c.solo = !channel.solo;
                        on_update.call(c);
                    },
                    "S"
                }
            }
        }
    }
}

#[component]
fn TeleprompterView() -> Element {
    let mut text = use_signal(|| String::new());
    let mut font_size = use_signal(|| 36u32);
    let mut speed = use_signal(|| 1.0f64);
    let mut is_playing = use_signal(|| false);
    let mut scroll_pos = use_signal(|| 0.0f64);

    rsx! {
        div { class: "teleprompter-view",
            div { class: "teleprompter-toolbar",
                label { "Font:" }
                input {
                    r#type: "range",
                    min: "12",
                    max: "72",
                    value: "{font_size}",
                    oninput: move |e| {
                        if let Ok(v) = e.value().parse::<u32>() {
                            font_size.set(v);
                        }
                    },
                }
                label { "Speed:" }
                input {
                    r#type: "range",
                    min: "0.1",
                    max: "5.0",
                    step: "0.1",
                    value: "{speed}",
                    oninput: move |e| {
                        if let Ok(v) = e.value().parse::<f64>() {
                            speed.set(v);
                        }
                    },
                }
                button {
                    class: "transport-btn",
                    onclick: move |_| is_playing.set(!is_playing()),
                    if is_playing() { "⏸ Pause" } else { "▶ Play" }
                }
                button {
                    class: "transport-btn",
                    onclick: move |_| {
                        scroll_pos.set(0.0);
                        is_playing.set(false);
                    },
                    "⏹ Stop"
                }
            }
            div { class: "teleprompter-text",
                textarea {
                    value: "{text}",
                    placeholder: "Paste your lyrics or script here...",
                    style: "font-size: {font_size}px",
                    oninput: move |e| text.set(e.value()),
                }
            }
        }
    }
}

#[component]
fn WriterView(
    mut title: Signal<String>,
    mut artist: Signal<String>,
    mut bpm: Signal<u32>,
    mut song_key: Signal<String>,
    mut sections: Signal<Vec<(String, u32)>>,
    mut chords_text: Signal<String>,
    mut lyrics_text: Signal<String>,
    mut song_description: Signal<String>,
) -> Element {
    let mut time_sig_num = use_signal(|| 4u32);

    rsx! {
        div { class: "writer-view",
            div { class: "writer-sidebar",
                div { class: "writer-metadata",
                    input {
                        placeholder: "Song Title (50 chars max)",
                        maxlength: "50",
                        value: "{title}",
                        oninput: move |e| title.set(e.value()),
                    }
                    input {
                        placeholder: "Artist",
                        value: "{artist}",
                        oninput: move |e| artist.set(e.value()),
                    }
                    label { "Description (2500 chars max):" }
                    textarea {
                        placeholder: "Describe the song... mood, instrumentation, style...",
                        maxlength: "2500",
                        value: "{song_description}",
                        oninput: move |e| song_description.set(e.value()),
                        style: "width: 100%; height: 80px; background: #1a1a1a; border: 1px solid #333; color: #ddd; padding: 6px; border-radius: 3px; resize: vertical; font-family: 'Consolas', monospace; font-size: 11px;",
                    }
                    div { class: "writer-params",
                        label { "BPM:" }
                        input {
                            r#type: "number",
                            value: "{bpm}",
                            min: "20", max: "300",
                            oninput: move |e| {
                                if let Ok(v) = e.value().parse::<u32>() {
                                    bpm.set(v);
                                }
                            },
                        }
                        label { "Key:" }
                        select {
                            value: "{song_key}",
                            oninput: move |e| song_key.set(e.value()),
                            for k in ["C", "G", "D", "A", "E", "B", "F#", "Db", "Ab", "Eb", "Bb", "F", "Am", "Em", "Dm", "Gm", "Cm", "Fm", "Bbm"] {
                                option { value: "{k}", "{k}" }
                            }
                        }
                        label { "Time:" }
                        select {
                            value: "{time_sig_num}/4",
                            oninput: move |e| {
                                if let Some(v) = e.value().split('/').next().and_then(|s| s.parse::<u32>().ok()) {
                                    time_sig_num.set(v);
                                }
                            },
                            for n in [2u32, 3, 4, 5, 6, 7, 8, 9, 12] {
                                option { value: "{n}/4", "{n}/4" }
                            }
                        }
                    }
                }
                div { class: "writer-sections",
                    h3 { "Sections" }
                    for s in ["Intro", "Verse", "Chorus", "Bridge", "Solo", "Outro"] {
                        button {
                            onclick: move |_| {
                                let mut secs = sections();
                                secs.push((s.to_string(), 8));
                                sections.set(secs);
                            },
                            "+ {s}"
                        }
                    }
                }
            }
            div { class: "writer-main",
                div { class: "writer-structure",
                    for (i, item) in sections.iter().enumerate() {
                        div { class: "structure-block",
                            span { "{item.0} ({item.1} bars)" }
                            button {
                                onclick: move |_| {
                                    let mut secs = sections();
                                    secs.remove(i);
                                    sections.set(secs);
                                },
                                "×"
                            }
                        }
                    }
                }
                div { class: "writer-chords",
                    textarea {
                        value: "{chords_text}",
                        placeholder: "Enter chord progression...\nC  G  Am  F",
                        oninput: move |e| chords_text.set(e.value()),
                    }
                }
                div { class: "writer-lyrics",
                    label { "Lyrics (5000 chars max):" }
                    textarea {
                        value: "{lyrics_text}",
                        placeholder: "Enter lyrics...",
                        maxlength: "5000",
                        oninput: move |e| lyrics_text.set(e.value()),
                    }
                }
            }
        }
    }
}

#[component]
fn BandView(
    song_title: Signal<String>,
    song_artist: Signal<String>,
    song_bpm: Signal<u32>,
    song_key: Signal<String>,
    song_sections: Signal<Vec<(String, u32)>>,
    song_chords: Signal<String>,
    song_lyrics: Signal<String>,
    song_description: Signal<String>,
    on_generate: EventHandler<SongInput>,
    pipeline_status: PipelineStatus,
) -> Element {
    let mut preset = use_signal(|| "basic_band".to_string());
    let mut ollama_endpoint = use_signal(|| "http://localhost:11434".to_string());
    let mut model_name = use_signal(|| "qwen2:1.5b".to_string());
    let mut temperature = use_signal(|| 0.7f64);
    let mut generate_drums = use_signal(|| true);
    let mut generate_bass = use_signal(|| true);
    let mut generate_guitar = use_signal(|| true);
    let mut generate_keys = use_signal(|| true);
    let mut generate_strings = use_signal(|| false);
    let mut generate_fiddle = use_signal(|| true);
    let mut arrangement_text = use_signal(|| String::new());
    let is_generating = matches!(pipeline_status, PipelineStatus::Generating(_));

    rsx! {
        div { class: "band-view",
            div { class: "band-controls",
                h2 { "AI Band Generation" }
                div { class: "band-presets",
                    label { "Preset:" }
                    select {
                        value: "{preset}",
                        oninput: move |e| preset.set(e.value()),
                        option { value: "basic_band", "Basic Band" }
                        option { value: "ballad", "Ballad" }
                        option { value: "train_beat", "Train Beat" }
                        option { value: "country_ballad", "Country Ballad" }
                        option { value: "bluegrass", "Bluegrass" }
                    }
                }
                div { class: "band-instruments",
                    h3 { "Instruments" }
                    div { class: "instrument-toggle",
                        label { "Drums" }
                        input {
                            r#type: "checkbox",
                            checked: "{generate_drums}",
                            oninput: move |_| generate_drums.set(!generate_drums()),
                        }
                    }
                    div { class: "instrument-toggle",
                        label { "Bass" }
                        input {
                            r#type: "checkbox",
                            checked: "{generate_bass}",
                            oninput: move |_| generate_bass.set(!generate_bass()),
                        }
                    }
                    div { class: "instrument-toggle",
                        label { "Guitar" }
                        input {
                            r#type: "checkbox",
                            checked: "{generate_guitar}",
                            oninput: move |_| generate_guitar.set(!generate_guitar()),
                        }
                    }
                    div { class: "instrument-toggle",
                        label { "Keys" }
                        input {
                            r#type: "checkbox",
                            checked: "{generate_keys}",
                            oninput: move |_| generate_keys.set(!generate_keys()),
                        }
                    }
                    div { class: "instrument-toggle",
                        label { "Strings" }
                        input {
                            r#type: "checkbox",
                            checked: "{generate_strings}",
                            oninput: move |_| generate_strings.set(!generate_strings()),
                        }
                    }
                    div { class: "instrument-toggle",
                        label { "Fiddle" }
                        input {
                            r#type: "checkbox",
                            checked: "{generate_fiddle}",
                            oninput: move |_| generate_fiddle.set(!generate_fiddle()),
                        }
                    }
                }
                div { class: "band-params",
                    label { "Ollama:" }
                    input {
                        value: "{ollama_endpoint}",
                        oninput: move |e| ollama_endpoint.set(e.value()),
                    }
                    label { "Model:" }
                    input {
                        value: "{model_name}",
                        oninput: move |e| model_name.set(e.value()),
                    }
                    label { "Temperature: {temperature}" }
                    input {
                        r#type: "range",
                        min: "0.1", max: "2.0", step: "0.1",
                        value: "{temperature}",
                        oninput: move |e| {
                            if let Ok(v) = e.value().parse::<f64>() {
                                temperature.set(v);
                            }
                        },
                    }
                }
                button {
                    class: "generate-btn",
                    disabled: is_generating,
                    onclick: move |_| {
                        let mut instruments = Vec::new();
                        if generate_drums() { instruments.push("drums".to_string()); }
                        if generate_bass() { instruments.push("bass".to_string()); }
                        if generate_guitar() { instruments.push("guitar".to_string()); }
                        if generate_keys() { instruments.push("keys".to_string()); }
                        if generate_strings() { instruments.push("strings".to_string()); }
                        if generate_fiddle() { instruments.push("fiddle".to_string()); }
                        on_generate.call(SongInput {
                            title: song_title(),
                            artist: song_artist(),
                            key: song_key(),
                            bpm: song_bpm(),
                            sections: song_sections(),
                            chords: song_chords(),
                            lyrics: song_lyrics(),
                            description: song_description(),
                            instruments,
                            ollama_endpoint: ollama_endpoint(),
                            model_name: model_name(),
                            temperature: temperature(),
                        });
                        arrangement_text.set("Generating arrangement...".to_string());
                    },
                    if is_generating { "Generating..." } else { "Generate Band" }
                }
            }
            div { class: "band-output",
                h3 { "Arrangement" }
                textarea {
                    class: "arrangement-text",
                    readonly: true,
                    value: "{arrangement_text}",
                    placeholder: "AI-generated arrangement will appear here...",
                }
            }
        }
    }
}

#[component]
fn VoiceView(
    voice_status: PipelineStatus,
    on_process: EventHandler<VoiceCommand>,
) -> Element {
    let mut model_path = use_signal(|| "models/svc.pth".to_string());
    let mut input_audio = use_signal(|| String::new());
    let mut output_path = use_signal(|| "output/svc_output.wav".to_string());
    let mut pitch_shift = use_signal(|| 0.0f64);
    let mut speed = use_signal(|| 1.0f64);
    let msg = match &voice_status {
        PipelineStatus::Generating(m) | PipelineStatus::Rendering(m)
        | PipelineStatus::Done(m) | PipelineStatus::Error(m) => Some(m.clone()),
        _ => None,
    };
    let is_processing = voice_status != PipelineStatus::Idle
        && !matches!(voice_status, PipelineStatus::Done(_) | PipelineStatus::Error(_));

    rsx! {
        div { class: "voice-view",
            div { class: "voice-controls",
                h2 { "Voice Inference (So-VITS-SVC)" }
                div { class: "voice-params",
                    label { "Model Path:" }
                    input {
                        value: "{model_path}",
                        oninput: move |e| model_path.set(e.value()),
                    }
                    label { "Input Audio:" }
                    input {
                        value: "{input_audio}",
                        placeholder: "Path to input audio file...",
                        oninput: move |e| input_audio.set(e.value()),
                    }
                    label { "Output Path:" }
                    input {
                        value: "{output_path}",
                        oninput: move |e| output_path.set(e.value()),
                    }
                    label { "Pitch Shift (semitones): {pitch_shift}st" }
                    input {
                        r#type: "range",
                        min: "-12", max: "12", step: "1",
                        value: "{pitch_shift}",
                        oninput: move |e| {
                            if let Ok(v) = e.value().parse::<f64>() {
                                pitch_shift.set(v);
                            }
                        },
                    }
                    label { "Speed: {speed}x" }
                    input {
                        r#type: "range",
                        min: "0.5", max: "2.0", step: "0.1",
                        value: "{speed}",
                        oninput: move |e| {
                            if let Ok(v) = e.value().parse::<f64>() {
                                speed.set(v);
                            }
                        },
                    }
                }
                div { class: "voice-status",
                    {msg.as_ref().map(|m| rsx! { p { "{m}" } })}
                }
                button {
                    class: "generate-btn",
                    disabled: is_processing,
                    onclick: move |_| {
                        on_process.call(VoiceCommand::Process {
                            model_path: model_path(),
                            input_path: input_audio(),
                            output_path: output_path(),
                            pitch_shift: pitch_shift(),
                            speed: speed(),
                        });
                    },
                    if is_processing { "Processing..." } else { "Convert Voice" }
                }
            }
        }
    }
}

#[component]
fn SettingsView(
    zoom: Signal<u32>,
    on_zoom_in: EventHandler<()>,
    on_zoom_out: EventHandler<()>,
    on_zoom_reset: EventHandler<()>,
) -> Element {
    let mut audio_device = use_signal(|| "default".to_string());
    let mut sample_rate = use_signal(|| 44100u32);
    let mut buffer_size = use_signal(|| 256u32);
    let mut midi_input = use_signal(|| String::new());
    let mut midi_output = use_signal(|| String::new());
    let mut ollama_endpoint = use_signal(|| "http://localhost:11434".to_string());
    let mut datasets_path = use_signal(|| "datasets".to_string());
    let mut output_path = use_signal(|| "output".to_string());
    let mut theme = use_signal(|| "dark".to_string());
    let mut autosave = use_signal(|| true);

    rsx! {
        div { class: "settings-view",
            h2 { "Settings" }
            div { class: "settings-grid",
                div { class: "settings-section",
                    h3 { "Audio" }
                    label { "Device:" }
                    input {
                        value: "{audio_device}",
                        oninput: move |e| audio_device.set(e.value()),
                    }
                    label { "Sample Rate:" }
                    input {
                        value: "{sample_rate}",
                        r#type: "number",
                        oninput: move |e| {
                            if let Ok(v) = e.value().parse::<u32>() {
                                sample_rate.set(v);
                            }
                        },
                    }
                    label { "Buffer Size:" }
                    input {
                        value: "{buffer_size}",
                        r#type: "number",
                        oninput: move |e| {
                            if let Ok(v) = e.value().parse::<u32>() {
                                buffer_size.set(v);
                            }
                        },
                    }
                }
                div { class: "settings-section",
                    h3 { "MIDI" }
                    label { "Input:" }
                    input {
                        value: "{midi_input}",
                        oninput: move |e| midi_input.set(e.value()),
                    }
                    label { "Output:" }
                    input {
                        value: "{midi_output}",
                        oninput: move |e| midi_output.set(e.value()),
                    }
                }
                div { class: "settings-section",
                    h3 { "AI" }
                    label { "Ollama Endpoint:" }
                    input {
                        value: "{ollama_endpoint}",
                        oninput: move |e| ollama_endpoint.set(e.value()),
                    }
                }
                div { class: "settings-section",
                    h3 { "Project" }
                    label { "Datasets:" }
                    input {
                        value: "{datasets_path}",
                        oninput: move |e| datasets_path.set(e.value()),
                    }
                    label { "Output:" }
                    input {
                        value: "{output_path}",
                        oninput: move |e| output_path.set(e.value()),
                    }
                    label { "Theme:" }
                    select {
                        value: "{theme}",
                        oninput: move |e| theme.set(e.value()),
                        option { value: "dark", "Dark" }
                        option { value: "light", "Light" }
                        option { value: "country", "Country" }
                    }
                    div { class: "settings-checkbox",
                        label { "Autosave:" }
                        input {
                            r#type: "checkbox",
                            checked: "{autosave}",
                            oninput: move |_| autosave.set(!autosave()),
                        }
                    }
                }
                div { class: "settings-section",
                    h3 { "Display" }
                    label { "Zoom:" }
                    div { style: "display: flex; align-items: center; gap: 8px; margin-top: 4px;",
                        button {
                            class: "zoom-btn",
                            onclick: move |_| on_zoom_out.call(()),
                            "-"
                        }
                        span { style: "color: #d4a84b; font-weight: 700; font-size: 14px; min-width: 48px; text-align: center;",
                            "{zoom}%"
                        }
                        button {
                            class: "zoom-btn",
                            onclick: move |_| on_zoom_in.call(()),
                            "+"
                        }
                        button {
                            class: "zoom-reset-btn",
                            onclick: move |_| on_zoom_reset.call(()),
                            "Reset"
                        }
                    }
                }
            }
        }
    }
}

fn main() {
    LaunchBuilder::desktop().launch(App);
}
