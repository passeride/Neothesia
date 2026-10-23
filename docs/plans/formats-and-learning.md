# MIDI, notasjon og veien videre for Tangent

Dato: 23. oktober 2026. Status: **retning og utviklingsplan**, ikke en liste over ferdig implementerte funksjoner.

Tangent skal gjøre det lett å koble til et piano, forstå hva man spiller og øve målrettet. Arcade skal gjøre framgang motiverende, mens vanlig avspilling og endeløs note- og akkordøving fortsatt skal være nyttige uten poeng.

Formatretningen er **MIDI for framføring, MusicXML for skrevet notasjon og en liten JSON-fil for Tangent-spesifikke metadata og arrangementer**. Vi skal bevare vanlige pianotoner og kunne åpne en enkelt MIDI-fil uten en innholdspakke. Clone Hero er inspirasjon for spillfølelse; formatene og gitarspor-konvensjonene deres skal ikke implementeres.

## Når er MIDI tilstrekkelig?

Standard MIDI Files lagrer tidsfestede hendelser. NoteOn og NoteOff beskriver blant annet tonehøyde, anslag og hvor lenge tangenten holdes. Tempo, taktart, toneart, tekst og spornavn kan også finnes i filen. At formatet tillater metadata, betyr ikke at en bestemt fil inneholder dem, eller at Tangent bruker dem. [1]

| Behov | Hva MIDI gir | Hva som må suppleres eller tolkes |
| --- | --- | --- |
| Fallende noter og avspilling | Tonehøyde, tidspunkt, varighet, velocity og kontrollhendelser | Lydmotor og instrumentvalg; MIDI er ikke et lydopptak |
| Arcade med riktig tangent og timing | Nøyaktige måltidspunkter og tonehøyder | Regler, kalibrering og valg av hvilke spor spilleren skal svare på |
| Akkordgjenkjenning | Samlingen av klingende/spilte toner | Akkordnavn, grunntone og alternativ harmonisk tolkning |
| Enkelt notebilde | Tonehøyder og tidsforløp | Stavemåte, rytmisk kvantisering, stemmer og layout |
| Et godt noteark | Noen nødvendige grunndata | Ofte mangler pauser, buer, noteverdier, fingersetting og eksplisitt fordeling mellom notelinjer |
| Høyre og venstre hånd | Spor eller kanaler kan være delt etter hånd | Håndfordeling må være erklært eller foreslått og bekreftet; spor er ikke automatisk hender |
| Lett, middels og vanskelig | Separate filer eller eksplisitt valgte spor kan inneholde ulike arrangementer | Forholdet mellom arrangementene og nivåene må erklæres |
| Bibliotek og omslag | Filnavn og eventuelle tekstfelt | Pålitelig tittel, komponist, arrangør, kilde, lisens, omslag og stabile arrangement-ID-er |

En noteprogram-eksport med ryddig rytme, tempokart og stemmeinndeling er et bedre grunnlag for notasjon enn et fritt pianoopptak. Det er likevel ingen garanti for at den skrevne notasjonen kan rekonstrueres riktig.

I et framføringsopptak kan tonene i en akkord begynne noen millisekunder fra hverandre. Rubato og ujevne anslag er en del av framføringen, men sier ikke entydig hvilke noteverdier som var ment. Et opptak uten et tilpasset tempokart kan kreve egen taktslagstolkning. Vi skal ikke stille endre kilden for å få den til å passe et rutenett.

MIDI-tone 61 kan for eksempel være ciss eller dess. En eventuell toneartsangivelse hjelper, men bestemmer ikke stavemåten til hver enkelt tone. På samme måte er lengden mellom NoteOn og NoteOff ikke alltid den skrevne noteverdien, og pedal kan la lyden fortsette etter at tangenten er sluppet.

**Konsekvens for Arcade:** en framførings-MIDI gir en oppgave i å etterligne den framføringens anslag. Den er ikke automatisk et pedagogisk rytmefasitsvar. En senere kvantisert eller forenklet utgave må merkes som et eget, avledet arrangement med egne rekorder. Originalen skal bevares.

## Hva støtter appen nå?

- Bibliotek og filvelger åpner `.mid` og `.midi`.
- MIDI-parseren bruker metrisk tick-oppløsning og leser tempoendringer. SMPTE/timecode-timing avvises i dag.
- Taktmarkeringene beregnes foreløpig i grupper på fire firedeler i `midi-file/src/file.rs`. En taktartsangivelse i filen er dermed ikke full støtte for andre taktarter i appen. Toneartsbasert stavemåte er heller ikke en komplett importfunksjon.
- Uavhengige sekvenser i SMF type 2 trenger eksplisitt håndtering. At parseren kan lese filen, er ikke dokumentert støtte for å spille sekvensene korrekt; de skal ikke behandles som samtidige spor ved en framtidig støttekontrakt.
- Arcade bruker originalarrangementet med manuelt sporvalg. Bare anslag vurderes. NoteOff, sustainlengde og pedalteknikk gir ikke poeng.
- MusicXML, Tangent-manifest og vanskelighetspakker er **ikke implementert**. Dagens Staff-visning er en generert visning, ikke en full notetrykkmotor.

Se [Arcade-guiden](../pages/arcade.md) for gjeldende regler og [Arcade-planen](arcade-mode.md) for detaljer om poeng og presentasjon.

## Formater vi skal støtte videre

| Format | Beslutning | Omfang |
| --- | --- | --- |
| Standard MIDI Files: `.mid`, `.midi` | Behold som grunnformat | Prioriter eksplisitt og testet støtte for SMF type 0 og 1 med metrisk timing. Kartlegg og meld øvrige begrensninger tydelig. |
| `song.tangent.json` med vanlige MIDI-filer | Neste innholdsformat | Åpen, versjonert metadatafil med navngitte arrangementer, nivåer, deler/hender og omslag. Erstatter ikke MIDI. |
| MusicXML: `.musicxml`, samt `.xml` identifisert som MusicXML | Planlagt notasjonsimport | Bevar skrevet tonehøyde, pauser, stemmer, notelinjer, taktart, toneart og relevante notasjonselementer. Start med en dokumentert pianodelmengde. [2][3] |
| Komprimert MusicXML: `.mxl` | Del av MusicXML-arbeidet | Les standardens ZIP-container og `META-INF/container.xml`. En `.mxl` er ikke en vilkårlig ZIP-fil med nytt navn. [4] |
| MusicXML opus/samlinger | Senere vurdering | Kan samle flere partiturer, men vi må fortsatt erklære hvilke som er nivåer av samme stykke. Ingen automatisk Easy/Hard-tolkning. |
| ZIP for distribusjon av Tangent-mapper | Senere, etter stabilt manifest | Pakk eksisterende filer; ikke lag en ny koding av noter. Filendelse og distribusjonskontrakt er ikke fastlagt. |
| `.synthesia`, MEI og andre piano-/notasjonsformater | Ikke prioritert | Kan vurderes ved konkrete behov. Ingen lovet importer. |
| Clone Hero/Rock Band-spillspor og `.chart` | Utenfor retningen | Ingen importer eller avhengighet av spillenes tone-/vanskelighetskonvensjoner. |
| PDF, bilde, MP3 og WAV som notegrunnlag | Utenfor denne planen | Krever notegjenkjenning eller lydtranskripsjon. Lyd kan senere være separat referanse/akkompagnement, men er ikke tonefasit i seg selv. |

### MIDI + JSON: arrangementer uten eget noteformat

Modellen er **stykke → arrangement → spillbar del**. Easy, Medium og Hard skal være faktisk tilrettelagte arrangementer, ikke en gjetning basert på tempo, filstørrelse eller antall spor. Et vanskelig arrangement ved 50 prosent hastighet er fortsatt det samme arrangementet.

Manifestet skal kunne beskrive:

- Skjemaversjon, stabil stykke-ID, tittel, komponist, arrangør, kilde/lisens og valgfritt omslag.
- Stabil arrangement-ID og revisjon, nivånavn og referanse til MIDI-filen.
- Navngitte deler og eksplisitte sporvalg, med frivillig merking av høyre/venstre hånd. Notelinje og hånd er ikke alltid det samme, særlig ved håndkryssing.
- Senere: valgfri MusicXML-referanse, øvingsseksjoner og informasjon om koblingen mellom notasjon og framføring.

Flere arrangementer kan ligge i separate MIDI-filer eller bruke eksplisitt erklærte utvalg fra samme fil. Eksempelet i [Arcade-planen](arcade-mode.md#proposed-song-manifest) er et utkast, ikke et ferdig filformat eller en funksjon som virker nå. Før implementering skal eksempelet og et maskinlesbart skjema samordnes, inkludert entydige felt for komponist og arrangør.

Titler, omslag og filplassering skal ikke alene nullstille en rekord. Endringer i målnoter, timing, akkompagnement, hastighet eller vurderingsregler må gi riktig separat resultatkategori. En metadata-ID er ikke tilstrekkelig identitet for en spillrunde.

### MusicXML: notasjon med en tydelig støttekontrakt

MusicXML er laget for utveksling av skrevet musikk. Det kan bevare opplysninger som en vanlig MIDI-note ikke uttrykker, blant annet diatonisk navn og fortegn, stemmer og notelinjer. Det gjør formatet til riktig retning for bedre notelesing, men løser ikke layout eller avspilling automatisk. [2][3]

Første import skal avgrenses til vanlig pianonotasjon: toner, akkorder, pauser, noteverdier, bindebuer, stemmer, to notelinjer, nøkler, taktart, toneart og tempo. Repetisjoner, opptakter og tupleter må enten håndteres korrekt eller gi en tydelig begrensning før de brukes til poeng. Ingen stille tap av elementer som påvirker tonehøyde, rekkefølge eller timing. Visuelle elementer som ikke støttes må opplyses om.

Importer skal levere til en felles intern modell, med kildehenvisninger for hver note. MIDI gir framføringshendelser; MusicXML gir musikalsk struktur og kan senere generere en avspillingsplan. Når begge følger samme arrangement, må koblingen valideres gjennom blant annet repetisjoner og tempokart. Ikke anta at første XML-note svarer til første MIDI-note, eller vurder spilleren mot et noteark som viser en annen versjon.

Ved avvik skal brukeren få vite hvilken kilde som styrer avspilling og poeng. En uverifisert kobling skal ikke gi en tilsynelatende sammenlignbar Arcade-rekord. Full MusicXML-import og eksport kommer ikke som et løfte i første versjon.

## Prioritert vei videre

1. **Trygg tilkobling og timing.** Test Windows og fysisk piano, frakobling/gjentilkobling, pedal og lydruting. Lag en enkel MIDI-/lydveiviser og veiledet kalibrering. MIDI-testdata alene måler ikke faktisk forsinkelse.
2. **Riktig musikalsk tidsmodell.** Bruk taktart og taktartsskifter i taktmarkeringer. Gjør støtte for SMF-varianter, tempokart og standardverdier eksplisitt. Skill kildefeil fra manglende appstøtte.
3. **Målrettet øving.** Velg og gjenta et parti med redusert hastighet. Knytt Arcade-resultatet til «øv på dette partiet». Slike forsøk er øving og skal ikke erstatte sammenhengende rekorder.
4. **Bedre læringsprogresjon.** Introduser få noter om gangen, repeter vanlige feil og vis framgang i timing, bom og hastighet. Behold endeløs fri øving som et valg.
5. **Manifest og håndlagde nivåer.** Fastsett skjemaet, lag et lite originalt eksempelstykke i flere nivåer, og implementer bibliotek, delvalg, omslag og separate rekorder. Ingen automatisk reduksjon i første utgave.
6. **MusicXML og bedre Staff.** Implementer og dokumenter en avgrenset pianostøtte, først med korrekt struktur, deretter bedre layout. Test med eksport fra flere noteprogrammer og med innhold som gir tydelige avvisninger.
7. **Utvidelser etter erfaring.** Vurder distribusjonspakker, fingersetting, forfatterverktøy, merkede automatiske forenklinger og eventuell pedal-/varighetsvurdering. Nye vurderingsregler må versjoneres.

Hvert trinn skal ha en spillbar og testbar leveranse. Flere formater skal ikke gå foran pålitelig piano-input og nyttig øving.

## Krav før ny formatstøtte regnes som ferdig

- Dokumentert støttet delmengde, gjenkjennelige feilmeldinger og eksempelinnhold vi kan distribuere.
- Gyldige noter og timing må bevares gjennom importen, med tester for tempoendringer, samtidige toner, gjentatte toner og relevante takt-/stemmeegenskaper.
- Metadataimport skal ikke endre kildefiler. Filreferanser må holdes innenfor innholdsmappen, også via symbolske lenker. XML skal ikke hente eksterne entiteter eller nettverksressurser; arkiver må ha begrensninger for størrelse og utpakking.
- Hver ny formatfamilie trenger egne valideringskriterier. «Parseren åpnet filen» er ikke nok til å love korrekt notasjon eller poeng.
- Dokumentasjon og UI skal skille mellom metadata fra forfatteren og Tangents egne forslag, som akkordtolkning, håndfordeling eller kvantisert rytme.

Bakoverkompatibilitet med ubrukte prototyper er ikke en prioritet. Vi skal likevel versjonere innhold og vurderingsregler slik at senere endringer kan forklares og rekorder ikke blandes feil.

## Kilder og grunnlag

Formatbeskrivelsene bygger på primærkildene nedenfor, kontrollert 23. oktober 2026. Prioriteringene og Tangent-manifestet er prosjektets egne valg.

1. [MIDI Association: Standard MIDI Files](https://midi.org/standard-midi-files) — tidsfestede MIDI-strømmer, spor og metadata.
2. [W3C MusicXML 4.0: Introduction](https://www.w3.org/2021/06/musicxml40/tutorial/introduction/) — formålet som notasjonsformat.
3. [W3C MusicXML 4.0: XML Schema](https://www.w3.org/2021/06/musicxml40/listings/musicxml.xsd/) — blant annet pitch, voice, staff og notasjonselementer.
4. [W3C MusicXML 4.0: Compressed MXL Files](https://www.w3.org/2021/06/musicxml40/tutorial/compressed-mxl-files/) — filendelser og containerstruktur.

Nåstatus er kontrollert mot `midi-file/src/file.rs`, `midi-file/src/tempo_track.rs`, `tangent-core/src/midi_library.rs` og gjeldende [Arcade-guide](../pages/arcade.md). Nye implementasjoner skal oppdatere denne planen når støttekontrakten endres.
