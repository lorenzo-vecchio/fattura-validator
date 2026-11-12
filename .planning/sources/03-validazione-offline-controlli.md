# Fatture Elettroniche italiane — Parte 3/3

## Validazione offline: catalogo dei controlli, regole aritmetiche, algoritmi e tabelle

**Data:** 17 settembre 2026 · **Ambito:** solo ciò che è verificabile **localmente sul file**, senza SdI, Anagrafe tributaria, IndicePA o VIES.  
**Fonti primarie utilizzate:** _Elenco dei controlli effettuati sul file fattura del Sistema di Interscambio_, **v1.8** del 01/10/2022 (testo integrale letto il 17/09/2026, con pseudocodice) [V]; XSD `Schema_VFPR12_v1.2.3.xsd` (17/09/2026) [V]; Allegato A — Specifiche tecniche **1.9.1** (agg. 31/03/2026) [V/S]; _Elenco controlli_ **v2.0** (31/01/2025) [S].  
**Legenda:** **[V]** verificato su fonte primaria · **[S]** fonte secondaria · **[?]** da verificare · ✅ offline · 🟡 offline parziale · ❌ online

* * *

## 1. Tassonomia ufficiale delle verifiche SdI e mappa offline/online

Classificazione **verbatim** dei controlli (Elenco controlli, "Verifiche effettuate sul file fattura") [V]:
| # | Categoria ufficiale | Cosa verifica | Dati necessari | Offline? | Codici |
| --- | --- | --- | --- | --- | --- |
| A | **Nomenclatura ed unicità del file trasmesso** | nome conforme e mai usato prima | nome file + **storico SdI** | 🟡 (forma sì, unicità no) | 00001, 00002 |
| B | **Dimensioni del file** | rispetto dei limiti del canale | byte del file | ✅ | 00003 |
| C | **Integrità del documento** | il file non è stato modificato dopo la firma | file firmato + crittografia | 🟡 | 00102 |
| D | **Autenticità del certificato di firma** | certificato non scaduto/revocato/sospeso; CA affidabile | certificato + **CRL/OCSP** + elenco CA AgID | 🟡/❌ | 00100, 00101, 00104, 00107 |
| E | **Verifica del file e del formato** | conformità a XSD e regole tecniche | XML + XSD | ✅ | 00103, 00105, 00106, 00200, 00201 |
| F | **Verifiche sul contenuto** | coerenza di contenuto ex regole tecniche | **solo XML** | ✅ (quasi totale) | 00400–00445, 00471–00477 |
| G | **Verifica di validità del contenuto** | presenza/validità dei dati necessari all'inoltro | **Anagrafe tributaria, IndicePA, registri canali** | ❌ (in gran parte) | 00300–00330, 00398, 00399 |
| H | **Verifica di unicità della fattura** | fattura già trasmessa/elaborata | **archivio SdI** | ❌ (solo intra-file) | 00404, 00409 |

Definizione ufficiale della verifica di validità (la frase chiave per capire il confine) [V]:

> _"La verifica viene effettuata per accertare la presenza e la validità dei dati necessari al corretto inoltro del documento al destinatario… in particolare viene effettuato un controllo: sull'elemento_ `<CodiceDestinatario>`_: si verifica che sia presente e attivo nell'anagrafica di riferimento (_**_IndicePA_**_)…; sulla validità dei codici fiscali e delle partite IVA… attraverso una verifica di presenza nell'_**_anagrafe tributaria_**_; il controllo non è effettuato per gli identificativi fiscali assegnati da autorità estere; sulla coerenza tra partita IVA e codice fiscale…"_

**Conseguenza pratica:** i codici **003xx** sono (quasi) tutti fuori portata per un validatore locale; i codici **004xx** sono (quasi) tutti implementabili.

* * *

## 2. Catalogo completo dei controlli

### 2.1 File — nomenclatura, dimensione, integrità, certificato, formato

| Codice | Descrizione ufficiale (verbatim) | Offline? | Note |
| --- | --- | --- | --- |
| `00001` | **Nome file non valido** | 🟡 | verifica solo la forma: `ISO3166alpha2 + idUnivoco(11–16 se IT, 2–28 altrimenti) + "_" + progressivo(≤5 alfanum) + (.xml/.xml.p7m/.zip)` |
| `00002` | **Nome file duplicato** | ❌ | richiede lo storico dei nomi ricevuti dal SdI. _Replicabile solo come "duplicato all'interno del batch che stai inviando"_ |
| `00003` | **Le dimensioni del file superano quelle ammesse** | ✅ | singolo file fattura/zip **≤ 5 MB**; messaggio PEC ≤ 30 MB; supporto SFTP ≤ 150 MB |
| `00100` | Certificato di firma **scaduto** | 🟡 | date `notBefore`/`notAfter` vs `signing time`/now |
| `00101` | Certificato di firma **revocato** | ❌ | CRL/OCSP |
| `00102` | **La firma elettronica apposta al file non risulta valida** (integrità) | 🟡/✅ | verifica XMLDSig (XAdES) o CMS/PKCS#7 (`.p7m`); ✅ se hai la chiave pubblica del certificato |
| `00103` | **Alla firma elettronica apposta al file manca il riferimento temporale** | ✅ | assenza dell'attributo `signing time` |
| `00104` | **La CA che ha emesso il certificato di firma non risulta nell'elenco delle CA affidabili** | 🟡 | richiede un trust store (AgID) da spedire e mantenere |
| `00105` | **Il riferimento temporale associato alla firma è successivo alla data di ricezione del file** | 🟡 | confrontabile con "adesso"; non con la data reale di ricezione SdI |
| `00106` | **File / archivio vuoto o corrotto** | ✅ | validazione ZIP (integrità, presenza contenuti) |
| `00107` | **Il certificato di firma non è valido** | 🟡 | formato/struttura del certificato |
| `00200` | **File non conforme al formato** (la descrizione riporta l'indicazione puntuale della non conformità) | ✅ | **validazione XSD + regole strutturali** |
| `00201` | **Non è possibile procedere con ulteriori controlli, perché gli errori di formato presenti nel file superano il massimo previsto (50)** | ✅ | soglia di 50 errori riportati per file; policy SdI |

### 2.2 Contenuto (004xx) — il cuore del validatore locale

Descrizioni **verbatim** dall'Elenco controlli [V] (integrate, dove indicato, con le novità 1.9/1.9.1 [S]).
| Codice | Descrizione ufficiale (verbatim) | Offline? |
| --- | --- | --- |
| `00400` | 2.2.1.14 `<Natura>` non presente a fronte di 2.2.1.12 `<AliquotaIVA>` pari a zero | ✅ |
| `00401` | 2.2.1.14 `<Natura>` presente a fronte di 2.2.1.12 `<AliquotaIVA>` diversa da zero | ✅ |
| `00403` | 2.1.1.3 `<Data>` successiva alla data di ricezione | 🟡 (vs "oggi") |
| `00404` | **Fattura duplicata** | ❌ (🟡 solo intra-file) |
| `00409` | **Fattura duplicata nel lotto** | ✅ (confronto tra i `FatturaElettronicaBody` dello stesso file) |
| `00411` | 2.1.1.5 `<DatiRitenuta>` non presente a fronte di almeno un blocco 2.2.1 `<DettaglioLinee>` con 2.2.1.13 `<Ritenuta>` uguale a SI | ✅ |
| `00413` | 2.1.1.7.7 `<Natura>` non presente a fronte di 2.1.1.7.5 `<AliquotaIVA>` pari a zero (cassa previdenziale) | ✅ |
| `00414` | 2.1.1.7.7 `<Natura>` presente a fronte di 2.1.1.7.5 `<AliquotaIVA>` diversa da zero | ✅ |
| `00415` | 2.1.1.5 `<DatiRitenuta>` non presente a fronte di 2.1.1.7.6 `<Ritenuta>` uguale a SI | ✅ |
| `00417` | 1.4.1.1 `<IdFiscaleIVA>` e 1.4.1.2 `<CodiceFiscale>` non valorizzati (almeno uno dei due deve essere valorizzato) | ✅ |
| `00418` | 2.1.1.3 `<Data>` antecedente a 2.1.6.3 `<Data>` (nota di credito vs fattura collegata) | ✅ |
| `00419` | 2.2.2 `<DatiRiepilogo>` non presente in corrispondenza di almeno un valore di 2.1.1.7.5 `<AliquotaIVA>` o 2.2.1.12 `<AliquotaIVA>` | ✅ |
| `00420` | 2.2.2.2 `<Natura>` con valore di tipo **N6** a fronte di 2.2.2.7 `<EsigibilitaIVA>` uguale a **S** (scissione pagamenti) | ✅ |
| `00421` | 2.2.2.6 `<Imposta>` non calcolato secondo le regole definite nelle specifiche tecniche | ✅ (tolleranza ±0,01 €) |
| `00422` | 2.2.2.5 `<ImponibileImporto>` non calcolato secondo le regole definite nelle specifiche tecniche | ✅ (tolleranza ±1 €) |
| `00423` | 2.2.1.11 `<PrezzoTotale>` non calcolato secondo le regole definite nelle specifiche tecniche | ✅ (tolleranza ±0,01 €) |
| `00424` | 2.2.1.12 `<AliquotaIVA>` o 2.2.2.1 `<AliquotaIVA>` o 2.1.1.7.5 `<AliquotaIVA>` non indicata in termini percentuali | ✅ |
| `00425` | 2.1.1.4 `<Numero>` non contenente caratteri numerici | ✅ |
| `00426` | 1.1.6 `<PECDestinatario>` non valorizzato a fronte di 1.1.4 `<CodiceDestinatario>` = 0000000, **o** PEC valorizzata a fronte di CodiceDestinatario ≠ 0000000 | ⚠️ **presente nell'Elenco v1.8 [V], segnalato assente nella v2.0 [S]** — vedi §7 |
| `00427` | 1.1.4 `<CodiceDestinatario>` di 7 caratteri non ammesso con 1.1.3 `<FormatoTrasmissione>` = FPA12, o 6 caratteri non ammesso con FPR12/FSM10 | ✅ |
| `00428` | 1.1.3 `<FormatoTrasmissione>` non coerente con il valore dell'attributo VERSIONE | ✅ |
| `00429` | 2.2.2.2 `<Natura>` non presente a fronte di 2.2.2.1 `<AliquotaIVA>` pari a zero | ✅ |
| `00430` | 2.2.2.2 `<Natura>` presente a fronte di 2.2.2.1 `<AliquotaIVA>` diversa da zero | ✅ |
| `00437` | 2.1.1.8.2 `<Percentuale>` e 2.1.1.8.3 `<Importo>` non presenti a fronte di 2.1.1.8.1 `<Tipo>` valorizzato (ScontoMaggiorazione di documento) | ✅ |
| `00438` | 2.2.1.10.2 `<Percentuale>` e 2.2.1.10.3 `<Importo>` non presenti a fronte di 2.2.1.10.1 `<Tipo>` valorizzato (ScontoMaggiorazione di linea) | ✅ |
| `00443` | Non c'è corrispondenza tra i valori indicati nell'elemento 2.2.1.12 `<AliquotaIVA>` o 2.1.1.7.5 `<AliquotaIVA>` e quelli dell'elemento 2.2.2.1 `<AliquotaIVA>` | ✅ |
| `00444` | Non c'è corrispondenza tra i valori indicati nell'elemento 2.2.1.14 `<Natura>` o 2.1.1.7.7 `<Natura>` e quelli dell'elemento 2.2.2.2 `<Natura>` | ✅ |
| `00445` | _(in vigore dal 01/01/2021)_ Non è più ammesso il valore generico **N2, N3 o N6** come codice natura dell'operazione | ✅ |
| `00460` | Importo complessivo della fattura semplificata superiore al limite dell'art. **21-bis** DPR 633/72 (soglia storicamente **400 €**; dal **1.9** esonero per regime forfettario / RF20) | ✅ [?] confermare soglia/deroga su 1.9.1 |
| `00471` | Per il valore di 2.1.1.1 `<TipoDocumento>` il **cedente/prestatore non può essere uguale** al cessionario/committente | ✅ |
| `00472` | Per il valore di 2.1.1.1 `<TipoDocumento>` il **cedente/prestatore deve essere uguale** al cessionario/committente | ✅ |
| `00473` | Per il valore di 2.1.1.1 `<TipoDocumento>` il valore in 1.2.1.1.1 `<IdPaese>` non è ammesso | ✅ |
| `00474` | Per il valore di 2.1.1.1 `<TipoDocumento>` non sono ammesse linee di dettaglio con 2.2.1.12 `<AliquotaIVA>` = zero | ✅ |
| `00475` | Per il valore di 2.1.1.1 `<TipoDocumento>` deve essere presente 1.4.1.1 `<IdFiscaleIVA>` del cessionario/committente | ✅ |
| `00476` | Gli elementi 1.2.1.1.1 `<IdPaese>` e 1.4.1.1.1 `<IdPaese>` non possono essere entrambi valorizzati con codice diverso da IT | ✅ |
| `00477` | **Fattura recante titolo di non imponibilità ai fini IVA ai sensi dell'art. 8, comma 1, lett. c) DPR 633/72, con Dichiarazione d'intento invalidata** | ❌ (banca dati AdE) |

### 2.3 Validità del contenuto (003xx) — quasi tutte online

| Codice | Descrizione ufficiale (verbatim) | Offline? | Cosa può fare un validatore locale |
| --- | --- | --- | --- |
| `00300` | 1.1.1.2 `<IdCodice>` non valido (trasmittente) | ❌ | ✅ **solo** formato/checksum |
| `00301` | 1.2.1.1.2 `<IdCodice>` non valido (cedente; include P.IVA cessata) | ❌ | ✅ checksum P.IVA/CF |
| `00302` | 1.2.1.2 `<CodiceFiscale>` non valido (cedente) | ❌ | ✅ checksum/struttura |
| `00303` | 1.3.1.1.2 `<IdCodice>` o 1.4.4.1.2 `<IdCodice>` non valido (rappresentante fiscale) | ❌ | ✅ checksum |
| `00304` | 1.3.1.2 `<CodiceFiscale>` non valido (rappresentante fiscale) | ❌ | ✅ checksum |
| `00305` | 1.4.1.1.2 `<IdCodice>` non valido (cessionario) | ❌ | ✅ checksum |
| `00306` | 1.4.1.2 `<CodiceFiscale>` non valido (cessionario) | ❌ | ✅ checksum |
| `00311` | 1.1.4 `<CodiceDestinatario>` **non valido** (non presente nel registro canali / IPA) | ❌ | 🟡 forma + eventuale tabella IPA locale |
| `00312` | 1.1.4 `<CodiceDestinatario>` **non attivo** | ❌ | — |
| `00313` | `CodiceDestinatario = "XXXXXXX"` ammesso **solo** verso soggetti esteri: `1.4.1.1.1 <IdPaese>` deve essere ≠ "IT" | ✅ | controllo pienamente locale |
| `00320` | 1.2.1.1 `<IdFiscaleIVA>` e 1.2.1.2 `<CodiceFiscale>` non coerenti (cedente) | ❌ | 🟡 euristica: se CF è numerico a 11 cifre dovrebbe coincidere con la P.IVA |
| `00321` | 1.2.1.2 `<CodiceFiscale>` di soggetto non partecipante al gruppo IVA | ❌ | — |
| `00322` | 1.2.1.2 `<CodiceFiscale>` non presente a fronte di 1.2.1.1 `<IdFiscaleIVA>` di gruppo IVA | ❌ | — |
| `00323` | 1.2.1.1.2 `<IdCodice>` corrispondente a una partita IVA cessata da oltre 5 anni | ❌ | — |
| `00324` | 1.4.1.1 `<IdFiscaleIVA>` e 1.4.1.2 `<CodiceFiscale>` non coerenti (cessionario) | ❌ | 🟡 euristica come 00320 |
| `00325` | 1.4.1.2 `<CodiceFiscale>` di soggetto non partecipante al gruppo IVA | ❌ | — |
| `00326` | 1.4.1.2 `<CodiceFiscale>` non presente a fronte di 1.4.1.1 `<IdFiscaleIVA>` di gruppo IVA | ❌ | — |
| `00327` | **NUOVO in 1.9.1** — `1.4.1.2 <CodiceFiscale>` di **gruppo IVA** non riferito ad un partecipante, in assenza di `IdFiscaleIVA` di gruppo IVA: il CF va valorizzato con quello della **società partecipante**, non del gruppo | ❌ | — |
| `00330` | 1.1.6 `<PECDestinatario>` contenente un valore **non ammesso** (es. caselle SdI `sdixx@pec.fatturapa.it`) | 🟡 | ✅ con una **blacklist locale** dei domini/caselle SdI |
| `00398` | Codice Ufficio presente ed univocamente identificabile nell'anagrafica IPA, con `CodiceDestinatario` = codice ufficio "Centrale" | ❌ | — |
| `00399` | CessionarioCommittente presente nell'anagrafica IPA con `FormatoTrasmissione = "FPR12"` | ❌ | — |

### 2.4 Altri codici non SdI citati nella pratica

| Codice | Significato | Fonte |
| --- | --- | --- |
| `00459` | `RegimeFiscale` = RF03 (regime abrogato) | **[S]** implementato nella libreria di riferimento .NET; non trovato nell'Elenco controlli ufficiale → trattare come _warning di dominio_, non come "il SdI scarta con 00459" |

* * *

## 3. Pseudocodice dei controlli complessi (verbatim dall'Elenco controlli, rielaborato)

### 3.1 `00419` — presenza di un `DatiRiepilogo` per ogni aliquota

```
AliquoteIVA = insieme vuoto
per ogni DatiCassaPrevidenziale:  aggiungi AliquotaIVA a AliquoteIVA
per ogni DettaglioLinee:          aggiungi AliquotaIVA a AliquoteIVA
TotAliquoteIVA = |AliquoteIVA|
TotRiepiloghi  = numero di blocchi DatiRiepilogo
IF (TotRiepiloghi < TotAliquoteIVA) → ERRORE 00419
```

> Attenzione: il controllo confronta **numeri di blocchi** con **numero di aliquote distinte**, non con un insieme. Un `DatiRiepilogo` in più non produce 00419 ma può produrre 00443/00444.

### 3.2 `00420` — split payment vs reverse charge

```
per ogni DatiRiepilogo:
  IF (EsigibilitaIVA == "S" AND Natura ∈ {N6, N6.1..N6.9}) → ERRORE 00420
```

> Nota: `EsigibilitaIVA = S` è la **scissione dei pagamenti** (split payment), **non** una natura. `D` = differita, `I` = immediata. I documenti dell'elenco v1.8 elencavano esplicitamente `N6` e `N6.1`…`N6.9`.

### 3.3 `00421` — calcolo dell'Imposta

```
per ogni DatiRiepilogo:
  atteso = (AliquotaIVA * ImponibileImporto) / 100
  atteso = arrotonda_2decimali(atteso)        // difetto se 3ª cifra < 5, eccesso se ≥ 5
  IF ( |Imposta − atteso| ≥ 0.01 ) → ERRORE 00421
```

Esempi ufficiali: `1500.2448 → 1500.24`; `1500.2458 → 1500.25`. **Tolleranza: 1 centesimo di euro.**

### 3.4 `00422` — calcolo dell'ImponibileImporto (per aliquota)

```
per ogni valore distinto di AliquotaIVA:
  Σ ImponibileImporto[x]  deve  =  Σ PrezzoTotale[y] + Σ ImportoContributoCassa[z] + Σ Arrotondamento[x]
  IF ( |differenza| ≥ 1.00 ) → ERRORE 00422
```

dove `x` = blocchi `DatiRiepilogo` con quella aliquota, `y` = blocchi `DettaglioLinee` con quella aliquota, `z` = blocchi `DatiCassaPrevidenziale` con quella aliquota.  
**Tolleranza: 1 euro.**

> ⚠️ Il testo ufficiale riporta `+ Σ Arrotondamento` nella formula ma non precisa se esso vada sommato o sottratto in funzione del segno: `Arrotondamento` è un `Amount8DecimalType` **con segno ammesso** (`[\-]?…`). Nella pratica va trattato come **addendo algebrico** (il segno è nel valore). **[?] da confermare** — è una delle divergenze note tra implementazioni.

### 3.5 `00423` — calcolo del PrezzoTotale (con sconti/maggiorazioni a cascata)

```
per ogni DettaglioLinee:
  Quantita_eff = Quantita ?? 1
  PrezzoTotale_atteso = ( PrezzoUnitario ± ScontoTot / MaggiorazioneTot ) * Quantita_eff
  IF ( |PrezzoTotale − PrezzoTotale_atteso| ≥ 0.01 ) → ERRORE 00423
```

Regole ufficiali sul calcolo degli sconti/maggiorazioni **multiple**:

1.  Se rappresentati **come importo**: il totale è la **somma degli importi**.
    
2.  Se rappresentati **come percentuale**: si applica l'effetto **"a cascata"**, cioè ogni percentuale si applica al prezzo **già scontato/maggiorato** dal blocco precedente:  
    `(PrezzoUnitario_corrente * Percentuale) / 100` con aggiornamento progressivo.
    
3.  **In caso di presenza contemporanea di** `Importo` **e** `Percentuale` **nello stesso blocco, si considera solo l'**`Importo` (verbatim).
    
4.  **Tolleranza: 1 centesimo di euro.** Attenzione al numero di decimali: `PrezzoUnitario` è `Amount8DecimalType` ma gli **sconti a 3+ decimali** sono stati fonte di scarti reali → verificare i decimali effettivi.
    

Esempi ufficiali (verbatim):

*   **Caso 1** (tre sconti in importo, 10.05 + 5.25 + 4.70): `[500.00 − 20.00] × 100.00 = 48000.00`
    
*   **Caso 2** (tre sconti in percentuale, 8.50 → 5.15 → 3.00, a cascata): risultato `42092.05875`; _"per effetto della tolleranza è ammissibile anche 42092.05 oppure 42092.06"_
    
*   **Caso 3** (mix importo/percentuale): `41056.12`
    

> **Implicazione implementativa obbligatoria:** usare **decimali esatti a precisione arbitraria o interi in centesimi/milionesimi** — mai `f64`. Le librerie mature hanno storicamente sbagliato proprio qui (tolleranze "strette" invece che "larghe" su 00421/00423).

### 3.6 `00427` / `00428` — coerenza formato/versione

```
IF (FormatoTrasmissione == "FPA12" AND lunghezza(CodiceDestinatario) == 7) → ERRORE 00427
IF (FormatoTrasmissione ∈ {FPR12, FSM10} AND lunghezza(CodiceDestinatario) == 6) → ERRORE 00427
IF (FormatoTrasmissione != valore_attributo_versione_sulla_radice) → ERRORE 00428
```

> Attenzione: il controllo sulla **lunghezza** è binario (6 vs 7). Il pattern XSD è `{6,7}`: un `CodiceDestinatario` di 5 o 8 caratteri fallisce già a livello XSD (00200), non 00427.

### 3.7 `00429` / `00430` e simmetrici — Natura ↔ aliquota

```
# DettaglioLinee → 00400 / 00401
IF (AliquotaIVA == 0.00)  { IF (Natura assente) → 00400 }  ELSE { IF (Natura presente) → 00401 }
# DatiCassaPrevidenziale → 00413 / 00414   (identico, ripetuto per ogni blocco)
# DatiRiepilogo → 00429 / 00430            (identico, ripetuto per ogni blocco)
```

**Eccezione TD16**: se `TipoDocumento = TD16` è ammesso avere **contemporaneamente** Natura e `AliquotaIVA ≠ 0`.

### 3.8 `00437` / `00438` — ScontoMaggiorazione

```
IF (esiste Tipo) AND (non esiste Percentuale) AND (non esiste Importo) → ERRORE 00437 (documento) / 00438 (linea)
```

> Le due direzioni **non** sono simmetriche nel testo ufficiale: il controllo scatta sul `Tipo` **valorizzato senza né percentuale né importo**. La presenza di **entrambi** `Percentuale` e `Importo` è _tollerata_ dal SdI (l'XSD li ammette entrambi opzionali) anche se la linea guida di compilazione suggerisce l'esclusività; ai fini del calcolo si usa l'`Importo`.

### 3.9 `00443` / `00444` — corrispondenza aliquote/nature con il riepilogo

```
per ogni DatiCassaPrevidenziale:  IF (AliquotaIVA ∉ AliquoteIVA_dei_Riepiloghi) → ERRORE 00443
per ogni DettaglioLinee:          IF (AliquotaIVA ∉ AliquoteIVA_dei_Riepiloghi) → ERRORE 00443
per ogni DatiCassaPrevidenziale:  IF (Natura    ∉ Nature_dei_Riepiloghi)    → ERRORE 00444
per ogni DettaglioLinee:          IF (Natura    ∉ Nature_dei_Riepiloghi)    → ERRORE 00444
```

(nel testo ufficiale, per refuso, il secondo blocco 00444 riporta "00443" — è **00444**.)

### 3.10 `00445` — Natura generiche

```
IF (Natura ∈ {N2, N3, N6}) in DettaglioLinee / DatiCassaPrevidenziale / DatiRiepilogo → ERRORE 00445
(per la fattura semplificata: {N2, N3})
```

### 3.11 `00471`–`00476` — regole legate al `TipoDocumento`

```
# 00471 — cedente ≠ cessionario
IF (TipoDocumento ∈ {TD01,TD02,TD03,TD06,TD07,TD16,TD17,TD18,TD19,TD20,TD24,TD25,TD28,TD29}
    AND IdFiscaleIVA(cedente) == IdFiscaleIVA(cessionario)) → ERRORE 00471

# 00472 — cedente = cessionario
IF (TipoDocumento ∈ {TD21, TD27} AND IdFiscaleIVA(cedente) != IdFiscaleIVA(cessionario)) → ERRORE 00472

# 00473 — IdPaese del cedente
IF (TipoDocumento ∈ {TD17,TD18,TD19} AND IdPaese(cedente) == "IT") → ERRORE 00473
IF (TipoDocumento == TD28 AND IdPaese(cedente) != "SM")          → ERRORE 00473
IF (TipoDocumento == TD29 AND IdPaese(cedente) != "IT")          → ERRORE 00473

# 00474 — niente linee a IVA zero
IF (TipoDocumento == TD21 AND esiste DettaglioLinee con AliquotaIVA == 0.00) → ERRORE 00474

# 00475 — IdFiscaleIVA del cessionario obbligatorio
IF (TipoDocumento ∈ {TD16,TD17,TD18,TD19,TD20,TD22,TD23,TD28,TD29} AND IdFiscaleIVA(cessionario) assente) → ERRORE 00475

# 00476 — non entrambi esteri
IF (IdPaese(cedente) != "IT" AND IdPaese(cessionario) != "IT") → ERRORE 00476
```

**Nota da implementazione:** nel controllo 00471 il testo ufficiale precisa che _"nei casi in cui in fattura sia presente il solo IDCessionarioCF per il cessionario, si risale all'IDCessionarioPIVA attraverso interrogazione di Anagrafe Tributaria"_ → **quando il cessionario ha solo il codice fiscale, il confronto offline non è pienamente replicabile**. Un validatore locale deve: (a) confrontare quando entrambi hanno `IdFiscaleIVA`; (b) quando il cessionario ha solo il CF, applicare l'euristica "CF numerico a 11 cifre ≡ P.IVA" e **degradare a warning** altrimenti.

### 3.12 `00404` / `00409` — duplicazione (il controllo più complesso)

Chiave: **(identificativo cedente/prestatore, anno della data fattura, numero fattura)**, con queste deviazioni:

*   se `Art73 = "SI"` → la chiave usa la **data completa**, non l'anno;
    
*   se `SoggettoEmittente = "CC"` → il soggetto chiave è il **cessionario/committente** invece del cedente;
    
*   **eccezione TD04**: due documenti con stessa chiave sono ammessi **solo se uno dei due è TD04**; se entrambi sono TD04 con stessa chiave → errore;
    
*   per i documenti con `TipoDocumento ∈ {TD16…TD20, TD22, TD23, TD28, TD29}` la verifica è in capo al **cessionario**.
    

Per **00409** l'ambito è il singolo **lotto** (tutti i `FatturaElettronicaBody` dello stesso file): **replicabile offline**.  
Per **00404** l'ambito è lo **storico SdI**: **non replicabile**. Utile comunque implementare la stessa logica _intra-file_ come `00409` e come **warning "possibile duplicato interno"**.

* * *

## 4. Regole di coerenza aggiuntive (non SdI, raccomandate)

Questi controlli **non** producono uno scarto SdI ufficiale: sono "buone validazioni" che un validatore locale può offrire come warning. Sono quelli che emergono dalla prassi e dalle implementazioni di riferimento.
| # | Regola | Note |
| --- | --- | --- |
| E1 | `ImportoTotaleDocumento` **vs Σ(**`ImponibileImporto` **+** `Imposta`**) di tutti i** `DatiRiepilogo` (± tolleranza) | non esiste un codice SdI dedicato; `ImportoTotaleDocumento` è opzionale |
| E2 | **Σ** `ImportoPagamento` **vs totale documento** | idem |
| E3 | `CondizioniPagamento` **= TP01 ("a rate") ⇒ più** `DettaglioPagamento`; TP02/TP03 ⇒ un solo blocco | linea guida di compilazione, non controllo SdI |
| E4 | **Bollo**: `BolloVirtuale = SI` ⇒ importo dovuto **2,00 €** per fattura (determinato dall'AdE a posteriori); `ImportoBollo` è **opzionale** dalla 1.6 e _"il contenuto del campo Importo bollo non è rilevante"_; la soglia dei **77,47 €** e le esclusioni (TD16–TD19, TD28; regimi RF05–RF11) sono elaborazioni **back-office** AdE, non scarti SdI | implementare come **informativo**; uso tipico: bollo su fatture senza IVA sopra soglia con Natura N2.1/N2.2/N3.5/N3.6/N4 |
| E5 | `ImportoRitenuta` **=** `AliquotaRitenuta`**% × base imponibile** | nessun codice SdI |
| E6 | `ImportoContributoCassa` **=** `AlCassa`**% ×** `ImponibileCassa` | nessun codice SdI |
| E7 | **Coerenza segni nelle note di credito** (TD04/TD08/TD09 con importi negativi) e con la fattura collegata | l'XSD ammette il segno meno; nessun controllo SdI di segno |
| E8 | **IBAN**: validazione **mod-97** + lunghezza/struttura per paese; se presenti **ABI/CAB**, verificarne la coerenza con l'IBAN (posizioni 6-10 e 11-15 dell'IBAN italiano) | l'XSD valida solo il pattern; il SdI non incrocia |
| E9 | `ImportoTotaleDocumento` **assente** con `DatiPagamento` presente, o viceversa incoerenze di pagamento | informativo |
| E10 | **CAP ↔ Comune ↔ Provincia**: coerenza con tabelle ISTAT/Poste | anche il SdI non la verifica (è "un postino") → **warning, mai errore bloccante** |
| E11 | **Email/PEC**: forma RFC (dominio con TLD non numerico); **non** usare la regex restrittiva dell'XSD come regola bloccante | vedi §7 |
| E12 | `Data` **nel futuro** (00403) è già SdI, ma offline va confrontata con "oggi"; con `DataInizioPeriodo`/`DataFinePeriodo` verificare `inizio ≤ fine` | informativo |
| E13 | `RiferimentoNumeroLinea` deve puntare a un `NumeroLinea` esistente | non è un controllo SdI esplicito; utile |
| E14 | `NumeroLinea` **univoci** all'interno di ciascun `DettaglioLinee` | idem |
| E15 | `AltriDatiGestionali/TipoDato` ∈ set documentato; `ALI-COMP` **richiede** `RiferimentoNumero`; `ESENZSPORT` (1.9.1) su redditi sportivi | informativo/strutturale |
| E16 | `FormatoAttachment`**/**`NomeAttachment`: whitelist di estensioni e limite di dimensione dell'allegato (l'XSD non la contiene, ma le implementazioni reali sì) | informativo |
| E17 | `DatiTrasporto`: `DataOraRitiro ≤ DataOraConsegna`; `PesoLordo ≥ PesoNetto` | informativo |
| E18 | `DatiVeicoli.TotalePercorso` obbligatorio solo per mezzi di trasporto nuovi (art. 38 c.4 DL 331/1993) | informativo |

* * *

## 5. Algoritmi di validazione degli identificatori (implementabili offline)

### 5.1 Partita IVA (11 cifre) — Luhn modificato AdE

1.  Formato: `^[0-9]{11}$`. **Trattare come stringa** (zero iniziali significativi: `07643520567`).
    
2.  Siano `d1…d10` le prime 10 cifre (1-indexed). Posizioni **dispari** (1,3,5,7,9): somma le cifre così come sono → `X`. Posizioni **pari** (2,4,6,8,10): raddoppia e **se il doppio ≥ 10 sottrai 9** → somma → `Y`.  
    ⚠️ Non è `mod 9`: `9×2 = 18 → 9` (mentre `18 mod 9 = 0`).
    
3.  `T = (X + Y) mod 10`; cifra di controllo `C = (10 − T) mod 10`.
    
4.  Verifica "one-shot": includi anche l'11ª cifra tra le posizioni dispari e richiedi `totale mod 10 == 0`.
    
5.  Esempio: `0764352056?` → X = 0+6+3+2+5 = 16; Y = 5+8+1+0+3 = 17; T = 3; C = 7 → `07643520567`.
    
6.  `00000000000` **è formalmente valida** (il check digit torna) ma non esiste. → Un validatore deve separare **"formalmente valido"** da **"esistente"**: la seconda è **online** (servizio AdE "Controllo partita IVA", VIES per l'UE).
    

### 5.2 Codice Fiscale — persona fisica (16 caratteri)

**Struttura:** `[1-3]` cognome (prime 3 consonanti, poi vocali, riempi con `X`) · `[4-6]` nome (1ª, 3ª, 4ª consonante; se < 4 consonanti → prime 3; se < 3 → come il cognome) · `[7-8]` anno · `[9]` mese (`A B C D E H L M P R S T`) · `[10-11]` giorno (maschi 01–31; femmine **+40** → 41–71; **32–40 mai validi**) · `[12]` lettera del codice catastale · `[13-15]` parte numerica · `[16]` carattere di controllo (lettera).
**Check character** — si convertono i 15 caratteri secondo la tabella del proprio posto (dispari = 1,3,…,15; pari = 2,4,…,14), si somma, `somma mod 26`, mappa `0→A … 25→Z`.

*   **Posti pari**: cifre = valore; lettere `A=0 … Z=25`.
    
*   **Posti dispari**: tabella dedicata:  
    `0/A=1 · 1/B=0 · 2/C=5 · 3/D=7 · 4/E=9 · 5/F=13 · 6/G=15 · 7/H=17 · 8/I=19 · 9/J=21 · K=2 · L=4 · M=18 · N=20 · O=11 · P=3 · Q=6 · R=8 · S=12 · T=14 · U=16 · V=10 · W=22 · X=25 · Y=24 · Z=23`
    
*   Esempio verificato: `ABCDEF12B23P432P` → somma 67 → 67 mod 26 = 15 → **P** ✅
    

**Omocodia** — quando due soggetti produrrebbero lo stesso CF, l'Agenzia sostituisce una o più **cifre nelle posizioni numeriche** (**7, 8, 10, 11, 13, 14, 15**), partendo da **destra**, con le lettere:  
`0→L · 1→M · 2→N · 3→P · 4→Q · 5→R · 6→S · 7→T · 8→U · 9→V`  
(⚠️ `0→L`, **non** `0→O`: la lettera `O` non è ammessa in quelle posizioni.)
Riconoscimento/normalizzazione di un CF omocodico valido:

1.  applicare l'algoritmo del check character **al letterale** (le lettere omocodiche sono caratteri legittimi per le tabelle);
    
2.  pattern: pos. 1–6 `[A-Z]`; pos. 9 ∈ `{A,B,C,D,E,H,L,M,P,R,S,T}`; pos. 12 `[A-Z]`; pos. 16 `[A-Z]`;
    
3.  ogni posizione numerica ∈ `[0-9] ∪ {L,M,N,P,Q,R,S,T,U,V}`;
    
4.  **contiguità da destra**: scorrendo le 7 posizioni da destra verso sinistra, dopo la prima cifra incontrata tutte le posizioni a sinistra devono essere cifre (le sostituzioni parziali "a destra" sono invalide);
    
5.  per ottenere il **CF base**: mappa inversa `L→0 … V→9`, poi **ricalcola** il check character (che cambia).
    

**CF di enti / persone giuridiche:** codice fiscale **numerico di 11 cifre**, validato con **lo stesso algoritmo della partita IVA**. Alla costituzione CF e P.IVA **coincidono**, ma **non sempre restano uguali** (cambio di provincia, ecc.): **non assumere** `CF == P.IVA` **come invariante** — validare entrambi. Esistono inoltre CF provvisori numerici di 11 cifre assegnati a persone fisiche in casi eccezionali: vanno validati come una P.IVA.
Nel tracciato `CodiceFiscale` ammette `[A-Z0-9]{11,16}`: una validazione rigorosa distingue:

*   **16** caratteri → persona fisica (pattern + check char + eventuale omocodia);
    
*   **11** cifre → ente / provvisorio (Luhn P.IVA);
    
*   altre lunghezze (12–15) → sintatticamente ammesse dal pattern XSD, ma **sospette**: warning.
    

### 5.3 IBAN — mod 97 (ISO 13616)

1.  rimuovi spazi, maiuscolo; 2. sposta i primi 4 caratteri in coda; 3. sostituisci ogni lettera con `A=10 … Z=35`; 4. calcola con aritmetica big-integer il valore `mod 97`: valido se **= 1**.  
    Formato italiano (27): `IT` + 2 check + **CIN** (1 lettera) + **ABI** (5) + **CAB** (5) + conto (12).  
    Nello schema il campo è `[a-zA-Z]{2}[0-9]{2}[a-zA-Z0-9]{11,30}` (15–34 complessivi): **serve una tabella di struttura/lunghezza per paese** (IBAN Registry SWIFT) per un controllo rigoroso.
    

### 5.4 BIC (ISO 9362)

`^[A-Z]{6}[A-Z0-9]{2}([A-Z0-9]{3})?$` → 8 o 11 caratteri; i caratteri 5-6 sono il **codice paese ISO 3166-1 alpha-2**. Nessun checksum: solo formato; la registrazione è online.

### 5.5 CAP, Provincia, Nazione, Divisa

| Elemento | Regola offline | Tabella necessaria |
| --- | --- | --- |
| `CAP` | `^[0-9]{5}$`; **zeri iniziali significativi** (`00187`); per l'estero convenzione `00000` | coerenza CAP↔comune solo con tabella (ISTAT / Poste / "Anagrafe dei comuni") → **warning** |
| `Provincia` | `^[A-Z]{2}$` | lista delle sigle (~107), incluse le **storicamente valide** (CI, VS, OG, OT) e `EE` per "estero" (convenzione, non confermata dalle specifiche → warning) |
| `Nazione` / `IdPaese` | `^[A-Z]{2}$` | ISO 3166-1 alpha-2 (~249); attenzione a `GR`/`EL`, `XK`, `OO`, `XI` |
| `Divisa` | `^[A-Z]{3}$` | ISO 4217 |
| `TipoResa` | `^[A-Z]{3}$` | Incoterms (nessuna enumerazione ufficiale) |

### 5.6 PEC / email

Sintassi: mailbox RFC 5321/5322, `local-part@dominio`, dominio con label `[A-Za-z0-9-]` e TLD non puramente numerico. **Recapitabilità: non verificabile offline** (richiede DNS/MX e l'appartenenza del dominio a un **gestore PEC accreditato** — elenco AgID). Un validatore locale può però tenere una **blacklist dei domini/caselle SdI** (per 00330). Nel tracciato, `PECDestinatario` e `Email` sono `maxLength 256`.

### 5.7 CIG e CUP

| Codice | Formati ufficiali | Check |
| --- | --- | --- |
| **CIG** | `[0-9]{7}[0-9A-F]{3}` (CIG) · `[V-Z][0-9A-F]{9}` (SmartCIG) · `[A-U][0-9A-F]{9}` (CIG Unico) | **solo formato**, nessun checksum pubblico; esistenza online (ANAC/SIMOG) |
| **CUP** | 15 caratteri alfanumerici; struttura "natura/tipologia/settore/localizzazione/copertura" definita nel corredo informativo, non forzata da un algoritmo | solo formato; esistenza online (Sistema CUP / OpenCUP) |

⚠️ Nel tracciato i campi `CodiceCIG`/`CodiceCUP` sono `String15Type`: lo schema **non** impone il formato a 10 del CIG → validare `String15` strutturalmente e applicare i formati ANAC come **warning di dominio**.

### 5.8 Codice univoco ufficio (IPA) — controllo di appartenenza offline

`CodiceDestinatario` per la PA = **6 caratteri** `[A-Z0-9]{6}` corrispondenti al _Codice Ufficio_ dell'**IndicePA**. Il dataset IPA è scaricabile (Open Data IPA: CSV/TXT/JSON/XML/XLS via CKAN + SPARQL endpoint; web service `WS06_OU_CODUNI`). È quindi possibile un controllo **offline di appartenenza** (tabella da aggiornare). Restano **online**: l'attivazione, la data di avvio del servizio di fatturazione, l'unicità dell'ufficio associato al CF del cessionario (00398/00399).

* * *

## 6. Tabelle da incorporare e relativa manutenzione

| Tabella | Dimensione | Cadenza | Fonte |
| --- | --- | --- | --- |
| Enumerazioni del tracciato (da XSD) | piccole | a ogni provvedimento | XSD ufficiali |
| Descrizioni dei codici (TipoDocumento, Natura, RegimeFiscale, TipoCassa, ModalitàPagamento…) | piccole | a ogni provvedimento | Allegato A / "Rappresentazione tabellare" XLS |
| `CausalePagamento` (descrizioni) | piccola | **annuale** (modello CU) | istruzioni CU |
| `TipoDato` documentati | piccola | a ogni provvedimento | Allegato A |
| ISO 3166-1 alpha-2 | ~249 | rara | ISO |
| ISO 4217 | ~180 | rara | ISO |
| Sigle province | ~107 | rara | ISTAT |
| Codici catastali Belfiore + codici `Z` Stati esteri | ~7.900 comuni + ~216 stati + storici | periodica | Archivio Comuni e Stati esteri (AdE) |
| IndicePA (codici ufficio 6 char) | decine di migliaia | periodica | Open Data IPA |
| IBAN structure per paese | ~86 | periodica | SWIFT IBAN Registry |
| Trust store CA / CRL | variabile | periodica | AgID / QTSP |
| esempi/hash di riferimento ufficiali | piccola | a ogni versione | "Esempi fattura, hash e ricevute" (AdE) |

**Principio:** ogni tabella va **versionata e datata**, e il validatore deve poter rispondere alla domanda _"questa fattura è conforme alle regole in vigore alla data X?"_.

* * *

## 7. Cosa NON è validabile offline (e perché) — da dichiarare esplicitamente all'utente

| Condizione | Codice | Perché serve la rete |
| --- | --- | --- |
| Esistenza/attività di P.IVA e CF di trasmittente, cedente, rappresentante fiscale, cessionario | 00300–00306 | **Anagrafe tributaria** |
| P.IVA cessata / cessata da oltre 5 anni | 00301, 00323 | stato del registro |
| Coerenza P.IVA ↔ CF dello stesso soggetto | 00320, 00324 | registro |
| Appartenenza al **gruppo IVA** | 00321, 00322, 00325, 00326, **00327** | registro |
| `CodiceDestinatario` esistente / attivo / ufficio IPA univoco | 00311, 00312, 00398, 00399 | **registro canali SdI + IndicePA** |
| Validità della **dichiarazione d'intento** | 00477 | banca dati AdE |
| **Fattura duplicata** rispetto allo storico | 00404 | archivio SdI |
| Revoca/sospensione del certificato di firma | 00101 | CRL/OCSP |
| CA affidabile | 00104 | elenco CA (da spedire) |
| Conseguenze di consegna: RC / MC / AT / DT, esito committente EC01/EC02 | — | processo SdI/destinatario |
| Esistenza CIG (ANAC) / CUP (MEF) | — | banche dati esterne |
| VIES (partita IVA UE) | — | servizio UE |
| Elaborazioni **back-office** AdE: bollo dovuto (2 €, soglia 77,47 €, elenchi A/B) | — | elaborazione trimestrale AdE |

**Conseguenza di prodotto:** il validatore deve produrre un **report a livelli** (es. `L1 OK`, `L2 OK`, `L3 3 warning`, `L4 non verificato — richiede servizi online`) e **non** un booleano "valida/non valida".

* * *

## 8. Trappole note e divergenze documentate tra implementazioni

Raccolte dai changelog delle librerie di riferimento (in particolare FatturaElettronica.NET, BSD-3, e `slam/php-validatore-fattura-elettronica`, MIT) [S]:
**Plumbing XML**

1.  **Encoding non sempre UTF-8**: circolano file in `windows-1252` → gestire l'encoding dichiarato, non assumere UTF-8.
    
2.  **BOM / stream non a posizione 0**: riportare sempre lo stream all'inizio prima del parsing.
    
3.  **Processing instruction** (`<?xml-stylesheet?>`) e commenti possono far fallire parser ingenuo.
    
4.  `xsi:schemaLocation` ha causato problemi con client Java: non pretenderlo.
    
5.  `xml:space` **/ spazi significativi**: non fare `trim()` sui testi (`Descrizione`, `Causale` sono a lunghezza controllata e gli spazi contano).
    
6.  **Namespace misti + firma XAdES innestata**: la strategia robusta è il **matching per local-name, namespace-agnostico**.
    

**Modello documentale**  
7. `FatturaElettronicaBody` **multiplo ("lotto")**: molte implementazioni processano solo il primo body → è l'errore più frequente; il lotto è la norma, non l'eccezione.  
8. **Archivio ZIP**: il `.zip` non si firma; ogni file interno sì; i controlli sono per singolo file. Spesso non implementato.  
9. `.p7m`: Base64 con a-capo CRLF, certificati con `UniversalString` ASN.1: gestire i casi limite.  
10. **Fattura semplificata validata con lo schema ordinaria** (o viceversa): errore storico documentato.
**Aritmetica**  
11. **Tolleranze "larghe"** su 00421 (±0,01) e 00423 (±0,01), **±1 €** su 00422: implementarle come **costanti per-controllo**, non come epsilon globale.  
12. **Decimali**: `PrezzoUnitario` ammette 8 decimali, `PrezzoTotale` 2–8; gli **sconti a 3+ decimali** hanno causato scarti reali. Mai `f64`.  
13. `AliquotaIVA`: ammettere **1%** (il controllo 00424 è `< 1.00` **e** `≠ 0.00`); rifiutare **> 100%**.
**Tabelle**  
14. `Natura`: `N2/N3/N6` **non** vanno rimossi dallo schema ma vanno segnalati (00445); non riutilizzare mai la lista della fattura semplificata per l'ordinaria.  
15. `RegimeFiscale`: RF03 rimosso, RF19 presente, **RF20** nuovo (presente nell'XSD 1.2.3 corrente ma assente in copie vendored).  
16. `CausalePagamento`: la tabella **cambia ogni anno**; `Z` non valido dal 2021; i codici `F/J/K/V2` sono della CU e **non** ammessi in fattura.  
17. ISO 3166: casi storici (Islanda mancante/Irlanda duplicata), `EL` vs `GR`, `XK`, `OO`, `XI`; P.IVA che iniziano per `EU`.  
18. `TipoCassa`/`ModalitaPagamento`: allineare a XSD 1.2.3 (MP23 PagoPA incluso).
**Duplicazione e ciclo di vita**  
19. **00426** — vedi sotto.  
20. 00404/00409: le deviazioni (Art73 → data completa; SoggettoEmittente CC → soggetto chiave è il cessionario; eccezione TD04; TD16–TD29 in capo al cessionario) sono state riscritte più volte e sono la fonte principale di falsi positivi.

### 8.1 Il caso `00426` (da gestire con cautela)

*   **Elenco controlli v1.8 (01/10/2022)** [V]: `00426` presente con **doppia implicazione** — errore sia se manca la PEC con `CodiceDestinatario = 0000000`, sia se la PEC è presente con codice ≠ `0000000`.
    
*   **Elenco controlli v2.0 (31/01/2025)** [S]: 00426 **non** riportato.
    
*   **Allegato A 1.9.1** [V]: prevede espressamente il caso `0000000` **senza** PEC ("soggetto per il quale non si conosce il canale telematico") → la PEC è **facoltativa**.
    
*   **Conclusione operativa:** implementare **solo** `PEC valorizzata ⇒ CodiceDestinatario = 0000000` come **errore**, e **non** richiedere la PEC con `0000000`. Mantenere il secondo ramo come **warning configurabile**. **Verificare** sull'Appendice 1 di Allegato A 1.9.1 prima di una release.
    

* * *

## 9. Risorse di test e corpus di riferimento

| Risorsa | Contenuto | URL/nota |
| --- | --- | --- |
| **Esempi ufficiali FatturaPA/FPR (v1.2)** | `IT01234567890_FPA01.xml` (1 linea→PA), `_FPA02` (più linee→PA), `_FPA03` (lotto→PA), `_FPR01`, `_FPR02`, `_FPR03` (privati) | fatturapa.gov.it → _Norme e regole → Documentazione Fattura elettronica → FatturaPA_ |
| **"Esempi fattura, hash e ricevute"** (1.9.1) | bundle con **hash** e ricevute: il corpus di riferimento più aggiornato | pagina AdE "Specifiche tecniche versione 1.9.1" |
| **Fogli di stile (XSLT)** | rendering di fatture e ricevute | utile per ispezione manuale/round-trip |
| **Tool online "Controlla fattura"** | pre-check AdE nell'area riservata: **l'oracolo ufficiale** per dirimere divergenze | Fatture e Corrispettivi → Funzionalità di supporto |
| **Ambiente di test SdI** (WS/SFTP) | test del processo di fatturazione via canali accreditati | sistema di accreditamento |
| `italia/fatturapa-testsdi` | emulatore SdI completo (issuer/SdI/recipient), AGPL-3 | utile per test di processo, **non** è un oracolo di validazione; esclude ZIP, multi-body, firma |
| **Fixture delle librerie** | FatturaElettronica.NET (`Test/`), slam/php (`tests/`, `xsd/`), Python `fatturapa` (`tests/tracciati/`), `invopop/gobl.fatturapa` (`schemas/`) | utili come regression suite cross-implementazione |

**Strategia di test consigliata:** (1) round-trip sui 6 esempi ufficiali; (2) **differential testing** contro `xmllint --schema` (validazione XSD) e contro FatturaElettronica.NET per i 004xx; (3) property-based testing sull'aritmetica (00421/00422/00423) con generazione di sconti a cascata misti; (4) fuzzing su XML malformati/BOM/encoding; (5) casi limite espliciti per TD16 (Natura + IVA ≠ 0), TD04 (duplicazione), lotto multiplo, `0000000` con/senza PEC, `XXXXXXX` con cessionario IT (00313).

* * *

## 10. Elenco delle fonti primarie (da riscaricare e versionare)

**B2B/B2C (regole tecniche, privati)**

*   AdE — _Specifiche tecniche versione 1.9.1 — Documentazione utilizzabile dal 15 maggio 2026_: `https://www.agenziaentrate.gov.it/portale/specifiche-tecniche-versione-1.9.1-%C2%A0-utilizzabili-dal-15-maggio-2026-`  
    → Allegato A 1.9.1 (PDF, agg. 31/03/2026) · Appendix A (EN) · Rappresentazione tabellare ordinaria/semplificata (PDF+XLS) · `Schema VFSM10` · `Schema_VFPR12` · `MessaggiFatturaTypes_v1.0` · QR Code partita IVA · Esempi fattura/hash/ricevute · Fogli di stile per ricevute
    
*   AdE — _Variazioni alle specifiche tecniche (changelog)_: `https://www.agenziaentrate.gov.it/portale/documents/d/guest/variazioni-alle-specifiche-tecniche-fatture-elettroniche2025-01-31`
    
*   _Elenco dei controlli effettuati sul file fattura del Sistema di Interscambio_ **v1.8** (01/10/2022): `https://www.fatturapa.gov.it/export/documenti/Elenco_Controlli_version_1.8.pdf` (e variante `Elenco-Controlli-versione-1.8.pdf`)
    
*   _Elenco controlli_ **v2.0** (31/01/2025): `https://www.fatturapa.gov.it/export/documenti/fatturapa/v1.4/Elenco-Controlli-versione-2.0.pdf`
    
*   _Elenco controlli versione 1.9_: `https://www.fatturapa.gov.it/export/documenti/Elenco-Controlli-versione-v1.9.pdf`
    

**PA / FPA12**

*   fatturapa.gov.it — _Formato FatturaPA_: `https://www.fatturapa.gov.it/it/norme-e-regole/documentazione-fattura-elettronica/formato-fatturapa/`  
    → Schema FatturaPA **1.2.3** · Schema Fattura Ordinaria · Schema Fattura Semplificata **1.0.2** · Specifiche tecniche formato FatturaPA **1.4** (PDF) · Rappresentazioni tabellari · fogli di stile · esempi FPA01–03/FPR01–03 · PEC `sdi01@pec.fatturapa.it`
    
*   XSD verificato: `https://www.fatturapa.gov.it/export/documenti/fatturapa/v1.4/Schema_VFPR12_v1.2.3.xsd`
    

**Tabelle e anagrafiche esterne**

*   Archivio Comuni e Stati esteri (AdE) — codici catastali e codici `Z`
    
*   Open Data IPA (AgID) — dataset codici ufficio: `https://indicepa.gov.it/ipa-files/manuale-ckan/index.html`
    
*   ISTAT — Codici dei comuni, delle province e delle regioni
    
*   SWIFT IBAN Registry — struttura IBAN per paese
    
*   AGID — elenco gestori PEC · elenco CA affidabili
    
*   Istruzioni modello CU (annuale) — descrizioni `CausalePagamento`
    

* * *

## 11. Checklist finale per la progettazione di un validatore locale

1.  **Scaricare e hashare** gli XSD ufficiali correnti (ordinaria 1.2.3, semplificata 1.0.2, FatturaPA 1.2.3) e registrarne data+hash nel build.
    
2.  Separare in moduli: **L0 file** · **L1 well-formedness** · **L2 XSD** · **L3 regole di contenuto** · **L4 (out of scope, dichiarato)** · **L5 (out of scope, dichiarato)**.
    
3.  Implementare i controlli **004xx** con **tolleranze per-controllo** e aritmetica **decimale esatta**.
    
4.  Implementare le **tabelle con effective-date**, in particolare `CausalePagamento` e le deprecazioni (`N2/N3/N6`, `Z`, `RF03`).
    
5.  Implementare gli **algoritmi di checksum** (P.IVA, CF/omocodia, IBAN) come _livello distinto_ dal "formato".
    
6.  Non usare la **regex email dell'XSD** come regola bloccante; usare una validazione di forma permissiva.
    
7.  Gestire **lotto multiplo**, **ZIP**, `.p7m` e **matches per local-name**.
    
8.  Produrre un **report con severità** (errore SdI / controllo raccomandato / informativo) e con **riferimento al codice ufficiale** e al paragrafo.
    
9.  Esplicitare nel report **cosa non è stato verificato** (L4/L5) e suggerire il pre-check online AdE.
    
10.  Predisporre una **suite di test** con gli esempi ufficiali + differential testing + property-based sull'aritmetica.