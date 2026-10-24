# Biblioteker for Bevy-/web-sporet

Undersøkt 24. oktober 2026. Dette er en teknologiliste, ikke en ferdig testet
Cargo-konfigurasjon. Oppgitte lisenser er pakkens publiserte metadata, ikke en
full revisjon av transitive avhengigheter, eksempelassets eller SoundFonts.

[Hovedplanen](bevy-clean-room.md) beskriver prosess, plattformgrenser og leveranser.
Ingen lokale crates eller wrapperkode fra Neothesia-forken skal brukes i den nye
implementasjonen. Et uavhengig tredjepartsbibliotek kan være kandidat selv om
forken også brukte det; installer fra originalkilden og loggfør opprinnelsen.

## Førstevalg og alternativer

| Oppgave | Kandidat / undersøkt utgave | Dekning og lisensmetadata | Anbefaling |
| --- | --- | --- | --- |
| Motor, ECS, 2D, tekst og UI | [Bevy 0.20.0](https://docs.rs/bevy/0.20.0/bevy/) | Native/WASM; MIT OR Apache-2.0 | Førstevalg. Start med innebygde 2D-/UI-verktøy og bare nødvendige features |
| MIDI-filer og MIDI-meldinger | [midly 0.5.3](https://docs.rs/midly/0.5.3/midly/) | Parser/skriver; Unlicense | Første kandidat for bytes-basert import. Undersøk minimale features uten unødvendig tråding på web; avspillingsplanen skriver vi selv |
| MIDI-enheter | [midir 0.11.1](https://github.com/Boddlnagg/midir) | Native backends og Web MIDI; MIT | Første kandidat bak en liten adapter. Verifiser asynkron tilgang, tillatelser og tidsstempler; bruk direkte web-sys-bro dersom abstraksjonen ikke dekker behovet |
| Browser-API-er | [wasm-bindgen / web-sys](https://wasm-bindgen.github.io/wasm-bindgen/) | Rust/JS- og DOM-bindinger; MIT/Apache-familien | Nødvendig plattformbro. Velg API-features eksplisitt; DOM-objekter skal ikke lekke inn i domenemodeller |
| Pianolyd | [RustySynth 1.3.8](https://github.com/sinshu/rustysynth) | Ren Rust, SoundFont-syntese; MIT | Kandidat for native og separat WASM i AudioWorklet. WASM/latens og feil på ugyldige SoundFonts må testes; ikke en ferdig Bevy-integrasjon |
| Enkle samples/UI-lyder | Bevy audio; eventuelt [bevy_kira_audio 0.27.1](https://github.com/NiklasEi/bevy_kira_audio) | Kira-plugin for Bevy 0.20, webstøtte, MIT OR Apache-2.0 | Bruk én lydvei per formål. Kira er et alternativ for samples, ikke automatisk en MIDI-synth eller presis piano-transport |
| Lasteskjermer og assets | [bevy_asset_loader 0.28.1](https://github.com/NiklasEi/bevy_asset_loader) | Bevy 0.20; MIT OR Apache-2.0 | God kandidat for font/tema/lydpakke og ferdiglastet tilstand; ikke et bibliotek-/databaseprodukt |
| Utviklingspaneler | [bevy_egui 0.43.1](https://github.com/vladbat00/bevy_egui) | Bevy 0.20, native/web; MIT | Nyttig for diagnostikk/kalibreringsspike. Produkt-UI bør få egen visuell utforming; egui er ikke et designvalg alene |
| Enkel animasjon | Bevy-tid/kurver/systemer; [bevy_tweening 0.16.0](https://github.com/djeedai/bevy_tweening) som senere alternativ | Publisert tweening-utgave krever Bevy 0.19; MIT OR Apache-2.0 | Bruk innebygde midler først. Ikke ta med to motorversjoner eller nedgrader motoren for pulser/fades |
| Arcade-partikler | Egne små sprite-effekter; [bevy_hanabi 0.19.0](https://github.com/djeedai/bevy_hanabi) senere | Publisert Hanabi krever Bevy 0.19; MIT OR Apache-2.0; web krever WebGPU/compute | Ikke baseline. Repoets main omtaler 0.20, men det er ikke en verifisert publisert 0.20-kombinasjon. Ha alltid enkel effektvariant |
| Filvelger | HTML File API / [rfd 0.16.1](https://github.com/PolyMeilex/rfd) | rfd: MIT; web/native API må verifiseres | Native-dialog og valgfri webdialog bak adapter. Importøren tar bytes, ikke native paths. Filvelger gir ikke vedvarende mappeovervåking |
| Weblagring | IndexedDB via [rexie 0.6.2](https://docs.rs/rexie/0.6.2/rexie/) | WASM-wrapper; MIT/Apache-2.0 | Kandidat for importerte blobs, metadata og rekorder. Egen native lagringsadapter; små preferanser kan bruke localStorage |
| MusicXML-struktur | [roxmltree 0.21.2](https://github.com/RazrFalcon/roxmltree) | Read-only XML; MIT OR Apache-2.0 | Kandidat for en ny, selvstendig parser basert på W3C MusicXML. Ikke en notasjonsmotor eller full MusicXML-validator |
| MXL-container | [zip 9.1.0](https://github.com/zip-rs/zip2) | ZIP; MIT | Kandidat med begrenset featurevalg. Behandle container.xml, filstier og utpakkingsgrenser uttrykkelig; ingen gammel importer gjenbrukes |
| MusicXML-notegjengivelse | [OpenSheetMusicDisplay](https://github.com/opensheetmusicdisplay/opensheetmusicdisplay) | TypeScript/VexFlow, web/SVG; BSD-3-Clause | Sterk kandidat for webpanelet. Ikke en Bevy-plugin. Krever JS-bro, mapping til avspillingsforekomster og egen native-strategi |
| Musikksymboler | [Bravura](https://github.com/steinbergmedia/bravura) / SMuFL | Uavhengig musikkfont; kontroller OFL-filen i valgt assetutgave | Kandidat for enkel egen Staff. Font gir symboler og metrikk, ikke automatisk rytmisk spacing eller gravering |
| WASM-pakking | [Trunk](https://github.com/trunk-rs/trunk) | MIT OR Apache-2.0 | Første bygg-/serve-kandidat. Lås utgitt verktøyversjon og test worklet-/JS-assets og relativ deploy-sti |

Versjoner for andre støttepakker (serde, feiltyper, hashing, native lydbackend og
nettlesertester) velges først i CR-01 med kontrollert avhengighetsgraf. Unngå å
låse dem til den gamle forkens Cargo.lock. RustySynth bruker egen SoundFont-
innlasting; test kun uavhengig anskaffet og lisensiert lydinnhold. Eksisterende
standard.sf2/default.sf2 skal ikke følge med bare fordi den finnes i forken.

## Hvorfor ikke bevy_midix som grunnmur?

[bevy_midix 3.3.0](https://docs.rs/crate/bevy_midix/3.3.0/source/Cargo.toml.orig)
annonserer MIDI-input og SoundFont-lyd og en web-feature, men den publiserte
manifestfilen krever **Bevy 0.16.0**. Den inkluderer også sin egen MIDI-/lyd-
integrasjon og andre avhengigheter som må evalueres. Det er ikke dokumentasjon på
at den fungerer uendret med Bevy 0.20 eller oppfyller våre timingkrav.

Den er relevant å kjenne til, men ikke et førstevalg. Bruk `midir`, `midly` og
synth som uavhengige deler bak små adaptere. Dette gir kontroll på tillatelser,
tidsstempler og lydtråden uten å vedlikeholde en port av en stor plugin tidlig.

## Kompatibilitet som faktisk er undersøkt

Publiserte crates.io-avhengigheter ble lest for disse konkrete versjonene:

| Pakke | Publisert Bevy-krav | Vurdering mot 0.20 |
| --- | --- | --- |
| bevy_egui 0.43.1 | `^0.20.0` | Passer deklarert motorserie |
| bevy_asset_loader 0.28.1 | `^0.20.0` | Passer deklarert motorserie |
| bevy_kira_audio 0.27.1 | `^0.20` | Passer deklarert motorserie |
| bevy_tweening 0.16.0 | `^0.19` | Vent / bruk innebygd animasjon |
| bevy_hanabi 0.19.0 | `^0.19` | Vent / bruk sprite-effekter |
| bevy_midix 3.3.0 | `^0.16.0` | Ikke baseline |

Dette er **manifestkontroll, ikke en kompilert og prøvd samlet stack**. Motorens
nyeste utgave og en plugins nyeste utgave er ikke automatisk kompatible.
Lås eksakte utgaver i lockfile etter CR-01; oppgrader i egne endringer med native-
og webtester. Ikke bygg på main-grener bare fordi README sier «latest».

## Avhengighetsgodkjenning i det nye prosjektet

For hver faktisk valgt utgave: noter originalrepo, pakke/hash, direkte/transitive
lisenser, aktiverte features, notices og medfølgende assets. Gjør en wasm-byggtest
og en nettlesertest før pakken blir baseline. En deklarert tillatende lisens er
ikke dokumentasjon på at alt som følger med er kontrollert.

Kjør lisens-/advisory-kontroll med eksempelvis cargo-deny, men gjennomgå også JS-
pakker, font- og lydfiler. Et automatisk verktøy alene avgjør ikke clean room-status.
Ikke kopier implementasjonsmønstre fra Neothesia for å koble bibliotekene sammen.

## Kilder og etterprøving

Lenkene i tabellene peker til primærprosjekter/pakkedokumentasjon. Eksakte
manifestkrav og lisensmetadata kan etterprøves via crates.io sitt API:
`https://crates.io/api/v1/crates/{pakke}/{versjon}` og tilsvarende
`.../{versjon}/dependencies`. Undersøkelsen er datert; nye utgaver må kontrolleres
på nytt når implementeringen starter.

Supplerende primærkilder:

- [Bevy 0.20 release](https://bevy.org/news/bevy-0-20/).
- [Web MIDI](https://www.w3.org/TR/webmidi/) og [midir-backends](https://github.com/Boddlnagg/midir#features).
- [Web Audio](https://www.w3.org/TR/webaudio/) og [wasm-bindgen-worklet-demo](https://rustwasm.github.io/docs/wasm-bindgen/examples/wasm-audio-worklet.html).
- [IndexedDB](https://www.w3.org/TR/IndexedDB-3/) og [File System Access](https://wicg.github.io/file-system-access/).
- [MusicXML](https://www.w3.org/2021/06/musicxml40/).
- [OSMD-lisensen](https://github.com/opensheetmusicdisplay/opensheetmusicdisplay/blob/develop/LICENSE).
