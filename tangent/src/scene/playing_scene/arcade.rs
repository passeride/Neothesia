use super::{Keyboard, midi_player::MidiPlayer};
use crate::{
    TangentEvent,
    context::Context,
    records::{Record, modified},
    song::{PlayerConfig, Song},
};
use midi_file::midly::MidiMessage;
use sha2::{Digest, Sha256};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tangent_core::arcade::{Chart, Feedback, Grade, Judge, RULES_VERSION, Target, multiplier};

const LEAD: Duration = Duration::from_secs(3);

pub(crate) struct Compiled {
    chart: Chart,
    key: String,
    tracks: Vec<usize>,
    finish: Duration,
}
pub(crate) fn compile(
    song: &Song,
    speed: u32,
    range: std::ops::RangeInclusive<u8>,
) -> Result<Compiled, String> {
    if let Some(path) = &song.source_path {
        let meta = std::fs::metadata(path)
            .map_err(|_| "The MIDI file is unavailable. Return to the gallery and reload it.")?;
        if meta.len() != song.source_bytes || modified(meta.modified().ok()) != song.source_modified
        {
            return Err("The MIDI file changed. Return to the gallery and reload it before starting Arcade.".into());
        }
    }
    if !(250..=2000).contains(&speed) {
        return Err("Arcade speed must be between 25% and 200%.".into());
    }
    let mut targets = Vec::new();
    let mut tracks = Vec::new();
    let mut hash = Sha256::new();
    hash.update(b"tangent-arcade-chart-v1");
    hash.update(RULES_VERSION.to_le_bytes());
    hash.update(speed.to_le_bytes());
    let mut last = Duration::ZERO;
    for track in song.file.tracks.iter() {
        let config = &song.config.tracks[track.track_id];
        hash.update((track.track_id as u64).to_le_bytes());
        hash.update([match config.player {
            PlayerConfig::Human => 1,
            PlayerConfig::Auto => 2,
            PlayerConfig::Mute => 3,
        }]);
        // Hash semantic event data, excluding path/title, visibility and velocity
        // of user input. Changing the backing arrangement creates a new category.
        for event in track
            .events
            .iter()
            .filter(|_| config.player != PlayerConfig::Mute)
        {
            last = last.max(event.timestamp);
            hash.update(event.timestamp.as_nanos().to_le_bytes());
            hash.update([event.channel]);
            hash.update(format!("{:?}", event.message).as_bytes());
        }
        if config.player != PlayerConfig::Human {
            continue;
        }
        if track.notes.iter().any(|n| n.channel == 9) {
            return Err(format!(
                "Track {} contains percussion. Split piano and percussion into separate tracks before Arcade, or use normal playback.",
                track.track_id + 1
            ));
        }
        let mut pitched = false;
        for note in track
            .notes
            .iter()
            .filter(|n| n.channel != 9 && n.velocity > 0)
        {
            if !range.contains(&note.note) {
                return Err(format!(
                    "Track {} includes MIDI {} outside your keyboard range. Adjust range, choose another track, or use normal playback.",
                    track.track_id + 1,
                    note.note
                ));
            }
            pitched = true;
            let at = note
                .start
                .as_micros()
                .checked_mul(1000)
                .and_then(|t| t.checked_div(speed as u128))
                .and_then(|t| i64::try_from(t).ok())
                .ok_or("Song timestamp is too large")?;
            targets.push(Target {
                at,
                pitch: note.note,
            });
            last = last.max(note.end);
        }
        if pitched {
            tracks.push(track.track_id);
        }
    }
    let chart = Chart::new(targets).map_err(str::to_owned)?;
    let key = hash.finalize().iter().map(|b| format!("{b:02x}")).collect();
    let finish = Duration::from_secs_f64(last.as_secs_f64() * 1000.0 / speed as f64)
        + LEAD
        + Duration::from_millis(500);
    Ok(Compiled {
        chart,
        key,
        tracks,
        finish,
    })
}
pub(crate) fn speed(ctx: &Context) -> u32 {
    (ctx.config.speed_multiplier() * 1000.0)
        .round()
        .clamp(250.0, 2000.0) as u32
}

struct Clock {
    anchor: Instant,
    elapsed: Duration,
    paused: bool,
}
impl Clock {
    fn new(now: Instant) -> Self {
        Self {
            anchor: now,
            elapsed: Duration::ZERO,
            paused: false,
        }
    }
    fn at(&self, now: Instant) -> Option<Duration> {
        (!self.paused && now >= self.anchor)
            .then(|| self.elapsed + now.saturating_duration_since(self.anchor))
    }
    fn display(&self, now: Instant) -> Duration {
        self.at(now).unwrap_or(self.elapsed)
    }
    fn pause(&mut self, now: Instant) {
        self.elapsed = self.display(now);
        self.paused = true;
    }
    fn resume(&mut self, now: Instant) {
        self.anchor = now + Duration::from_secs(2);
        self.paused = false;
    }
}

pub(super) struct Session {
    pub judge: Judge,
    clock: Clock,
    speed: u32,
    offset: i64,
    key: String,
    tracks: Vec<usize>,
    finish: Duration,
    pub finished: bool,
    finished_at: Option<Instant>,
    tier_changed_at: Option<Instant>,
    previous_multiplier: u8,
    pub invalid: Option<String>,
    effects: Vec<(Feedback, Instant)>,
    last_feedback: Option<(Feedback, Instant)>,
    old_best: u32,
    saved: bool,
    save_error: Option<String>,
    record: Option<Record>,
    device: Option<String>,
    device_check: Instant,
}
impl Session {
    pub fn new(ctx: &Context, song: &Song) -> Result<Self, String> {
        let speed = speed(ctx);
        let Compiled {
            chart,
            key,
            tracks,
            finish,
        } = compile(song, speed, ctx.config.piano_range())?;
        let old_best = ctx.records.best(&key).map(|r| r.score).unwrap_or(0);
        let now = Instant::now();
        Ok(Self {
            judge: Judge::new(chart),
            clock: Clock::new(now),
            speed,
            offset: ctx.config.arcade_input_offset_ms() as i64 * 1000,
            key,
            tracks,
            finish,
            finished: false,
            finished_at: None,
            tier_changed_at: None,
            previous_multiplier: 1,
            invalid: None,
            effects: Vec::new(),
            last_feedback: None,
            old_best,
            saved: false,
            save_error: None,
            record: None,
            device: ctx.input_manager.connected_name(),
            device_check: now,
        })
    }
    pub fn paused(&self) -> bool {
        self.clock.paused
    }
    fn input_time(&self, at: Instant) -> Option<i64> {
        let elapsed = self.clock.at(at)?;
        Some(calibrated_time(elapsed, self.offset))
    }
    pub fn update(&mut self, ctx: &mut Context, song: &Song) -> Duration {
        let (inputs, now, overflow) = ctx.input_manager.take_scoring_input();
        if overflow {
            self.invalid = Some("Input queue overflow — practice result".into());
        }
        if !self.finished && !self.paused() {
            for input in inputs {
                if input.channel == 9 {
                    continue;
                }
                if let MidiMessage::NoteOn { key, vel } = input.message
                    && vel.as_int() > 0
                    && let Some(at) = self.input_time(input.at)
                    && !self.judge.attack(at, key.as_int())
                {
                    self.invalid =
                        Some("Input timestamps arrived out of order — practice result".into());
                }
            }
            if let Some(at) = self.input_time(now) {
                self.judge.advance(at);
            }
            for feedback in self.judge.take_feedback() {
                self.last_feedback = Some((feedback, now));
                if self.effects.len() == 64 {
                    self.effects.remove(0);
                }
                self.effects.push((feedback, now));
            }
            let tier = multiplier(self.judge.stats.streak);
            if tier > self.previous_multiplier {
                self.tier_changed_at = Some(now);
            }
            self.previous_multiplier = tier;
            if self.judge.finished() && self.clock.display(now) >= self.finish {
                self.finished = true;
                self.finished_at = Some(now);
                self.clock.pause(now);
                if self.invalid.is_none() {
                    let source = song.source_path.clone();

                    self.record = Some(Record {
                        schema: 1,
                        rules: RULES_VERSION,
                        key: self.key.clone(),
                        score: self.judge.score(),
                        accuracy_milli: (self
                            .judge
                            .stats
                            .accuracy(self.judge.chart().target_count())
                            * 1000.0)
                            .round() as u32,
                        full_combo: self.judge.full_combo(),
                        extras: self.judge.stats.extras,
                        tracks: self.tracks.clone(),
                        speed_milli: self.speed,
                        source,
                        source_bytes: song.source_bytes,
                        source_modified: song.source_modified,
                        created: SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_nanos()
                            .min(u64::MAX as u128) as u64,
                    });
                    self.save(ctx);
                }
            }
        }
        self.effects
            .retain(|(_, at)| now.saturating_duration_since(*at) < Duration::from_millis(450));
        let elapsed = self.clock.display(now);
        if elapsed <= LEAD {
            elapsed
        } else {
            LEAD + (elapsed - LEAD).mul_f64(self.speed as f64 / 1000.0)
        }
    }
    fn save(&mut self, ctx: &mut Context) {
        if let Some(record) = self.record.clone() {
            match ctx.records.save(record) {
                Ok(()) => {
                    self.saved = true;
                    self.save_error = None;
                }
                Err(e) => self.save_error = Some(format!("Could not save: {e}")),
            }
        }
    }
    pub fn pause(&mut self, ctx: &Context, player: &mut MidiPlayer) {
        if self.finished || self.paused() {
            return;
        }
        self.clock.pause(Instant::now());
        player.pause();
        self.invalid = Some("Paused or interrupted — practice result".into());
        ctx.input_manager.enable_scoring(true);
    }
    pub fn toggle_pause(&mut self, ctx: &Context, player: &mut MidiPlayer) {
        if self.finished {
            return;
        }
        if self.paused() {
            ctx.input_manager.enable_scoring(true);
            self.clock.resume(Instant::now());
            player.resume();
        } else {
            self.pause(ctx, player);
        }
    }
    pub fn check_device(&mut self, ctx: &Context, player: &mut MidiPlayer) {
        let now = Instant::now();
        if now.duration_since(self.device_check) < Duration::from_secs(1) {
            return;
        }
        self.device_check = now;
        if let Some(device) = &self.device
            && !self.paused()
            && !self.finished
            && !ctx
                .input_manager
                .inputs()
                .iter()
                .any(|p| p.to_string() == *device)
        {
            self.pause(ctx, player);
            self.invalid = Some("MIDI input disconnected — practice result".into());
        }
    }
    pub fn ui(
        &mut self,
        ctx: &mut Context,
        player: &mut MidiPlayer,
        keyboard: &Keyboard,
        ui: &mut nuon::Ui,
    ) {
        let now = Instant::now();
        let w = ctx.window_state.logical_size.width;
        let h = ctx.window_state.logical_size.height;
        nuon::quad().size(w, 94.0).color(ui.theme.surface).build(ui);
        text(
            ui,
            24.0,
            8.0,
            180.0,
            32.0,
            27.0,
            format!("{:07}", self.judge.score()),
        );
        text(
            ui,
            w * 0.34,
            8.0,
            150.0,
            32.0,
            22.0 + if ctx.config.arcade_effects() {
                self.tier_changed_at
                    .map(|at| {
                        (1.0 - now.duration_since(at).as_secs_f32() / 0.4).clamp(0.0, 1.0) * 5.0
                    })
                    .unwrap_or(0.0)
            } else {
                0.0
            },
            format!(
                "{}  ×{}",
                self.judge.stats.streak,
                multiplier(self.judge.stats.streak)
            ),
        );
        text(
            ui,
            w - 290.0,
            8.0,
            160.0,
            32.0,
            21.0,
            format!(
                "{:.1}%",
                self.judge
                    .stats
                    .accuracy((self.judge.stats.hits() + self.judge.stats.misses) as usize)
            ),
        );
        text(
            ui,
            24.0,
            42.0,
            180.0,
            20.0,
            11.0,
            format!("ARCADE · {}% · attack scoring", self.speed as f64 / 10.0),
        );
        text(
            ui,
            w * 0.34,
            42.0,
            170.0,
            20.0,
            11.0,
            format!("COMBO · BEST {}", self.judge.stats.longest_streak),
        );
        text(ui, w - 290.0, 42.0, 160.0, 20.0, 11.0, "ACCURACY".into());
        let streak = self.judge.stats.streak;
        let thresholds = [10u64, 25, 50];
        for (i, threshold) in thresholds.into_iter().enumerate() {
            let before = if i == 0 { 0 } else { thresholds[i - 1] };
            let ratio = ((streak.saturating_sub(before)) as f32 / (threshold - before) as f32)
                .clamp(0.0, 1.0);
            nuon::quad()
                .pos(w * 0.34 + i as f32 * 43.0, 72.0)
                .size(36.0, 3.0)
                .color(ui.theme.border)
                .build(ui);
            nuon::quad()
                .pos(w * 0.34 + i as f32 * 43.0, 72.0)
                .size(36.0 * ratio, 3.0)
                .color(ui.theme.accent)
                .build(ui);
        }
        if let Some((feedback, at)) = self.last_feedback
            && now.duration_since(at) < Duration::from_millis(700)
        {
            let label = match feedback.grade {
                Grade::Good | Grade::Great => format!(
                    "{} · {}",
                    feedback.grade.label(),
                    if feedback.error < 0 { "Early" } else { "Late" }
                ),
                _ => feedback.grade.label().into(),
            };
            nuon::label()
                .pos(24.0, 65.0)
                .size(240.0, 20.0)
                .font_size(14.0)
                .color(colour(feedback.grade))
                .text(label)
                .build(ui);
        }
        let progress =
            (self.clock.display(now).as_secs_f32() / self.finish.as_secs_f32()).clamp(0.0, 1.0);
        nuon::quad()
            .pos(0.0, 91.0)
            .size(w * progress, 3.0)
            .color(ui.theme.accent)
            .build(ui);
        if !self.finished
            && button(
                ui,
                "arcade-pause",
                w - 116.0,
                18.0,
                92.0,
                34.0,
                if self.paused() { "Resume" } else { "Pause" },
            )
        {
            self.toggle_pause(ctx, player);
        }
        if ctx.config.arcade_effects() {
            for (effect, at) in &self.effects {
                let Some(key) = keyboard.layout().keys.iter().find(|k| {
                    k.id() + keyboard.layout().range.start() as usize == effect.pitch as usize
                }) else {
                    continue;
                };
                let age = now.duration_since(*at).as_secs_f32() / 0.45;
                let x = keyboard.pos().x + key.x() + key.width() / 2.0;
                let y = keyboard.pos().y;
                let r = 8.0 + age * 26.0;
                let mut colour = colour(effect.grade);
                colour.a = 1.0 - age;
                nuon::quad()
                    .pos(x - r, y - r)
                    .size(r * 2.0, r * 2.0)
                    .border_radius([r; 4])
                    .color(nuon::Color::new(0., 0., 0., 0.))
                    .border_color(colour)
                    .border_width(2.0)
                    .build(ui);
                if effect.grade == Grade::Perfect {
                    for i in 0..6 {
                        let angle = i as f32 * std::f32::consts::TAU / 6.0;
                        nuon::quad()
                            .pos(x + angle.cos() * r * 1.4, y + angle.sin() * r * 0.7)
                            .size(2.0, 4.0)
                            .color(colour)
                            .build(ui);
                    }
                }
            }
        }
        let waiting = now < self.clock.anchor;
        if self.paused() || self.finished {
            nuon::quad()
                .pos(0., 94.)
                .size(w, (h - 94.).max(0.))
                .color(nuon::Color::new(0.025, 0.04, 0.065, 0.82))
                .build(ui);
            let pw = (w - 48.).min(680.);
            let ph = if self.finished { 386. } else { 220. };
            let x = (w - pw) / 2.;
            let y = ((h - ph) / 2.).max(110.);
            nuon::quad()
                .pos(x, y)
                .size(pw, ph)
                .color(ui.theme.surface)
                .border_radius([20.; 4])
                .border_width(1.)
                .border_color(ui.theme.accent)
                .build(ui);
            if self.finished {
                let score = self.judge.score();
                let stars = [200_000, 400_000, 600_000, 800_000, 950_000]
                    .iter()
                    .filter(|&&v| score >= v)
                    .count();
                let title = if self.judge.perfect_run() {
                    "PERFECT RUN"
                } else if self.judge.full_combo() {
                    "FULL COMBO"
                } else {
                    "PERFORMANCE COMPLETE"
                };
                text(ui, x + 16., y + 14., pw - 32., 30., 19., title.into());
                let shown_score = if ctx.config.arcade_effects() {
                    let progress = self
                        .finished_at
                        .map(|at| (now.duration_since(at).as_secs_f64() / 0.65).clamp(0.0, 1.0))
                        .unwrap_or(1.0);
                    (score as f64 * (1.0 - (1.0 - progress).powi(3))).round() as u32
                } else {
                    score
                };
                text(
                    ui,
                    x + 16.,
                    y + 47.,
                    pw - 32.,
                    53.,
                    43.,
                    format!("{shown_score:07}"),
                );
                text(
                    ui,
                    x + 16.,
                    y + 103.,
                    pw - 32.,
                    25.,
                    18.,
                    format!(
                        "{} / 5 stars · {:.1}% accuracy",
                        stars,
                        self.judge.stats.accuracy(self.judge.chart().target_count())
                    ),
                );
                let s = &self.judge.stats;
                text(
                    ui,
                    x + 16.,
                    y + 136.,
                    pw - 32.,
                    25.,
                    14.,
                    format!(
                        "Perfect {}  ·  Great {}  ·  Good {}",
                        s.perfect, s.great, s.good
                    ),
                );
                text(
                    ui,
                    x + 16.,
                    y + 165.,
                    pw - 32.,
                    25.,
                    14.,
                    format!(
                        "Miss {}  ·  Extra {}  ·  Best combo {}",
                        s.misses, s.extras, s.longest_streak
                    ),
                );
                let mean = if s.hits() > 0 {
                    s.timing_sum_us as f64 / s.hits() as f64 / 1000.
                } else {
                    0.
                };
                text(
                    ui,
                    x + 16.,
                    y + 194.,
                    pw - 32.,
                    25.,
                    13.,
                    format!(
                        "Timing {mean:+.1} ms · Early {} / Late {} · {}/{} notes",
                        s.early,
                        s.late,
                        s.hits(),
                        self.judge.chart().target_count()
                    ),
                );
                let max = s.histogram.iter().copied().max().unwrap_or(1).max(1) as f32;
                for (i, count) in s.histogram.iter().enumerate() {
                    let bar = *count as f32 / max * 26.;
                    nuon::quad()
                        .pos(x + pw / 2. - 90. + i as f32 * 12., y + 248. - bar)
                        .size(8., bar)
                        .color(ui.theme.accent)
                        .build(ui);
                }
                let status = if let Some(reason) = &self.invalid {
                    reason.clone()
                } else if let Some(error) = &self.save_error {
                    error.clone()
                } else if self.saved {
                    if score > self.old_best {
                        format!("NEW PERSONAL BEST · +{}", score - self.old_best)
                    } else {
                        format!("Saved · Personal best {:07}", self.old_best)
                    }
                } else {
                    "Saving record…".into()
                };
                text(ui, x + 16., y + 265., pw - 32., 40., 14., status);
                if self.save_error.is_some()
                    && button(
                        ui,
                        "arcade-save-retry",
                        x + pw / 2. - 50.,
                        y + 307.,
                        100.,
                        26.,
                        "Retry save",
                    )
                {
                    self.save(ctx);
                }
            } else {
                text(ui, x + 16., y + 20., pw - 32., 35., 28., "PAUSED".into());
                text(
                    ui,
                    x + 16.,
                    y + 65.,
                    pw - 32.,
                    50.,
                    14.,
                    self.invalid.clone().unwrap_or_default(),
                );
                if button(
                    ui,
                    "arcade-resume",
                    x + pw / 2. - 70.,
                    y + 124.,
                    140.,
                    34.,
                    "Resume",
                ) {
                    self.toggle_pause(ctx, player);
                }
            }
            let by = y + ph - 47.;
            if button(
                ui,
                "arcade-retry",
                x + 24.,
                by,
                (pw - 60.) / 2.,
                32.,
                "Retry from start",
            ) {
                ctx.proxy
                    .send_event(TangentEvent::Arcade(player.song().clone()))
                    .ok();
            }
            if button(
                ui,
                "arcade-gallery",
                x + 36. + (pw - 60.) / 2.,
                by,
                (pw - 60.) / 2.,
                32.,
                "Gallery",
            ) {
                ctx.proxy
                    .send_event(TangentEvent::MainMenu(Some(player.song().clone())))
                    .ok();
            }
        } else if waiting || self.clock.display(now) < LEAD {
            let seconds = if waiting {
                self.clock.anchor.duration_since(now).as_secs_f32()
            } else {
                (LEAD - self.clock.display(now)).as_secs_f32()
            };
            text(
                ui,
                0.,
                h * 0.36,
                w,
                85.,
                64.,
                format!("{}", seconds.ceil().max(1.) as u32),
            );
            text(
                ui,
                0.,
                h * 0.36 + 85.,
                w,
                30.,
                16.,
                "Get ready · exact pitch and octave".into(),
            );
        }
    }
}
fn calibrated_time(elapsed: Duration, offset: i64) -> i64 {
    (elapsed.as_micros().min(i64::MAX as u128) as i64)
        .saturating_sub(LEAD.as_micros() as i64)
        .saturating_sub(offset)
}
fn colour(grade: Grade) -> nuon::Color {
    match grade {
        Grade::Perfect => [116, 215, 209],
        Grade::Great => [153, 203, 242],
        Grade::Good => [232, 184, 116],
        Grade::Miss | Grade::Extra => [244, 132, 119],
    }
    .into()
}
pub(super) fn text(ui: &mut nuon::Ui, x: f32, y: f32, w: f32, h: f32, size: f32, text: String) {
    nuon::label()
        .pos(x, y)
        .size(w, h)
        .font_size(size)
        .text(text)
        .build(ui);
}
pub(super) fn button(
    ui: &mut nuon::Ui,
    id: &str,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    label: &str,
) -> bool {
    nuon::button()
        .id(id)
        .pos(x, y)
        .size(w, h)
        .label(label.to_owned())
        .border_radius([8.; 4])
        .build(ui)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn calibration_corrects_delayed_input_with_a_positive_offset() {
        assert_eq!(
            calibrated_time(LEAD + Duration::from_millis(1020), 20_000),
            1_000_000
        );
        assert_eq!(
            calibrated_time(LEAD + Duration::from_millis(980), -20_000),
            1_000_000
        );
        assert_eq!(calibrated_time(Duration::ZERO, 0), -3_000_000);
    }
    #[test]
    fn pause_resume_clock_excludes_break_and_reentry_countdown() {
        let now = Instant::now();
        let mut c = Clock::new(now);
        assert_eq!(
            c.at(now + Duration::from_secs(1)),
            Some(Duration::from_secs(1))
        );
        c.pause(now + Duration::from_secs(2));
        assert!(c.at(now + Duration::from_secs(20)).is_none());
        c.resume(now + Duration::from_secs(20));
        assert!(c.at(now + Duration::from_secs(21)).is_none());
        assert_eq!(
            c.at(now + Duration::from_secs(23)),
            Some(Duration::from_secs(3))
        );
    }
    #[test]
    fn chart_identity_ignores_name_but_separates_speed_tracks_and_backing() {
        let mut song = Song::new(midi_file::MidiFile::new("../test.mid").unwrap());
        for t in &mut song.config.tracks {
            t.player = PlayerConfig::Human;
        }
        let a = compile(&song, 1000, 0..=127).unwrap();
        song.file.name = "renamed.mid".into();
        assert_eq!(a.key, compile(&song, 1000, 0..=127).unwrap().key);
        assert_ne!(a.key, compile(&song, 800, 0..=127).unwrap().key);
        assert!(compile(&song, 1000, 60..=60).is_err());
        let mut hidden = song.clone();
        for t in &mut hidden.config.tracks {
            t.visible = false;
        }
        assert_eq!(a.key, compile(&hidden, 1000, 0..=127).unwrap().key);
        let mut mixed = song.clone();
        let tracks = std::sync::Arc::make_mut(&mut mixed.file.tracks);
        let track = tracks.iter_mut().find(|t| !t.notes.is_empty()).unwrap();
        std::sync::Arc::make_mut(&mut track.notes)[0].channel = 9;
        assert!(
            compile(&mixed, 1000, 0..=127)
                .err()
                .unwrap()
                .contains("percussion")
        );

        for t in &mut song.config.tracks {
            t.player = PlayerConfig::Auto;
        }
        assert!(compile(&song, 1000, 0..=127).is_err());
    }
}
