# Teknisk gjeld og refaktorering

Oppdatert 23. oktober 2026, etter MusicXML-importen (`d69d3e9`).
Dette er en prioritert arbeidsliste. Ingen av refaktoreringene nedenfor er ferdige.
[Format- og læringsplanen](docs/plans/formats-and-learning.md) beskriver produktretningen;
[ADR-ene](docs/adr/index.md) beskriver valgene som skal styre implementeringen.

Prioritet: **P0** før bredere MusicXML-/notasjonsstøtte, **P1** neste arkitekturrunde,
**P2** når berørte områder videreutvikles. ID-ene skal beholdes når punktene flyttes.

## P0 — felles musikkmodell (ARCH-01)

- [ ] Skill skrevet musikk fra avspillingshendelser og Arcade-mål, jf. [ADR-0001](docs/adr/0001-music-model.md).
- [ ] Definer musikalsk posisjon/varighet med eksakt representasjon, tempokart og stabile referanser mellom kildenoter, skrevne segmenter og klingende anslag. Unngå at en fast MIDI-oppløsning bestemmer hvilke rytmer modellen kan uttrykke.
- [ ] Innfør en formatnøytral lastet dokumenttype ved siden av dagens `Song`/`MidiFile`; flytt spor-/visningsvalg ut av dokumentinnholdet. MIDI skal kunne mangle autoritativ skrevet notasjon.
- [ ] La MusicXML bevare strukturert stavemåte, pauser, stemmer, notelinjer, taktart, toneart og nøkler. Behold original XML som kilde, ikke som eneste representasjon av disse egenskapene.
- [ ] La Staff lese skrevet musikk når den finnes, med tydelig merket generering som alternativ for MIDI. Unngå rekonstruksjon fra MIDI når autoritative MusicXML-data allerede finnes.

**Hvorfor:** `midi-file/src/musicxml.rs` lager i dag et syntetisk SMF; `MidiFile.score`
beholder skrevne data ved siden av. `ScoreOverlay::new` bygger likevel notasjonen
fra MIDI-tonehøyder og sekunder. Informasjonen får dermed ingen gjennomgående vei
fra import til visning.

**Ferdig når:** begge importører leverer til formatnøytrale modeller; en test viser
at ciss/dess, pauser og to notelinjer fra MusicXML når visningsmodellen uten å bli
gjettet på nytt. En bundet tone har flere skrevne segmenter, men ett Arcade-anslag.
Eksisterende MIDI-/timing-/poengtester skal fortsatt passere.

## P1 — stabil identitet for rekorder (ARCH-02)

- [ ] Erstatt `format!("{:?}", event.message)` i Arcade-hashen med eksplisitt, versjonert koding, jf. [ADR-0002](docs/adr/0002-record-identity.md).
- [ ] Dokumenter felt, sortering, tidsenhet, tagger og lengdekoding. Skill identitetskodingens versjon fra poengreglenes versjon.
- [ ] Lag faste testvektorer, inkludert en stabil hash med forventet verdi og tilfeller for hastighet, menneske-/bakgrunns-/mute-valg og innholdsendringer.
- [ ] Test at titler, omslag, filnavn, flytting og visningsvalg ikke endrer kategorien. Ekskluder ikke musikalske endringer som faktisk påvirker gjennomspillingen.
- [ ] Hold gamle og nye identiteter adskilt. Ingen migrering eller bakoverkompatibilitet er påkrevd; gamle resultater skal aldri få en ny identitet uten verifisering.

**Hvorfor:** `compile` i `playing_scene/arcade.rs` bruker en avhengighets
Debug-representasjon som del av en varig lagringskontrakt.

**Ferdig når:** rekordnøkkelen kan testes uten UI og ikke avhenger av Debug-tekst,
filplassering eller tilfeldig samlingsrekkefølge. Gjeldende kategorier og
versjonsovergang er dokumentert.

## P1 — Arcade-kompilering og sesjon ut av UI (ARCH-03)

- [ ] Flytt validering/kompilering av spillemål og identitet ut av `playing_scene/arcade.rs`, jf. [ADR-0003](docs/adr/0003-domain-and-ui-boundaries.md).
- [ ] Skill sesjonstilstand fra HUD, effekter og resultatskjerm. La tid og input tilføres eksplisitt; la lagring og enhetshåndtering skje via appens adaptere.
- [ ] Gjør nedtelling, spilling, pause/gjeninntelling og resultat til tydelige tilstander; skill rekordberettigelse og lagringsstatus fra grafiske animasjoner.
- [ ] Bevar eksisterende monotone tidsgrunnlag og input-watermark. Dokumenter omregningen mellom inputtid, sangtid, kalibrering og hastighet; ikke innfør en uavhengig poengklokke i UI-et.
- [ ] Test overgangene uten GPU: ferdig runde, pause, retry, fokus-/enhetstap, ugyldig input og lagringsfeil med nytt forsøk. Behold tester for samme vurdering ved ulike bildefrekvenser.

**Hvorfor:** poengmotoren i `tangent-core/src/arcade.rs` er allerede adskilt, men
skjermens `Session` håndterer også rekordbygging/-lagring, klokke, enhetssjekk,
UI, effekter og navigasjon.

**Ferdig når:** kompilering og sesjon kan testes uten `Context`, vindu, GPU eller
filsystem. UI viser tilstand og sender handlinger; adapterne utfører I/O.
Poenggrenser og avbruddsregler endres ikke som del av oppryddingen.

## P1 — trinnvis MusicXML-import og strukturerte feil (ARCH-04)

Avhenger av den første modellavklaringen i ARCH-01, men fil-/containerlaget kan
skilles ut tidligere.

- [ ] Del containerlesing, XML-tolkning, musikalsk validering og avspillingskompilering i egne moduler.
- [ ] Innfør importfeiltyper som skiller ugyldig fil, ressursgrense og gyldig, men foreløpig ustøttet musikkstruktur. Ta med del, takt og relevant element når det finnes.
- [ ] Samle eksplisitte støttebegrensninger; bevar grenser for størrelse/noder/noter og forbud mot eksterne entiteter og usikre arkivstier.
- [ ] Utvid eksempelsamlingen med distribuerbare eksporter fra flere noteprogrammer. Legg til repetisjoner og andre nye strukturer først når deres avspillingsrekkefølge er representert og testet.

**Hvorfor:** dagens MusicXML-modul gjør alle disse oppgavene og returnerer
`String`-feil. Flere støttede elementer vil gjøre én stor importfunksjon vanskeligere
å verifisere. Dette følger importgrensen i ADR-0001; egen ADR trengs først hvis
vi senere velger en annen parser-/utvidelsesstrategi.

**Ferdig når:** container, musikkstruktur og avspilling har egne testgrenser,
og UI kan vise en presis importfeil uten å tolke innholdet i en tekststreng.

## P2 — musikkjerne uten grafikkavhengigheter (ARCH-05)

- [ ] Skill modeller, akkordlogikk, vurdering og nødvendige tidsberegninger fra rendering og vindustilstand. Bruk modulgrenser først; trekk ut en liten crate når det gir en faktisk avhengighetsgrense, jf. ADR-0003.
- [ ] Flytt WGPU-/glyphon-/fontavhengigheter til presentasjonslaget. Unngå en ny samling generelle tjenester uten konkrete brukere.
- [ ] Kjør domenetestene med ordinær Cargo-kommando uten grafikk-/lydenheter; erstatt de direkte `rustc --test`-omveiene i CI når crate-grensen gjør dem overflødige.

**Hvorfor:** `tangent-core` inneholder både poeng/akkorder og rendering og trekker
inn WGPU og tekstmotoren, selv om mye av domenelogikken ikke trenger dem.

**Ferdig når:** de rene musikk-/poengtestene kan bygges uten grafikk- og lydbackend.
Appens renderingskode bruker den samme domenelogikken som testene.

## P2 — bibliotekets tilstand, lasting og visning (ARCH-06)

- [ ] Skill katalog-/søk-/sorterings-/sideinndeling fra UI-tegning og filvelgere.
- [ ] Samle lasting fra bibliotek, filvelger, kommandolinje og sist åpnet fil bak en formatnøytral lastetjeneste. Bevar synlige feil og MusicXML-informasjonen ved åpning.
- [ ] Definer eierskap til asynkrone forespørsler og foreldede svar, slik at et sent svar ikke kan overstyre et nyere mappe-/filvalg.
- [ ] Behold metadata-skanning uten full import av alle filer; ikke les alle MusicXML-/MIDI-filer bare for å tegne omslag eller rekorder.

**Hvorfor:** `menu_scene/library.rs` blander katalogstyring, asynkron lasting,
filvelgere, søk og korttegning. `Song::from_env` har en annen lastesti som blant
annet gjør lastefeil om til `None`.

**Ferdig når:** tilstand og lasting kan testes uten vindu, og alle innganger bruker
samme import-/feilpolicy. Ingen endring i bibliotekets lagrede mappe eller søkeregler.

## Foreslått gjennomføring

1. Avklar modellene og skriv karakteriseringstester for eksisterende oppførsel (ARCH-01/03).
2. Lever stabil rekordidentitet som en liten separat endring (ARCH-02).
3. Flytt Arcade-kompilering og sesjon med eksisterende atferd som fasit (ARCH-03).
4. Lever musikkmodell, importtrinn og tilkobling til Staff i gjennomgående, testbare deler (ARCH-01/04).
5. Ta avhengighetsgrensen og biblioteket når de berøres videre (ARCH-05/06).

P0 betyr at ARCH-01 skal avklares før mer kompleks notasjon bygges oppå dagens
MIDI-omvei; hele notegjengivelsen trenger ikke være ferdig før de mindre P1-grepene.

Behold dagens lokale rekordlagring inntil et konkret skalerings-/søkeproblem
begrunner noe annet. Database, pluginrammeverk og total omskriving er ikke del av
runden. Fysisk piano, pedal, forsinkelse og Windows-GUI trenger fortsatt egen
verifikasjon; en refaktorering gjør ikke disse punktene automatisk ferdige.
