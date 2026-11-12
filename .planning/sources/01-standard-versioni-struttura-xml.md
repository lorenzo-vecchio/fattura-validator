# Fatture Elettroniche italiane — Analisi degli standard e delle validazioni locali

## Parte 1/3 — Standard, versioni, struttura XML, tipi, vincoli di file e firma

**Data di redazione:** 17 settembre 2026 · **Ambito:** validazione _offline_ (solo il file XML/signed file, senza SdI, senza anagrafe tributaria, senza IndicePA, senza VIES)  
**Documenti collegati:** _Parte 2/3 — Codifiche complete (code list)_ · _Parte 3/3 — Validazione offline: catalogo controlli, algoritmi, tabelle_
**Legenda affidabilità delle informazioni**

*   **[V]** = verificato direttamente su fonte primaria (fetch del documento/XSD il 17/09/2026)
    
*   **[S]** = riportato da fonte secondaria attendibile o da documentazione di terze parti, non verificato riga per riga
    
*   **[?]** = da verificare — non usare come assunto in un validatore senza controllo
    

* * *

## 0. Sintesi esecutiva

1.  Non esiste **una** "fattura elettronica", ma **tre tracciati** con lo stesso impianto concettuale: **FatturaPA verso PA (**`FPA12`**)**, **Fattura ordinaria verso privati (**`FPR12`**)**, **Fattura semplificata (**`FSM10`**)**. Sono tre XSD distinti (il primo e il secondo condividono lo stesso namespace ma hanno enumerazioni diverse).
    
2.  Lo **standard tecnico corrente (17/09/2026)** è la **"Allegato A — Specifiche tecniche versione 1.9.1"**, aggiornata al 31 marzo 2026, **utilizzabile dal 15 maggio 2026**. Lato PA (fatturapa.gov.it) è valida dal 1 aprile 2025 la **"Specifiche tecniche del formato della FatturaPA versione 1.4"**, con **XSD 1.2.3** e **Fattura Semplificata 1.0.2**.
    
3.  **Il numero di versione delle specifiche NON è il numero di versione dell'XSD.** Specifiche: 1.0 → 1.9.1. XSD ordinaria: 1.2 → 1.2.1 → 1.2.2 → **1.2.3** (namespace fermo a `…/fatture/v1.2`). Il processo SdI ha una numerazione separata (attualmente **1.8.4**). Confondere le tre numerazioni è il primo errore da evitare.
    
4.  L'attributo `versione` dell'elemento radice vale `FPA12` **|** `FPR12` **|** `FSM10` — _non_ "1.2.3" — e deve coincidere con `<FormatoTrasmissione>` (controllo **00428**).
    
5.  Esiste una **doppia sorgente ufficiale** con contenuti parzialmente sovrapposti: **agenziaentrate.gov.it** (regole tecniche B2B/B2C: Allegato A + `Elenco controlli`) e **fatturapa.gov.it** (regole PA/FPA12: DM 55/2013 + tracciato + esempi). Un validatore completo deve leggere entrambe.
    
6.  **La validazione locale è possibile per: struttura XSD, ordine degli elementi, vincoli di formato, coerenze aritmetiche e logiche (i controlli 00400–00477), integrità della firma, vincoli di file (dimensione/estensione/encoding).** Circa il 60–70% dei controlli SdI è replicabile offline.
    
7.  **Non è replicabile offline**: esistenza/attività di partite IVA e codici fiscali (Anagrafe tributaria), esistenza/attività del `CodiceDestinatario` (registro dei canali SdI), codici ufficio IPA e IndicePA (00398/00399), duplicazione rispetto allo storico SdI (00404), dichiarazioni d'intento (00477), revoca dei certificati di firma (00101), CA affidabili (00104).
    
8.  L'XSD ufficiale ha **bug e rigidità note**: la regex email dell'XSD AdE rifiuta indirizzi che il SdI accetta [S]; il modello di validazione del "lotto" (più `FatturaElettronicaBody` in un file) è dove più implementazioni sbagliano.
    
9.  I punti dove i validatori di terze parti divergono dal SdI sono **quasi sempre gli stessi**: arrotondamenti e tolleranze (00421/00422/00423), tabelle di codici che cambiano nel tempo (Natura, CausalePagamento, RegimeFiscale, ISO 3166), `CodiceDestinatario`/PEC, e i controlli legati al `TipoDocumento` (00471–00476) che sono stati riscritti **quattro volte** tra il 2020 e il 2025.
    
10.  **Nessun crate Rust per FatturaPA esiste** ([S], verificato su crates.io). Nessuno **Schematron ufficiale** per FatturaPA è pubblicato da AdE: le regole di business esistono solo come prosa/pseudocodice nei PDF ufficiali e come codice nelle librerie di terze parti.
     

* * *

## 1. Cosa significa "validazione locale" e tassonomia dei livelli

Il SdI esegue una pipeline di verifiche ufficialmente descritta in **"Elenco dei controlli effettuati sul file fattura del Sistema di Interscambio"**. La classificazione ufficiale delle tipologie di verifica è:

> _"Le tipologie di controllo effettuate mirano a verificare: nomenclatura ed unicità del file trasmesso; dimensioni del file; integrità del documento se firmato; autenticità del certificato di firma; conformità del formato fattura; coerenza e validità del contenuto della fattura; unicità della fattura; recapitabilità della fattura."_ [V — Elenco controlli v1.8, pag. 3]

Da qui si derivano **6 livelli**, con diversa replicabilità locale:
| Livello | Cosa verifica | Offline? | Note |
| --- | --- | --- | --- |
| **L0 — File** | nome file, dimensione, estensione, zip, encoding | **Sì** | la _global_ unicità del nome richiede lo storico SdI → solo parziale |
| **L1 — Well-formedness** | XML parsabile, encoding dichiarato coerente con i byte | **Sì** | attenzione a BOM, encoding legacy, processing instruction |
| **L2 — XSD** | conformità allo schema (ordine, cardinalità, enumerazioni, pattern, lunghezze) | **Sì** | è la base, ma da sola **non è** un validatore FatturaPA |
| **L3 — Regole extra-XSD di contenuto** | coerenze logiche e aritmetiche (i codici 00400–00477) | **Sì** nella quasi totalità | il cuore del valore aggiunto di un validatore locale |
| **L4 — Anagrafiche esterne** | esistenza/attività di P.IVA, CF, `CodiceDestinatario`, codici IPA, dichiarazioni d'intento | **No** | richiede Anagrafe tributaria / IndicePA / registri SdI |
| **L5 — Storico e ciclo di vita** | duplicati rispetto allo storico, consegna, notifiche (RC/NS/MC/NE/EC/DT/AT/MT) | **No** | è il processo, non il documento |

**Regola d'oro per un validatore locale:** dichiarare sempre _quale_ livello ha prodotto un finding, e **non** emettere mai una sentenza di "fattura valida" tout court — solo "conforme ai livelli L0–L3". L'L4 e l'L5 sono per definizione fuori portata.

* * *

## 2. Famiglia di standard e inventario documenti (stato al 17/09/2026)

### 2.1 I tre tracciati

|  | **FPA12 — FatturaPA** | **FPR12 — Fattura ordinaria** | **FSM10 — Fattura semplificata** |
| --- | --- | --- | --- |
| Destinatari | Pubbliche Amministrazioni | Privati (B2B/B2C) | Privati, casi art. 21-bis DPR 633/72 |
| Namespace | `http://ivaservizi.agenziaentrate.gov.it/docs/xsd/fatture/v1.2` | idem | `http://ivaservizi.agenziaentrate.gov.it/docs/xsd/fatture/v1.0` |
| Elemento radice | `FatturaElettronica` | `FatturaElettronica` | `FatturaElettronicaSemplificata` |
| `versione` / `FormatoTrasmissione` | `FPA12` | `FPR12` | `FSM10` |
| XSD corrente | 1.2.3 | 1.2.3 (`Schema_VFPR12`) | 1.0.2 (`Schema VFSM10`) |
| `CodiceDestinatario` | **6** caratteri (codice ufficio IPA) | **7** caratteri | **7** caratteri |
| Documento di riferimento | Specifiche tecniche formato FatturaPA **1.4** (dal 01/04/2025); DM 55/2013 | Allegato A — Specifiche tecniche **1.9.1** (dal 15/05/2026) | Allegato A — Specifiche tecniche **1.9.1** |
| Firma digitale | **obbligatoria** (firma qualificata + riferimento temporale) | **facoltativa** | **facoltativa** |
| Tipi documento ammessi | sottoinsieme (TD01…) | TD01–TD06, TD16–TD29 | **solo TD07, TD08, TD09** |
| Natura ammesse | sottoinsieme | N1 … N7 (incl. N6.x) | **solo N1, N2, N2.1, N2.2, N3, N3.1–N3.6, N4, N5** (no N6.x, no N7) [S] |

> Nota: la separazione in **due file XSD distinti** per PA e privati è stata introdotta con la versione `1.2.3` (i file `Schema_VFPA12_V1.2.3.xsd` e `Schema_VFPR12_v1.2.3.xsd` differiscono, sostanzialmente, solo per il valore ammesso in `FormatoTrasmissioneType`: `FPA12` vs `FPR12`). [V — confronto dei due XSD / [S] sul nome file]

### 2.2 Inventario dei documenti ufficiali correnti

**Lato B2B/B2C — agenziaentrate.gov.it, "Specifiche tecniche versione 1.9.1 — Documentazione utilizzabile dal 15 maggio 2026"** [V]:
| Documento | Formato | Note |
| --- | --- | --- |
| Allegato A — Specifiche tecniche vers. 1.9.1 | PDF (agg. 31/03/2026) | **il documento normativo principale** |
| Appendix A — Technical Specifications vers. 1.9.1 | PDF (EN) | traduzione non ufficiale ai fini legali |
| Rappresentazione tabellare del tracciato fattura ordinaria | PDF + XLS | **la migliore fonte machine-scannable campo-per-campo** |
| Rappresentazione tabellare fattura semplificata | PDF + XLSX | idem, per FSM10 |
| **Schema VFSM10** | XSD | fattura semplificata, v1.0.2 |
| **Schema_VFPR12** | XSD | fattura ordinaria, v1.2.3 |
| MessaggiFatturaTypes_v1.0 | XSD | **notifiche/ricevute SdI**, non la fattura |
| Qr Code partita IVA — Specifiche tecniche | PDF | novità recente, fuori dal tracciato fattura |
| Esempi fattura, hash e ricevute | bundle | **corpus di test ufficiale, include gli hash** |
| Fogli di stile per ricevute | XSLT | rendering |

**Lato PA — fatturapa.gov.it, "Documentazione valida dal 1 aprile 2025"** [V]:
| Documento | Note |
| --- | --- |
| Schema del file xml FatturaPA **versione 1.2.3** | XSD (67 KB) |
| Schema del file xml Fattura Ordinaria | XSD (68 KB) |
| Schema del file xml Fattura Semplificata **versione 1.0.2** | XSD (24 KB) |
| Fogli di stile (FatturaPA 1.2.3 / Ordinaria 1.2.3 / Semplificata 1.0.2) | XSLT |
| **Specifiche tecniche del formato della FatturaPA versione 1.4** | PDF (781 KB) |
| Rappresentazione tabellare (ordinaria / semplificata) | PDF + Excel |
| _Documentazione valida fino al 31 marzo 2025_ | FatturaPA **1.2.2** + specifiche **1.3.2** |
| _Documentazione valida fino al 31 gennaio 2024_ | FatturaPA 1.2.2 + specifiche 1.3.2 |
| **Esempi (versione 1.2, ancora pubblicati)**: `IT01234567890_FPA01/FPA02/FPA03.xml` (PA: 1 linea, più linee, lotto) e `IT01234567890_FPR01/FPR02/FPR03.xml` (privati: idem) | corpus di riferimento |
| PEC di ricezione: `sdi01@pec.fatturapa.it` | canale PEC |

**Documenti SdI (processo, non formato)** [S]: "Specifiche tecniche relative al Sistema di Interscambio" v**1.8.4** (dal 15/05/2026), "Elenco dei controlli effettuati sul file fattura del Sistema di Interscambio" — edizioni note: **v1.8** (01/10/2022) e **v2.0** (31/01/2025) — più "Specifiche tecniche notifiche di esito".
URL utili:

*   `https://www.agenziaentrate.gov.it/portale/specifiche-tecniche-versione-1.9.1-%C2%A0-utilizzabili-dal-15-maggio-2026-`
    
*   `https://www.fatturapa.gov.it/it/norme-e-regole/documentazione-fattura-elettronica/formato-fatturapa/`
    
*   `https://www.fatturapa.gov.it/export/documenti/Elenco-Controlli-versione-1.8.pdf` e `https://www.fatturapa.gov.it/export/documenti/fatturapa/v1.4/Elenco-Controlli-versione-2.0.pdf`
    
*   XSD verificato in questa analisi: `https://www.fatturapa.gov.it/export/documenti/fatturapa/v1.4/Schema_VFPR12_v1.2.3.xsd`
    

* * *

## 3. Matrice versioni e le tre "trappole" di numerazione

### 3.1 Tre numerazioni indipendenti

| Numerazione | Cosa numera | Valori storici | Valore corrente (17/09/2026) |
| --- | --- | --- | --- |
| **Specifiche tecniche (formato fattura), Allegato A** | le _regole_ B2B/B2C | 1.0 → 1.1 → 1.2 → 1.3 → 1.4 → 1.4.1 → 1.4.2 → 1.5 → 1.6 → 1.6.1 → 1.6.2 → 1.6.3 → 1.6.4 → 1.7 → 1.7.1 → 1.8 → 1.9 → **1.9.1** | **1.9.1** (dal 15/05/2026) |
| **Specifiche tecniche formato FatturaPA (PA)** | le regole FPA12 | … → 1.3.1 → 1.3.2 → **1.4** | **1.4** (dal 01/04/2025) |
| **Specifiche tecniche SdI** | il processo/interscambio | … → 1.8.1 → 1.8.2 → 1.8.3 → **1.8.4** | **1.8.4** (dal 15/05/2026) |
| **XSD** | lo schema XML | 1.2 → 1.2.1 → 1.2.2 → **1.2.3** (ordinaria/PA); 1.0 → 1.0.1 → **1.0.2** (semplificata) | **1.2.3** / **1.0.2** |

### 3.2 Trappole

1.  **"Versione 1.2.2" vs "versione 1.9.1"**: il namespace è ancora `…/fatture/v1.2` e l'attributo `version` dell'XSD è `1.2.3`. Le specifiche sono a 1.9.1. Entrambi i numeri sono "giusti" ma si riferiscono a cose diverse.
    
2.  **L'attributo** `versione` **nel file non è la versione dello schema**: vale `FPR12`/`FPA12`/`FSM10` (controllo 00428). Un validatore che confronta `versione` con "1.2.3" sbaglia.
    
3.  **La coppia di siti**: `agenziaentrate.gov.it` e `fatturapa.gov.it` pubblicano XSD con etichette di versione potenzialmente disallineate tra loro [S — segnalato esplicitamente dalla libreria `slam/php-validatore-fattura-elettronica`]. **Scarica sempre l'XSD dal sito e calcola un checksum**, non fidarti del numero nel nome.
    

### 3.3 Storia delle modifiche rilevanti per un validatore [S, da changelog ufficiale "Variazioni alle specifiche tecniche"]

| Versione | Data | Cosa cambia (selezione rilevante per la validazione) |
| --- | --- | --- |
| 1.2 | 03/10/2018 | introdotto il controllo **00460** (importo max fattura semplificata) |
| 1.4.2 | 20-24/05/2019 | 00460: soglia **100 € → 400 €** |
| 1.5 | 30/07/2019 | nuovi controlli **00313, 00320–00326, 00330** |
| 1.6 | 04/05/2020 | nuovi `Natura` (N2.1/N2.2, N3.1–N3.6, N6.1–N6.9), `TipoDocumento` TD16–TD27, `TipoRitenuta` RT03–RT06, `ModalitaPagamento` MP19–MP23; `ImportoBollo` opzionale; controlli **00443, 00444, 00445, 00471–00474**; `DatiRitenuta` diventa 0..N |
| 1.6.1 | 20/04/2020 | **N2, N3, N6 non più validi per fatture emesse dal 01/01/2021** (→ 00445) |
| 1.6.3 | 02/07/2021 | nuove codifiche `AltriDatiGestionali/TipoDato`: INVCONT, NB1, NB2, NB3, N.DOC.COMM |
| 1.6.4 / 1.7 | 23/12/2021 / 28/10/2021 | nuovo controllo **00475**; 00313 aggiornato per la semplificata; **data transfrontalieri via tracciato FatturaPA** (abolizione del flusso "dati fattura") |
| 1.7.1 | 01/08/2022 | **TD28**; codici `TipoDato` F24 e deposito IVA; nuovo controllo **00476** |
| 1.8 | 12/12/2023 | nuovo controllo **00477** (dichiarazione d'intento invalidata); codici agricoli ALI-COMP/NO-COMP/OCC34BIS; TD28 esteso |
| 1.9 | 31/01/2025 | **TD29** (omessa/irregolare fatturazione, art. 6 c.8 D.Lgs. 471/97); **RF20** (franchigia IVA transfrontaliera, Dir. UE 2020/285); riscrittura descrizioni 00471/00473/00475; eccezione 400 € per forfettari/RF20 |
| **1.9.1** | **31/03/2026** | **nuovo controllo 00327** (gruppo IVA, ciclo passivo); nuova codifica `ESENZSPORT` in `AltriDatiGestionali` (art. 36 c.6 D.Lgs. 36/2021); aggiornate procedure di accreditamento WS/SFTP; **max codici destinatario 100 → 300** |

> **Regola operativa**: le code list sono _datate_. Un validatore serio deve avere un **catalogo di tabelle con effective-date**, non un singolo set hard-coded. `CausalePagamento` cambia con il modello CU ogni anno; `Natura`/`TipoDocumento`/`RegimeFiscale` cambiano per provvedimento.

* * *

## 4. Quadro normativo: cosa è legge e cosa è verificabile a macchina

Distinzione essenziale per un validatore:
| Tipo | Esempi | Verificabile offline? |
| --- | --- | --- |
| **Legge / regolamento** | DPR 633/72 artt. 21 e 21-bis; DM 55/2013 (PA); DM 17/06/2014 (bollo); L. 205/2017 (obbligo B2B/B2C); DL 119/2018; DL 34/2019 (San Marino); D.Lgs. 148/2018 (EN 16931) | solo indirettamente |
| **Provvedimenti tecnici AdE** | Provv. 30/04/2018 n. 89757 (regole tecniche), poi **Provv. 24/11/2022 n. 433608** (mod. Provv. 08/03/2024) per i privati; DM 55/2013 per la PA | sono la fonte delle regole |
| **Specifiche tecniche (Allegato A)** | formato, tracciato, regole di compilazione, controlli | **sì** (è il documento da implementare) |
| **Elenco controlli** | i codici 00xxx con pseudocodice | **sì, per la parte L0–L3** |
| **Anagrafiche** | Anagrafe tributaria, IndicePA, registri canali SdI, VIES | **no** |

Punti noti del quadro applicativo:

*   **Per la PA restano valide le regole del DM 55/2013** (Allegato A = formato, Allegato B = regole di interscambio); per i privati valgono le regole del Provvedimento 24/11/2022.
    
*   **La fattura verso PA deve essere firmata digitalmente** con firma qualificata; per i privati la firma è **facoltativa** (_"Il SdI gestisce sia fatture elettroniche prive di firma elettronica che fatture elettroniche alle quali sia apposta firma elettronica"_) [V/S — testo §1.2.1 delle specifiche].
    
*   **Obbligo di firma ≠ verificabilità locale**: un validatore può verificare _integrità_ e _presenza del riferimento temporale_, non l'autenticità del certificato rispetto a CRL/OCSP.
    
*   **Eccezione sanitaria**: chi invia i dati delle spese sanitarie al **Sistema Tessera Sanitaria** _non può_ emettere quelle fatture via SdI (art. 10-bis DL 119/2018): un validatore di dominio deve saper riconoscere questo caso d'uso e non segnalare come "mancante" una fattura che per legge non deve passare dal SdI.
    

* * *

## 5. Struttura XML: namespace, radice, attributi, firma

### 5.1 Prologo dello schema (verbatim, `Schema_VFPA12_V1.2.3.xsd`) [V]

```xml
<?xml version="1.0" encoding="utf-8"?>
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema"
    xmlns:ds="http://www.w3.org/2000/09/xmldsig#"
    xmlns="http://ivaservizi.agenziaentrate.gov.it/docs/xsd/fatture/v1.2"
    targetNamespace="http://ivaservizi.agenziaentrate.gov.it/docs/xsd/fatture/v1.2"
    version="1.2.3">
  <xs:import namespace="http://www.w3.org/2000/09/xmldsig#" schemaLocation="xmldsig-core-schema.xsd" />
  <xs:element name="FatturaElettronica" type="FatturaElettronicaType">
    <xs:annotation><xs:documentation>XML schema fatture destinate a PA (FPA12) versione 1.2.3</xs:documentation></xs:annotation>
  </xs:element>
```

Fatti conseguenti, **critici** per l'implementazione:

1.  `elementFormDefault` **non è dichiarato** → vale `unqualified`: **solo l'elemento radice è qualificato dal namespace**; tutti i figli sono in _nessun namespace_ (`FatturaElettronicaHeader`, non `p:FatturaElettronicaHeader`). Un validatore/parser deve gestire questo, e in lettura conviene adottare il **matching per local-name, namespace-agnostico**.
    
2.  **La firma è un elemento** `ds:Signature` **opzionale, dopo i body** (`<xs:element ref="ds:Signature" minOccurs="0"/>`), con import relativo di `xmldsig-core-schema.xsd`.
    
3.  **Elemento radice**: `FatturaElettronica` con
    *   `@versione` — **obbligatorio**, tipo `FormatoTrasmissioneType`
        
    *   `@SistemaEmittente` — **opzionale**, tipo `String10Type` (aggiunto in 1.2.1)
        
4.  **Fattura semplificata**: radice `FatturaElettronicaSemplificata`, namespace `…/xsd/fatture/v1.0`, `@versione="FSM10"`.
    

### 5.2 Radice tipica di un'istanza

```xml
<?xml version="1.0" encoding="UTF-8"?>
<p:FatturaElettronica versione="FPR12" SistemaEmittente="…"
   xmlns:ds="http://www.w3.org/2000/09/xmldsig#"
   xmlns:p="http://ivaservizi.agenziaentrate.gov.it/docs/xsd/fatture/v1.2"
   xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">
  <FatturaElettronicaHeader> … </FatturaElettronicaHeader>
  <FatturaElettronicaBody> … </FatturaElettronicaBody>
</p:FatturaElettronica>
```

> **Attenzione:** `xsi:schemaLocation` **non è richiesto** e in alcuni contesti (client Java di terze parti) ha causato problemi [S]. Un validatore non deve pretenderlo né rifiutarlo.

* * *

## 6. Albero completo degli elementi (XSD 1.2.3, fattura ordinaria)

Cardinalità `min..max` (default `1..1`). Tipi come definiti nello schema. **[V — XSD 1.2.3]**

```
FatturaElettronica                                  (root)
│  @versione (required, FormatoTrasmissioneType)
│  @SistemaEmittente (optional, String10Type)
├─ FatturaElettronicaHeader                        1..1
│  ├─ DatiTrasmissione                             1..1
│  │  ├─ IdTrasmittente                            1..1  (IdFiscaleType: IdPaese 1..1 + IdCodice 1..1)
│  │  ├─ ProgressivoInvio                          1..1  String10Type
│  │  ├─ FormatoTrasmissione                       1..1  FormatoTrasmissioneType
│  │  ├─ CodiceDestinatario                        1..1  CodiceDestinatarioType
│  │  ├─ ContattiTrasmittente                      0..1  (Telefono 0..1, Email 0..1)
│  │  └─ PECDestinatario                           0..1  EmailType
│  ├─ CedentePrestatore                            1..1
│  │  ├─ DatiAnagrafici                            1..1
│  │  │  ├─ IdFiscaleIVA                           1..1
│  │  │  ├─ CodiceFiscale                          0..1  CodiceFiscaleType
│  │  │  ├─ Anagrafica                             1..1  → choice( Denominazione | Nome + Cognome ) + Titolo? + CodEORI?
│  │  │  ├─ AlboProfessionale                      0..1  String60LatinType
│  │  │  ├─ ProvinciaAlbo                          0..1  ProvinciaType
│  │  │  ├─ NumeroIscrizioneAlbo                   0..1  String60Type
│  │  │  ├─ DataIscrizioneAlbo                     0..1  xs:date
│  │  │  └─ RegimeFiscale                          1..1  RegimeFiscaleType
│  │  ├─ Sede                                      1..1  IndirizzoType
│  │  ├─ StabileOrganizzazione                     0..1  IndirizzoType
│  │  ├─ IscrizioneREA                             0..1
│  │  │  ├─ Ufficio 1..1 (ProvinciaType) · NumeroREA 1..1 (String20Type)
│  │  │  ├─ CapitaleSociale 0..1 (Amount2DecimalType) · SocioUnico 0..1 · StatoLiquidazione 1..1
│  │  ├─ Contatti                                  0..1  (Telefono 0..1, Fax 0..1, Email 0..1)
│  │  └─ RiferimentoAmministrazione                0..1  String20Type
│  ├─ RappresentanteFiscale                        0..1
│  │  └─ DatiAnagrafici 1..1 (IdFiscaleIVA 1..1, CodiceFiscale 0..1, Anagrafica 1..1)
│  ├─ CessionarioCommittente                       1..1
│  │  ├─ DatiAnagrafici                            1..1  (IdFiscaleIVA 0..1 !, CodiceFiscale 0..1, Anagrafica 1..1)
│  │  ├─ Sede                                      1..1  IndirizzoType
│  │  ├─ StabileOrganizzazione                     0..1  IndirizzoType
│  │  └─ RappresentanteFiscale                     0..1  (IdFiscaleIVA 1..1 + choice Denominazione|Nome+Cognome)
│  ├─ TerzoIntermediarioOSoggettoEmittente         0..1
│  │  └─ DatiAnagrafici 1..1 (IdFiscaleIVA 0..1, CodiceFiscale 0..1, Anagrafica 1..1)
│  └─ SoggettoEmittente                            0..1  SoggettoEmittenteType
├─ FatturaElettronicaBody                          1..unbounded   ← "lotto"
│  ├─ DatiGenerali                                 1..1
│  │  ├─ DatiGeneraliDocumento                     1..1   (ordine rigido — vedi §8)
│  │  ├─ DatiOrdineAcquisto                        0..unbounded  DatiDocumentiCorrelatiType
│  │  ├─ DatiContratto                             0..unbounded  idem
│  │  ├─ DatiConvenzione                           0..unbounded  idem
│  │  ├─ DatiRicezione                             0..unbounded  idem
│  │  ├─ DatiFattureCollegate                      0..unbounded  idem
│  │  ├─ DatiSAL                                   0..unbounded  (RiferimentoFase 1..1)
│  │  ├─ DatiDDT                                   0..unbounded  (NumeroDDT 1..1, DataDDT 1..1, RiferimentoNumeroLinea 0..unbounded)
│  │  ├─ DatiTrasporto                             0..1
│  │  └─ FatturaPrincipale                         0..1
│  ├─ DatiBeniServizi                              1..1
│  │  ├─ DettaglioLinee                            1..unbounded
│  │  └─ DatiRiepilogo                             1..unbounded
│  ├─ DatiVeicoli                                  0..1  (Data 1..1, TotalePercorso 1..1)
│  ├─ DatiPagamento                                0..unbounded  (CondizioniPagamento 1..1 + DettaglioPagamento 1..unbounded)
│  └─ Allegati                                     0..unbounded
└─ ds:Signature                                    0..1 (XML-DSig)
```

`DatiDocumentiCorrelatiType` (usato da DatiOrdineAcquisto/Contratto/Convenzione/Ricezione/FattureCollegate), in ordine:  
`RiferimentoNumeroLinea 0..∞ (integer 1..9999)` · `IdDocumento 1..1 (String20)` · `Data 0..1 (xs:date)` · `NumItem 0..1 (String20)` · `CodiceCommessaConvenzione 0..1 (String100Latin)` · `CodiceCUP 0..1 (String15)` · `CodiceCIG 0..1 (String15)`
`DatiGeneraliDocumento` (ordine obbligatorio):  
`TipoDocumento 1..1` · `Divisa 1..1` · `Data 1..1 (DataFatturaType)` · `Numero 1..1 (String20)` · `DatiRitenuta 0..∞` · `DatiBollo 0..1` · `DatiCassaPrevidenziale 0..∞` · `ScontoMaggiorazione 0..∞` · `ImportoTotaleDocumento 0..1 (Amount2Decimal)` · `Arrotondamento 0..1 (Amount2Decimal)` · `Causale 0..∞ (String200Latin)` · `Art73 0..1`

*   `DatiRitenuta`: `TipoRitenuta 1..1` · `ImportoRitenuta 1..1 (Amount2)` · `AliquotaRitenuta 1..1 (RateType)` · `CausalePagamento 1..1`
    
*   `DatiBollo`: `BolloVirtuale 1..1 (solo "SI")` · `ImportoBollo 0..1 (Amount2)`
    
*   `DatiCassaPrevidenziale`: `TipoCassa 1..1` · `AlCassa 1..1 (RateType)` · `ImportoContributoCassa 1..1 (Amount2)` · `ImponibileCassa 0..1 (Amount2)` · `AliquotaIVA 1..1 (RateType)` · `Ritenuta 0..1 ("SI")` · `Natura 0..1` · `RiferimentoAmministrazione 0..1 (String20)`
    
*   `ScontoMaggiorazione`: `Tipo 1..1 (SC|MG)` · `Percentuale 0..1 (RateType)` · `Importo 0..1 (Amount8Decimal)`
    

`DettaglioLinee` (ordine obbligatorio):  
`NumeroLinea 1..1 (1..9999)` · `TipoCessionePrestazione 0..1` · `CodiceArticolo 0..∞` (CodiceTipo 1..1 + CodiceValore 1..1) · `Descrizione 1..1 (String1000Latin)` · `Quantita 0..1 (QuantitaType)` · `UnitaMisura 0..1 (String10)` · `DataInizioPeriodo 0..1 (xs:date)` · `DataFinePeriodo 0..1 (xs:date)` · `PrezzoUnitario 1..1 (Amount8Decimal)` · `ScontoMaggiorazione 0..∞` · `PrezzoTotale 1..1 (Amount8Decimal)` · `AliquotaIVA 1..1 (RateType)` · `Ritenuta 0..1 ("SI")` · `Natura 0..1` · `RiferimentoAmministrazione 0..1 (String20)` · `AltriDatiGestionali 0..∞` (TipoDato 1..1 String10 · RiferimentoTesto 0..1 String60Latin · RiferimentoNumero 0..1 Amount8Decimal · RiferimentoData 0..1 xs:date)
`DatiRiepilogo`:  
`AliquotaIVA 1..1 (RateType)` · `Natura 0..1` · `SpeseAccessorie 0..1 (Amount2)` · `Arrotondamento 0..1 (Amount8Decimal)` · `ImponibileImporto 1..1 (Amount2)` · `Imposta 1..1 (Amount2)` · `EsigibilitaIVA 0..1 (D|I|S)` · `RiferimentoNormativo 0..1 (String100Latin)`
`DettaglioPagamento`:  
`Beneficiario 0..1 (String200Latin)` · `ModalitaPagamento 1..1` · `DataRiferimentoTerminiPagamento 0..1` · `GiorniTerminiPagamento 0..1 (0..999)` · `DataScadenzaPagamento 0..1` · `ImportoPagamento 1..1 (Amount2)` · `CodUfficioPostale 0..1 (String20)` · `CognomeQuietanzante 0..1 (String60Latin)` · `NomeQuietanzante 0..1 (String60Latin)` · `CFQuietanzante 0..1 (CodiceFiscalePFType — **esattamente 16**)` · `TitoloQuietanzante 0..1` · `IstitutoFinanziario 0..1 (String80Latin)` · `IBAN 0..1` · `ABI 0..1 (5 cifre)` · `CAB 0..1 (5 cifre)` · `BIC 0..1` · `ScontoPagamentoAnticipato 0..1 (Amount2)` · `DataLimitePagamentoAnticipato 0..1` · `PenalitaPagamentiRitardati 0..1 (Amount2)` · `DataDecorrenzaPenale 0..1` · `CodicePagamento 0..1 (String60)`
`Allegati`: `NomeAttachment 1..1 (String60Latin)` · `AlgoritmoCompressione 0..1 (String10)` · `FormatoAttachment 0..1 (String10)` · `DescrizioneAttachment 0..1 (String100Latin)` · `Attachment 1..1 (**xs:base64Binary**)`
`Anagrafica`: `choice( Denominazione (String80Latin) | (Nome (String60Latin), Cognome (String60Latin)) )` + `Titolo?` + `CodEORI?`
`DatiTrasporto` (tutti 0..1, in ordine): `DatiAnagraficiVettore` (IdFiscaleIVA 1..1, CodiceFiscale 0..1, Anagrafica 1..1, NumeroLicenzaGuida 0..1) · `MezzoTrasporto (String80Latin)` · `CausaleTrasporto (String100Latin)` · `NumeroColli (1..9999)` · `Descrizione (String100Latin)` · `UnitaMisuraPeso (String10)` · `PesoLordo (PesoType)` · `PesoNetto (PesoType)` · `DataOraRitiro (xs:dateTime)` · `DataInizioTrasporto (xs:date)` · `TipoResa ([A-Z]{3})` · `IndirizzoResa (IndirizzoType)` · `DataOraConsegna (xs:dateTime)`
`IndirizzoType` (ordine rigido): `Indirizzo 1..1 (String60Latin)` · `NumeroCivico 0..1 (String8)` · `CAP 1..1 (5 cifre)` · `Comune 1..1 (String60Latin)` · `Provincia 0..1 ([A-Z]{2})` · `Nazione 1..1 ([A-Z]{2}, default IT)`

### 6.1 Fattura semplificata (FSM10) — differenze note

Ricostruito dalle fonti disponibili (da completare contro `Schema VFSM10` v1.0.2) [V/S, con punti **[?]**]:

*   Radice `FatturaElettronicaSemplificata`, namespace `…/fatture/v1.0`, XSD 1.0.2, `@versione="FSM10"`; `FormatoTrasmissione` ammette solo `FSM10`.
    
*   **Assenti/non previsti**: `RappresentanteFiscale`, `TerzoIntermediarioOSoggettoEmittente`, `ContattiTrasmittente`, `DatiPagamento`, `DatiVeicoli` [?], `DatiTrasporto` [?], `DatiOrdineAcquisto`/`Contratto`/`Convenzione`/`Ricezione`/`FattureCollegate` [?], `DatiDDT` [?], `FatturaPrincipale` [?], `Allegati` [?].
    
*   `BolloVirtuale` compare **direttamente sotto** `DatiGeneraliDocumento` (senza blocco `DatiBollo` e senza `ImportoBollo`).
    
*   `Natura` ammette **solo** `N1, N2, N2.1, N2.2, N3, N3.1–N3.6, N4, N5` (nessun N6.x, nessun N7).
    
*   `TipoDocumento` ammette **solo** `TD07, TD08, TD09`.
    
*   `CodiceDestinatario` = 7 caratteri (`[A-Z0-9]{7}`).
    
*   Vincolo sostanziale: importo complessivo entro il limite dell'art. 21-bis DPR 633/72 (controllo **00460**) — la soglia è stata **400 €**, con **esonero dal 1.9** in caso di regime forfettario o RF20 [?].
    

* * *

## 7. Tipi semplici nominati e relativi facet (verbatim dallo XSD 1.2.3)

**Fatto importante:** l'XSD **non usa** `xs:totalDigits`**/**`fractionDigits`: tutti i vincoli numerici sono espressi come `xs:pattern` su `xs:decimal`. E **non ci sono restrizioni inline** dentro gli elementi: i facet stanno sempre in `simpleType` nominati (vincoli sempre _indiretti_). **[V]**

### 7.1 Numerici e interi

```xml
PesoType              : xs:decimal  pattern "[0-9]{1,4}\.[0-9]{1,2}"
Amount8DecimalType    : xs:decimal  pattern "[\-]?[0-9]{1,11}\.[0-9]{2,8}"
Amount2DecimalType    : xs:decimal  pattern "[\-]?[0-9]{1,11}\.[0-9]{2}"
RateType              : xs:decimal  maxInclusive="100.00"  pattern "[0-9]{1,3}\.[0-9]{2}"
QuantitaType          : xs:decimal  pattern "[0-9]{1,12}\.[0-9]{2,8}"
RiferimentoNumeroLineaType : xs:integer  minInclusive 1  maxInclusive 9999
NumeroLineaType            : xs:integer  1..9999
NumeroColliType            : xs:integer  1..9999
RiferimentoFaseType        : xs:integer  1..999
GiorniTerminePagamentoType : xs:integer  0..999
DataFatturaType            : xs:date     minInclusive "1970-01-01"
```

> **Nota tecnica fondamentale:** `RateType` **non ammette il segno meno** e impone **esattamente 2 decimali** e max 100.00. `Amount2DecimalType` impone **esattamente 2 decimali**; `Amount8DecimalType`/`QuantitaType` **da 2 a 8 decimali**. Il separatore è il **punto**, mai la virgola; niente separatore delle migliaia; niente notazione esponenziale. Un validatore che normalizza "12,50" o "1.234,56" sta correggendo un file che il SdI scarterebbe.

### 7.2 Stringhe e codici (verbatim)

```xml
TitoloType           : normalizedString, whiteSpace=collapse, pattern "(\p{IsBasicLatin}{2,10})"
String10Type         : pattern "(\p{IsBasicLatin}{1,10})"
String15Type         : pattern "(\p{IsBasicLatin}{1,15})"
String20Type         : pattern "(\p{IsBasicLatin}{1,20})"
String35Type         : pattern "(\p{IsBasicLatin}{1,35})"
String35LatinExtType : normalizedString, minLength 1, maxLength 35   (NESSUN pattern: unica stringa "estesa")
String60Type         : pattern "(\p{IsBasicLatin}{1,60})"
String80Type         : pattern "(\p{IsBasicLatin}{1,80})"
String100Type        : pattern "(\p{IsBasicLatin}{1,100})"
String60LatinType    : pattern "[\p{IsBasicLatin}\p{IsLatin-1Supplement}]{1,60}"
String80LatinType    : pattern "[\p{IsBasicLatin}\p{IsLatin-1Supplement}]{1,80}"
String100LatinType   : pattern "[\p{IsBasicLatin}\p{IsLatin-1Supplement}]{1,100}"
String200LatinType   : pattern "[\p{IsBasicLatin}\p{IsLatin-1Supplement}]{1,200}"
String1000LatinType  : pattern "[\p{IsBasicLatin}\p{IsLatin-1Supplement}]{1,1000}"
NumeroCivicoType     : pattern "(\p{IsBasicLatin}{1,8})"
TelFaxType           : pattern "(\p{IsBasicLatin}{5,12})"

EmailType            : xs:string, maxLength 256, pattern ".+@.+[.]+.+"
EmailContattiType    : xs:string, minLength 7, maxLength 256, pattern ".+@.+[.]+.+"

ProvinciaType        : "[A-Z]{2}"
NazioneType          : "[A-Z]{2}"
DivisaType           : "[A-Z]{3}"
TipoResaType         : "[A-Z]{3}"
CAPType              : "[0-9][0-9][0-9][0-9][0-9]"
ABIType              : "[0-9][0-9][0-9][0-9][0-9]"
CABType              : "[0-9][0-9][0-9][0-9][0-9]"
CodiceFiscaleType    : "[A-Z0-9]{11,16}"
CodiceFiscalePFType  : "[A-Z0-9]{16}"
CodEORIType          : minLength 13, maxLength 17
IBANType             : "[a-zA-Z]{2}[0-9]{2}[a-zA-Z0-9]{11,30}"
BICType              : "[A-Z]{6}[A-Z2-9][A-NP-Z0-9]([A-Z0-9]{3}){0,1}"
CodiceDestinatarioType: "[A-Z0-9]{6,7}"     (ordinaria)   /   "[A-Z0-9]{7}"  (semplificata)
CodiceType (IdCodice) : minLength 1, maxLength 28
```

### 7.3 Avvertenze sui pattern XSD (fonte tipica di falsi positivi)

1.  I `xs:pattern` sono **regex XML Schema**, non PCRE: **non** esistono `^`/`$` come metacaratteri (il pattern è implicitamente ancorato all'intera stringa), e le classi `\p{IsBasicLatin}` / `\p{IsLatin-1Supplement}` sono block escape **non supportati allo stesso modo da tutte le librerie**. **Testare sempre il comportamento del motore XSD scelto.**
    
2.  I pattern includono già i quantificatori di lunghezza: aggiungere `^…$` in una regex PCRE di validazione _manuale_ è corretto, ma è sbagliato validare "lunghezza ≤ 60" _e_ applicare il pattern "così com'è" con un motore XSD.
    
3.  `EmailType` dell'XSD ufficiale è **troppo restrittivo**: la libreria PHP di riferimento **sovrascrive** la regex dell'XSD con una più permissiva (`.+@.+[.]+.+`) dichiarando esplicitamente che la regex AdE "è costruita male" [S]. Anche la libreria .NET ha **rilassato** lo stesso campo. → **Un validatore non deve usare la regex email dell'XSD come regola bloccante.**
    
4.  `String35LatinExtType` è **l'unica stringa senza restrizione di blocco Unicode** (usata da `CodiceArticolo/CodiceValore`): non applicarle il controllo "solo Latin-1".
    
5.  Le stringhe sono `normalizedString` (`\r`, `\n`, `\t` → spazio) con `whiteSpace=collapse` per `Titolo`. **Non fare** `trim()` **arbitrari** in pre-processing: altera i valori.
    

* * *

## 8. Ordine e cardinalità: i vincoli che l'XSD impone _e_ quelli che l'app non deve violare

Tutti i complessi sono `xs:sequence`: **l'ordine documentale è obbligatorio**. Gli `xs:choice` sono **esattamente due**:

1.  `AnagraficaType`: `Denominazione` **XOR** (`Nome` + `Cognome`) — con `Titolo`/`CodEORI` opzionali dopo.
    
2.  `RappresentanteFiscaleCessionarioType`: `IdFiscaleIVA` + choice(`Denominazione` | `Nome`+`Cognome`).
    

Ordini canonici da rispettare in _scrittura_ (e da validare in _lettura_):
| Blocco | Ordine |
| --- | --- |
| Radice | `FatturaElettronicaHeader` → `FatturaElettronicaBody`(1..∞) → `ds:Signature`? |
| Header | `DatiTrasmissione` → `CedentePrestatore` → `RappresentanteFiscale`? → `CessionarioCommittente` → `TerzoIntermediarioOSoggettoEmittente`? → `SoggettoEmittente`? |
| DatiTrasmissione | `IdTrasmittente` → `ProgressivoInvio` → `FormatoTrasmissione` → `CodiceDestinatario` → `ContattiTrasmittente`? → `PECDestinatario`? |
| **DatiGeneraliDocumento** | `TipoDocumento` → `Divisa` → `Data` → `Numero` → `DatiRitenuta`* → `DatiBollo`? → `DatiCassaPrevidenziale`* → `ScontoMaggiorazione`* → `ImportoTotaleDocumento`? → `Arrotondamento`? → `Causale`* → `Art73`? |
| DatiGenerali | `DatiGeneraliDocumento` → `DatiOrdineAcquisto`* → `DatiContratto`* → `DatiConvenzione`* → `DatiRicezione`* → `DatiFattureCollegate`* → `DatiSAL`* → `DatiDDT`* → `DatiTrasporto`? → `FatturaPrincipale`? |
| Body | `DatiGenerali` → `DatiBeniServizi` → `DatiVeicoli`? → `DatiPagamento`* → `Allegati`* |
| DettaglioLinee | `NumeroLinea` → `TipoCessionePrestazione`? → `CodiceArticolo`* → `Descrizione` → `Quantita`? → `UnitaMisura`? → `DataInizioPeriodo`? → `DataFinePeriodo`? → `PrezzoUnitario` → `ScontoMaggiorazione`* → `PrezzoTotale` → `AliquotaIVA` → `Ritenuta`? → `Natura`? → `RiferimentoAmministrazione`? → `AltriDatiGestionali`* |

**Elementi obbligatori (minOccurs=1) da ricordare:** `@versione`; `FatturaElettronicaHeader`; ≥1 `FatturaElettronicaBody`; `DatiTrasmissione` + `IdTrasmittente`/`IdPaese`/`IdCodice` + `ProgressivoInvio` + `FormatoTrasmissione` + `CodiceDestinatario`; `CedentePrestatore` + `DatiAnagrafici` + `IdFiscaleIVA` + `Anagrafica` + `RegimeFiscale` + `Sede` (con `Indirizzo`, `CAP`, `Comune`, `Nazione`); `CessionarioCommittente` + `DatiAnagrafici` + `Anagrafica` + `Sede`; `DatiGenerali` + `DatiGeneraliDocumento` (`TipoDocumento`, `Divisa`, `Data`, `Numero`); `DatiBeniServizi` + ≥1 `DettaglioLinee` (`NumeroLinea`, `Descrizione`, `PrezzoUnitario`, `PrezzoTotale`, `AliquotaIVA`) + ≥1 `DatiRiepilogo` (`AliquotaIVA`, `ImponibileImporto`, `Imposta`).
**Asimmetria da non dimenticare:** nel `CessionarioCommittente` `IdFiscaleIVA` **è** `0..1` (non obbligatorio), mentre nel `CedentePrestatore` è `1..1`. La regola "almeno uno tra IdFiscaleIVA e CodiceFiscale" per il cessionario **non è nello XSD**: è il controllo **00417**.

* * *

## 9. Vincoli a livello di file [V, salvo diversa indicazione]

### 9.1 Nomenclatura

```
codicePaese + identificativoUnivoco + "_" + progressivoUnivoco + estensione
```

*   `codicePaese`: ISO 3166-1 alpha-2 (es. `IT`)
    
*   `identificativoUnivoco`: **11–16 caratteri** se il paese è `IT`, **2–28** altrimenti
    
*   `progressivoUnivoco`: max **5** caratteri alfanumerici `[a-z][A-Z][0-9]`
    
*   separatore: underscore (ASCII 95)
    
*   esempi ufficiali: `ITAAABBB99T99X999W_00001.xml`, `IT99999999999_00002.xml.p7m`, `ITAAABBB99T99X999W_00001.zip`
    
*   Controlli: **00001** (nome file non valido), **00002** (nome file duplicato — richiede lo storico SdI, quindi **non** replicabile)
    
*   Nota ufficiale: l'identificativo usato nel nome file _"non è soggetto a controlli di validità, esistenza o coerenza con i dati presenti in fattura"_.
    

### 9.2 Dimensioni

| Oggetto | Limite |
| --- | --- |
| Singolo file fattura (o archivio .zip) | **≤ 5 MB** (controllo **00003**) |
| Messaggio PEC (con allegati) | ≤ 30 MB |
| Supporto SFTP | ≤ 150 MB (con regola anti-frammentazione) |

### 9.3 Formato ed estensioni

*   Un file XML singolo **oppure** più file XML in un **archivio ZIP** (unica compressione ammessa).
    
*   Estensioni: `.xml` (firma **XAdES**) · `.xml.p7m` (firma **CAdES**) · `.zip` (archivio).
    
*   **L'archivio .zip non va firmato**: _"non è il file compresso (.zip) che deve essere firmato digitalmente, ma ogni singolo file in esso contenuto"_. I controlli su un archivio sono **per singolo file**, mentre su un "lotto" (`FatturaElettronicaBody` multipli) sono **sull'intero file**.
    
*   Controllo **00106**: file/archivio vuoto o corrotto.
    
*   Encoding: la dichiarazione degli esempi è UTF-8. Le stringhe del tracciato sono comunque limitate a Basic Latin / Latin-1 Supplement, quindi l'uso di caratteri fuori da quei blocchi è già vietato a livello XSD (eccetto `String35LatinExtType`).
    

### 9.4 Canali di trasmissione (contesto, non validazione locale)

PEC (`sdi01@pec.fatturapa.it` per il primo invio), web service **SdICoop**, **SdiFTP** (oggi SFTP), upload web dall'area riservata "Fatture e Corrispettivi". Canali accreditati richiedono accreditamento; il SdI risponde con notifiche (`RC`, `NS`, `MC`, `MT`, `NE`, `EC`, `SE`, `DT`, `AT`).

### 9.5 Notifiche (per chi valida anche i messaggi)

`MessaggiFatturaTypes_v1.0` è lo schema delle notifiche; la naming convention delle ricevute è `<nome-fattura-senza-estensione>_<Tipo>_<progressivo>.xml` con `Tipo ∈ {RC, NS, MC, MT}` (es. `ITAAABBB99T99X999W_00001_NS_001.xml`). Il tetto di **50 errori di formato** riportati (codice **00201**) è una policy SdI.

* * *

## 10. Firma digitale: cosa è obbligatorio e cosa è verificabile offline

### 10.1 Obbligo e formati

|  | Obbligo | Formati ammessi |
| --- | --- | --- |
| **FPA12 (PA)** | **Firma digitale obbligatoria** (firma elettronica qualificata + riferimento temporale) | CAdES-BES (ETSI TS 101 733) / XAdES-BES (ETSI TS 101 903 v1.4.1), oltre a CAdES con certificato AdE |
| **FPR12 / FSM10 (privati)** | **Facoltativa** — il SdI gestisce file firmati e non firmati | come sopra |

Vincoli tecnici rilevanti (verbatim dalle specifiche, §1.2.1) [V/S]:

*   _"Nell'ambito del formato di firma XML l'unica modalità accettata è quella_ **_enveloped_**_"_.
    
*   La firma XAdES deve avere `Reference` con `URI=""` **oppure** `URI="#iddoc"` — l'attributo `URI` non può essere omesso.
    
*   Il **riferimento temporale** è l'attributo `signing time`, che deve essere presente e **non successivo al momento di ricezione**.
    
*   Il certificato AdE **non è una CA qualificata**: _"la verifica di questo tipo di firma può essere fatta solo utilizzando il servizio di verifica di Agenzia delle Entrate"_.
    

### 10.2 Controlli SdI sulla firma e replicabilità

| Codice | Descrizione ufficiale | Offline? |
| --- | --- | --- |
| **00100** | Certificato di firma scaduto | **Parziale** (date di validità sì) |
| **00101** | Certificato di firma revocato | **No** (CRL/OCSP) |
| **00102** | La firma elettronica apposta al file non risulta valida (integrità) | **Parziale/sì** (verifica crittografica se hai la catena; senza trust store è parziale) |
| **00103** | Alla firma elettronica apposta al file manca il riferimento temporale | **Sì** |
| **00104** | La CA che ha emesso il certificato non risulta nell'elenco delle CA affidabili | **Parziale** (richiede un trust store AgID da mantenere) |
| **00105** | Riferimento temporale successivo alla data di ricezione | **Parziale** (confrontabile con "now"; non con la data reale di ricezione SdI) |
| **00106** | File/archivio vuoto o corrotto | **Sì** |
| **00107** | Il certificato di firma non è valido | **Parziale** |

### 10.3 Cosa può fare concretamente un validatore offline

1.  **Integrità** — verifica XMLDSig per XAdES (`Canonicalization`, `DigestValue` su `Reference URI=""`/`#iddoc`, `SignatureValue` con la chiave pubblica) o CMS/PKCS#7 per `.p7m`. **Fattibile offline.**
    
2.  **Catena di certificazione** — fattibile **solo spedendo** (o aggiornando periodicamente) i root/intermedi di AgID. Non esiste un'API offline dell'elenco AgID.
    
3.  **Validità temporale** — `notBefore`/`notAfter` rispetto a `signing time` dichiarato e/o a "now".
    
4.  **Revoca** — richiede CRL/OCSP. Opzioni: (a) spedire e aggiornare le CRL dei QTSP italiani; (b) caching OCSP; (c) **fail-open**: dichiarare "firma non verificabile per revoca" e **non** affermare mai "firma valida".
    
5.  **Riferimento temporale** — attenzione: il SdI usa `SigningTime` (tempo _dichiarato_, non token TSA). Un validatore può solo verificarne **presenza** e **non-futurità**.
    
6.  **Certificati AdE non qualificati** — **non verificabili offline per definizione**.
    

* * *

## 11. Punti aperti / da verificare prima di congelare un validatore

**[?]** elenco da chiudere con i PDF ufficiali (Allegato A 1.9.1 e Appendice 1):

1.  Contenuto esatto e completo della **Appendice 1** di Allegato A 1.9.1 (tutti i codici ordinaria **e** semplificata, con i path nella numerazione semplificata).
    
2.  Se **00426** (coerenza `PECDestinatario` ↔ `CodiceDestinatario = 0000000`) sia effettivamente **abrogato**: presente nell'Elenco v1.8 del 01/10/2022 [V], risulta **assente** dall'edizione v2.0 [S], e il testo 1.9.1 rende la PEC _facoltativa_ con `0000000`. → **Trattare la sola implicazione "PEC valorizzata ⇒ CodiceDestinatario = 0000000" come vincolo, e non bloccare l'assenza di PEC.**
    
3.  Albero completo e definitivo di `Schema VFSM10` **v1.0.2** (differenze esatte vs ordinaria).
    
4.  Soglia e deroga esatte del **00460** in 1.9.1 (400 € / forfettari / RF20).
    
5.  Precisione dei tipi data-ora (`xs:dateTime`): se siano ammessi **offset di fuso** e **frazioni di secondo** (nelle versioni verificate non comparivano nei campi data).
    
6.  Se la **PEC** debba rispettare formati/lunghezze ulteriori rispetto a `EmailType` (max 256) e se esistano blacklist SdI replicabili (es. `sdixx@pec.fatturapa.it` → 00330, replicabile con una lista locale).
    
7.  Servizi 2025/2026 che _non_ toccano l'XML della fattura ma vanno conosciuti: **QR Code partita IVA**, **integrazione CUP** (Provv. 10/12/2025, servizio dal 27/01/2026) e **codice CUN** (Provv. 18/03/2026) [?].